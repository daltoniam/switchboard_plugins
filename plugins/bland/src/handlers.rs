use std::collections::HashMap;
use switchboard_guest_sdk as sdk;

use crate::{bland_delete, bland_get, bland_patch, bland_post, path_escape, query_escape};

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

fn required(
    args: &HashMap<String, serde_json::Value>,
    key: &str,
) -> Result<String, sdk::ToolResult> {
    let value = sdk::arg_str(args, key);
    if value.is_empty() {
        Err(sdk::err_result(&format!("{key} is required")))
    } else {
        Ok(value)
    }
}

fn add_str_param(
    args: &HashMap<String, serde_json::Value>,
    params: &mut Vec<(String, String)>,
    arg: &str,
    key: &str,
) {
    let value = sdk::arg_str(args, arg);
    if !value.is_empty() {
        params.push((key.to_string(), value));
    }
}

fn add_int_param(
    args: &HashMap<String, serde_json::Value>,
    params: &mut Vec<(String, String)>,
    arg: &str,
    key: &str,
) {
    if let Some(value) = sdk::arg_int(args, arg) {
        params.push((key.to_string(), value.to_string()));
    }
}

fn add_bool_param(
    args: &HashMap<String, serde_json::Value>,
    params: &mut Vec<(String, String)>,
    arg: &str,
    key: &str,
) {
    if let Some(value) = sdk::arg_bool(args, arg) {
        params.push((key.to_string(), value.to_string()));
    }
}

fn append_query(path: &str, params: &[(String, String)]) -> String {
    if params.is_empty() {
        return path.to_string();
    }
    let query = params
        .iter()
        .map(|(key, value)| format!("{}={}", query_escape(key), query_escape(value)))
        .collect::<Vec<_>>()
        .join("&");
    format!("{path}?{query}")
}

fn insert_string_body(
    args: &HashMap<String, serde_json::Value>,
    body: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
) {
    let value = sdk::arg_str(args, key);
    if !value.is_empty() {
        body.insert(key.to_string(), serde_json::json!(value));
    }
}

fn insert_i64_body(
    args: &HashMap<String, serde_json::Value>,
    body: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
) {
    if let Some(value) = sdk::arg_int(args, key) {
        body.insert(key.to_string(), serde_json::json!(value));
    }
}

fn insert_f64_body(
    args: &HashMap<String, serde_json::Value>,
    body: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<(), String> {
    match args.get(key) {
        Some(serde_json::Value::Number(n)) => {
            if let Some(value) = n.as_f64() {
                body.insert(key.to_string(), serde_json::json!(value));
            }
        }
        Some(serde_json::Value::String(s)) if !s.is_empty() => {
            let value = s
                .parse::<f64>()
                .map_err(|e| format!("{key} must be a number: {e}"))?;
            body.insert(key.to_string(), serde_json::json!(value));
        }
        _ => {}
    }
    Ok(())
}

fn insert_bool_body(
    args: &HashMap<String, serde_json::Value>,
    body: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
) {
    if let Some(value) = sdk::arg_bool(args, key) {
        body.insert(key.to_string(), serde_json::json!(value));
    }
}

fn insert_json_body(
    args: &HashMap<String, serde_json::Value>,
    body: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<(), String> {
    match args.get(key) {
        Some(serde_json::Value::String(s)) if !s.is_empty() => {
            let parsed: serde_json::Value =
                serde_json::from_str(s).map_err(|e| format!("{key} must be valid JSON: {e}"))?;
            body.insert(key.to_string(), parsed);
        }
        Some(v) if !v.is_null() => {
            body.insert(key.to_string(), v.clone());
        }
        _ => {}
    }
    Ok(())
}

pub fn list_calls(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let mut params: Vec<(String, String)> = vec![(
        "limit".into(),
        sdk::arg_int(&args, "limit")
            .unwrap_or(20)
            .clamp(1, 100)
            .to_string(),
    )];
    add_int_param(&args, &mut params, "from", "from");
    add_int_param(&args, &mut params, "to", "to");
    add_bool_param(&args, &mut params, "ascending", "ascending");
    add_str_param(&args, &mut params, "sort_by", "sort_by");
    add_str_param(&args, &mut params, "start_date", "start_date");
    add_str_param(&args, &mut params, "end_date", "end_date");
    add_str_param(&args, &mut params, "batch_id", "batch_id");
    add_str_param(&args, &mut params, "answered_by", "answered_by");
    add_bool_param(&args, &mut params, "inbound", "inbound");
    add_bool_param(&args, &mut params, "completed", "completed");
    add_str_param(&args, &mut params, "from_number", "from_number");
    add_str_param(&args, &mut params, "to_number", "to_number");
    add_int_param(&args, &mut params, "duration_gt", "duration_gt");
    add_int_param(&args, &mut params, "duration_lt", "duration_lt");

    let v = call!(bland_get(&append_query("/calls", &params)));
    json_result(&v)
}

pub fn list_active_calls(_args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let v = call!(bland_get("/calls/active"));
    json_result(&v)
}

pub fn get_call(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "call_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(bland_get(&format!("/calls/{}", path_escape(&id))));
    json_result(&v)
}

pub fn send_call(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let phone_number = match required(&args, "phone_number") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let task = sdk::arg_str(&args, "task");
    let pathway_id = sdk::arg_str(&args, "pathway_id");
    if task.is_empty() && pathway_id.is_empty() {
        return sdk::err_result("task or pathway_id is required");
    }

    let mut body = serde_json::Map::new();
    body.insert("phone_number".into(), serde_json::json!(phone_number));
    if !task.is_empty() {
        body.insert("task".into(), serde_json::json!(task));
    }
    if !pathway_id.is_empty() {
        body.insert("pathway_id".into(), serde_json::json!(pathway_id));
    }

    for key in [
        "voice",
        "first_sentence",
        "model",
        "language",
        "from",
        "webhook",
        "transfer_phone_number",
        "timezone",
        "background_track",
    ] {
        insert_string_body(&args, &mut body, key);
    }
    for key in ["max_duration", "interruption_threshold"] {
        insert_i64_body(&args, &mut body, key);
    }
    for key in ["wait_for_greeting", "record"] {
        insert_bool_body(&args, &mut body, key);
    }
    if let Err(e) = insert_f64_body(&args, &mut body, "temperature") {
        return sdk::err_result(&e);
    }
    for key in ["metadata", "dynamic_data", "tools"] {
        if let Err(e) = insert_json_body(&args, &mut body, key) {
            return sdk::err_result(&e);
        }
    }

    let v = call!(bland_post("/calls", &serde_json::Value::Object(body)));
    json_result(&v)
}

pub fn stop_call(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "call_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(bland_post(
        &format!("/calls/{}/stop", path_escape(&id)),
        &serde_json::json!({})
    ));
    json_result(&v)
}

pub fn analyze_call(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "call_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let goal = match required(&args, "goal") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let questions = match args.get("questions") {
        Some(serde_json::Value::String(s)) if !s.is_empty() => {
            match serde_json::from_str::<serde_json::Value>(s) {
                Ok(v) => v,
                Err(e) => return sdk::err_result(&format!("questions must be valid JSON: {e}")),
            }
        }
        Some(v) if !v.is_null() => v.clone(),
        _ => return sdk::err_result("questions is required"),
    };
    if !questions.is_array() {
        return sdk::err_result("questions must be a JSON array");
    }

    let body = serde_json::json!({
        "goal": goal,
        "questions": questions,
    });
    let v = call!(bland_post(
        &format!("/calls/{}/analyze", path_escape(&id)),
        &body
    ));
    json_result(&v)
}

pub fn list_voices(_args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let v = call!(bland_get("/voices"));
    json_result(&v)
}

pub fn get_voice(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "voice_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(bland_get(&format!("/voices/{}", path_escape(&id))));
    json_result(&v)
}

pub fn list_pathways(_args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let v = call!(bland_get("/pathway"));
    json_result(&v)
}

pub fn get_pathway(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "pathway_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(bland_get(&format!("/pathway/{}", path_escape(&id))));
    json_result(&v)
}

pub fn list_numbers(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let mut params: Vec<(String, String)> = Vec::new();
    add_str_param(&args, &mut params, "encrypted_key", "encrypted_key");
    let v = call!(bland_get(&append_query("/inbound", &params)));
    json_result(&v)
}

pub fn get_number(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let phone_number = match required(&args, "phone_number") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(bland_get(&format!(
        "/inbound/{}",
        path_escape(&phone_number)
    )));
    json_result(&v)
}

pub fn list_knowledge_bases(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let mut params: Vec<(String, String)> = vec![
        (
            "page".into(),
            sdk::arg_int(&args, "page").unwrap_or(1).max(1).to_string(),
        ),
        (
            "limit".into(),
            sdk::arg_int(&args, "limit")
                .unwrap_or(20)
                .clamp(1, 100)
                .to_string(),
        ),
    ];
    add_str_param(&args, &mut params, "status", "status");
    let v = call!(bland_get(&append_query("/knowledge", &params)));
    json_result(&v)
}

pub fn get_knowledge_base(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "knowledge_base_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(bland_get(&format!("/knowledge/{}", path_escape(&id))));
    json_result(&v)
}

pub fn get_me(_args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let v = call!(bland_get("/me"));
    json_result(&v)
}

pub fn create_org(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let name = match required(&args, "name") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let body = serde_json::json!({ "name": name });
    let v = call!(bland_post("/orgs/create", &body));
    json_result(&v)
}

pub fn get_org(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let org_id = match required(&args, "org_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(bland_get(&format!("/orgs/{}", path_escape(&org_id))));
    json_result(&v)
}

pub fn delete_org(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let org_id = match required(&args, "org_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let delete_confirm = match required(&args, "delete_confirm") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let body = serde_json::json!({ "delete_confirm": delete_confirm });
    let v = call!(bland_delete(
        &format!("/orgs/{}", path_escape(&org_id)),
        Some(&body)
    ));
    json_result(&v)
}

pub fn update_org_properties(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let org_id = match required(&args, "org_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let mut updates = serde_json::Map::new();
    insert_string_body(&args, &mut updates, "org_display_name");

    let mut preferences = serde_json::Map::new();
    insert_bool_body(&args, &mut preferences, "use_bland_url");
    insert_i64_body(&args, &mut preferences, "recording_lifespan_days");
    if !preferences.is_empty() {
        updates.insert("preferences".into(), serde_json::Value::Object(preferences));
    }
    if let Err(e) = insert_json_body(&args, &mut updates, "preferences") {
        return sdk::err_result(&e);
    }
    if updates.is_empty() {
        return sdk::err_result("at least one update field is required");
    }

    let body = serde_json::json!({ "updates": updates });
    let v = call!(bland_patch(
        &format!("/orgs/{}/properties", path_escape(&org_id)),
        &body
    ));
    json_result(&v)
}

pub fn list_org_members(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let org_id = match required(&args, "org_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(bland_get(&format!(
        "/orgs/{}/members",
        path_escape(&org_id)
    )));
    json_result(&v)
}

pub fn update_org_members(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let org_id = match required(&args, "org_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let action = match required(&args, "action") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let target = match required(&args, "target") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let mut body = serde_json::Map::new();
    body.insert("action".into(), serde_json::json!(action));
    body.insert("target".into(), serde_json::json!(target));
    if let Err(e) = insert_json_body(&args, &mut body, "permissions") {
        return sdk::err_result(&e);
    }
    insert_bool_body(&args, &mut body, "is_invite");
    let v = call!(bland_patch(
        &format!("/orgs/{}/members", path_escape(&org_id)),
        &serde_json::Value::Object(body)
    ));
    json_result(&v)
}

pub fn update_org_member_permissions(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let org_id = match required(&args, "org_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let action = match required(&args, "action") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let target = match required(&args, "target") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let mut body = serde_json::Map::new();
    body.insert("action".into(), serde_json::json!(action));
    body.insert("target".into(), serde_json::json!(target));
    if let Err(e) = insert_json_body(&args, &mut body, "permissions") {
        return sdk::err_result(&e);
    }
    if !body.contains_key("permissions") {
        return sdk::err_result("permissions is required");
    }
    let v = call!(bland_patch(
        &format!("/orgs/{}/members/permissions", path_escape(&org_id)),
        &serde_json::Value::Object(body),
    ));
    json_result(&v)
}

pub fn list_my_org_memberships(_args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let v = call!(bland_get("/orgs/self/memberships"));
    json_result(&v)
}

pub fn leave_org(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let org_id = match required(&args, "org_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let body = serde_json::json!({ "org_id": org_id });
    let v = call!(bland_delete("/orgs/self/leave", Some(&body)));
    json_result(&v)
}

pub fn get_org_billing(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let org_id = match required(&args, "org_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(bland_get(&format!(
        "/orgs/{}/billing",
        path_escape(&org_id)
    )));
    json_result(&v)
}

pub fn get_org_billing_refill(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let org_id = match required(&args, "org_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(bland_get(&format!(
        "/orgs/{}/billing/refill",
        path_escape(&org_id)
    )));
    json_result(&v)
}

fn org_service_path(
    args: &HashMap<String, serde_json::Value>,
    suffix: &str,
) -> Result<String, sdk::ToolResult> {
    let org_id = required(args, "org_id")?;
    let service = sdk::arg_str(args, "service");
    let service = if service.is_empty() {
        "api_server".to_string()
    } else {
        service
    };
    Ok(format!(
        "/orgs/{}/versions/{}{}",
        path_escape(&org_id),
        path_escape(&service),
        suffix
    ))
}

pub fn get_org_current_version(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let path = match org_service_path(&args, "/current") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(bland_get(&path));
    json_result(&v)
}

pub fn list_org_versions(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let path = match org_service_path(&args, "/list") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(bland_get(&path));
    json_result(&v)
}

pub fn update_org_version(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let path = match org_service_path(&args, "") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let version = match required(&args, "version") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let body = serde_json::json!({ "version": version });
    let v = call!(bland_patch(&path, &body));
    json_result(&v)
}

pub fn list_audit_logs(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let mut params: Vec<(String, String)> = vec![
        (
            "page".into(),
            sdk::arg_int(&args, "page").unwrap_or(1).max(1).to_string(),
        ),
        (
            "page_size".into(),
            sdk::arg_int(&args, "page_size")
                .unwrap_or(50)
                .clamp(1, 100)
                .to_string(),
        ),
    ];
    add_str_param(&args, &mut params, "event_type", "event_type");
    add_str_param(&args, &mut params, "actor_id", "actor_id");
    add_str_param(&args, &mut params, "created_after", "created_after");
    add_str_param(&args, &mut params, "created_before", "created_before");
    let v = call!(bland_get(&append_query("/audit/logs", &params)));
    json_result(&v)
}
