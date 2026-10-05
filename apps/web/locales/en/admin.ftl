# Mailwoman — Admin panel strings (source locale: en, SPEC §19/§21/§22).
# The admin screen is code-split and reached only via lazy(import); its catalog is
# lazily loaded with it. Ids are module-prefixed `admin-*`.

# -- Shell / nav -------------------------------------------------------------
admin-brand = Mailwoman admin
admin-nav = Admin sections
admin-sign-out = Sign out
admin-nav-domains = Domains
admin-nav-users = Users
admin-nav-integrations = Integrations
admin-nav-observability = Observability
admin-nav-plugins = Plugins
admin-nav-assist = Assist
admin-nav-sso = Single sign-on
admin-nav-servermeta = Server metadata
admin-nav-rethread = Maintenance
admin-nav-2fa = Require two-factor

# Shared admin actions / states
admin-delete = Delete
admin-revoke = Revoke
admin-remove = Remove
admin-saved = Saved.

# -- Sign-in gate (separate admin session) -----------------------------------
admin-login-form = Admin sign in
admin-login-note = This panel runs under a separate admin session. The session ends after 30 minutes without activity, and 12 hours after sign-in at the latest.
admin-login-session-ended = Your admin session has ended. Sign in again.
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
admin-domains-note = A domain is registered by name. Registered names are offered on the Require two-factor screen when a rule applies to one domain. Registering a domain does not route or filter mail.
admin-domains-save = Save domain
admin-domains-empty = No domains yet.
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
admin-users-zeroaccess-help = A record only. Zero-access storage is turned on and off by the user, in their own settings, with their own key. Ticking this box does not encrypt the account's stored mail, and clearing it does not decrypt it.
admin-users-disable-for = Disable { $account }
admin-users-disabled = disabled
admin-users-disabled-help = Blocks sign-in and ends every open session for this account. Its API keys and tokens are refused. This does not disable the mailbox on the mail server: other mail clients can still reach it until you disable it there too.
admin-users-force-change-for = Force password change for { $account }
admin-users-force-change = force change
admin-users-force-change-help = The user is held at a password-change screen, at the next sign-in and in sessions already open, and cannot reach mail until the change succeeds. This needs a working password-change backend (MW_PASSWD_BACKEND). Without one the change cannot succeed and the user stays at that screen until you clear this box.
admin-users-revoke-for = Revoke sessions for { $account }

# -- Integrations ------------------------------------------------------------
admin-integrations-title = Integrations
admin-integrations-load-error = Could not load integrations
admin-integrations-revoke-error = Could not revoke the key
admin-integrations-ldap = LDAP / GAL directory
admin-integrations-nextcloud = Nextcloud bridge
admin-integrations-active = Active
admin-integrations-configured = Configured
admin-integrations-not-configured = Not configured
admin-integrations-unknown = Status unknown
admin-integrations-config-note = "Configured" means this deployment has settings for the integration: an enabled LDAP directory entry in the database, or the three MW_NEXTCLOUD_* environment variables. Both are read when the server starts. It does not mean the remote service was contacted or is reachable.
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
admin-integrations-col-unattended = Unattended send
admin-integrations-unattended-not-requested = Not requested
admin-integrations-unattended-requested = Requested, not approved
admin-integrations-unattended-approved = Approved
admin-integrations-unattended-approve = Approve
admin-integrations-unattended-withdraw = Withdraw
admin-integrations-unattended-approve-key = Approve unattended send for key { $prefix }
admin-integrations-unattended-withdraw-key = Withdraw approval of unattended send for key { $prefix }
admin-integrations-unattended-approve-title = Approve unattended send for this key?
admin-integrations-unattended-withdraw-title = Withdraw the approval for this key?
admin-integrations-unattended-owner = Owner: { $account }
admin-integrations-unattended-key = Key: { $prefix }
admin-integrations-unattended-scope = Permissions: { $permissions }. MCP tools: { $tools }.
admin-integrations-unattended-scope-unreadable = The scope of this key could not be read.
admin-integrations-scope-read = read
admin-integrations-scope-send = send
admin-integrations-scope-delete = delete
admin-integrations-scope-mail = mail
admin-integrations-scope-pim = calendar, tasks, notes and contacts
admin-integrations-scope-none = none
admin-integrations-unattended-approve-meaning = Messages sent with this key through MCP are transmitted without a person releasing them.
admin-integrations-unattended-withdraw-meaning = Messages sent with this key through MCP wait in the owner's Outbox until a person releases them.
admin-integrations-unattended-approve-effect = The approval takes effect when the server restarts.
admin-integrations-unattended-withdraw-effect = The withdrawal takes effect when the server restarts. Until then this key still sends without release.
admin-integrations-unattended-cancel = Cancel
admin-integrations-unattended-saving = Saving…
admin-integrations-unattended-approved-done = Unattended send approved for key { $prefix }. It takes effect when the server restarts.
admin-integrations-unattended-withdrawn-done = Approval withdrawn for key { $prefix }. It takes effect when the server restarts.
admin-integrations-unattended-error-gone = Key { $prefix } no longer exists or was revoked. Nothing was changed.
admin-integrations-unattended-error-conflict = Key { $prefix } is revoked or its owner did not request unattended send. Nothing was approved.
admin-integrations-unattended-error = Could not change the approval for key { $prefix }.

# -- Observability -----------------------------------------------------------
admin-obs-title = Observability
admin-obs-load-error = Could not load observability data
admin-obs-export-error = Could not export the audit log
admin-obs-ban-add-error = Could not add the ban
admin-obs-unban-error = Could not remove the ban
admin-obs-telemetry-note = Logging and telemetry are not set here. The log filter (MW_LOG), the OTLP collector (MW_OTLP_ENDPOINT) and the metrics endpoint (MW_METRICS_TOKEN) are read from the server's environment when it starts.
admin-obs-audit = Audit log
admin-obs-export = Export JSONL
admin-obs-audit-empty = No audit entries.
admin-obs-col-time = Time
admin-obs-col-actor = Actor
admin-obs-col-action = Action
admin-obs-col-target = Target
admin-obs-bans = Login monitor / ban list
admin-obs-bans-note = This list is a record. Mailwoman does not refuse connections from the addresses on it, and an entry has no effect on sign-in. Each failed admin sign-in writes a line to the server log that a fail2ban jail can act on, and five failures from one address add that address here. Blocking is done by that jail or by your firewall.
admin-obs-ban-add = Add ban
admin-obs-ban-ip = IP address
admin-obs-ban-reason = Reason
admin-obs-ban-btn = Ban IP
admin-obs-bans-empty = No active bans.
admin-obs-unban-for = Unban { $ip }
admin-obs-unban-btn = Unban

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

# -- Search index: status and rebuild (t28-e13) --------------------------------
# GET /admin/maintenance/search-index and POST /admin/maintenance/reindex.
admin-searchindex-title = Search index
admin-searchindex-intro = Search reads an index built from stored mail. The server rebuilds it at start when it does not match the stored mail. Mail of zero-access accounts is not indexed, so search finds nothing for those accounts.
admin-searchindex-count = { $documents } of { $messages } stored message(s) are indexed.
admin-searchindex-on-disk = The index is kept on disk.
admin-searchindex-in-memory = The index is kept in memory and is rebuilt each time the server starts.
admin-searchindex-progress = A rebuild is running: { $done } of { $total } message(s) done.
admin-searchindex-run = Rebuild search index
admin-searchindex-running = Rebuilding…
admin-searchindex-summary = Read { $messages } stored message(s) in { $accounts } account(s): { $indexed } indexed, { $removed } removed from the index, { $failed } could not be read. { $zeroAccess } zero-access account(s) left unindexed.
admin-searchindex-busy = A rebuild is already running. Wait for it to finish.
admin-searchindex-error = The rebuild failed. The index keeps what it held; check the server logs and try again.
admin-searchindex-status-error = Could not read the search index status.
admin-searchindex-unavailable = This server runs in proxy mode and has no search index.

# -- Require two-factor policy (DQ2, t16 26.16) ------------------------------
# The require-2FA policy (global / per-domain). Any user may enrol a factor on
# their own; this panel governs where a second factor is REQUIRED. A required but
# not-yet-enrolled account is prompted to enrol on its next sign-in.
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

# -- Egress proxy routes (26.20 t22-e16) -------------------------------------
admin-nav-egress = Egress
admin-egress-heading = Outbound proxy routes
admin-egress-intro = Route the server's outbound fetches through an HTTP CONNECT or SOCKS5 proxy. Mailwoman still enforces its own address policy — the proxy is never asked to resolve a name.
admin-egress-empty = No routes configured. Outbound fetches go direct.
admin-egress-none-active = No route is in use. Outbound fetches go direct. A saved route carries traffic only after you choose "Use this route".
admin-egress-in-use = Outbound fetches use the route "{ $id }".
admin-egress-deactivate = Stop using a proxy
admin-egress-activate = Use this route
admin-egress-activate-for = Use route { $id }
admin-egress-activate-error = Could not switch to the route
admin-egress-deactivate-error = Could not stop using the route
admin-egress-col-state = State
admin-egress-state-active = In use
admin-egress-state-inactive = Not in use
admin-egress-test-scope = Only the route in use can be tested. A test fetches one fixed URL through the route and reports how far it got; it changes nothing.
admin-egress-probe-url = Fetched: { $url }
admin-egress-load-error = Could not load the egress routes
admin-egress-save-error = Could not save the route
admin-egress-delete-error = Could not delete the route
admin-egress-invalid = An id, a host and a port above zero are required.
admin-egress-username-required = A password needs a username; without one the route would authenticate as nobody.
admin-egress-col-id = Id
admin-egress-col-route = Route
admin-egress-col-username = Username
admin-egress-col-credentials = Password
admin-egress-col-plaintext = Plaintext
admin-egress-col-actions = Actions
admin-egress-cred-set = Set
admin-egress-cred-none = None
admin-egress-plaintext-on = Allowed
admin-egress-plaintext-off = Refused
admin-egress-add-heading = Add a route
admin-egress-edit-heading = Edit route
admin-egress-id = Id
admin-egress-scheme = Scheme
admin-egress-host = Host
admin-egress-port = Port
admin-egress-username = Username
admin-egress-password = Password
admin-egress-password-unchanged = Leave blank to keep the stored password
admin-egress-allow-plaintext = Allow plaintext http origins through this route
admin-egress-allow-plaintext-note = Off by default. An https origin is what keeps a proxy from reading the traffic it carries; allowing http means the proxy sees it.
admin-egress-test = Test
admin-egress-testing = Testing…
admin-egress-test-failed = The test could not be run, so nothing is known about this route.
admin-egress-outcome-connected = Connected
admin-egress-outcome-auth-rejected = Authentication rejected
admin-egress-outcome-refused-by-policy = Refused by policy
admin-egress-outcome-dns-failed = Name did not resolve
admin-egress-outcome-unreachable = Unreachable
admin-egress-outcome-origin-tls-failed = Origin TLS failed
admin-egress-outcome-route-invalid = Route is not valid
admin-egress-outcome-unknown = Unrecognised result
admin-egress-stage-dns = stopped at name resolution
admin-egress-stage-connect = stopped while connecting to the proxy
admin-egress-stage-tunnel = stopped while opening the tunnel
admin-egress-stage-origin = stopped at the origin
admin-egress-stage-unknown = stopped at an unrecognised stage
admin-egress-proxied-yes = Traversed the proxy
admin-egress-proxied-no = Did not traverse the proxy
