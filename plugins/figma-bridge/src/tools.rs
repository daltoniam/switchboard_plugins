use std::collections::HashMap;
use switchboard_guest_sdk::ToolDefinition;

pub fn tool_definitions() -> Vec<ToolDefinition> {
    vec![
        // ── Connection ──────────────────────────────────────────────────
        ToolDefinition {
            name: "figma_bridge_status".into(),
            description: "Check the connection status to the Figma Desktop Bridge plugin. Start here to verify the bridge is running.".into(),
            parameters: HashMap::new(),
            required: vec![],
        },
        // ── File & Selection ────────────────────────────────────────────
        ToolDefinition {
            name: "figma_bridge_get_selection".into(),
            description: "Get the currently selected nodes in Figma Desktop (IDs, names, types, properties)".into(),
            parameters: HashMap::new(),
            required: vec![],
        },
        ToolDefinition {
            name: "figma_bridge_get_page_nodes".into(),
            description: "Get all top-level nodes on the current page".into(),
            parameters: HashMap::new(),
            required: vec![],
        },
        // ── Node Manipulation ───────────────────────────────────────────
        ToolDefinition {
            name: "figma_bridge_create_frame".into(),
            description: "Create a new frame on the current page".into(),
            parameters: {
                let mut m = HashMap::new();
                m.insert("name".into(), "Frame name".into());
                m.insert("x".into(), "X position (default 0)".into());
                m.insert("y".into(), "Y position (default 0)".into());
                m.insert("width".into(), "Width in pixels (default 400)".into());
                m.insert("height".into(), "Height in pixels (default 300)".into());
                m
            },
            required: vec!["name".into()],
        },
        ToolDefinition {
            name: "figma_bridge_create_text".into(),
            description: "Create a text node on the current page or inside a specified parent".into(),
            parameters: {
                let mut m = HashMap::new();
                m.insert("text".into(), "Text content".into());
                m.insert("parent_id".into(), "Parent node ID (optional, defaults to current page)".into());
                m.insert("x".into(), "X position (default 0)".into());
                m.insert("y".into(), "Y position (default 0)".into());
                m.insert("font_size".into(), "Font size in pixels (default 16)".into());
                m
            },
            required: vec!["text".into()],
        },
        ToolDefinition {
            name: "figma_bridge_create_rectangle".into(),
            description: "Create a rectangle node".into(),
            parameters: {
                let mut m = HashMap::new();
                m.insert("name".into(), "Node name (default 'Rectangle')".into());
                m.insert("parent_id".into(), "Parent node ID (optional)".into());
                m.insert("x".into(), "X position (default 0)".into());
                m.insert("y".into(), "Y position (default 0)".into());
                m.insert("width".into(), "Width (default 100)".into());
                m.insert("height".into(), "Height (default 100)".into());
                m.insert("fill_color".into(), "Fill color as hex (e.g. #FF5733)".into());
                m
            },
            required: vec![],
        },
        ToolDefinition {
            name: "figma_bridge_set_node_property".into(),
            description: "Set a property on an existing node (position, size, name, opacity, fills, etc.)".into(),
            parameters: {
                let mut m = HashMap::new();
                m.insert("node_id".into(), "Target node ID".into());
                m.insert("property".into(), "Property name (e.g. name, x, y, width, height, opacity, visible)".into());
                m.insert("value".into(), "New value (JSON for complex types, string/number for simple)".into());
                m
            },
            required: vec!["node_id".into(), "property".into(), "value".into()],
        },
        ToolDefinition {
            name: "figma_bridge_delete_node".into(),
            description: "Delete a node by ID".into(),
            parameters: {
                let mut m = HashMap::new();
                m.insert("node_id".into(), "Node ID to delete".into());
                m
            },
            required: vec!["node_id".into()],
        },
        ToolDefinition {
            name: "figma_bridge_clone_node".into(),
            description: "Clone/duplicate a node by ID".into(),
            parameters: {
                let mut m = HashMap::new();
                m.insert("node_id".into(), "Node ID to clone".into());
                m
            },
            required: vec!["node_id".into()],
        },
        // ── Components & Instances ──────────────────────────────────────
        ToolDefinition {
            name: "figma_bridge_create_component".into(),
            description: "Convert a frame/group into a component, or create a new component".into(),
            parameters: {
                let mut m = HashMap::new();
                m.insert("node_id".into(), "Existing node ID to convert (optional)".into());
                m.insert("name".into(), "Component name".into());
                m.insert("width".into(), "Width if creating new (default 100)".into());
                m.insert("height".into(), "Height if creating new (default 100)".into());
                m
            },
            required: vec!["name".into()],
        },
        ToolDefinition {
            name: "figma_bridge_create_instance".into(),
            description: "Create an instance of a local component by its ID".into(),
            parameters: {
                let mut m = HashMap::new();
                m.insert("component_id".into(), "Component node ID".into());
                m.insert("x".into(), "X position (default 0)".into());
                m.insert("y".into(), "Y position (default 0)".into());
                m
            },
            required: vec!["component_id".into()],
        },
        // ── Auto Layout ─────────────────────────────────────────────────
        ToolDefinition {
            name: "figma_bridge_set_auto_layout".into(),
            description: "Apply or modify auto layout on a frame".into(),
            parameters: {
                let mut m = HashMap::new();
                m.insert("node_id".into(), "Frame node ID".into());
                m.insert("direction".into(), "HORIZONTAL or VERTICAL".into());
                m.insert("spacing".into(), "Item spacing in pixels".into());
                m.insert("padding".into(), "Padding (single number for all sides, or JSON object with top/right/bottom/left)".into());
                m.insert("align".into(), "Primary axis alignment: MIN, CENTER, MAX, SPACE_BETWEEN".into());
                m
            },
            required: vec!["node_id".into(), "direction".into()],
        },
        // ── Styles & Variables ──────────────────────────────────────────
        ToolDefinition {
            name: "figma_bridge_set_fills".into(),
            description: "Set fill colors on a node".into(),
            parameters: {
                let mut m = HashMap::new();
                m.insert("node_id".into(), "Target node ID".into());
                m.insert("fills".into(), "JSON array of fill paints (e.g. [{\"type\":\"SOLID\",\"color\":{\"r\":1,\"g\":0,\"b\":0}}])".into());
                m
            },
            required: vec!["node_id".into(), "fills".into()],
        },
        ToolDefinition {
            name: "figma_bridge_set_strokes".into(),
            description: "Set stroke colors on a node".into(),
            parameters: {
                let mut m = HashMap::new();
                m.insert("node_id".into(), "Target node ID".into());
                m.insert("strokes".into(), "JSON array of stroke paints".into());
                m.insert("stroke_weight".into(), "Stroke weight in pixels (optional)".into());
                m
            },
            required: vec!["node_id".into(), "strokes".into()],
        },
        // ── Execute Raw Plugin Code ─────────────────────────────────────
        ToolDefinition {
            name: "figma_bridge_execute".into(),
            description: "Execute arbitrary Figma Plugin API JavaScript code in the desktop app context. Use for advanced operations not covered by other tools.".into(),
            parameters: {
                let mut m = HashMap::new();
                m.insert("code".into(), "JavaScript code to execute (has access to the `figma` global object)".into());
                m
            },
            required: vec!["code".into()],
        },
    ]
}
