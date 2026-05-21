mod handlers;
mod tools;

use std::collections::HashMap;
use std::sync::Mutex;
use switchboard_guest_sdk as sdk;

#[cfg(test)]
#[no_mangle]
pub extern "C" fn host_http_request(_ptr_size: u64) -> u64 {
    0
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn host_log(_ptr: u32, _size: u32) {}

const API_BASE: &str = "https://api.clerk.com/v1";

struct Config {
    secret_key: String,
}

static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

fn with_config<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&Config) -> R,
{
    let guard = CONFIG.lock().map_err(|e| e.to_string())?;
    match guard.as_ref() {
        Some(c) => Ok(f(c)),
        None => Err("clerk: not configured".into()),
    }
}

#[no_mangle]
pub extern "C" fn name() -> u64 {
    sdk::leaked_string("clerk")
}

#[no_mangle]
pub extern "C" fn metadata() -> u64 {
    sdk::leaked_metadata(&sdk::PluginMetadata {
        name: "clerk".into(),
        version: "0.1.0".into(),
        abi_version: 1,
        description: "Clerk authentication and identity management — users, sessions, organizations, memberships, invitations, and allow/block list identifiers via the Clerk Backend API.".into(),
        author: "daltoniam".into(),
        homepage: "https://github.com/daltoniam/switchboard_plugins".into(),
        license: "MIT".into(),
        capabilities: vec!["http".into()],
        credential_keys: vec!["secret_key".into()],
        plain_text_keys: vec![],
        optional_keys: vec![],
        placeholders: HashMap::from([(
            "secret_key".into(),
            "Clerk secret key (sk_test_... or sk_live_...)".into(),
        )]),
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
        Err(e) => return sdk::leaked_string(&format!("clerk: invalid credentials JSON: {e}")),
    };

    let secret_key = creds.get("secret_key").cloned().unwrap_or_default();
    if secret_key.is_empty() {
        return sdk::leaked_string("clerk: secret_key is required");
    }

    *CONFIG.lock().unwrap() = Some(Config { secret_key });
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
    // JWKS is the cheapest authenticated read; succeeds iff the secret key is valid.
    match clerk_get("/jwks") {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn compact_specs() -> u64 {
    sdk::leaked_compact_specs(&compact_spec_map())
}

type HandlerFn = fn(HashMap<String, serde_json::Value>) -> sdk::ToolResult;

fn dispatch(tool_name: &str, args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let handler: Option<HandlerFn> = match tool_name {
        // Users
        "clerk_list_users" => Some(handlers::list_users),
        "clerk_get_user" => Some(handlers::get_user),
        "clerk_create_user" => Some(handlers::create_user),
        "clerk_update_user" => Some(handlers::update_user),
        "clerk_delete_user" => Some(handlers::delete_user),
        "clerk_ban_user" => Some(handlers::ban_user),
        "clerk_unban_user" => Some(handlers::unban_user),
        "clerk_lock_user" => Some(handlers::lock_user),
        "clerk_unlock_user" => Some(handlers::unlock_user),
        "clerk_list_user_organization_memberships" => {
            Some(handlers::list_user_organization_memberships)
        }
        // Sessions
        "clerk_list_sessions" => Some(handlers::list_sessions),
        "clerk_get_session" => Some(handlers::get_session),
        "clerk_revoke_session" => Some(handlers::revoke_session),
        // Organizations
        "clerk_list_organizations" => Some(handlers::list_organizations),
        "clerk_get_organization" => Some(handlers::get_organization),
        "clerk_create_organization" => Some(handlers::create_organization),
        "clerk_update_organization" => Some(handlers::update_organization),
        "clerk_delete_organization" => Some(handlers::delete_organization),
        "clerk_list_organization_memberships" => Some(handlers::list_organization_memberships),
        "clerk_create_organization_membership" => Some(handlers::create_organization_membership),
        "clerk_update_organization_membership" => Some(handlers::update_organization_membership),
        "clerk_delete_organization_membership" => Some(handlers::delete_organization_membership),
        "clerk_list_organization_invitations" => Some(handlers::list_organization_invitations),
        "clerk_create_organization_invitation" => Some(handlers::create_organization_invitation),
        "clerk_revoke_organization_invitation" => Some(handlers::revoke_organization_invitation),
        // Invitations
        "clerk_list_invitations" => Some(handlers::list_invitations),
        "clerk_create_invitation" => Some(handlers::create_invitation),
        "clerk_revoke_invitation" => Some(handlers::revoke_invitation),
        // Allow/Block list
        "clerk_list_allowlist_identifiers" => Some(handlers::list_allowlist_identifiers),
        "clerk_create_allowlist_identifier" => Some(handlers::create_allowlist_identifier),
        "clerk_delete_allowlist_identifier" => Some(handlers::delete_allowlist_identifier),
        "clerk_list_blocklist_identifiers" => Some(handlers::list_blocklist_identifiers),
        "clerk_create_blocklist_identifier" => Some(handlers::create_blocklist_identifier),
        "clerk_delete_blocklist_identifier" => Some(handlers::delete_blocklist_identifier),
        _ => None,
    };

    match handler {
        Some(f) => f(args),
        None => sdk::err_result(&format!("unknown tool: {tool_name}")),
    }
}

pub(crate) fn clerk_get(path: &str) -> Result<serde_json::Value, String> {
    do_request("GET", path, None)
}

pub(crate) fn clerk_post(
    path: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    do_request("POST", path, Some(body))
}

pub(crate) fn clerk_patch(
    path: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    do_request("PATCH", path, Some(body))
}

pub(crate) fn clerk_delete(path: &str) -> Result<serde_json::Value, String> {
    do_request("DELETE", path, None)
}

fn do_request(
    method: &str,
    path: &str,
    body: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let secret_key = with_config(|c| c.secret_key.clone())?;

    let mut headers = HashMap::new();
    headers.insert("Authorization".into(), format!("Bearer {secret_key}"));
    headers.insert("Accept".into(), "application/json".into());

    let body_str = if let Some(b) = body {
        headers.insert("Content-Type".into(), "application/json".into());
        serde_json::to_string(b).map_err(|e| format!("clerk: encode request: {e}"))?
    } else {
        String::new()
    };

    let req = sdk::HttpRequest {
        method: method.into(),
        url: format!("{API_BASE}{path}"),
        headers,
        body: body_str,
        ..Default::default()
    };

    let resp = sdk::host_http_request(&req)?;
    if resp.status >= 400 {
        return Err(format!("clerk API error ({}): {}", resp.status, resp.body));
    }
    if resp.status == 204 || resp.body.is_empty() {
        return Ok(serde_json::json!({"status": "success"}));
    }
    serde_json::from_str(&resp.body).map_err(|e| format!("clerk: decode response: {e}"))
}

pub(crate) fn path_escape(s: &str) -> String {
    encode_component(s)
}

pub(crate) fn query_escape(s: &str) -> String {
    encode_component(s)
}

fn encode_component(s: &str) -> String {
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

fn compact_spec_map() -> HashMap<String, Vec<String>> {
    let mut s: HashMap<String, Vec<String>> = HashMap::new();

    // Users list — Clerk returns either a bare array or {data, total_count}.
    // Spec both shapes so compaction always finds a match.
    s.insert(
        "clerk_list_users".into(),
        vec![
            "total_count".into(),
            "data[].id".into(),
            "data[].username".into(),
            "data[].first_name".into(),
            "data[].last_name".into(),
            "data[].primary_email_address_id".into(),
            "data[].primary_phone_number_id".into(),
            "data[].email_addresses[].id".into(),
            "data[].email_addresses[].email_address".into(),
            "data[].email_addresses[].verification.status".into(),
            "data[].phone_numbers[].id".into(),
            "data[].phone_numbers[].phone_number".into(),
            "data[].banned".into(),
            "data[].locked".into(),
            "data[].created_at".into(),
            "data[].updated_at".into(),
            "data[].last_sign_in_at".into(),
            "data[].last_active_at".into(),
            // Fallback for bare-array responses
            "[].id".into(),
            "[].username".into(),
            "[].first_name".into(),
            "[].last_name".into(),
            "[].primary_email_address_id".into(),
            "[].email_addresses[].email_address".into(),
            "[].phone_numbers[].phone_number".into(),
            "[].banned".into(),
            "[].locked".into(),
            "[].created_at".into(),
            "[].last_sign_in_at".into(),
            "[].last_active_at".into(),
        ],
    );
    s.insert(
        "clerk_list_user_organization_memberships".into(),
        vec![
            "total_count".into(),
            "data[].id".into(),
            "data[].role".into(),
            "data[].role_name".into(),
            "data[].created_at".into(),
            "data[].organization.id".into(),
            "data[].organization.name".into(),
            "data[].organization.slug".into(),
            "data[].organization.members_count".into(),
        ],
    );
    s.insert(
        "clerk_list_sessions".into(),
        vec![
            "[].id".into(),
            "[].user_id".into(),
            "[].client_id".into(),
            "[].status".into(),
            "[].last_active_at".into(),
            "[].expire_at".into(),
            "[].abandon_at".into(),
            "[].created_at".into(),
            "[].latest_activity.country".into(),
            "[].latest_activity.city".into(),
            "[].latest_activity.is_mobile".into(),
            "[].latest_activity.browser_name".into(),
            "[].latest_activity.device_type".into(),
        ],
    );
    s.insert(
        "clerk_list_organizations".into(),
        vec![
            "total_count".into(),
            "data[].id".into(),
            "data[].name".into(),
            "data[].slug".into(),
            "data[].members_count".into(),
            "data[].max_allowed_memberships".into(),
            "data[].created_by".into(),
            "data[].created_at".into(),
            "data[].updated_at".into(),
        ],
    );
    s.insert(
        "clerk_list_organization_memberships".into(),
        vec![
            "total_count".into(),
            "data[].id".into(),
            "data[].role".into(),
            "data[].role_name".into(),
            "data[].created_at".into(),
            "data[].public_user_data.user_id".into(),
            "data[].public_user_data.identifier".into(),
            "data[].public_user_data.first_name".into(),
            "data[].public_user_data.last_name".into(),
            "data[].organization.id".into(),
            "data[].organization.slug".into(),
        ],
    );
    s.insert(
        "clerk_list_organization_invitations".into(),
        vec![
            "total_count".into(),
            "data[].id".into(),
            "data[].email_address".into(),
            "data[].role".into(),
            "data[].role_name".into(),
            "data[].status".into(),
            "data[].organization_id".into(),
            "data[].created_at".into(),
            "data[].updated_at".into(),
        ],
    );
    s.insert(
        "clerk_list_invitations".into(),
        vec![
            "[].id".into(),
            "[].email_address".into(),
            "[].status".into(),
            "[].revoked".into(),
            "[].created_at".into(),
            "[].updated_at".into(),
            "[].expires_at".into(),
            "[].url".into(),
        ],
    );
    s.insert(
        "clerk_list_allowlist_identifiers".into(),
        vec![
            "[].id".into(),
            "[].identifier".into(),
            "[].identifier_type".into(),
            "[].instance_id".into(),
            "[].created_at".into(),
            "[].updated_at".into(),
        ],
    );
    s.insert(
        "clerk_list_blocklist_identifiers".into(),
        vec![
            "[].id".into(),
            "[].identifier".into(),
            "[].identifier_type".into(),
            "[].instance_id".into(),
            "[].created_at".into(),
            "[].updated_at".into(),
        ],
    );

    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn tool_definitions_have_required_metadata() {
        let defs = tools::tool_definitions();
        assert_eq!(defs.len(), 34);

        let mut seen = HashSet::new();
        for def in defs {
            assert!(def.name.starts_with("clerk_"), "{}", def.name);
            assert!(!def.description.is_empty(), "{}", def.name);
            assert!(seen.insert(def.name));
        }
    }

    #[test]
    fn entry_point_tools_have_start_here_guidance() {
        for def in tools::tool_definitions() {
            if matches!(
                def.name.as_str(),
                "clerk_list_users"
                    | "clerk_list_sessions"
                    | "clerk_list_organizations"
                    | "clerk_list_invitations"
                    | "clerk_list_allowlist_identifiers"
                    | "clerk_list_blocklist_identifiers"
            ) {
                assert!(def.description.contains("Start here"), "{}", def.name);
            }
        }
    }

    #[test]
    fn compact_specs_reference_known_tools() {
        let defs = tools::tool_definitions()
            .into_iter()
            .map(|def| def.name)
            .collect::<HashSet<_>>();
        for name in compact_spec_map().keys() {
            assert!(defs.contains(name), "{name}");
        }
    }

    #[test]
    fn unknown_tool_returns_error_result() {
        let result = dispatch("clerk_missing_tool", HashMap::new());
        assert!(result.is_error);
        assert!(result.data.contains("unknown tool"));
    }

    #[test]
    fn dispatch_covers_every_tool() {
        // No host_http_request available in unit tests, so handlers will return an
        // error, but it must NOT be the "unknown tool" error from the dispatch
        // fallthrough. Any other error proves the tool name routes to a handler.
        for def in tools::tool_definitions() {
            let result = dispatch(&def.name, HashMap::new());
            assert!(
                !result.data.contains("unknown tool"),
                "tool {} is not in dispatch",
                def.name
            );
        }
    }

    #[test]
    fn escapes_path_and_query_components() {
        assert_eq!(path_escape("user_2abc/def+ghi"), "user_2abc%2Fdef%2Bghi");
        assert_eq!(query_escape("dale@example.com"), "dale%40example.com");
    }
}
