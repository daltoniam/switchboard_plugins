mod tools;

use std::collections::HashMap;
use std::sync::Mutex;
use switchboard_guest_sdk as sdk;

static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

struct Config {
    bridge_url: String,
}

fn with_config<F, R>(f: F) -> R
where
    F: FnOnce(&Config) -> R,
{
    let guard = CONFIG.lock().unwrap();
    f(guard.as_ref().expect("not configured"))
}

fn bridge_url() -> String {
    with_config(|c| c.bridge_url.clone())
}

#[no_mangle]
pub extern "C" fn name() -> u64 {
    sdk::leaked_string("figma_bridge")
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
        Err(e) => return sdk::leaked_string(&format!("invalid credentials JSON: {e}")),
    };

    let url = creds
        .get("bridge_url")
        .map(|s| s.trim_end_matches('/').to_string())
        .unwrap_or_else(|| "http://127.0.0.1:9223".into());

    *CONFIG.lock().unwrap() = Some(Config { bridge_url: url });
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
    match bridge_request("status", &serde_json::Value::Null) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn metadata() -> u64 {
    sdk::leaked_metadata(&sdk::PluginMetadata {
        name: "figma_bridge".into(),
        version: "0.1.0".into(),
        abi_version: 1,
        description: "Figma Desktop Bridge — access the Figma Plugin API for write operations (create frames, components, auto layout, etc.) via a companion plugin running in Figma Desktop.".into(),
        author: "daltoniam".into(),
        homepage: "https://github.com/daltoniam/switchboard_plugins".into(),
        license: "MIT".into(),
        capabilities: vec!["http".into()],
        credential_keys: vec!["bridge_url".into()],
        plain_text_keys: vec!["bridge_url".into()],
        optional_keys: vec!["bridge_url".into()],
        placeholders: HashMap::from([(
            "bridge_url".into(),
            "http://127.0.0.1:9223 (default — Figma Desktop Bridge port)".into(),
        )]),
    })
}

// ── Dispatch ────────────────────────────────────────────────────────────────

type HandlerFn = fn(HashMap<String, serde_json::Value>) -> sdk::ToolResult;

fn dispatch(tool_name: &str, args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let handler: Option<HandlerFn> = match tool_name {
        "figma_bridge_status" => Some(handle_status),
        "figma_bridge_get_selection" => Some(handle_get_selection),
        "figma_bridge_get_page_nodes" => Some(handle_get_page_nodes),
        "figma_bridge_create_frame" => Some(handle_create_frame),
        "figma_bridge_create_text" => Some(handle_create_text),
        "figma_bridge_create_rectangle" => Some(handle_create_rectangle),
        "figma_bridge_set_node_property" => Some(handle_set_node_property),
        "figma_bridge_delete_node" => Some(handle_delete_node),
        "figma_bridge_clone_node" => Some(handle_clone_node),
        "figma_bridge_create_component" => Some(handle_create_component),
        "figma_bridge_create_instance" => Some(handle_create_instance),
        "figma_bridge_set_auto_layout" => Some(handle_set_auto_layout),
        "figma_bridge_set_fills" => Some(handle_set_fills),
        "figma_bridge_set_strokes" => Some(handle_set_strokes),
        "figma_bridge_execute" => Some(handle_execute),
        _ => None,
    };

    match handler {
        Some(f) => f(args),
        None => sdk::err_result(&format!("unknown tool: {tool_name}")),
    }
}

// ── Bridge Communication ────────────────────────────────────────────────────

fn bridge_request(command: &str, params: &serde_json::Value) -> Result<String, String> {
    let body = serde_json::json!({
        "command": command,
        "params": params
    });

    let req = sdk::HttpRequest {
        method: "POST".into(),
        url: format!("{}/api/command", bridge_url()),
        headers: {
            let mut h = HashMap::new();
            h.insert("Content-Type".into(), "application/json".into());
            h
        },
        body: body.to_string(),
        body_base64: String::new(),
    };

    let resp = sdk::host_http_request(&req)?;
    if resp.status >= 400 {
        return Err(format!(
            "Figma Bridge error ({}): {}",
            resp.status, resp.body
        ));
    }
    Ok(resp.body)
}

// ── Handlers ────────────────────────────────────────────────────────────────

fn handle_status(_args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    match bridge_request("status", &serde_json::Value::Null) {
        Ok(data) => sdk::raw_result(data),
        Err(e) => sdk::err_result(&format!(
            "Bridge not connected: {e}. Ensure the Figma Desktop Bridge plugin is running."
        )),
    }
}

fn handle_get_selection(_args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    match bridge_request("getSelection", &serde_json::Value::Null) {
        Ok(data) => sdk::raw_result(data),
        Err(e) => sdk::err_result(&e),
    }
}

fn handle_get_page_nodes(_args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    match bridge_request("getPageNodes", &serde_json::Value::Null) {
        Ok(data) => sdk::raw_result(data),
        Err(e) => sdk::err_result(&e),
    }
}

fn handle_create_frame(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let name = sdk::arg_str(&args, "name");
    if name.is_empty() {
        return sdk::err_result("name is required");
    }
    let params = serde_json::json!({
        "name": name,
        "x": sdk::arg_int(&args, "x").unwrap_or(0),
        "y": sdk::arg_int(&args, "y").unwrap_or(0),
        "width": sdk::arg_int(&args, "width").unwrap_or(400),
        "height": sdk::arg_int(&args, "height").unwrap_or(300),
    });
    match bridge_request("createFrame", &params) {
        Ok(data) => sdk::raw_result(data),
        Err(e) => sdk::err_result(&e),
    }
}

fn handle_create_text(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let text = sdk::arg_str(&args, "text");
    if text.is_empty() {
        return sdk::err_result("text is required");
    }
    let params = serde_json::json!({
        "text": text,
        "parentId": sdk::arg_str(&args, "parent_id"),
        "x": sdk::arg_int(&args, "x").unwrap_or(0),
        "y": sdk::arg_int(&args, "y").unwrap_or(0),
        "fontSize": sdk::arg_int(&args, "font_size").unwrap_or(16),
    });
    match bridge_request("createText", &params) {
        Ok(data) => sdk::raw_result(data),
        Err(e) => sdk::err_result(&e),
    }
}

fn handle_create_rectangle(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let params = serde_json::json!({
        "name": sdk::arg_str(&args, "name"),
        "parentId": sdk::arg_str(&args, "parent_id"),
        "x": sdk::arg_int(&args, "x").unwrap_or(0),
        "y": sdk::arg_int(&args, "y").unwrap_or(0),
        "width": sdk::arg_int(&args, "width").unwrap_or(100),
        "height": sdk::arg_int(&args, "height").unwrap_or(100),
        "fillColor": sdk::arg_str(&args, "fill_color"),
    });
    match bridge_request("createRectangle", &params) {
        Ok(data) => sdk::raw_result(data),
        Err(e) => sdk::err_result(&e),
    }
}

fn handle_set_node_property(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let node_id = sdk::arg_str(&args, "node_id");
    let property = sdk::arg_str(&args, "property");
    let value = sdk::arg_str(&args, "value");
    if node_id.is_empty() || property.is_empty() {
        return sdk::err_result("node_id and property are required");
    }
    let parsed_value: serde_json::Value =
        serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));
    let params = serde_json::json!({
        "nodeId": node_id,
        "property": property,
        "value": parsed_value,
    });
    match bridge_request("setNodeProperty", &params) {
        Ok(data) => sdk::raw_result(data),
        Err(e) => sdk::err_result(&e),
    }
}

fn handle_delete_node(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let node_id = sdk::arg_str(&args, "node_id");
    if node_id.is_empty() {
        return sdk::err_result("node_id is required");
    }
    let params = serde_json::json!({ "nodeId": node_id });
    match bridge_request("deleteNode", &params) {
        Ok(data) => sdk::raw_result(data),
        Err(e) => sdk::err_result(&e),
    }
}

fn handle_clone_node(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let node_id = sdk::arg_str(&args, "node_id");
    if node_id.is_empty() {
        return sdk::err_result("node_id is required");
    }
    let params = serde_json::json!({ "nodeId": node_id });
    match bridge_request("cloneNode", &params) {
        Ok(data) => sdk::raw_result(data),
        Err(e) => sdk::err_result(&e),
    }
}

fn handle_create_component(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let name = sdk::arg_str(&args, "name");
    if name.is_empty() {
        return sdk::err_result("name is required");
    }
    let params = serde_json::json!({
        "nodeId": sdk::arg_str(&args, "node_id"),
        "name": name,
        "width": sdk::arg_int(&args, "width").unwrap_or(100),
        "height": sdk::arg_int(&args, "height").unwrap_or(100),
    });
    match bridge_request("createComponent", &params) {
        Ok(data) => sdk::raw_result(data),
        Err(e) => sdk::err_result(&e),
    }
}

fn handle_create_instance(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let component_id = sdk::arg_str(&args, "component_id");
    if component_id.is_empty() {
        return sdk::err_result("component_id is required");
    }
    let params = serde_json::json!({
        "componentId": component_id,
        "x": sdk::arg_int(&args, "x").unwrap_or(0),
        "y": sdk::arg_int(&args, "y").unwrap_or(0),
    });
    match bridge_request("createInstance", &params) {
        Ok(data) => sdk::raw_result(data),
        Err(e) => sdk::err_result(&e),
    }
}

fn handle_set_auto_layout(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let node_id = sdk::arg_str(&args, "node_id");
    let direction = sdk::arg_str(&args, "direction");
    if node_id.is_empty() || direction.is_empty() {
        return sdk::err_result("node_id and direction are required");
    }
    let params = serde_json::json!({
        "nodeId": node_id,
        "direction": direction,
        "spacing": sdk::arg_int(&args, "spacing").unwrap_or(0),
        "padding": sdk::arg_str(&args, "padding"),
        "align": sdk::arg_str(&args, "align"),
    });
    match bridge_request("setAutoLayout", &params) {
        Ok(data) => sdk::raw_result(data),
        Err(e) => sdk::err_result(&e),
    }
}

fn handle_set_fills(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let node_id = sdk::arg_str(&args, "node_id");
    let fills_str = sdk::arg_str(&args, "fills");
    if node_id.is_empty() || fills_str.is_empty() {
        return sdk::err_result("node_id and fills are required");
    }
    let fills: serde_json::Value = match serde_json::from_str(&fills_str) {
        Ok(v) => v,
        Err(e) => return sdk::err_result(&format!("invalid JSON for fills: {e}")),
    };
    let params = serde_json::json!({
        "nodeId": node_id,
        "fills": fills,
    });
    match bridge_request("setFills", &params) {
        Ok(data) => sdk::raw_result(data),
        Err(e) => sdk::err_result(&e),
    }
}

fn handle_set_strokes(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let node_id = sdk::arg_str(&args, "node_id");
    let strokes_str = sdk::arg_str(&args, "strokes");
    if node_id.is_empty() || strokes_str.is_empty() {
        return sdk::err_result("node_id and strokes are required");
    }
    let strokes: serde_json::Value = match serde_json::from_str(&strokes_str) {
        Ok(v) => v,
        Err(e) => return sdk::err_result(&format!("invalid JSON for strokes: {e}")),
    };
    let params = serde_json::json!({
        "nodeId": node_id,
        "strokes": strokes,
        "strokeWeight": sdk::arg_int(&args, "stroke_weight"),
    });
    match bridge_request("setStrokes", &params) {
        Ok(data) => sdk::raw_result(data),
        Err(e) => sdk::err_result(&e),
    }
}

fn handle_execute(args: HashMap<String, serde_json::Value>) -> sdk::ToolResult {
    let code = sdk::arg_str(&args, "code");
    if code.is_empty() {
        return sdk::err_result("code is required");
    }
    let params = serde_json::json!({ "code": code });
    match bridge_request("execute", &params) {
        Ok(data) => sdk::raw_result(data),
        Err(e) => sdk::err_result(&e),
    }
}
