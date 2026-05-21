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
        // ── Users ────────────────────────────────────────────────────────────
        tool!(
            "clerk_list_users",
            "Search and list Clerk users by email, phone, username, name, or user ID. Start here for Clerk user management, identity lookup, account audit, finding signed-up users, B2C auth debugging, login history, or banned/locked account review. Covers users, accounts, identities, members, customers.",
            params!(
                "limit" => "Maximum users to return (default 10, max 500)",
                "offset" => "Pagination offset (default 0)",
                "order_by" => "Sort field with optional direction, e.g. -created_at, +last_active_at, +last_sign_in_at, +email_address, +username (default -created_at)",
                "query" => "Free-text query matched against email, phone, username, first/last name, user ID",
                "email_address" => "Comma-separated email addresses to filter by (exact match)",
                "phone_number" => "Comma-separated phone numbers to filter by (exact match)",
                "username" => "Comma-separated usernames to filter by (exact match)",
                "user_id" => "Comma-separated user IDs to filter by",
                "external_id" => "Comma-separated external IDs to filter by",
                "organization_id" => "Comma-separated org IDs — return users who belong to any of these organizations",
                "banned" => "Filter to banned users only (true/false)",
                "locked" => "Filter to locked users only (true/false)",
                "last_active_at_since" => "Unix epoch ms — return users active since this time"
            )
        ),
        tool!(
            "clerk_get_user",
            "Get a Clerk user's full profile, including email addresses, phone numbers, external auth accounts, public/private metadata, and timestamps. Use after list_users when inspecting a specific account.",
            params!(
                "user_id" => "Clerk user ID (e.g. user_2abc...)"
            ),
            &["user_id"]
        ),
        tool!(
            "clerk_create_user",
            "Create a new Clerk user with email, phone, username, password, name, or external ID. Optionally attach public/private/unsafe metadata.",
            params!(
                "email_address" => "JSON array string of email addresses, e.g. [\"a@b.com\"]",
                "phone_number" => "JSON array string of phone numbers in E.164 format",
                "web3_wallet" => "JSON array string of Web3 wallet addresses",
                "username" => "Username for the user",
                "password" => "Plain-text password (Clerk will hash)",
                "password_digest" => "Pre-hashed password digest",
                "password_hasher" => "Hash algorithm for password_digest (e.g. bcrypt, argon2i)",
                "first_name" => "User's first name",
                "last_name" => "User's last name",
                "external_id" => "External system ID for this user",
                "skip_password_checks" => "Skip password complexity checks (true/false)",
                "skip_password_requirement" => "Allow user creation without a password (true/false)",
                "public_metadata" => "JSON object string of public metadata",
                "private_metadata" => "JSON object string of private metadata",
                "unsafe_metadata" => "JSON object string of unsafe metadata",
                "created_at" => "Backfill created_at timestamp (RFC3339 or Unix seconds)"
            )
        ),
        tool!(
            "clerk_update_user",
            "Update a Clerk user's profile, primary identifier, metadata, password, or activity flags. Use after list_users or get_user.",
            params!(
                "user_id" => "Clerk user ID",
                "first_name" => "Update first name",
                "last_name" => "Update last name",
                "username" => "Update username",
                "primary_email_address_id" => "ID of the email_address record to mark primary",
                "primary_phone_number_id" => "ID of the phone_number record to mark primary",
                "primary_web3_wallet_id" => "ID of the web3 wallet to mark primary",
                "profile_image_id" => "Image ID to use as the profile picture",
                "password" => "New password (Clerk will hash)",
                "password_digest" => "Pre-hashed password digest",
                "password_hasher" => "Hash algorithm for password_digest",
                "sign_out_of_other_sessions" => "Sign user out of other active sessions after password change (true/false)",
                "external_id" => "External system ID",
                "public_metadata" => "JSON object string of public metadata (replaces existing)",
                "private_metadata" => "JSON object string of private metadata (replaces existing)",
                "unsafe_metadata" => "JSON object string of unsafe metadata (replaces existing)"
            ),
            &["user_id"]
        ),
        tool!(
            "clerk_delete_user",
            "Permanently delete a Clerk user. Use after list_users or get_user when removing an account.",
            params!(
                "user_id" => "Clerk user ID to delete"
            ),
            &["user_id"]
        ),
        tool!(
            "clerk_ban_user",
            "Ban a Clerk user, preventing all future sign-ins. Reversible via unban_user.",
            params!("user_id" => "Clerk user ID to ban"),
            &["user_id"]
        ),
        tool!(
            "clerk_unban_user",
            "Lift a ban on a Clerk user, restoring their ability to sign in.",
            params!("user_id" => "Clerk user ID to unban"),
            &["user_id"]
        ),
        tool!(
            "clerk_lock_user",
            "Lock a Clerk user out of new sign-ins (without banning). Useful for temporary holds during fraud review.",
            params!("user_id" => "Clerk user ID to lock"),
            &["user_id"]
        ),
        tool!(
            "clerk_unlock_user",
            "Unlock a previously locked Clerk user, restoring sign-in.",
            params!("user_id" => "Clerk user ID to unlock"),
            &["user_id"]
        ),
        tool!(
            "clerk_list_user_organization_memberships",
            "List all Clerk organizations a user belongs to, with their role and organization metadata. Use after list_users to map a user to their tenants/workspaces.",
            params!(
                "user_id" => "Clerk user ID",
                "limit" => "Maximum memberships to return (default 10, max 100)",
                "offset" => "Pagination offset"
            ),
            &["user_id"]
        ),

        // ── Sessions ─────────────────────────────────────────────────────────
        tool!(
            "clerk_list_sessions",
            "List Clerk authentication sessions (active sign-ins). Start here for session debugging, auditing who is signed in, revoking suspicious logins, and seeing which clients/devices a user is using.",
            params!(
                "client_id" => "Filter sessions for a specific client/device",
                "user_id" => "Filter sessions for a specific Clerk user",
                "status" => "Filter by session status: abandoned, active, ended, expired, removed, replaced, revoked"
            )
        ),
        tool!(
            "clerk_get_session",
            "Get a Clerk session by ID, including user, client, status, expiry, and latest activity (browser, device, location). Use after list_sessions when debugging a specific sign-in.",
            params!("session_id" => "Clerk session ID (e.g. sess_2abc...)"),
            &["session_id"]
        ),
        tool!(
            "clerk_revoke_session",
            "Revoke a Clerk session, signing the user out of that client/device. Use after list_sessions or get_session for security incident response.",
            params!("session_id" => "Clerk session ID to revoke"),
            &["session_id"]
        ),

        // ── Organizations ────────────────────────────────────────────────────
        tool!(
            "clerk_list_organizations",
            "List or search Clerk organizations (tenants, workspaces, teams, accounts). Start here for B2B tenant management, customer audit, finding which organizations exist, or onboarding/billing reviews.",
            params!(
                "limit" => "Maximum organizations to return (default 10, max 500)",
                "offset" => "Pagination offset (default 0)",
                "include_members_count" => "Include each organization's member count (true/false)",
                "order_by" => "Sort field with optional direction, e.g. -created_at, +name, +members_count (default -created_at)",
                "query" => "Free-text query matched against organization name, slug, or ID",
                "user_id" => "Comma-separated user IDs — return organizations that any of these users belong to"
            )
        ),
        tool!(
            "clerk_get_organization",
            "Get a Clerk organization by ID or slug, including members count, max allowed memberships, creator, metadata, and timestamps. Use after list_organizations.",
            params!(
                "organization_id" => "Organization ID (e.g. org_2abc...) or slug",
                "include_members_count" => "Include the organization's member count (true/false)"
            ),
            &["organization_id"]
        ),
        tool!(
            "clerk_create_organization",
            "Create a new Clerk organization (tenant, workspace, team) with a name, optional slug, creator user, and metadata.",
            params!(
                "name" => "Organization display name",
                "created_by" => "Clerk user ID who will be the initial owner",
                "slug" => "URL-safe slug (auto-generated if omitted)",
                "max_allowed_memberships" => "Cap on total members (0 or omit for unlimited)",
                "public_metadata" => "JSON object string of public metadata",
                "private_metadata" => "JSON object string of private metadata"
            ),
            &["name", "created_by"]
        ),
        tool!(
            "clerk_update_organization",
            "Update a Clerk organization's name, slug, member cap, or metadata. Use after list_organizations or get_organization.",
            params!(
                "organization_id" => "Organization ID or slug",
                "name" => "New organization name",
                "slug" => "New URL-safe slug",
                "max_allowed_memberships" => "New cap on total members (0 for unlimited)",
                "admin_delete_enabled" => "Allow admins to delete the organization (true/false)",
                "public_metadata" => "JSON object string of public metadata (replaces existing)",
                "private_metadata" => "JSON object string of private metadata (replaces existing)"
            ),
            &["organization_id"]
        ),
        tool!(
            "clerk_delete_organization",
            "Permanently delete a Clerk organization. Removes all memberships and invitations.",
            params!("organization_id" => "Organization ID or slug to delete"),
            &["organization_id"]
        ),
        tool!(
            "clerk_list_organization_memberships",
            "List members of a Clerk organization with their role and user data. Use after list_organizations to see who belongs to a tenant/workspace/team.",
            params!(
                "organization_id" => "Organization ID or slug",
                "limit" => "Maximum memberships to return (default 10, max 500)",
                "offset" => "Pagination offset",
                "order_by" => "Sort field, e.g. -created_at, +last_active_at, +first_name"
            ),
            &["organization_id"]
        ),
        tool!(
            "clerk_create_organization_membership",
            "Add an existing Clerk user to an organization with a role. Use to invite/onboard team members programmatically when they already have an account.",
            params!(
                "organization_id" => "Organization ID or slug",
                "user_id" => "Clerk user ID to add",
                "role" => "Role for the new member (e.g. admin, basic_member, or a custom role key)"
            ),
            &["organization_id", "user_id", "role"]
        ),
        tool!(
            "clerk_update_organization_membership",
            "Change a Clerk organization member's role (e.g. promote to admin).",
            params!(
                "organization_id" => "Organization ID or slug",
                "user_id" => "Clerk user ID of the member",
                "role" => "New role key"
            ),
            &["organization_id", "user_id", "role"]
        ),
        tool!(
            "clerk_delete_organization_membership",
            "Remove a user from a Clerk organization. Use after list_organization_memberships.",
            params!(
                "organization_id" => "Organization ID or slug",
                "user_id" => "Clerk user ID of the member to remove"
            ),
            &["organization_id", "user_id"]
        ),
        tool!(
            "clerk_list_organization_invitations",
            "List pending, accepted, or revoked invitations to a Clerk organization. Use to audit outstanding invites before resending or revoking.",
            params!(
                "organization_id" => "Organization ID or slug",
                "limit" => "Maximum invitations to return (default 10, max 500)",
                "offset" => "Pagination offset",
                "status" => "Comma-separated statuses to filter: pending, accepted, revoked"
            ),
            &["organization_id"]
        ),
        tool!(
            "clerk_create_organization_invitation",
            "Invite a user by email to join a Clerk organization with a role. Sends an email and creates a pending invitation.",
            params!(
                "organization_id" => "Organization ID or slug",
                "email_address" => "Recipient email address",
                "role" => "Role to grant on acceptance (e.g. admin, basic_member, or a custom role key)",
                "inviter_user_id" => "Clerk user ID of the inviter (shown in the invitation email)",
                "redirect_url" => "URL to send the recipient to after accepting",
                "public_metadata" => "JSON object string of public metadata",
                "private_metadata" => "JSON object string of private metadata"
            ),
            &["organization_id", "email_address", "role"]
        ),
        tool!(
            "clerk_revoke_organization_invitation",
            "Revoke a pending Clerk organization invitation. Use after list_organization_invitations.",
            params!(
                "organization_id" => "Organization ID or slug",
                "invitation_id" => "Invitation ID to revoke",
                "requesting_user_id" => "Clerk user ID performing the revoke (for audit)"
            ),
            &["organization_id", "invitation_id"]
        ),

        // ── Invitations (instance-level) ─────────────────────────────────────
        tool!(
            "clerk_list_invitations",
            "List Clerk instance-level invitations (not tied to an organization). Start here for auditing outstanding sign-up invites sent to email addresses.",
            params!(
                "limit" => "Maximum invitations to return (default 10, max 500)",
                "offset" => "Pagination offset",
                "status" => "Filter by status: pending, accepted, revoked",
                "query" => "Free-text query matched against the invitation email address",
                "order_by" => "Sort field, e.g. -created_at, -updated_at"
            )
        ),
        tool!(
            "clerk_create_invitation",
            "Send a Clerk instance-level invitation to an email address. The recipient gets a sign-up link.",
            params!(
                "email_address" => "Recipient email address",
                "redirect_url" => "URL to send the recipient to after accepting",
                "notify" => "Send the invitation email (true/false, default true)",
                "ignore_existing" => "Don't error if an invitation already exists for this email (true/false)",
                "expires_in_days" => "Invitation lifetime in days",
                "template_slug" => "Specific invitation email template slug to use",
                "public_metadata" => "JSON object string of public metadata"
            ),
            &["email_address"]
        ),
        tool!(
            "clerk_revoke_invitation",
            "Revoke a pending Clerk instance-level invitation. Use after list_invitations.",
            params!("invitation_id" => "Invitation ID to revoke"),
            &["invitation_id"]
        ),

        // ── Allow / Block list ───────────────────────────────────────────────
        tool!(
            "clerk_list_allowlist_identifiers",
            "List identifiers (emails, phones, domains) on the Clerk sign-up allow list. Start here when auditing who is allowed to register for the application.",
            params!()
        ),
        tool!(
            "clerk_create_allowlist_identifier",
            "Add an email address, phone number, or domain (e.g. @example.com) to the Clerk sign-up allow list.",
            params!(
                "identifier" => "Email address, E.164 phone number, or @domain to allow",
                "notify" => "Send a notification to the identifier when added (true/false)"
            ),
            &["identifier"]
        ),
        tool!(
            "clerk_delete_allowlist_identifier",
            "Remove an identifier from the Clerk sign-up allow list.",
            params!("identifier_id" => "Allowlist identifier ID returned by list_allowlist_identifiers"),
            &["identifier_id"]
        ),
        tool!(
            "clerk_list_blocklist_identifiers",
            "List identifiers (emails, phones, domains) on the Clerk sign-up block list. Start here when auditing which addresses are blocked from registering.",
            params!()
        ),
        tool!(
            "clerk_create_blocklist_identifier",
            "Add an email address, phone number, or domain (e.g. @example.com) to the Clerk sign-up block list.",
            params!("identifier" => "Email address, E.164 phone number, or @domain to block"),
            &["identifier"]
        ),
        tool!(
            "clerk_delete_blocklist_identifier",
            "Remove an identifier from the Clerk sign-up block list.",
            params!("identifier_id" => "Blocklist identifier ID returned by list_blocklist_identifiers"),
            &["identifier_id"]
        ),
    ]
}
