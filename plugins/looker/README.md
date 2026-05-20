# looker

[Looker](https://cloud.google.com/looker) BI integration for Switchboard — query dashboards, Looks, LookML models, ad-hoc analytics, and the SQL Runner from any MCP-compatible client.

## Install

In the Switchboard web UI, **Plugin Marketplace → Add manifest URL**:

```
https://raw.githubusercontent.com/daltoniam/switchboard_plugins/main/manifest.json
```

Then enable the `looker` plugin and configure credentials.

## Credentials

| Key | Required | Plaintext | Example |
|-----|----------|-----------|---------|
| `base_url` | ✅ | yes | `https://your-instance.cloud.looker.com:19999` (with or without `/api/4.0`) |
| `client_id` | ✅ | yes | Looker API3 client_id |
| `client_secret` | ✅ | no  | Looker API3 client_secret |

Create an API3 key from **Admin → Users → Edit user → API3 Keys** in your Looker instance.

## Tools (23)

| Domain | Tools |
|--------|-------|
| Search | `looker_search_content` (entry point) |
| Folders | `looker_list_folders`, `looker_get_folder` |
| Looks | `looker_list_looks`, `looker_get_look`, `looker_run_look` |
| Dashboards | `looker_list_dashboards`, `looker_get_dashboard` |
| Queries | `looker_run_inline_query` (BI entry point), `looker_run_query`, `looker_get_query`, `looker_create_query` |
| SQL Runner | `looker_run_sql_query` |
| Models | `looker_list_models`, `looker_get_model`, `looker_get_model_explore` |
| Connections | `looker_list_connections`, `looker_get_connection` |
| Users | `looker_get_me`, `looker_list_users`, `looker_get_user` |
| Schedules | `looker_list_scheduled_plans`, `looker_get_scheduled_plan` |

All `_run_*` and `_create_*` tools clamp `limit` to a max of **5,000 rows** (default 100) to prevent runaway responses.

## Auth

Uses Looker's [API3 client_id/client_secret](https://cloud.google.com/looker/docs/api-auth) login flow. Tokens are cached in WASM module state with a 30s safety window and transparently refreshed on `401`.

## Build

From the workspace root:

```bash
cargo build --release --target wasm32-wasip1 -p looker-wasm
cp target/wasm32-wasip1/release/looker_wasm.wasm dist/looker.wasm
```
