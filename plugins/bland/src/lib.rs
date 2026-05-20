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

const API_BASE: &str = "https://api.bland.ai/v1";

struct Config {
    api_key: String,
    org_id: String,
}

static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

fn with_config<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&Config) -> R,
{
    let guard = CONFIG.lock().map_err(|e| e.to_string())?;
    match guard.as_ref() {
        Some(c) => Ok(f(c)),
        None => Err("bland: not configured".into()),
    }
}

#[no_mangle]
pub extern "C" fn name() -> u64 {
    sdk::leaked_string("bland")
}

#[no_mangle]
pub extern "C" fn metadata() -> u64 {
    sdk::leaked_metadata(&sdk::PluginMetadata {
        name: "bland".into(),
        version: "0.1.0".into(),
        abi_version: 1,
        description: "Bland.ai voice AI integration for calls, transcripts, agents, voices, pathways, numbers, and knowledge bases.".into(),
        author: "daltoniam".into(),
        homepage: "https://github.com/daltoniam/switchboard_plugins".into(),
        license: "MIT".into(),
        capabilities: vec!["http".into()],
        credential_keys: vec!["api_key".into(), "org_id".into()],
        plain_text_keys: vec!["org_id".into()],
        optional_keys: vec!["org_id".into()],
        placeholders: HashMap::from([
            ("api_key".into(), "Bland.ai API key".into()),
            ("org_id".into(), "Optional Bland organization ID".into()),
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
        Err(e) => return sdk::leaked_string(&format!("bland: invalid credentials JSON: {e}")),
    };

    let api_key = creds.get("api_key").cloned().unwrap_or_default();
    if api_key.is_empty() {
        return sdk::leaked_string("bland: api_key is required");
    }

    *CONFIG.lock().unwrap() = Some(Config {
        api_key,
        org_id: creds.get("org_id").cloned().unwrap_or_default(),
    });
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
    match bland_get("/voices") {
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
        "bland_list_calls" => Some(handlers::list_calls),
        "bland_list_active_calls" => Some(handlers::list_active_calls),
        "bland_get_call" => Some(handlers::get_call),
        "bland_send_call" => Some(handlers::send_call),
        "bland_stop_call" => Some(handlers::stop_call),
        "bland_analyze_call" => Some(handlers::analyze_call),
        "bland_list_voices" => Some(handlers::list_voices),
        "bland_get_voice" => Some(handlers::get_voice),
        "bland_list_pathways" => Some(handlers::list_pathways),
        "bland_get_pathway" => Some(handlers::get_pathway),
        "bland_list_numbers" => Some(handlers::list_numbers),
        "bland_get_number" => Some(handlers::get_number),
        "bland_list_knowledge_bases" => Some(handlers::list_knowledge_bases),
        "bland_get_knowledge_base" => Some(handlers::get_knowledge_base),
        "bland_get_me" => Some(handlers::get_me),
        "bland_create_org" => Some(handlers::create_org),
        "bland_get_org" => Some(handlers::get_org),
        "bland_delete_org" => Some(handlers::delete_org),
        "bland_update_org_properties" => Some(handlers::update_org_properties),
        "bland_list_org_members" => Some(handlers::list_org_members),
        "bland_update_org_members" => Some(handlers::update_org_members),
        "bland_update_org_member_permissions" => Some(handlers::update_org_member_permissions),
        "bland_list_my_org_memberships" => Some(handlers::list_my_org_memberships),
        "bland_leave_org" => Some(handlers::leave_org),
        "bland_get_org_billing" => Some(handlers::get_org_billing),
        "bland_get_org_billing_refill" => Some(handlers::get_org_billing_refill),
        "bland_get_org_current_version" => Some(handlers::get_org_current_version),
        "bland_list_org_versions" => Some(handlers::list_org_versions),
        "bland_update_org_version" => Some(handlers::update_org_version),
        "bland_list_audit_logs" => Some(handlers::list_audit_logs),
        _ => None,
    };

    match handler {
        Some(f) => f(args),
        None => sdk::err_result(&format!("unknown tool: {tool_name}")),
    }
}

pub(crate) fn bland_get(path: &str) -> Result<serde_json::Value, String> {
    do_request("GET", path, None)
}

pub(crate) fn bland_post(
    path: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    do_request("POST", path, Some(body))
}

pub(crate) fn bland_patch(
    path: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    do_request("PATCH", path, Some(body))
}

pub(crate) fn bland_delete(
    path: &str,
    body: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    do_request("DELETE", path, body)
}

fn do_request(
    method: &str,
    path: &str,
    body: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let (api_key, org_id) = with_config(|c| (c.api_key.clone(), c.org_id.clone()))?;

    let mut headers = HashMap::new();
    headers.insert("Authorization".into(), api_key);
    headers.insert("Accept".into(), "application/json".into());
    if !org_id.is_empty() {
        headers.insert("x-bland-org-id".into(), org_id);
    }

    let body_str = if let Some(b) = body {
        headers.insert("Content-Type".into(), "application/json".into());
        serde_json::to_string(b).map_err(|e| format!("bland: encode request: {e}"))?
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
        return Err(format!("bland API error ({}): {}", resp.status, resp.body));
    }
    if resp.status == 204 || resp.body.is_empty() {
        return Ok(serde_json::json!({"status": "success"}));
    }
    serde_json::from_str(&resp.body).map_err(|e| format!("bland: decode response: {e}"))
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

    s.insert(
        "bland_list_calls".into(),
        vec![
            "count".into(),
            "total_count".into(),
            "calls[].call_id".into(),
            "calls[].created_at".into(),
            "calls[].to".into(),
            "calls[].from".into(),
            "calls[].call_length".into(),
            "calls[].completed".into(),
            "calls[].queue_status".into(),
            "calls[].status".into(),
            "calls[].error_message".into(),
            "calls[].answered_by".into(),
            "calls[].inbound".into(),
            "calls[].batch_id".into(),
        ],
    );
    s.insert(
        "bland_list_active_calls".into(),
        vec![
            "calls[].call_id".into(),
            "calls[].phone_number".into(),
            "calls[].to".into(),
            "calls[].from".into(),
            "calls[].status".into(),
            "calls[].created_at".into(),
            "calls[].started_at".into(),
        ],
    );
    s.insert(
        "bland_get_call".into(),
        vec![
            "call_id".into(),
            "created_at".into(),
            "to".into(),
            "from".into(),
            "call_length".into(),
            "status".into(),
            "completed".into(),
            "answered_by".into(),
            "summary".into(),
            "concatenated_transcript".into(),
            "transcripts[].created_at".into(),
            "transcripts[].user".into(),
            "transcripts[].text".into(),
            "analysis".into(),
            "variables".into(),
            "metadata".into(),
        ],
    );
    s.insert(
        "bland_list_voices".into(),
        vec![
            "voices[].id".into(),
            "voices[].name".into(),
            "voices[].description".into(),
            "voices[].public".into(),
            "voices[].tags".into(),
            "voices[].total_ratings".into(),
            "voices[].average_rating".into(),
        ],
    );
    s.insert(
        "bland_get_voice".into(),
        vec![
            "id".into(),
            "name".into(),
            "description".into(),
            "public".into(),
            "tags".into(),
            "total_ratings".into(),
            "average_rating".into(),
            "language".into(),
        ],
    );
    s.insert(
        "bland_list_pathways".into(),
        vec![
            "pathways[].id".into(),
            "pathways[].pathway_id".into(),
            "pathways[].name".into(),
            "pathways[].description".into(),
            "pathways[].created_at".into(),
            "pathways[].updated_at".into(),
        ],
    );
    s.insert(
        "bland_get_pathway".into(),
        vec![
            "id".into(),
            "pathway_id".into(),
            "name".into(),
            "description".into(),
            "created_at".into(),
            "updated_at".into(),
            "nodes".into(),
            "edges".into(),
        ],
    );
    s.insert(
        "bland_list_numbers".into(),
        vec![
            "numbers[].phone_number".into(),
            "numbers[].label".into(),
            "numbers[].inbound_agent".into(),
            "numbers[].created_at".into(),
        ],
    );
    s.insert(
        "bland_get_number".into(),
        vec![
            "phone_number".into(),
            "label".into(),
            "inbound_agent".into(),
            "webhook".into(),
            "created_at".into(),
        ],
    );
    s.insert(
        "bland_list_knowledge_bases".into(),
        vec![
            "knowledge_bases[].id".into(),
            "knowledge_bases[].name".into(),
            "knowledge_bases[].description".into(),
            "knowledge_bases[].created_at".into(),
            "knowledge_bases[].updated_at".into(),
            "knowledge_bases[].status".into(),
        ],
    );
    s.insert(
        "bland_get_knowledge_base".into(),
        vec![
            "id".into(),
            "name".into(),
            "description".into(),
            "created_at".into(),
            "updated_at".into(),
            "status".into(),
            "documents".into(),
        ],
    );
    s.insert(
        "bland_get_me".into(),
        vec![
            "status".into(),
            "billing.current_balance".into(),
            "billing.refill_to".into(),
            "total_calls".into(),
        ],
    );
    s.insert(
        "bland_get_org".into(),
        vec![
            "data.id".into(),
            "data.org_slug".into(),
            "data.org_display_name".into(),
            "data.org_plan".into(),
            "data.org_creation_date".into(),
            "data.kyc_level".into(),
            "data.is_deleted".into(),
            "data.is_stripe_overdue".into(),
            "data.is_suspended".into(),
            "data.org_rate_limit".into(),
            "data.org_type".into(),
            "data.entitlements".into(),
            "data.preferences".into(),
        ],
    );
    s.insert(
        "bland_list_org_members".into(),
        vec![
            "data[].org_id".into(),
            "data[].user_id".into(),
            "data[].permissions".into(),
            "data[].is_owner".into(),
            "data[].is_org_creator".into(),
            "data[].joined_at".into(),
            "data[].first_name".into(),
            "data[].last_name".into(),
            "data[].member_email".into(),
            "data[].member_phone_number".into(),
        ],
    );
    s.insert(
        "bland_update_org_member_permissions".into(),
        vec!["data.newPermissions".into(), "errors".into()],
    );
    s.insert(
        "bland_list_my_org_memberships".into(),
        vec![
            "data[].org_id".into(),
            "data[].org_slug".into(),
            "data[].org_display_name".into(),
            "data[].permissions".into(),
            "data[].is_owner".into(),
            "data[].is_org_creator".into(),
            "data[].joined_at".into(),
        ],
    );
    s.insert(
        "bland_get_org_billing".into(),
        vec![
            "data.current_balance".into(),
            "data.refill_amount".into(),
            "data.refill_at".into(),
            "errors".into(),
        ],
    );
    s.insert(
        "bland_get_org_billing_refill".into(),
        vec!["data".into(), "errors".into()],
    );
    s.insert(
        "bland_get_org_current_version".into(),
        vec!["data.version".into(), "errors".into()],
    );
    s.insert(
        "bland_list_org_versions".into(),
        vec![
            "data.versions[].id".into(),
            "data.versions[].friendly_name".into(),
            "data.versions[].created_at".into(),
            "data.versions[].git_sha".into(),
            "data.versions[].tags".into(),
            "data.versions[].currently_supported".into(),
            "data.versions[].recommended_upgrade_to".into(),
            "data.versions[].service".into(),
            "data.versions[].placement_group".into(),
        ],
    );
    s.insert(
        "bland_list_audit_logs".into(),
        vec![
            "data.events[].id".into(),
            "data.events[].org_id".into(),
            "data.events[].actor_id".into(),
            "data.events[].event_type".into(),
            "data.events[].resource_type".into(),
            "data.events[].resource_id".into(),
            "data.events[].metadata".into(),
            "data.events[].created_at".into(),
            "data.total".into(),
            "data.total_pages".into(),
            "data.current_page".into(),
            "data.page_size".into(),
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
        assert_eq!(defs.len(), 30);

        let mut seen = HashSet::new();
        for def in defs {
            assert!(def.name.starts_with("bland_"));
            assert!(!def.description.is_empty());
            assert!(seen.insert(def.name));
        }
    }

    #[test]
    fn entry_point_tools_have_start_here_guidance() {
        for def in tools::tool_definitions() {
            if matches!(
                def.name.as_str(),
                "bland_list_calls"
                    | "bland_list_voices"
                    | "bland_list_pathways"
                    | "bland_list_numbers"
                    | "bland_list_knowledge_bases"
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
        let result = dispatch("bland_missing_tool", HashMap::new());
        assert!(result.is_error);
        assert!(result.data.contains("unknown tool"));
    }

    #[test]
    fn escapes_path_and_query_components() {
        assert_eq!(path_escape("+1555 123/abc"), "%2B1555%20123%2Fabc");
        assert_eq!(query_escape("created_at desc"), "created_at%20desc");
    }
}
