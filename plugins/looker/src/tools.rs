use std::collections::HashMap;
use switchboard_guest_sdk::ToolDefinition;

macro_rules! tool {
    ($name:expr, $desc:expr, $params:expr) => {
        ToolDefinition {
            name: $name.into(),
            description: $desc.into(),
            parameters: $params,
            required: vec![],
        }
    };
    ($name:expr, $desc:expr, $params:expr, $req:expr) => {
        ToolDefinition {
            name: $name.into(),
            description: $desc.into(),
            parameters: $params,
            required: $req.iter().map(|s: &&str| s.to_string()).collect(),
        }
    };
}

macro_rules! params {
    () => { HashMap::new() };
    ($($k:expr => $v:expr),+ $(,)?) => {{
        let mut m: HashMap<String, String> = HashMap::new();
        $(m.insert($k.into(), $v.into());)+
        m
    }};
}

pub fn tool_definitions() -> Vec<ToolDefinition> {
    vec![
        // ── Search / discovery ──────────────────────────────────────────
        tool!(
            "looker_search_content",
            "Search Looker dashboards, Looks (saved reports), and folders by title or description. \
             Start here for BI, business intelligence, analytics, reports, visualizations, and finding existing data assets in Looker.",
            params!(
                "terms" => "Search terms (matches title/description)",
                "types" => "Comma-separated content types to include: dashboard, look, folder (default: all)",
                "limit" => "Maximum results to return (default: 25)",
                "offset" => "Pagination offset (default: 0)"
            ),
            &["terms"]
        ),

        // ── Folders ─────────────────────────────────────────────────────
        tool!(
            "looker_list_folders",
            "List Looker folders (spaces) used to organize dashboards and Looks. Use for navigating BI content organization.",
            params!(
                "fields" => "Optional comma-separated field selection"
            )
        ),
        tool!(
            "looker_get_folder",
            "Get a Looker folder's metadata and child content (dashboards, Looks). Use after list_folders or search_content.",
            params!(
                "folder_id" => "Folder ID"
            ),
            &["folder_id"]
        ),

        // ── Looks ───────────────────────────────────────────────────────
        tool!(
            "looker_list_looks",
            "List Looks — saved Looker reports and queries used for BI and analytics dashboards. \
             Use to discover existing saved analytics queries before creating new ones.",
            params!(
                "limit" => "Maximum Looks to return (default: 25)",
                "offset" => "Pagination offset (default: 0)",
                "fields" => "Optional comma-separated field selection"
            )
        ),
        tool!(
            "looker_get_look",
            "Get a Look's metadata, owner, folder, and underlying query reference. Use after list_looks to inspect a saved BI report.",
            params!(
                "look_id" => "Look ID"
            ),
            &["look_id"]
        ),
        tool!(
            "looker_run_look",
            "Run a saved Look and return its data rows as JSON. Use for fetching saved BI report results and analytics data. \
             Row count is capped (default 100, max 5000) — pass `limit` to adjust.",
            params!(
                "look_id" => "Look ID",
                "limit" => "Row limit (default: 100, max: 5000)",
                "apply_formatting" => "Apply Looker value formatting (default: false)",
                "apply_vis" => "Apply visualization options (default: false)",
                "cache" => "Use cache (default: true)"
            ),
            &["look_id"]
        ),

        // ── Dashboards ──────────────────────────────────────────────────
        tool!(
            "looker_list_dashboards",
            "List Looker dashboards — interactive BI reports and analytics visualizations. \
             Use for discovering existing dashboards before exploring data.",
            params!(
                "limit" => "Maximum dashboards to return (default: 25)",
                "offset" => "Pagination offset (default: 0)",
                "fields" => "Optional comma-separated field selection"
            )
        ),
        tool!(
            "looker_get_dashboard",
            "Get a dashboard's metadata, tiles, and filter definitions. Use after list_dashboards or search_content to inspect dashboard contents.",
            params!(
                "dashboard_id" => "Dashboard ID (numeric for user dashboards, slug-like for LookML)"
            ),
            &["dashboard_id"]
        ),

        // ── Queries (the analytics workhorse) ───────────────────────────
        tool!(
            "looker_run_inline_query",
            "Run an ad-hoc Looker analytics query against a LookML model/explore and return rows as JSON. \
             This is the main BI tool — compose model + view + fields + filters + sorts to explore data. \
             Use list_models and get_model_explore first to discover available fields. \
             Row count is capped (default 100, max 5000).",
            params!(
                "model" => "LookML model name (from list_models)",
                "view" => "LookML explore/view name (from get_model_explore)",
                "fields" => "Comma-separated field names (e.g., 'users.id,users.email,orders.count')",
                "filters" => "JSON object of filter expressions, e.g., {\"orders.created_date\":\"7 days\"}",
                "sorts" => "Comma-separated sort expressions (e.g., 'orders.count desc')",
                "limit" => "Row limit (default: 100, max: 5000)",
                "pivots" => "Optional comma-separated pivot field names"
            ),
            &["model", "view", "fields"]
        ),
        tool!(
            "looker_run_query",
            "Run a previously-saved Looker query by ID and return rows as JSON. Use when chaining off get_look (look.query_id) or after create_query.",
            params!(
                "query_id" => "Query ID",
                "limit" => "Row limit override (default: 100, max: 5000)",
                "apply_formatting" => "Apply Looker value formatting (default: false)",
                "cache" => "Use cache (default: true)"
            ),
            &["query_id"]
        ),
        tool!(
            "looker_get_query",
            "Get the definition of a saved query (model, view, fields, filters, sorts). Use to inspect a Look's query before re-running it.",
            params!(
                "query_id" => "Query ID"
            ),
            &["query_id"]
        ),
        tool!(
            "looker_create_query",
            "Create a saved Looker query definition (does not run it). Returns the query_id. Use with run_query when you want to reuse the same query multiple times.",
            params!(
                "model" => "LookML model name",
                "view" => "LookML explore/view name",
                "fields" => "Comma-separated field names",
                "filters" => "JSON object of filter expressions",
                "sorts" => "Comma-separated sort expressions",
                "limit" => "Saved row limit (default: 500)",
                "pivots" => "Optional comma-separated pivot field names"
            ),
            &["model", "view", "fields"]
        ),

        // ── SQL Runner ──────────────────────────────────────────────────
        tool!(
            "looker_run_sql_query",
            "Run a raw SQL query through Looker's SQL Runner against a configured connection. \
             Use for ad-hoc SQL analytics, BI exploration, and bypassing the LookML semantic layer when needed.",
            params!(
                "connection_name" => "Looker connection name (from list_connections)",
                "sql" => "SQL query string"
            ),
            &["connection_name", "sql"]
        ),

        // ── LookML Models ───────────────────────────────────────────────
        tool!(
            "looker_list_models",
            "List all LookML models available in Looker. LookML is Looker's semantic data modeling layer. Use before run_inline_query to discover models and their explores.",
            params!()
        ),
        tool!(
            "looker_get_model",
            "Get a LookML model's metadata and list of explores. Use after list_models to find explore (view) names for run_inline_query.",
            params!(
                "model_name" => "LookML model name"
            ),
            &["model_name"]
        ),
        tool!(
            "looker_get_model_explore",
            "Get a LookML explore's full field metadata (dimensions, measures, filters). Use to discover field names for run_inline_query.",
            params!(
                "model_name" => "LookML model name",
                "explore_name" => "Explore (view) name"
            ),
            &["model_name", "explore_name"]
        ),

        // ── Connections ─────────────────────────────────────────────────
        tool!(
            "looker_list_connections",
            "List database connections configured in Looker. Use to find connection_name for run_sql_query.",
            params!()
        ),
        tool!(
            "looker_get_connection",
            "Get details of a Looker database connection (host, dialect, schema).",
            params!(
                "connection_name" => "Connection name"
            ),
            &["connection_name"]
        ),

        // ── Users ───────────────────────────────────────────────────────
        tool!(
            "looker_get_me",
            "Get the current authenticated Looker user. Use to verify credentials and discover your user_id.",
            params!()
        ),
        tool!(
            "looker_list_users",
            "List Looker users. Use for admin tasks like finding owners of Looks and dashboards.",
            params!(
                "limit" => "Maximum users (default: 25)",
                "offset" => "Pagination offset (default: 0)"
            )
        ),
        tool!(
            "looker_get_user",
            "Get a Looker user's profile, roles, and email.",
            params!(
                "user_id" => "User ID"
            ),
            &["user_id"]
        ),

        // ── Schedules ───────────────────────────────────────────────────
        tool!(
            "looker_list_scheduled_plans",
            "List Looker scheduled plans (recurring email/Slack deliveries of dashboards and Looks).",
            params!(
                "user_id" => "Optional: filter to plans owned by this user",
                "all" => "Set 'true' to list across all users (requires admin)"
            )
        ),
        tool!(
            "looker_get_scheduled_plan",
            "Get a scheduled plan's details (recipients, frequency, content).",
            params!(
                "scheduled_plan_id" => "Scheduled plan ID"
            ),
            &["scheduled_plan_id"]
        ),
    ]
}
