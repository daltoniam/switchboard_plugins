# Bland.ai Switchboard plugin

Bland.ai voice AI integration for calls, transcripts, voices, pathways, inbound numbers, knowledge bases, account details, org management, billing, service versions, and audit logs.

## Credentials

| Key | Required | Description |
|-----|----------|-------------|
| `api_key` | yes | Bland.ai API key used as the `Authorization` header |
| `org_id` | no | Optional organization ID sent as `x-bland-org-id` for org-scoped endpoints |

## Tools

| Tool | Description |
|------|-------------|
| `bland_list_calls` | List call history and filter by date, number, status, batch, direction, or duration |
| `bland_list_active_calls` | List currently active calls |
| `bland_get_call` | Get call details, summary, transcript, analysis, variables, and metadata |
| `bland_send_call` | Start an outbound AI phone call using `task` or `pathway_id` |
| `bland_stop_call` | Stop an active call |
| `bland_analyze_call` | Run AI analysis over a completed call transcript |
| `bland_list_voices` | List available voices |
| `bland_get_voice` | Get voice details |
| `bland_list_pathways` | List conversational pathways |
| `bland_get_pathway` | Get pathway configuration |
| `bland_list_numbers` | List inbound phone numbers |
| `bland_get_number` | Get inbound number details |
| `bland_list_knowledge_bases` | List knowledge bases |
| `bland_get_knowledge_base` | Get knowledge base details |
| `bland_get_me` | Get account details, balance, and total call count |
| `bland_create_org` | Create an organization |
| `bland_get_org` | Get organization details |
| `bland_delete_org` | Delete an organization with slug confirmation |
| `bland_update_org_properties` | Update organization display name and preferences |
| `bland_list_org_members` | List organization members and permissions |
| `bland_update_org_members` | Add or remove organization members/invites |
| `bland_update_org_member_permissions` | Add, remove, reset, or set member permissions |
| `bland_list_my_org_memberships` | List organizations for the current user |
| `bland_leave_org` | Leave an organization |
| `bland_get_org_billing` | Get organization billing balance and refill settings |
| `bland_get_org_billing_refill` | Get organization billing refill threshold |
| `bland_get_org_current_version` | Get current org service version for `api_server` or `ws_server` |
| `bland_list_org_versions` | List org service versions for `api_server` or `ws_server` |
| `bland_update_org_version` | Update org service version |
| `bland_list_audit_logs` | List enterprise audit log events |
