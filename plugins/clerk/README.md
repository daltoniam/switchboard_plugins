# Clerk Switchboard plugin

Clerk authentication and identity management integration — users, sessions, organizations, memberships, invitations, and allow/block list identifiers via the Clerk Backend API.

## Credentials

| Key | Required | Description |
|-----|----------|-------------|
| `secret_key` | yes | Clerk Backend API secret key (`sk_test_...` or `sk_live_...`) from the Clerk Dashboard under API Keys → Secret keys. Sent as `Authorization: Bearer <secret_key>`. |

## Tools

### Users

| Tool | Description |
|------|-------------|
| `clerk_list_users` | Search and list Clerk users by email, phone, username, or user ID; filter by organization, banned/locked state, last activity |
| `clerk_get_user` | Get a user's full profile, email addresses, phone numbers, external accounts, metadata, and timestamps |
| `clerk_create_user` | Create a new Clerk user with email, phone, username, password, name, and metadata |
| `clerk_update_user` | Update a user's profile, metadata, password, or primary identifier |
| `clerk_delete_user` | Permanently delete a Clerk user |
| `clerk_ban_user` | Ban a user, preventing all sign-ins |
| `clerk_unban_user` | Lift a ban on a user |
| `clerk_lock_user` | Lock a user out of new sign-ins |
| `clerk_unlock_user` | Unlock a previously locked user |
| `clerk_list_user_organization_memberships` | List all organizations a user belongs to with their role |

### Sessions

| Tool | Description |
|------|-------------|
| `clerk_list_sessions` | List Clerk sessions filtered by user, client, or status |
| `clerk_get_session` | Get session details including user, status, expiry, and last activity |
| `clerk_revoke_session` | Revoke a session, signing the user out of that client |

### Organizations

| Tool | Description |
|------|-------------|
| `clerk_list_organizations` | List or search Clerk organizations (tenants, workspaces, teams) |
| `clerk_get_organization` | Get organization details by ID or slug |
| `clerk_create_organization` | Create a new organization with a name, slug, and creator user |
| `clerk_update_organization` | Update an organization's name, slug, max memberships, or metadata |
| `clerk_delete_organization` | Permanently delete an organization |
| `clerk_list_organization_memberships` | List members of an organization with their role |
| `clerk_create_organization_membership` | Add a user to an organization with a role |
| `clerk_update_organization_membership` | Change a member's role within an organization |
| `clerk_delete_organization_membership` | Remove a user from an organization |
| `clerk_list_organization_invitations` | List pending/accepted/revoked invitations for an organization |
| `clerk_create_organization_invitation` | Invite a user (by email) to an organization with a role |
| `clerk_revoke_organization_invitation` | Revoke a pending organization invitation |

### Invitations (instance-level)

| Tool | Description |
|------|-------------|
| `clerk_list_invitations` | List instance-level invitations sent to email addresses |
| `clerk_create_invitation` | Send an instance-level invitation to an email address |
| `clerk_revoke_invitation` | Revoke a pending instance-level invitation |

### Allow / Block list

| Tool | Description |
|------|-------------|
| `clerk_list_allowlist_identifiers` | List identifiers (emails, phones, domains) allowed to sign up |
| `clerk_create_allowlist_identifier` | Add an identifier to the sign-up allow list |
| `clerk_delete_allowlist_identifier` | Remove an identifier from the sign-up allow list |
| `clerk_list_blocklist_identifiers` | List identifiers blocked from signing up |
| `clerk_create_blocklist_identifier` | Add an identifier to the sign-up block list |
| `clerk_delete_blocklist_identifier` | Remove an identifier from the sign-up block list |
