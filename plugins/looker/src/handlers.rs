use std::collections::HashMap;
use switchboard_guest_sdk as sdk;

use crate::{clamp_limit, looker_get, looker_post, path_escape, query_escape, DEFAULT_ROW_LIMIT};

// ── Small helpers ──────────────────────────────────────────────────────────

fn split_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

/// Return the inner Value on success or an `err_result` ToolResult.
macro_rules! call {
    ($call:expr) => {
        match $call {
            Ok(v) => v,
            Err(e) => return sdk::err_result(&e),
        }
    };
}

fn json_result(v: &serde_json::Value) -> sdk::ToolResult {
    match serde_json::to_string(v) {
        Ok(s) => sdk::raw_result(s),
        Err(e) => sdk::err_result(&format!("encode response: {e}")),
    }
}

// ── Search ─────────────────────────────────────────────────────────────────

/// Run unified search across dashboards, Looks, and folders. Looker has
/// separate `/dashboards/search`, `/looks/search`, `/folders/search`
/// endpoints (no unified `/search`). Fan out to whichever types are requested
/// and merge results.
pub fn search_content(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let terms = sdk::arg_str(&args, "terms");
    if terms.is_empty() {
        return sdk::err_result("terms is required");
    }
    let types = sdk::arg_str(&args, "types");
    let limit = sdk::arg_int(&args, "limit").unwrap_or(25);
    let offset = sdk::arg_int(&args, "offset").unwrap_or(0);

    let mut wanted = [true; 3]; // [dashboard, look, folder]
    if !types.is_empty() {
        wanted = [false; 3];
        for t in types.split(',') {
            match t.trim().to_lowercase().as_str() {
                "dashboard" => wanted[0] = true,
                "look" => wanted[1] = true,
                "folder" => wanted[2] = true,
                _ => {}
            }
        }
    }

    let mut out = serde_json::Map::new();

    if wanted[0] {
        let path = format!(
            "/dashboards/search?title={}&limit={}&offset={}",
            query_escape(&terms),
            limit,
            offset
        );
        let v = call!(looker_get(&path));
        out.insert("dashboards".into(), v);
    }
    if wanted[1] {
        let path = format!(
            "/looks/search?title={}&limit={}&offset={}",
            query_escape(&terms),
            limit,
            offset
        );
        let v = call!(looker_get(&path));
        out.insert("looks".into(), v);
    }
    if wanted[2] {
        // /folders/search uses `name` rather than `title`.
        let path = format!(
            "/folders/search?name={}&limit={}&offset={}",
            query_escape(&terms),
            limit,
            offset
        );
        let v = call!(looker_get(&path));
        out.insert("folders".into(), v);
    }

    json_result(&serde_json::Value::Object(out))
}

// ── Folders ────────────────────────────────────────────────────────────────

pub fn list_folders(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let fields = sdk::arg_str(&args, "fields");
    let path = if fields.is_empty() {
        "/folders".to_string()
    } else {
        format!("/folders?fields={}", query_escape(&fields))
    };
    let v = call!(looker_get(&path));
    json_result(&v)
}

pub fn get_folder(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = sdk::arg_str(&args, "folder_id");
    if id.is_empty() {
        return sdk::err_result("folder_id is required");
    }
    let v = call!(looker_get(&format!("/folders/{}", path_escape(&id))));
    json_result(&v)
}

// ── Looks ──────────────────────────────────────────────────────────────────

pub fn list_looks(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let limit = sdk::arg_int(&args, "limit").unwrap_or(25);
    let offset = sdk::arg_int(&args, "offset").unwrap_or(0);
    let fields = sdk::arg_str(&args, "fields");
    let mut path = format!("/looks?limit={limit}&offset={offset}");
    if !fields.is_empty() {
        path.push_str(&format!("&fields={}", query_escape(&fields)));
    }
    let v = call!(looker_get(&path));
    json_result(&v)
}

pub fn get_look(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = sdk::arg_str(&args, "look_id");
    if id.is_empty() {
        return sdk::err_result("look_id is required");
    }
    let v = call!(looker_get(&format!("/looks/{}", path_escape(&id))));
    json_result(&v)
}

pub fn run_look(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = sdk::arg_str(&args, "look_id");
    if id.is_empty() {
        return sdk::err_result("look_id is required");
    }
    let raw_limit = sdk::arg_int(&args, "limit").unwrap_or(0);
    let apply_fmt = sdk::arg_bool(&args, "apply_formatting").unwrap_or(false);
    let apply_vis = sdk::arg_bool(&args, "apply_vis").unwrap_or(false);
    let cache_arg = args.get("cache");

    let mut q = format!("limit={}", clamp_limit(raw_limit));
    if apply_fmt {
        q.push_str("&apply_formatting=true");
    }
    if apply_vis {
        q.push_str("&apply_vis=true");
    }
    if let Some(v) = cache_arg {
        let on = sdk::arg_bool(&args, "cache").unwrap_or(true);
        if !on && !v.is_null() {
            q.push_str("&cache=false");
        }
    }
    let v = call!(looker_get(&format!(
        "/looks/{}/run/json?{q}",
        path_escape(&id)
    )));
    json_result(&v)
}

// ── Dashboards ─────────────────────────────────────────────────────────────

pub fn list_dashboards(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let limit = sdk::arg_int(&args, "limit").unwrap_or(25);
    let offset = sdk::arg_int(&args, "offset").unwrap_or(0);
    let fields = sdk::arg_str(&args, "fields");
    let mut path = format!("/dashboards?limit={limit}&offset={offset}");
    if !fields.is_empty() {
        path.push_str(&format!("&fields={}", query_escape(&fields)));
    }
    let v = call!(looker_get(&path));
    json_result(&v)
}

pub fn get_dashboard(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = sdk::arg_str(&args, "dashboard_id");
    if id.is_empty() {
        return sdk::err_result("dashboard_id is required");
    }
    let v = call!(looker_get(&format!("/dashboards/{}", path_escape(&id))));
    json_result(&v)
}

// ── Queries ────────────────────────────────────────────────────────────────

/// Build the JSON body shared by `POST /queries` and `POST /queries/run/{format}`.
fn build_query_body(
    args: &HashMap<String, serde_json::Value>,
    limit_override: i64,
) -> Result<serde_json::Value, String> {
    let model = sdk::arg_str(args, "model");
    let view = sdk::arg_str(args, "view");
    let fields = sdk::arg_str(args, "fields");
    if model.is_empty() || view.is_empty() || fields.is_empty() {
        return Err("model, view, and fields are required".into());
    }
    let mut body = serde_json::json!({
        "model": model,
        "view": view,
        "fields": split_csv(&fields),
    });
    let obj = body.as_object_mut().unwrap();
    let sorts = sdk::arg_str(args, "sorts");
    if !sorts.is_empty() {
        obj.insert("sorts".into(), serde_json::json!(split_csv(&sorts)));
    }
    let pivots = sdk::arg_str(args, "pivots");
    if !pivots.is_empty() {
        obj.insert("pivots".into(), serde_json::json!(split_csv(&pivots)));
    }
    let filters = sdk::arg_str(args, "filters");
    if !filters.is_empty() {
        let parsed: serde_json::Value = serde_json::from_str(&filters)
            .map_err(|e| format!("filters must be a JSON object: {e}"))?;
        if !parsed.is_object() {
            return Err("filters must be a JSON object".into());
        }
        obj.insert("filters".into(), parsed);
    }
    if limit_override > 0 {
        obj.insert(
            "limit".into(),
            serde_json::json!(limit_override.to_string()),
        );
    }
    Ok(body)
}

pub fn run_inline_query(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let raw_limit = sdk::arg_int(&args, "limit").unwrap_or(0);
    let limit = clamp_limit(raw_limit);
    let body = match build_query_body(&args, limit) {
        Ok(b) => b,
        Err(e) => return sdk::err_result(&e),
    };
    let v = call!(looker_post("/queries/run/json", &body));
    json_result(&v)
}

pub fn create_query(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let raw_limit = sdk::arg_int(&args, "limit").unwrap_or(500);
    let body = match build_query_body(&args, clamp_limit(raw_limit)) {
        Ok(b) => b,
        Err(e) => return sdk::err_result(&e),
    };
    let v = call!(looker_post("/queries", &body));
    json_result(&v)
}

pub fn get_query(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = sdk::arg_str(&args, "query_id");
    if id.is_empty() {
        return sdk::err_result("query_id is required");
    }
    let v = call!(looker_get(&format!("/queries/{}", path_escape(&id))));
    json_result(&v)
}

pub fn run_query(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = sdk::arg_str(&args, "query_id");
    if id.is_empty() {
        return sdk::err_result("query_id is required");
    }
    let raw_limit = sdk::arg_int(&args, "limit").unwrap_or(0);
    let apply_fmt = sdk::arg_bool(&args, "apply_formatting").unwrap_or(false);

    let mut q = format!("limit={}", clamp_limit(raw_limit));
    if apply_fmt {
        q.push_str("&apply_formatting=true");
    }
    if let Some(v) = args.get("cache") {
        let on = sdk::arg_bool(&args, "cache").unwrap_or(true);
        if !on && !v.is_null() {
            q.push_str("&cache=false");
        }
    }
    let v = call!(looker_get(&format!(
        "/queries/{}/run/json?{q}",
        path_escape(&id)
    )));
    json_result(&v)
}

// ── SQL Runner ─────────────────────────────────────────────────────────────

/// Looker requires the two-step SQL Runner pattern: POST /sql_queries to
/// register a query, then GET /sql_queries/{slug}/run/{format} to run it.
pub fn run_sql_query(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let conn = sdk::arg_str(&args, "connection_name");
    let sql = sdk::arg_str(&args, "sql");
    if conn.is_empty() || sql.is_empty() {
        return sdk::err_result("connection_name and sql are required");
    }
    let body = serde_json::json!({
        "connection_name": conn,
        "sql": sql,
    });
    let created = call!(looker_post("/sql_queries", &body));
    let slug = created
        .get("slug")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if slug.is_empty() {
        return sdk::err_result("sql_query: missing slug in response");
    }
    let v = call!(looker_get(&format!(
        "/sql_queries/{}/run/json",
        path_escape(&slug)
    )));
    json_result(&v)
}

// ── LookML Models ──────────────────────────────────────────────────────────

pub fn list_models(_args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let v = call!(looker_get("/lookml_models"));
    json_result(&v)
}

pub fn get_model(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let name = sdk::arg_str(&args, "model_name");
    if name.is_empty() {
        return sdk::err_result("model_name is required");
    }
    let v = call!(looker_get(&format!(
        "/lookml_models/{}",
        path_escape(&name)
    )));
    json_result(&v)
}

pub fn get_model_explore(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let model = sdk::arg_str(&args, "model_name");
    let explore = sdk::arg_str(&args, "explore_name");
    if model.is_empty() || explore.is_empty() {
        return sdk::err_result("model_name and explore_name are required");
    }
    let v = call!(looker_get(&format!(
        "/lookml_models/{}/explores/{}",
        path_escape(&model),
        path_escape(&explore)
    )));
    json_result(&v)
}

// ── Connections ────────────────────────────────────────────────────────────

pub fn list_connections(_args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let v = call!(looker_get("/connections"));
    json_result(&v)
}

pub fn get_connection(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let name = sdk::arg_str(&args, "connection_name");
    if name.is_empty() {
        return sdk::err_result("connection_name is required");
    }
    let v = call!(looker_get(&format!("/connections/{}", path_escape(&name))));
    json_result(&v)
}

// ── Users ──────────────────────────────────────────────────────────────────

pub fn get_me(_args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let v = call!(looker_get("/user"));
    json_result(&v)
}

pub fn list_users(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let limit = sdk::arg_int(&args, "limit").unwrap_or(25);
    let offset = sdk::arg_int(&args, "offset").unwrap_or(0);
    let v = call!(looker_get(&format!("/users?limit={limit}&offset={offset}")));
    json_result(&v)
}

pub fn get_user(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = sdk::arg_str(&args, "user_id");
    if id.is_empty() {
        return sdk::err_result("user_id is required");
    }
    let v = call!(looker_get(&format!("/users/{}", path_escape(&id))));
    json_result(&v)
}

// ── Scheduled plans ────────────────────────────────────────────────────────

pub fn list_scheduled_plans(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let user_id = sdk::arg_str(&args, "user_id");
    let all = sdk::arg_bool(&args, "all").unwrap_or(false);
    let mut parts: Vec<String> = Vec::new();
    if !user_id.is_empty() {
        parts.push(format!("user_id={}", query_escape(&user_id)));
    }
    if all {
        parts.push("all_users=true".into());
    }
    let path = if parts.is_empty() {
        "/scheduled_plans".to_string()
    } else {
        format!("/scheduled_plans?{}", parts.join("&"))
    };
    let v = call!(looker_get(&path));
    json_result(&v)
}

pub fn get_scheduled_plan(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = sdk::arg_str(&args, "scheduled_plan_id");
    if id.is_empty() {
        return sdk::err_result("scheduled_plan_id is required");
    }
    let v = call!(looker_get(&format!(
        "/scheduled_plans/{}",
        path_escape(&id)
    )));
    json_result(&v)
}

// Suppress unused-import warning when DEFAULT_ROW_LIMIT is only conceptually used.
#[allow(dead_code)]
const _: i64 = DEFAULT_ROW_LIMIT;
