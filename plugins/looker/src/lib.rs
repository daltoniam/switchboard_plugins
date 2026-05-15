mod handlers;
mod tools;

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use switchboard_guest_sdk as sdk;

// ── Constants ──────────────────────────────────────────────────────────────

/// Hard cap on rows returned from any inline/saved query. Prevents
/// accidentally pulling millions of rows into context.
pub(crate) const MAX_ROW_LIMIT: i64 = 5000;

/// Default row cap when a caller omits `limit`.
pub(crate) const DEFAULT_ROW_LIMIT: i64 = 100;

/// Buffer subtracted from the Looker `access_token` expiry before treating it
/// as expired. Avoids racing on the wire-side TTL.
const TOKEN_SAFETY_WINDOW_SECS: u64 = 30;

/// Fallback expiry if the Looker login response omits `expires_in`.
const DEFAULT_TOKEN_TTL_SECS: u64 = 15 * 60;

// ── Config + token cache ───────────────────────────────────────────────────

struct Config {
    base_url: String, // includes /api/4.0
    client_id: String,
    client_secret: String,
}

struct TokenCache {
    token: String,
    /// Unix seconds when the token expires.
    expires_at: u64,
}

static CONFIG: Mutex<Option<Config>> = Mutex::new(None);
static TOKEN: Mutex<Option<TokenCache>> = Mutex::new(None);

fn with_config<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&Config) -> R,
{
    let guard = CONFIG.lock().map_err(|e| e.to_string())?;
    match guard.as_ref() {
        Some(c) => Ok(f(c)),
        None => Err("looker: not configured".into()),
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ── Required ABI exports ───────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn name() -> u64 {
    sdk::leaked_string("looker")
}

#[no_mangle]
pub extern "C" fn metadata() -> u64 {
    sdk::leaked_metadata(&sdk::PluginMetadata {
        name: "looker".into(),
        version: "0.1.0".into(),
        abi_version: 1,
        description: "Looker BI integration — dashboards, Looks, LookML models, inline analytics queries, SQL Runner.".into(),
        author: "daltoniam".into(),
        homepage: "https://github.com/daltoniam/switchboard_plugins".into(),
        license: "MIT".into(),
        capabilities: vec!["http".into()],
        credential_keys: vec!["base_url".into(), "client_id".into(), "client_secret".into()],
        plain_text_keys: vec!["base_url".into(), "client_id".into()],
        optional_keys: vec![],
        placeholders: HashMap::from([
            ("base_url".into(), "https://your-instance.cloud.looker.com:19999".into()),
            ("client_id".into(), "Looker API3 client_id".into()),
            ("client_secret".into(), "Looker API3 client_secret".into()),
        ]),
    })
}

#[no_mangle]
pub extern "C" fn tools() -> u64 {
    let defs = tools::tool_definitions();
    let data = serde_json::to_vec(&defs).unwrap_or_default();
    sdk::leaked_result(&data)
}

#[no_mangle]
pub extern "C" fn configure(ptr_size: u64) -> u64 {
    let input = sdk::read_input(ptr_size);
    let creds: HashMap<String, String> = match serde_json::from_slice(&input) {
        Ok(c) => c,
        Err(e) => return sdk::leaked_string(&format!("looker: invalid credentials JSON: {e}")),
    };

    let client_id = creds.get("client_id").cloned().unwrap_or_default();
    let client_secret = creds.get("client_secret").cloned().unwrap_or_default();
    let raw = creds
        .get("base_url")
        .map(|s| s.trim_end_matches('/').to_string())
        .unwrap_or_default();

    if client_id.is_empty() {
        return sdk::leaked_string("looker: client_id is required");
    }
    if client_secret.is_empty() {
        return sdk::leaked_string("looker: client_secret is required");
    }
    if raw.is_empty() {
        return sdk::leaked_string(
            "looker: base_url is required (e.g., https://your-instance.cloud.looker.com:19999)",
        );
    }

    // Normalize: ensure /api/4.0 suffix. If the user pasted only the host,
    // append it; if they pasted the API root, leave it alone.
    let base_url = if raw.contains("/api/") {
        raw
    } else {
        format!("{raw}/api/4.0")
    };

    *CONFIG.lock().unwrap() = Some(Config {
        base_url,
        client_id,
        client_secret,
    });
    // Reset any cached token from a previous configuration.
    *TOKEN.lock().unwrap() = None;
    0
}

#[no_mangle]
pub extern "C" fn execute(ptr_size: u64) -> u64 {
    let input = sdk::read_input(ptr_size);
    let req: sdk::ExecuteRequest = match serde_json::from_slice(&input) {
        Ok(r) => r,
        Err(e) => {
            let r = sdk::err_result(&format!("invalid request: {e}"));
            let data = serde_json::to_vec(&r).unwrap_or_default();
            return sdk::leaked_result(&data);
        }
    };

    let result = dispatch(&req.tool_name, req.args);
    let data = serde_json::to_vec(&result).unwrap_or_default();
    sdk::leaked_result(&data)
}

#[no_mangle]
pub extern "C" fn healthy() -> i32 {
    match looker_get("/user") {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn compact_specs() -> u64 {
    sdk::leaked_compact_specs(&compact_spec_map())
}

// ── Dispatch ───────────────────────────────────────────────────────────────

type HandlerFn = fn(HashMap<String, serde_json::Value>) -> sdk::ToolResult;

fn dispatch(tool_name: &str, args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let handler: Option<HandlerFn> = match tool_name {
        // Search
        "looker_search_content" => Some(handlers::search_content),
        // Folders
        "looker_list_folders" => Some(handlers::list_folders),
        "looker_get_folder" => Some(handlers::get_folder),
        // Looks
        "looker_list_looks" => Some(handlers::list_looks),
        "looker_get_look" => Some(handlers::get_look),
        "looker_run_look" => Some(handlers::run_look),
        // Dashboards
        "looker_list_dashboards" => Some(handlers::list_dashboards),
        "looker_get_dashboard" => Some(handlers::get_dashboard),
        // Queries
        "looker_run_inline_query" => Some(handlers::run_inline_query),
        "looker_run_query" => Some(handlers::run_query),
        "looker_get_query" => Some(handlers::get_query),
        "looker_create_query" => Some(handlers::create_query),
        // SQL Runner
        "looker_run_sql_query" => Some(handlers::run_sql_query),
        // Models
        "looker_list_models" => Some(handlers::list_models),
        "looker_get_model" => Some(handlers::get_model),
        "looker_get_model_explore" => Some(handlers::get_model_explore),
        // Connections
        "looker_list_connections" => Some(handlers::list_connections),
        "looker_get_connection" => Some(handlers::get_connection),
        // Users
        "looker_get_me" => Some(handlers::get_me),
        "looker_list_users" => Some(handlers::list_users),
        "looker_get_user" => Some(handlers::get_user),
        // Schedules
        "looker_list_scheduled_plans" => Some(handlers::list_scheduled_plans),
        "looker_get_scheduled_plan" => Some(handlers::get_scheduled_plan),
        _ => None,
    };

    match handler {
        Some(f) => f(args),
        None => sdk::err_result(&format!("unknown tool: {tool_name}")),
    }
}

// ── Auth ───────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct LoginResponse {
    access_token: String,
    #[serde(default)]
    expires_in: u64,
}

/// Return a cached, unexpired access token, fetching/refreshing as needed.
/// Process-scoped (module-instance-scoped) cache.
fn ensure_token() -> Result<String, String> {
    {
        let guard = TOKEN.lock().map_err(|e| e.to_string())?;
        if let Some(tc) = guard.as_ref() {
            if !tc.token.is_empty()
                && now_secs().saturating_add(TOKEN_SAFETY_WINDOW_SECS) < tc.expires_at
            {
                return Ok(tc.token.clone());
            }
        }
    }

    let (base_url, client_id, client_secret) = with_config(|c| {
        (
            c.base_url.clone(),
            c.client_id.clone(),
            c.client_secret.clone(),
        )
    })?;

    let body = format!(
        "client_id={}&client_secret={}",
        form_encode(&client_id),
        form_encode(&client_secret),
    );

    let mut headers = HashMap::new();
    headers.insert(
        "Content-Type".into(),
        "application/x-www-form-urlencoded".into(),
    );

    let req = sdk::HttpRequest {
        method: "POST".into(),
        url: format!("{base_url}/login"),
        headers,
        body,
        ..Default::default()
    };

    let resp = sdk::host_http_request(&req)?;
    if resp.status >= 400 {
        return Err(format!(
            "looker login failed ({}): {}",
            resp.status, resp.body
        ));
    }

    let lr: LoginResponse = serde_json::from_str(&resp.body)
        .map_err(|e| format!("looker login: decode response: {e}"))?;
    if lr.access_token.is_empty() {
        return Err("looker login: empty access_token".into());
    }

    let ttl = if lr.expires_in == 0 {
        DEFAULT_TOKEN_TTL_SECS
    } else {
        lr.expires_in
    };
    let expires_at = now_secs().saturating_add(ttl);

    *TOKEN.lock().map_err(|e| e.to_string())? = Some(TokenCache {
        token: lr.access_token.clone(),
        expires_at,
    });
    Ok(lr.access_token)
}

/// Form-encode a single value. Looker `client_id`/`client_secret` are typically
/// hex tokens, but we still escape to be safe.
fn form_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        let c = *b;
        let safe = c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.' | b'~');
        if safe {
            out.push(c as char);
        } else {
            out.push_str(&format!("%{c:02X}"));
        }
    }
    out
}

// ── HTTP helpers ───────────────────────────────────────────────────────────

/// Send a request to the Looker API and return the raw response body as a
/// `serde_json::Value`. On 401 transparently refresh the token and retry once.
fn do_request(
    method: &str,
    path: &str,
    body: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let (raw, status) = send(method, path, body)?;
    let (raw, status) = if status == 401 {
        *TOKEN.lock().map_err(|e| e.to_string())? = None;
        send(method, path, body)?
    } else {
        (raw, status)
    };
    if status >= 400 {
        return Err(format!("looker API error ({status}): {raw}"));
    }
    if status == 204 || raw.is_empty() {
        return Ok(serde_json::json!({"status": "success"}));
    }
    serde_json::from_str(&raw).map_err(|e| format!("looker: decode response: {e}"))
}

fn send(
    method: &str,
    path: &str,
    body: Option<&serde_json::Value>,
) -> Result<(String, i32), String> {
    let token = ensure_token()?;
    let base_url = with_config(|c| c.base_url.clone())?;

    let mut headers = HashMap::new();
    headers.insert("Authorization".into(), format!("token {token}"));
    headers.insert("Accept".into(), "application/json".into());
    let body_str = if let Some(b) = body {
        headers.insert("Content-Type".into(), "application/json".into());
        serde_json::to_string(b).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    let req = sdk::HttpRequest {
        method: method.into(),
        url: format!("{base_url}{path}"),
        headers,
        body: body_str,
        ..Default::default()
    };

    let resp = sdk::host_http_request(&req)?;
    Ok((resp.body, resp.status))
}

pub(crate) fn looker_get(path: &str) -> Result<serde_json::Value, String> {
    do_request("GET", path, None)
}

pub(crate) fn looker_post(
    path: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    do_request("POST", path, Some(body))
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Apply the default and hard-cap row limits to a user-supplied value.
pub(crate) fn clamp_limit(n: i64) -> i64 {
    if n <= 0 {
        DEFAULT_ROW_LIMIT
    } else if n > MAX_ROW_LIMIT {
        MAX_ROW_LIMIT
    } else {
        n
    }
}

/// Percent-encode a path segment. Identical safe-char set to Go's
/// `url.PathEscape`: encodes everything except RFC 3986 unreserved chars.
pub(crate) fn path_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        let c = *b;
        let safe = c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.' | b'~');
        if safe {
            out.push(c as char);
        } else {
            out.push_str(&format!("%{c:02X}"));
        }
    }
    out
}

/// Encode a query-string value (spaces -> %20, no `+` substitution).
pub(crate) fn query_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        let c = *b;
        let safe = c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.' | b'~');
        if safe {
            out.push(c as char);
        } else {
            out.push_str(&format!("%{c:02X}"));
        }
    }
    out
}

// ── Compact specs ──────────────────────────────────────────────────────────

fn compact_spec_map() -> HashMap<String, Vec<String>> {
    // Compaction specs cover read endpoints only. Tools that return user
    // analytics data (run_inline_query, run_query, run_look, run_sql_query)
    // intentionally have NO specs — their payload IS the answer and must pass
    // through untouched. Mutations (create_query) also pass through unmodified.
    let mut s: HashMap<String, Vec<String>> = HashMap::new();

    // Search — wrapper {dashboards:[],looks:[],folders:[]}.
    s.insert(
        "looker_search_content".into(),
        vec![
            "dashboards[].id".into(),
            "dashboards[].title".into(),
            "dashboards[].description".into(),
            "dashboards[].folder.name".into(),
            "dashboards[].user_id".into(),
            "dashboards[].view_count".into(),
            "dashboards[].updated_at".into(),
            "looks[].id".into(),
            "looks[].title".into(),
            "looks[].description".into(),
            "looks[].folder.name".into(),
            "looks[].user_id".into(),
            "looks[].query_id".into(),
            "looks[].view_count".into(),
            "looks[].updated_at".into(),
            "folders[].id".into(),
            "folders[].name".into(),
            "folders[].parent_id".into(),
        ],
    );

    // Folders
    s.insert(
        "looker_list_folders".into(),
        vec![
            "id".into(),
            "name".into(),
            "parent_id".into(),
            "child_count".into(),
        ],
    );
    s.insert(
        "looker_get_folder".into(),
        vec![
            "id".into(),
            "name".into(),
            "parent_id".into(),
            "child_count".into(),
            "dashboards".into(),
            "looks".into(),
        ],
    );

    // Looks
    s.insert(
        "looker_list_looks".into(),
        vec![
            "id".into(),
            "title".into(),
            "description".into(),
            "folder.name".into(),
            "user_id".into(),
            "query_id".into(),
            "view_count".into(),
            "favorite_count".into(),
            "public".into(),
            "updated_at".into(),
        ],
    );
    s.insert(
        "looker_get_look".into(),
        vec![
            "id".into(),
            "title".into(),
            "description".into(),
            "folder.name".into(),
            "folder.id".into(),
            "user_id".into(),
            "query_id".into(),
            "view_count".into(),
            "favorite_count".into(),
            "public".into(),
            "updated_at".into(),
            "query.model".into(),
            "query.view".into(),
            "query.fields".into(),
        ],
    );

    // Dashboards
    s.insert(
        "looker_list_dashboards".into(),
        vec![
            "id".into(),
            "title".into(),
            "description".into(),
            "folder.name".into(),
            "user_id".into(),
            "view_count".into(),
            "favorite_count".into(),
            "updated_at".into(),
        ],
    );
    s.insert(
        "looker_get_dashboard".into(),
        vec![
            "id".into(),
            "title".into(),
            "description".into(),
            "folder.name".into(),
            "folder.id".into(),
            "user_id".into(),
            "view_count".into(),
            "favorite_count".into(),
            "updated_at".into(),
            "dashboard_elements[].id".into(),
            "dashboard_elements[].title".into(),
            "dashboard_elements[].type".into(),
            "dashboard_elements[].query_id".into(),
            "dashboard_filters".into(),
        ],
    );

    // Queries
    s.insert(
        "looker_get_query".into(),
        vec![
            "id".into(),
            "model".into(),
            "view".into(),
            "fields".into(),
            "filters".into(),
            "sorts".into(),
            "limit".into(),
            "pivots".into(),
            "share_url".into(),
        ],
    );

    // Models / explores
    s.insert(
        "looker_list_models".into(),
        vec![
            "name".into(),
            "label".into(),
            "project_name".into(),
            "explores[].name".into(),
            "explores[].label".into(),
            "explores[].hidden".into(),
        ],
    );
    s.insert(
        "looker_get_model".into(),
        vec![
            "name".into(),
            "label".into(),
            "project_name".into(),
            "allowed_db_connection_names".into(),
            "explores[].name".into(),
            "explores[].label".into(),
            "explores[].description".into(),
            "explores[].hidden".into(),
        ],
    );
    s.insert(
        "looker_get_model_explore".into(),
        vec![
            "name".into(),
            "label".into(),
            "description".into(),
            "model_name".into(),
            "view_name".into(),
            "connection_name".into(),
            "fields".into(),
            "joins[].name".into(),
            "joins[].type".into(),
            "joins[].relationship".into(),
        ],
    );

    // Connections
    s.insert(
        "looker_list_connections".into(),
        vec![
            "name".into(),
            "dialect_name".into(),
            "host".into(),
            "port".into(),
            "database".into(),
            "schema".into(),
        ],
    );
    s.insert(
        "looker_get_connection".into(),
        vec![
            "name".into(),
            "dialect_name".into(),
            "host".into(),
            "port".into(),
            "database".into(),
            "schema".into(),
            "username".into(),
            "max_connections".into(),
            "ssl".into(),
        ],
    );

    // Users
    s.insert(
        "looker_get_me".into(),
        vec![
            "id".into(),
            "email".into(),
            "display_name".into(),
            "first_name".into(),
            "last_name".into(),
            "role_ids".into(),
            "is_disabled".into(),
        ],
    );
    s.insert(
        "looker_list_users".into(),
        vec![
            "id".into(),
            "email".into(),
            "display_name".into(),
            "role_ids".into(),
            "is_disabled".into(),
        ],
    );
    s.insert(
        "looker_get_user".into(),
        vec![
            "id".into(),
            "email".into(),
            "display_name".into(),
            "first_name".into(),
            "last_name".into(),
            "role_ids".into(),
            "group_ids".into(),
            "is_disabled".into(),
            "verified_looker_employee".into(),
        ],
    );

    // Scheduled plans
    s.insert(
        "looker_list_scheduled_plans".into(),
        vec![
            "id".into(),
            "name".into(),
            "user_id".into(),
            "look_id".into(),
            "dashboard_id".into(),
            "enabled".into(),
            "crontab".into(),
            "timezone".into(),
            "next_run_at".into(),
        ],
    );
    s.insert(
        "looker_get_scheduled_plan".into(),
        vec![
            "id".into(),
            "name".into(),
            "user_id".into(),
            "look_id".into(),
            "dashboard_id".into(),
            "enabled".into(),
            "crontab".into(),
            "timezone".into(),
            "next_run_at".into(),
            "scheduled_plan_destination".into(),
            "filters_string".into(),
        ],
    );

    s
}
