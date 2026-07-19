# Mailwoman — Admin panel strings (source locale: en, SPEC §19/§21/§22).
# The admin screen is code-split and reached only via lazy(import); its catalog is
# lazily loaded with it. Ids are module-prefixed `admin-*`.

# -- Shell / nav -------------------------------------------------------------
admin-brand = Mailwoman admin
admin-nav = Admin sections
admin-sign-out = Sign out
admin-nav-domains = Domains
admin-nav-users = Users
admin-nav-security = Security policy
admin-nav-integrations = Integrations
admin-nav-observability = Observability
admin-nav-appearance = Appearance
admin-nav-plugins = Plugins
admin-nav-assist = Assist
admin-nav-sso = Single sign-on
admin-nav-servermeta = Server metadata
admin-nav-rethread = Re-thread mailbox
admin-nav-2fa = Require two-factor

# Shared admin actions / states
admin-delete = Delete
admin-revoke = Revoke
admin-remove = Remove
admin-saved = Saved.

# -- Sign-in gate (separate admin session) -----------------------------------
admin-login-form = Admin sign in
admin-login-note = This panel runs under a separate admin session.
admin-login-username = Admin username
admin-login-password = Password
admin-login-sign-in = Sign in
admin-login-signing-in = Signing in…
admin-login-invalid = Invalid admin credentials
admin-login-unreachable = Could not reach the server

# -- Domains -----------------------------------------------------------------
admin-domains-title = Domains
admin-domains-load-error = Could not load domains
admin-domains-save-error = Could not save the domain
admin-domains-delete-error = Could not delete the domain
admin-domains-add = Add domain
admin-domains-name = Domain name
admin-domains-name-placeholder = example.com
admin-domains-upstream = Upstream (JSON)
admin-domains-allowlist = Allowlist
admin-domains-blocklist = Blocklist
admin-domains-one-per-line = one per line
admin-domains-save = Save domain
admin-domains-empty = No domains yet.
admin-domains-counts = ({ $allow } allow / { $block } block)
admin-domains-delete-for = Delete { $name }

# -- Users -------------------------------------------------------------------
admin-users-title = Users
admin-users-load-error = Could not load users
admin-users-provision-error = Could not provision the user
admin-users-flag-error = Could not update the flag
admin-users-revoke-error = Could not revoke sessions
admin-users-provision = Provision user
admin-users-username = Username
admin-users-domain = Domain
admin-users-domain-placeholder = example.com
admin-users-quota-bytes = Quota bytes (0 = unlimited)
admin-users-quota-msgs = Quota messages (0 = unlimited)
admin-users-empty = No users yet.
admin-users-col-account = Account
admin-users-col-quota = Quota (bytes/msgs)
admin-users-col-zeroaccess = Zero-access
admin-users-col-flags = Flags
admin-users-col-sessions = Sessions
admin-users-zeroaccess-for = Zero-access for { $account }
admin-users-disable-for = Disable { $account }
admin-users-disabled = disabled
admin-users-force-change-for = Force password change for { $account }
admin-users-force-change = force change
admin-users-cache-wipe-for = Remote cache wipe for { $account }
admin-users-cache-wipe = cache wipe
admin-users-revoke-for = Revoke sessions for { $account }

# -- Security policy ---------------------------------------------------------
admin-security-title = Security policy
admin-security-load-error = Could not load the security policy
admin-security-save-error = Could not save the security policy
admin-security-min-tls = Minimum TLS
admin-security-capture = Capture policy
admin-security-argon-mem = Argon2 memory cost (KiB)
admin-security-argon-time = Argon2 time cost
admin-security-argon-par = Argon2 parallelism
admin-security-dlp = DLP rules (JSON)
admin-security-require-2fa-label = Require two-factor authentication
admin-security-require-2fa = Require 2FA
admin-security-floor-label = Enforce maximum-security floor
admin-security-floor = Enforce maximum-security floor
admin-security-save = Save policy

# -- Integrations ------------------------------------------------------------
admin-integrations-title = Integrations
admin-integrations-load-error = Could not load integrations
admin-integrations-revoke-error = Could not revoke the key
admin-integrations-ldap = LDAP / GAL directory
admin-integrations-nextcloud = Nextcloud bridge
admin-integrations-deferred = Deferred
admin-integrations-active = Active
admin-integrations-deferred-note = LDAP and Nextcloud are configuration surfaces only in this release; they are not yet wired.
admin-integrations-webhooks = Webhooks
admin-integrations-webhooks-empty = No webhooks registered.
admin-integrations-keys = API & MCP keys
admin-integrations-keys-empty = No keys issued.
admin-integrations-col-prefix = Prefix
admin-integrations-col-account = Account
admin-integrations-col-scopes = Scopes
admin-integrations-col-status = Status
admin-integrations-status-revoked = revoked
admin-integrations-status-active = active
admin-integrations-revoke-key = Revoke key { $prefix }

# -- Observability -----------------------------------------------------------
admin-obs-title = Observability
admin-obs-load-error = Could not load observability data
admin-obs-save-error = Could not save observability config
admin-obs-export-error = Could not export the audit log
admin-obs-ban-add-error = Could not add the ban
admin-obs-unban-error = Could not remove the ban
admin-obs-config = Logging and telemetry
admin-obs-log-level = Log level
admin-obs-otlp = OTLP DSN
admin-obs-otlp-placeholder = https://otlp.example.org
admin-obs-metrics-label = Enable Prometheus metrics endpoint
admin-obs-metrics = Enable auth-gated Prometheus /metrics
admin-obs-save = Save telemetry
admin-obs-audit = Audit log
admin-obs-export = Export JSONL
admin-obs-audit-empty = No audit entries.
admin-obs-col-time = Time
admin-obs-col-actor = Actor
admin-obs-col-action = Action
admin-obs-col-target = Target
admin-obs-bans = Login monitor / ban list
admin-obs-ban-add = Add ban
admin-obs-ban-ip = IP address
admin-obs-ban-reason = Reason
admin-obs-ban-btn = Ban IP
admin-obs-bans-empty = No active bans.
admin-obs-unban-for = Unban { $ip }
admin-obs-unban-btn = Unban

# -- Appearance (deployment default) -----------------------------------------
admin-appearance-title = Appearance
admin-appearance-load-error = Could not load appearance
admin-appearance-save-error = Could not save appearance
admin-appearance-brand = Brand name
admin-appearance-theme = Default theme
admin-appearance-accent = Accent (hex, optional)
admin-appearance-accent-placeholder = #6d8a4e
admin-appearance-save = Save appearance

# -- Plugins (§22) -----------------------------------------------------------
# NB: the unsigned-plugin banner copy is a FROZEN, exported const (UNSIGNED_BANNER
# in Plugins/index.tsx) referenced by tests and the security model — not localised.
admin-plugins-title = Plugins
admin-plugins-intro = Engine plugins run in a capability-gated WebAssembly sandbox. Approve a plugin before it can be enabled, and grant only the capabilities it needs.
admin-plugins-empty = No plugins are registered.
admin-plugins-signed = Signed
admin-plugins-unsigned = Unsigned
admin-plugins-approved = Approved
admin-plugins-enabled = Enabled
admin-plugins-approve = Approve
admin-plugins-enable = Enable
admin-plugins-disable = Disable
admin-plugins-net = net: { $hosts }
admin-plugins-limits = limits: { $memory } MiB · { $deadline } ms
admin-plugins-limits-fuel = limits: { $memory } MiB · { $deadline } ms · { $fuel } fuel
admin-plugins-allow-unsigned-for = Allow unsigned plugin { $name }
admin-plugins-allow-unsigned = Allow this unsigned plugin to run
admin-plugins-version = v{ $version }

# -- Third-party plugin allowlist (§7.2 / t15 26.15) -------------------------
# The trust surface for loading non-first-party components. An operator drops a
# <id>.wasm into the third-party plugin directory; the server computes its SHA-256
# and shows it here. An admin approves that exact digest to let it load — nothing
# else does. Copy is factual: this is a security action, neither alarmist nor
# reassuring-marketing.
admin-allowlist-title = Third-party plugin allowlist
admin-allowlist-intro = A third-party (non-first-party) component loads only after an administrator approves its exact SHA-256 digest. The digest below is computed by the server over the component's bytes on disk; approving it pins those exact bytes. First-party components are pinned in the build and are not managed here.
admin-allowlist-load-error = Could not load the plugin allowlist.
admin-allowlist-present-heading = Components on disk
admin-allowlist-present-empty = No third-party components are present. Place a component in the third-party plugin directory for it to appear here.
admin-allowlist-digest-label = Computed SHA-256
admin-allowlist-status-approved = Approved
admin-allowlist-status-pending = Not approved
admin-allowlist-status-firstparty = First-party
# A component approved by digest without a signature is expected — a neutral note,
# not a warning. The digest pin is what authorizes loading.
admin-allowlist-unsigned-note = Admitted by digest pin. This component carries no signature; approval trusts the exact bytes whose digest is shown, which is the expected posture for a component approved this way.
# High-power capabilities are refused to third-party plugins at grant time by the
# server, regardless of admin action. Surfaced so an admin is not surprised by a
# rejected grant.
admin-allowlist-highpower-note = High-power capabilities ({ $caps }) cannot be granted to a third-party plugin. The server refuses them regardless of approval; they are available to first-party components only.
admin-allowlist-firstparty-note = This id matches a first-party component. The first-party pin always takes precedence, so this id cannot be approved as third-party.
admin-allowlist-approve = Approve digest
admin-allowlist-approve-for = Approve digest for { $id }
admin-allowlist-revoke = Revoke
admin-allowlist-revoke-for = Revoke pin for { $id }
admin-allowlist-uninstall = Uninstall
admin-allowlist-uninstall-for = Uninstall { $id }
admin-allowlist-pins-heading = Approved and revoked pins
admin-allowlist-pins-empty = No pins recorded.
admin-allowlist-pin-approved-by = Approved by { $by } on { $at }
admin-allowlist-pin-revoked = Revoked
admin-allowlist-cancel = Cancel

# Approve confirmation (shows the exact digest being trusted).
admin-allowlist-approve-title = Approve this component to load?
admin-allowlist-approve-detail = Approving pins the exact bytes whose SHA-256 is shown below. After approval, only bytes matching this digest will load for this id; any change to the component produces a different digest and will not load until re-approved. Approval grants no capabilities on its own.
admin-allowlist-approve-confirm = Approve digest

# Revoke confirmation.
admin-allowlist-revoke-title = Revoke this pin?
admin-allowlist-revoke-detail = Revoking removes approval for this digest and disables the plugin. It takes effect on the next load; an already-running instance is not stopped until then.
admin-allowlist-revoke-confirm = Revoke pin

# Uninstall confirmation (clear about what it deletes).
admin-allowlist-uninstall-title = Uninstall this plugin?
admin-allowlist-uninstall-detail = Uninstalling deletes the plugin's stored key/value data for every account, removes its allowlist pins, and disables it. The component file on disk is not deleted; it can be re-approved later.
admin-allowlist-uninstall-confirm = Uninstall plugin

# -- Assist governance (§14/§19) ---------------------------------------------
admin-assist-title = Assist
admin-assist-intro = Assist proxies selected message text to an AI endpoint you configure. It never sends, deletes, or accepts mail on a user's behalf — those always require a person. End-to-end-encrypted content and attachments are withheld unless you explicitly allow them below.
admin-assist-enable = Enable Assist tenant-wide
admin-assist-enabled = Assist enabled tenant-wide
admin-assist-off-note = Assist is off. The kill switch reports the gateway as disabled to every user.
admin-assist-allowlist = Endpoint allowlist
admin-assist-allowlist-note = Only these hosts may receive proxied requests. Anything else is refused.
admin-assist-host = Endpoint host
admin-assist-host-placeholder = api.openai.com
admin-assist-add-host = Add host
admin-assist-hosts-empty = No hosts yet.
admin-assist-remove-host = Remove { $host }
admin-assist-locks = Capability locks
admin-assist-locks-note = A locked capability is never offered, regardless of per-user grants.
admin-assist-locked = Locked
admin-assist-ceilings = Data-class ceilings
admin-assist-ceilings-note = These are hard limits. Even a granted capability cannot exceed them. Both are off by default.
admin-assist-allow-e2ee = Allow end-to-end-encrypted content to leave the deployment
admin-assist-allow-e2ee-label = Allow end-to-end-encrypted content to be sent
admin-assist-allow-attachments = Allow attachments to leave the deployment
admin-assist-allow-attachments-label = Allow attachments to be sent
admin-assist-save = Save policy
admin-assist-enabled-status = Assist enabled.
admin-assist-disabled-status = Assist disabled tenant-wide (kill switch).
admin-assist-kill-error = Could not change the kill switch.
admin-assist-save-error = Save failed.
admin-assist-load-error = Could not load Assist policy.
# Capability labels
admin-assist-cap-summarize = Summarize
admin-assist-cap-draft = Draft & rewrite
admin-assist-cap-grammar = Grammar
admin-assist-cap-dictation = Dictation
admin-assist-cap-search-semantic = Semantic search
admin-assist-cap-auto-tag = Auto-tag
admin-assist-cap-recap = Recap
admin-assist-cap-assistant = Assistant (chat)

# -- Single sign-on: OIDC + SAML login backends (t9, §18.3) -------------------
admin-sso-title = Single sign-on
admin-sso-intro = Configure OIDC and SAML 2.0 login backends. Enabled backends appear as "Sign in with…" buttons on the mailbox login, scoped deployment-wide or to one domain.
admin-sso-add = Add a login backend
admin-sso-edit = Edit
admin-sso-create = Add backend
admin-sso-update = Save changes
admin-sso-cancel = Cancel
admin-sso-empty = No SSO backends configured.
admin-sso-load-error = Could not load SSO backends.
admin-sso-save-error = Could not save the backend.
admin-sso-delete-error = Could not delete the backend.

# Common fields
admin-sso-id = Backend ID
admin-sso-id-placeholder = corp-oidc
admin-sso-display-name = Display name
admin-sso-display-name-placeholder = Sign in with Acme SSO
admin-sso-kind = Protocol
admin-sso-kind-oidc = OIDC
admin-sso-kind-saml = SAML 2.0
admin-sso-scope = Scope
admin-sso-scope-deployment = Whole deployment
admin-sso-scope-domain = One domain
admin-sso-domain = Domain
admin-sso-domain-placeholder = example.org
admin-sso-enabled = Enabled
admin-sso-first-login = First-login policy
admin-sso-policy-allowlist = Allowlist (deny unknown users)
admin-sso-policy-autocreate = Auto-create accounts on first login

# OIDC fields
admin-sso-issuer = Issuer URL
admin-sso-issuer-placeholder = https://idp.example.org/realms/acme
admin-sso-client-id = Client ID
admin-sso-client-secret = Client secret
admin-sso-secret-unchanged = Leave blank to keep the stored secret
admin-sso-redirect = Redirect URL
admin-sso-scopes = Scopes
admin-sso-metadata = SP metadata

# SAML fields
admin-sso-sp-entity-id = SP entity ID
admin-sso-acs-url = ACS URL
admin-sso-idp-metadata-url = IdP metadata URL
admin-sso-idp-metadata-url-placeholder = https://idp.example.org/saml/metadata
admin-sso-idp-metadata-xml = IdP metadata XML
admin-sso-idp-metadata-xml-placeholder = Paste the IdP metadata XML, or use the URL above
admin-sso-idp-sso-url = IdP SSO URL
admin-sso-idp-slo-url = IdP logout (SLO) URL
admin-sso-idp-certs = IdP signing certificates (PEM)
admin-sso-idp-certs-placeholder = One PEM certificate per block, separated by a blank line
admin-sso-nameid-format = NameID format
admin-sso-want-signed = Require signed assertions
admin-sso-want-encrypted = Require encrypted assertions

# Claim map
admin-sso-claims = Claim mapping
admin-sso-claim-email = Email claim
admin-sso-claim-username = Username claim
admin-sso-claim-display = Display-name claim
admin-sso-claim-groups = Groups claim

# List row
admin-sso-badge-enabled = Enabled
admin-sso-badge-disabled = Disabled
admin-sso-enable = Enable
admin-sso-disable = Disable
admin-sso-delete = Delete
# `name` is the backend's admin-set display name.
admin-sso-enable-for = Enable { $name }
admin-sso-disable-for = Disable { $name }
admin-sso-delete-for = Delete { $name }

# -- Server metadata editor (t14, RFC 5464 annotations under /admin) ----------
# The editor body (entry list, add form) reuses the servermeta.ftl catalog; these
# ids cover only the admin wrapper (account picker + framing).
admin-servermeta-title = Server metadata
admin-servermeta-intro = View and edit RFC 5464 server annotations for a provisioned account. Changes are written straight to the mail server, which decides whether the account may set them.
admin-servermeta-account = Account
admin-servermeta-select-option = Select an account…
admin-servermeta-select-prompt = Select an account to view and edit its server annotations.
admin-servermeta-load-error = Could not load the account list.
admin-servermeta-no-accounts = No accounts are provisioned.

# -- Re-thread mailbox: one-shot JWZ backfill (t14, admin opt-in) --------------
# Keys are disjoint from admin-servermeta-* (E4) — additive. This drives the
# admin-gated POST /admin/maintenance/rethread; the action is non-destructive by
# default and never fires without the explicit confirmation below.
admin-rethread-title = Re-thread mailbox
admin-rethread-intro = Re-runs conversation threading (JWZ) over a provisioned account's stored mail and re-keys its thread grouping. This is a one-time maintenance action, not something that runs automatically.
admin-rethread-account = Account
admin-rethread-select-option = Select an account…
admin-rethread-no-accounts = No accounts are provisioned.
admin-rethread-load-error = Could not load the account list.
admin-rethread-run = Re-thread mailbox
admin-rethread-confirm-title = Re-thread this mailbox?
admin-rethread-confirm-warning = Re-threading re-keys conversation grouping for this account. Existing threads may merge or split, and users may see conversations regrouped.
admin-rethread-confirm-detail = This is a one-time maintenance action. It runs once now; it is safe to re-run and does not delete any mail.
admin-rethread-confirm = Re-thread mailbox
admin-rethread-running = Re-threading…
admin-rethread-cancel = Cancel
admin-rethread-summary = Re-threaded { $messages } message(s) into { $threads } thread(s) across { $accounts } account(s); { $reassigned } message(s) moved to a different thread.
admin-rethread-error = The re-thread action failed. No thread grouping was changed if the server rejected the request; check the server logs and try again.

# -- Require two-factor policy (DQ2, t16 26.16) ------------------------------
# The require-2FA policy (global / per-domain). Any user may enrol a factor on
# their own; this panel governs where a second factor is REQUIRED. A required but
# not-yet-enrolled account is prompted to enrol on its next sign-in. Keys are
# disjoint from admin-security-* (SecurityPolicy has its own global require2fa
# checkbox; this panel adds the per-domain policy and shares the same intent).
admin-2fa-title = Require two-factor
admin-2fa-intro = Require a second factor (passkey or authenticator app) for sign-in. Any user may enrol a factor on their own; requiring it here forces accounts in scope to enrol on their next sign-in.
admin-2fa-load-error = Could not load the two-factor policy.
admin-2fa-save-error = Could not save the two-factor policy.
admin-2fa-global = Require two-factor for the whole deployment
admin-2fa-global-label = Require two-factor for the whole deployment
admin-2fa-global-note = When on, every account must have a second factor. A per-domain rule below can also require it for one domain without requiring it everywhere.
admin-2fa-domains-heading = Per-domain requirements
admin-2fa-domains-empty = No per-domain requirements set.
admin-2fa-col-domain = Domain
admin-2fa-col-require = Required
admin-2fa-require-for = Require two-factor for { $domain }
admin-2fa-add-domain = Domain
admin-2fa-add-domain-placeholder = example.com
admin-2fa-add-require = Require two-factor for this domain
admin-2fa-add-require-label = Require two-factor for this domain
admin-2fa-add-save = Add domain rule
