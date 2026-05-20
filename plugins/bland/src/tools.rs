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
        tool!(
            "bland_list_calls",
            "List Bland.ai voice AI phone calls with status, phone numbers, duration, completion, errors, and transcript availability. Start here for call history, call logs, outbound calls, inbound calls, conversations, and voice agent debugging.",
            params!(
                "limit" => "Maximum calls to return (default 20, max 100)",
                "from" => "Pagination start index (inclusive)",
                "to" => "Pagination end index (exclusive)",
                "ascending" => "Sort ascending instead of descending (true/false)",
                "sort_by" => "Sort field: created_at or updated_at (default created_at)",
                "start_date" => "Filter calls created on or after this ISO date/time",
                "end_date" => "Filter calls created on or before this ISO date/time",
                "batch_id" => "Filter by Bland batch ID",
                "answered_by" => "Filter by answered_by result",
                "inbound" => "Filter inbound calls (true/false)",
                "completed" => "Filter completed calls (true/false)",
                "from_number" => "Filter by originating phone number",
                "to_number" => "Filter by destination phone number",
                "duration_gt" => "Filter calls longer than this duration in seconds",
                "duration_lt" => "Filter calls shorter than this duration in seconds"
            )
        ),
        tool!(
            "bland_list_active_calls",
            "List currently active live Bland.ai calls. Use for monitoring in-progress phone conversations before stopping or inspecting them.",
            params!()
        ),
        tool!(
            "bland_get_call",
            "Get details for a specific Bland.ai call, including status, summary, transcript, corrected transcript fields, analysis, variables, metadata, and conversation timing. Use after list_calls when debugging a call or reading a transcript.",
            params!(
                "call_id" => "Bland call_id returned by list_calls or send_call"
            ),
            &["call_id"]
        ),
        tool!(
            "bland_send_call",
            "Send or schedule an outbound Bland.ai AI phone call. Creates a voice agent call using either a task prompt or pathway_id, with optional voice, first sentence, metadata, webhook, transfer number, and dynamic variables.",
            params!(
                "phone_number" => "Destination phone number in E.164 format (for example +15551234567)",
                "task" => "Plain-language voice agent task/instructions. Required unless pathway_id is provided",
                "pathway_id" => "Existing Bland pathway ID. Required unless task is provided",
                "voice" => "Voice ID or voice name",
                "first_sentence" => "Opening sentence the AI should say",
                "model" => "Bland model name",
                "language" => "Language code (for example en-US)",
                "from" => "Bland phone number to call from",
                "webhook" => "Webhook URL for call events/results",
                "metadata" => "JSON object string attached to the call",
                "dynamic_data" => "JSON object string for pathway/task variables",
                "tools" => "JSON array string of Bland tool definitions",
                "transfer_phone_number" => "Phone number to transfer to",
                "timezone" => "Timezone for scheduling and context",
                "max_duration" => "Maximum call duration in minutes",
                "wait_for_greeting" => "Wait for recipient greeting before speaking (true/false)",
                "record" => "Whether to record the call (true/false)",
                "temperature" => "Model temperature",
                "interruption_threshold" => "Interruption sensitivity threshold",
                "background_track" => "Background audio track name"
            ),
            &["phone_number"]
        ),
        tool!(
            "bland_stop_call",
            "Stop an active Bland.ai phone call. Use after list_active_calls or get_call when a live voice conversation should be ended.",
            params!(
                "call_id" => "Bland call_id for the active call to stop"
            ),
            &["call_id"]
        ),
        tool!(
            "bland_analyze_call",
            "Analyze a completed Bland.ai call transcript with AI using a goal and structured questions. Use after get_call to extract outcomes, classifications, lead qualification, sentiment, or custom fields.",
            params!(
                "call_id" => "Bland call_id to analyze",
                "goal" => "Analysis goal or rubric",
                "questions" => "JSON array of question objects or strings to answer from the call"
            ),
            &["call_id", "goal", "questions"]
        ),
        tool!(
            "bland_list_voices",
            "List Bland.ai voices for text-to-speech and phone calls, including voice IDs, names, descriptions, tags, public/private status, and ratings. Start here for choosing a call voice.",
            params!()
        ),
        tool!(
            "bland_get_voice",
            "Get details for a specific Bland.ai voice. Use after list_voices to inspect a voice before using it in send_call.",
            params!(
                "voice_id" => "Bland voice ID"
            ),
            &["voice_id"]
        ),
        tool!(
            "bland_list_pathways",
            "List Bland.ai conversational pathways and voice agent flows. Start here for discovering pathway IDs to use when sending calls.",
            params!()
        ),
        tool!(
            "bland_get_pathway",
            "Get a Bland.ai pathway's full configuration. Use after list_pathways to inspect a conversational flow before sending calls with pathway_id.",
            params!(
                "pathway_id" => "Bland pathway ID"
            ),
            &["pathway_id"]
        ),
        tool!(
            "bland_list_numbers",
            "List Bland.ai inbound phone numbers configured on the account. Start here for phone number inventory, inbound agents, and call routing setup.",
            params!(
                "encrypted_key" => "Optional encrypted key filter used by Bland for number lookup"
            )
        ),
        tool!(
            "bland_get_number",
            "Get details for a specific Bland.ai inbound phone number, including inbound agent and routing configuration. Use after list_numbers.",
            params!(
                "phone_number" => "Inbound phone number to inspect, usually in E.164 format"
            ),
            &["phone_number"]
        ),
        tool!(
            "bland_list_knowledge_bases",
            "List Bland.ai knowledge bases used by voice agents and pathways. Start here for discovering retrieval sources available to calls.",
            params!(
                "page" => "Page number (default 1)",
                "limit" => "Maximum knowledge bases per page (default 20)"
            )
        ),
        tool!(
            "bland_get_knowledge_base",
            "Get a Bland.ai knowledge base's details and documents. Use after list_knowledge_bases to inspect retrieval content available to voice agents.",
            params!(
                "knowledge_base_id" => "Bland knowledge base ID"
            ),
            &["knowledge_base_id"]
        ),
        tool!(
            "bland_get_me",
            "Get the current Bland.ai account details, billing balance, and total call count. Start here for account management and credential verification.",
            params!()
        ),
        tool!(
            "bland_create_org",
            "Create a new Bland.ai organization/workspace. Use for account and org management setup.",
            params!(
                "name" => "Organization display name"
            ),
            &["name"]
        ),
        tool!(
            "bland_get_org",
            "Get a Bland.ai organization's details, slug, plan, preferences, entitlements, rate limit, suspension, and deletion status. Use after list_my_org_memberships.",
            params!(
                "org_id" => "Bland organization ID"
            ),
            &["org_id"]
        ),
        tool!(
            "bland_delete_org",
            "Delete a Bland.ai organization. Requires delete_confirm with the organization slug for safety.",
            params!(
                "org_id" => "Bland organization ID",
                "delete_confirm" => "Organization slug confirmation required by Bland"
            ),
            &["org_id", "delete_confirm"]
        ),
        tool!(
            "bland_update_org_properties",
            "Update Bland.ai organization properties such as display name and preferences including use_bland_url and recording retention lifespan. Use after get_org.",
            params!(
                "org_id" => "Bland organization ID",
                "org_display_name" => "New organization display name (1-30 characters)",
                "use_bland_url" => "Whether to use Bland-hosted URLs (true/false)",
                "recording_lifespan_days" => "Recording retention in days (1-1825, or -1 to disable)",
                "preferences" => "JSON object string for advanced preferences override"
            ),
            &["org_id"]
        ),
        tool!(
            "bland_list_org_members",
            "List Bland.ai organization members with emails, phone numbers, permissions, owner/admin/operator/viewer roles, join dates, and org metadata. Start here for org user management.",
            params!(
                "org_id" => "Bland organization ID"
            ),
            &["org_id"]
        ),
        tool!(
            "bland_update_org_members",
            "Add or remove Bland.ai organization members and invites. Use for org member management after list_org_members.",
            params!(
                "org_id" => "Bland organization ID",
                "action" => "Member action: add or remove",
                "target" => "Target user ID",
                "permissions" => "JSON array of permissions for add: owner, admin, operator, viewer",
                "is_invite" => "Whether removal applies to an invite (true/false)"
            ),
            &["org_id", "action", "target"]
        ),
        tool!(
            "bland_update_org_member_permissions",
            "Update Bland.ai organization member permissions and roles. Supports add, remove, reset, or set permissions: owner, admin, operator, viewer. Use after list_org_members.",
            params!(
                "org_id" => "Bland organization ID",
                "action" => "Permission action: add, remove, reset, or set",
                "target" => "Target user ID",
                "permissions" => "JSON array of permissions: owner, admin, operator, viewer"
            ),
            &["org_id", "action", "target", "permissions"]
        ),
        tool!(
            "bland_list_my_org_memberships",
            "List Bland.ai organizations the current user belongs to, including org IDs, slugs, display names, permissions, owner status, and join dates. Start here to discover org_id values for management tools.",
            params!()
        ),
        tool!(
            "bland_leave_org",
            "Leave a Bland.ai organization as the current user. Use after list_my_org_memberships.",
            params!(
                "org_id" => "Bland organization ID to leave"
            ),
            &["org_id"]
        ),
        tool!(
            "bland_get_org_billing",
            "Get Bland.ai organization billing information including current balance, refill amount, and refill threshold. Start here for org billing management.",
            params!(
                "org_id" => "Bland organization ID"
            ),
            &["org_id"]
        ),
        tool!(
            "bland_get_org_billing_refill",
            "Get Bland.ai organization billing refill threshold information. Use after get_org_billing.",
            params!(
                "org_id" => "Bland organization ID"
            ),
            &["org_id"]
        ),
        tool!(
            "bland_get_org_current_version",
            "Get the current Bland.ai service version for an organization. Supports api_server and ws_server services.",
            params!(
                "org_id" => "Bland organization ID",
                "service" => "Service name: api_server (default) or ws_server"
            ),
            &["org_id"]
        ),
        tool!(
            "bland_list_org_versions",
            "List available Bland.ai service versions for an organization, including support status and recommended upgrades. Supports api_server and ws_server services.",
            params!(
                "org_id" => "Bland organization ID",
                "service" => "Service name: api_server (default) or ws_server"
            ),
            &["org_id"]
        ),
        tool!(
            "bland_update_org_version",
            "Update a Bland.ai organization's service version. Supports api_server and ws_server services. Use after list_org_versions.",
            params!(
                "org_id" => "Bland organization ID",
                "service" => "Service name: api_server (default) or ws_server",
                "version" => "Version identifier to switch to"
            ),
            &["org_id", "version"]
        ),
        tool!(
            "bland_list_audit_logs",
            "List Bland.ai audit logs for enterprise org compliance, security, admin activity, pathway changes, knowledge base updates, and SSO events. Start here for compliance and audit investigations.",
            params!(
                "event_type" => "Optional exact event type filter",
                "actor_id" => "Optional user ID filter",
                "created_after" => "Optional ISO 8601 lower bound",
                "created_before" => "Optional ISO 8601 upper bound",
                "page" => "Page number (default 1)",
                "page_size" => "Page size (default 50, max 100)"
            )
        ),
    ]
}
