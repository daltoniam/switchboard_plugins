use std::collections::HashMap;
use switchboard_guest_sdk as sdk;

use crate::{clerk_delete, clerk_get, clerk_patch, clerk_post, path_escape, query_escape};

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

/// Repeats an array-valued query parameter as `key=v1&key=v2` after splitting on commas.
fn add_repeated_param(
    args: &HashMap<String, serde_json::Value>,
    params: &mut Vec<(String, String)>,
    arg: &str,
    key: &str,
) {
    let value = sdk::arg_str(args, arg);
    if value.is_empty() {
        return;
    }
    for piece in value.split(',') {
        let trimmed = piece.trim();
        if !trimmed.is_empty() {
            params.push((key.to_string(), trimmed.to_string()));
        }
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
        Some(v) if !v.is_null() && !matches!(v, serde_json::Value::String(s) if s.is_empty()) => {
            body.insert(key.to_string(), v.clone());
        }
        _ => {}
    }
    Ok(())
}

/// Coerces a value that may arrive as a JSON array or a comma-separated string into a JSON array.
fn insert_string_array_body(
    args: &HashMap<String, serde_json::Value>,
    body: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<(), String> {
    match args.get(key) {
        Some(serde_json::Value::Array(arr)) => {
            body.insert(key.to_string(), serde_json::Value::Array(arr.clone()));
        }
        Some(serde_json::Value::String(s)) if !s.is_empty() => {
            let trimmed = s.trim();
            if trimmed.starts_with('[') {
                let parsed: serde_json::Value = serde_json::from_str(trimmed)
                    .map_err(|e| format!("{key} must be a JSON array: {e}"))?;
                if !parsed.is_array() {
                    return Err(format!("{key} must be a JSON array"));
                }
                body.insert(key.to_string(), parsed);
            } else {
                let values: Vec<serde_json::Value> = trimmed
                    .split(',')
                    .filter_map(|p| {
                        let t = p.trim();
                        if t.is_empty() {
                            None
                        } else {
                            Some(serde_json::Value::String(t.to_string()))
                        }
                    })
                    .collect();
                body.insert(key.to_string(), serde_json::Value::Array(values));
            }
        }
        _ => {}
    }
    Ok(())
}

// ── Users ───────────────────────────────────────────────────────────────────

pub fn list_users(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let mut params: Vec<(String, String)> = vec![
        (
            "limit".into(),
            sdk::arg_int(&args, "limit")
                .unwrap_or(10)
                .clamp(1, 500)
                .to_string(),
        ),
        (
            "offset".into(),
            sdk::arg_int(&args, "offset")
                .unwrap_or(0)
                .max(0)
                .to_string(),
        ),
    ];
    add_str_param(&args, &mut params, "order_by", "order_by");
    add_str_param(&args, &mut params, "query", "query");
    add_repeated_param(&args, &mut params, "email_address", "email_address");
    add_repeated_param(&args, &mut params, "phone_number", "phone_number");
    add_repeated_param(&args, &mut params, "username", "username");
    add_repeated_param(&args, &mut params, "user_id", "user_id");
    add_repeated_param(&args, &mut params, "external_id", "external_id");
    add_repeated_param(&args, &mut params, "organization_id", "organization_id");
    add_bool_param(&args, &mut params, "banned", "banned");
    add_bool_param(&args, &mut params, "locked", "locked");
    add_int_param(
        &args,
        &mut params,
        "last_active_at_since",
        "last_active_at_since",
    );

    let v = call!(clerk_get(&append_query("/users", &params)));
    json_result(&v)
}

pub fn get_user(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "user_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(clerk_get(&format!("/users/{}", path_escape(&id))));
    json_result(&v)
}

pub fn create_user(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let mut body = serde_json::Map::new();

    for key in ["email_address", "phone_number", "web3_wallet"] {
        if let Err(e) = insert_string_array_body(&args, &mut body, key) {
            return sdk::err_result(&e);
        }
    }
    for key in [
        "username",
        "password",
        "password_digest",
        "password_hasher",
        "first_name",
        "last_name",
        "external_id",
        "created_at",
    ] {
        insert_string_body(&args, &mut body, key);
    }
    for key in ["skip_password_checks", "skip_password_requirement"] {
        insert_bool_body(&args, &mut body, key);
    }
    for key in ["public_metadata", "private_metadata", "unsafe_metadata"] {
        if let Err(e) = insert_json_body(&args, &mut body, key) {
            return sdk::err_result(&e);
        }
    }

    let v = call!(clerk_post("/users", &serde_json::Value::Object(body)));
    json_result(&v)
}

pub fn update_user(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "user_id") {
        Ok(v) => v,
        Err(e) => return e,
    };

    let mut body = serde_json::Map::new();
    for key in [
        "first_name",
        "last_name",
        "username",
        "primary_email_address_id",
        "primary_phone_number_id",
        "primary_web3_wallet_id",
        "profile_image_id",
        "password",
        "password_digest",
        "password_hasher",
        "external_id",
    ] {
        insert_string_body(&args, &mut body, key);
    }
    insert_bool_body(&args, &mut body, "sign_out_of_other_sessions");
    for key in ["public_metadata", "private_metadata", "unsafe_metadata"] {
        if let Err(e) = insert_json_body(&args, &mut body, key) {
            return sdk::err_result(&e);
        }
    }

    let v = call!(clerk_patch(
        &format!("/users/{}", path_escape(&id)),
        &serde_json::Value::Object(body)
    ));
    json_result(&v)
}

pub fn delete_user(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "user_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(clerk_delete(&format!("/users/{}", path_escape(&id))));
    json_result(&v)
}

fn user_action(args: HashMap<String, serde_json::Value>, action: &str) -> sdk::ToolResult {
    let id = match required(&args, "user_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(clerk_post(
        &format!("/users/{}/{}", path_escape(&id), action),
        &serde_json::json!({})
    ));
    json_result(&v)
}

pub fn ban_user(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    user_action(args, "ban")
}

pub fn unban_user(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    user_action(args, "unban")
}

pub fn lock_user(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    user_action(args, "lock")
}

pub fn unlock_user(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    user_action(args, "unlock")
}

pub fn list_user_organization_memberships(
    args: HashMap<String, serde_json::Value>,
) -> sdk::ToolResult {
    let id = match required(&args, "user_id") {
        Ok(v) => v,
        Err(e) => return e,
    };

    let params: Vec<(String, String)> = vec![
        (
            "limit".into(),
            sdk::arg_int(&args, "limit")
                .unwrap_or(10)
                .clamp(1, 100)
                .to_string(),
        ),
        (
            "offset".into(),
            sdk::arg_int(&args, "offset")
                .unwrap_or(0)
                .max(0)
                .to_string(),
        ),
    ];

    let v = call!(clerk_get(&append_query(
        &format!("/users/{}/organization_memberships", path_escape(&id)),
        &params
    )));
    json_result(&v)
}

// ── Sessions ────────────────────────────────────────────────────────────────

pub fn list_sessions(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let mut params: Vec<(String, String)> = Vec::new();
    add_str_param(&args, &mut params, "client_id", "client_id");
    add_str_param(&args, &mut params, "user_id", "user_id");
    add_str_param(&args, &mut params, "status", "status");

    let has_filter = params
        .iter()
        .any(|(k, _)| k == "client_id" || k == "user_id");
    if !has_filter {
        return sdk::err_result("user_id or client_id is required");
    }

    let v = call!(clerk_get(&append_query("/sessions", &params)));
    json_result(&v)
}

pub fn get_session(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "session_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(clerk_get(&format!("/sessions/{}", path_escape(&id))));
    json_result(&v)
}

pub fn revoke_session(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "session_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(clerk_post(
        &format!("/sessions/{}/revoke", path_escape(&id)),
        &serde_json::json!({})
    ));
    json_result(&v)
}

// ── Organizations ───────────────────────────────────────────────────────────

pub fn list_organizations(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let mut params: Vec<(String, String)> = vec![
        (
            "limit".into(),
            sdk::arg_int(&args, "limit")
                .unwrap_or(10)
                .clamp(1, 500)
                .to_string(),
        ),
        (
            "offset".into(),
            sdk::arg_int(&args, "offset")
                .unwrap_or(0)
                .max(0)
                .to_string(),
        ),
    ];
    add_bool_param(
        &args,
        &mut params,
        "include_members_count",
        "include_members_count",
    );
    add_str_param(&args, &mut params, "order_by", "order_by");
    add_str_param(&args, &mut params, "query", "query");
    add_repeated_param(&args, &mut params, "user_id", "user_id");

    let v = call!(clerk_get(&append_query("/organizations", &params)));
    json_result(&v)
}

pub fn get_organization(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "organization_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let mut params: Vec<(String, String)> = Vec::new();
    add_bool_param(
        &args,
        &mut params,
        "include_members_count",
        "include_members_count",
    );
    let v = call!(clerk_get(&append_query(
        &format!("/organizations/{}", path_escape(&id)),
        &params
    )));
    json_result(&v)
}

pub fn create_organization(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let name = match required(&args, "name") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let created_by = match required(&args, "created_by") {
        Ok(v) => v,
        Err(e) => return e,
    };

    let mut body = serde_json::Map::new();
    body.insert("name".into(), serde_json::json!(name));
    body.insert("created_by".into(), serde_json::json!(created_by));
    insert_string_body(&args, &mut body, "slug");
    insert_i64_body(&args, &mut body, "max_allowed_memberships");
    for key in ["public_metadata", "private_metadata"] {
        if let Err(e) = insert_json_body(&args, &mut body, key) {
            return sdk::err_result(&e);
        }
    }

    let v = call!(clerk_post(
        "/organizations",
        &serde_json::Value::Object(body)
    ));
    json_result(&v)
}

pub fn update_organization(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "organization_id") {
        Ok(v) => v,
        Err(e) => return e,
    };

    let mut body = serde_json::Map::new();
    insert_string_body(&args, &mut body, "name");
    insert_string_body(&args, &mut body, "slug");
    insert_i64_body(&args, &mut body, "max_allowed_memberships");
    insert_bool_body(&args, &mut body, "admin_delete_enabled");
    for key in ["public_metadata", "private_metadata"] {
        if let Err(e) = insert_json_body(&args, &mut body, key) {
            return sdk::err_result(&e);
        }
    }

    let v = call!(clerk_patch(
        &format!("/organizations/{}", path_escape(&id)),
        &serde_json::Value::Object(body)
    ));
    json_result(&v)
}

pub fn delete_organization(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "organization_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(clerk_delete(&format!(
        "/organizations/{}",
        path_escape(&id)
    )));
    json_result(&v)
}

pub fn list_organization_memberships(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "organization_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let mut params: Vec<(String, String)> = vec![
        (
            "limit".into(),
            sdk::arg_int(&args, "limit")
                .unwrap_or(10)
                .clamp(1, 500)
                .to_string(),
        ),
        (
            "offset".into(),
            sdk::arg_int(&args, "offset")
                .unwrap_or(0)
                .max(0)
                .to_string(),
        ),
    ];
    add_str_param(&args, &mut params, "order_by", "order_by");

    let v = call!(clerk_get(&append_query(
        &format!("/organizations/{}/memberships", path_escape(&id)),
        &params
    )));
    json_result(&v)
}

pub fn create_organization_membership(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let org_id = match required(&args, "organization_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let user_id = match required(&args, "user_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let role = match required(&args, "role") {
        Ok(v) => v,
        Err(e) => return e,
    };

    let body = serde_json::json!({
        "user_id": user_id,
        "role": role,
    });
    let v = call!(clerk_post(
        &format!("/organizations/{}/memberships", path_escape(&org_id)),
        &body
    ));
    json_result(&v)
}

pub fn update_organization_membership(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let org_id = match required(&args, "organization_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let user_id = match required(&args, "user_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let role = match required(&args, "role") {
        Ok(v) => v,
        Err(e) => return e,
    };

    let body = serde_json::json!({ "role": role });
    let v = call!(clerk_patch(
        &format!(
            "/organizations/{}/memberships/{}",
            path_escape(&org_id),
            path_escape(&user_id)
        ),
        &body
    ));
    json_result(&v)
}

pub fn delete_organization_membership(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let org_id = match required(&args, "organization_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let user_id = match required(&args, "user_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(clerk_delete(&format!(
        "/organizations/{}/memberships/{}",
        path_escape(&org_id),
        path_escape(&user_id)
    )));
    json_result(&v)
}

pub fn list_organization_invitations(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "organization_id") {
        Ok(v) => v,
        Err(e) => return e,
    };

    let mut params: Vec<(String, String)> = vec![
        (
            "limit".into(),
            sdk::arg_int(&args, "limit")
                .unwrap_or(10)
                .clamp(1, 500)
                .to_string(),
        ),
        (
            "offset".into(),
            sdk::arg_int(&args, "offset")
                .unwrap_or(0)
                .max(0)
                .to_string(),
        ),
    ];
    add_repeated_param(&args, &mut params, "status", "status");

    let v = call!(clerk_get(&append_query(
        &format!("/organizations/{}/invitations", path_escape(&id)),
        &params
    )));
    json_result(&v)
}

pub fn create_organization_invitation(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let org_id = match required(&args, "organization_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let email_address = match required(&args, "email_address") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let role = match required(&args, "role") {
        Ok(v) => v,
        Err(e) => return e,
    };

    let mut body = serde_json::Map::new();
    body.insert("email_address".into(), serde_json::json!(email_address));
    body.insert("role".into(), serde_json::json!(role));
    insert_string_body(&args, &mut body, "inviter_user_id");
    insert_string_body(&args, &mut body, "redirect_url");
    for key in ["public_metadata", "private_metadata"] {
        if let Err(e) = insert_json_body(&args, &mut body, key) {
            return sdk::err_result(&e);
        }
    }

    let v = call!(clerk_post(
        &format!("/organizations/{}/invitations", path_escape(&org_id)),
        &serde_json::Value::Object(body)
    ));
    json_result(&v)
}

pub fn revoke_organization_invitation(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let org_id = match required(&args, "organization_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let invitation_id = match required(&args, "invitation_id") {
        Ok(v) => v,
        Err(e) => return e,
    };

    let mut body = serde_json::Map::new();
    insert_string_body(&args, &mut body, "requesting_user_id");

    let v = call!(clerk_post(
        &format!(
            "/organizations/{}/invitations/{}/revoke",
            path_escape(&org_id),
            path_escape(&invitation_id)
        ),
        &serde_json::Value::Object(body)
    ));
    json_result(&v)
}

// ── Invitations ─────────────────────────────────────────────────────────────

pub fn list_invitations(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let mut params: Vec<(String, String)> = vec![
        (
            "limit".into(),
            sdk::arg_int(&args, "limit")
                .unwrap_or(10)
                .clamp(1, 500)
                .to_string(),
        ),
        (
            "offset".into(),
            sdk::arg_int(&args, "offset")
                .unwrap_or(0)
                .max(0)
                .to_string(),
        ),
    ];
    add_str_param(&args, &mut params, "status", "status");
    add_str_param(&args, &mut params, "query", "query");
    add_str_param(&args, &mut params, "order_by", "order_by");

    let v = call!(clerk_get(&append_query("/invitations", &params)));
    json_result(&v)
}

pub fn create_invitation(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let email_address = match required(&args, "email_address") {
        Ok(v) => v,
        Err(e) => return e,
    };

    let mut body = serde_json::Map::new();
    body.insert("email_address".into(), serde_json::json!(email_address));
    insert_string_body(&args, &mut body, "redirect_url");
    insert_string_body(&args, &mut body, "template_slug");
    insert_bool_body(&args, &mut body, "notify");
    insert_bool_body(&args, &mut body, "ignore_existing");
    insert_i64_body(&args, &mut body, "expires_in_days");
    if let Err(e) = insert_json_body(&args, &mut body, "public_metadata") {
        return sdk::err_result(&e);
    }

    let v = call!(clerk_post("/invitations", &serde_json::Value::Object(body)));
    json_result(&v)
}

pub fn revoke_invitation(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "invitation_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(clerk_post(
        &format!("/invitations/{}/revoke", path_escape(&id)),
        &serde_json::json!({})
    ));
    json_result(&v)
}

// ── Allow / Block list ──────────────────────────────────────────────────────

pub fn list_allowlist_identifiers(_args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let v = call!(clerk_get("/allowlist_identifiers"));
    json_result(&v)
}

pub fn create_allowlist_identifier(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let identifier = match required(&args, "identifier") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let mut body = serde_json::Map::new();
    body.insert("identifier".into(), serde_json::json!(identifier));
    insert_bool_body(&args, &mut body, "notify");

    let v = call!(clerk_post(
        "/allowlist_identifiers",
        &serde_json::Value::Object(body)
    ));
    json_result(&v)
}

pub fn delete_allowlist_identifier(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "identifier_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(clerk_delete(&format!(
        "/allowlist_identifiers/{}",
        path_escape(&id)
    )));
    json_result(&v)
}

pub fn list_blocklist_identifiers(_args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let v = call!(clerk_get("/blocklist_identifiers"));
    json_result(&v)
}

pub fn create_blocklist_identifier(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let identifier = match required(&args, "identifier") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let body = serde_json::json!({ "identifier": identifier });

    let v = call!(clerk_post("/blocklist_identifiers", &body));
    json_result(&v)
}

pub fn delete_blocklist_identifier(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let id = match required(&args, "identifier_id") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let v = call!(clerk_delete(&format!(
        "/blocklist_identifiers/{}",
        path_escape(&id)
    )));
    json_result(&v)
}
