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
admin-nav-assist = AI assistant
admin-nav-sso = Single sign-on
admin-nav-servermeta = Server metadata
admin-nav-rethread = Maintenance
admin-nav-2fa = Require two-factor
admin-nav-security-policy = Security policy
admin-nav-oauth-clients = OAuth clients

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
admin-integrations-unattended-withdraw-meaning = A send made with this key through MCP is refused until the key is approved again. The message is not transmitted and is not placed in the owner's Outbox.
admin-integrations-unattended-approve-effect = The approval takes effect immediately, from the key's next send. No restart is needed.
admin-integrations-unattended-withdraw-effect = The withdrawal takes effect immediately: the key stops sending unattended from its next send. No restart is needed.
admin-integrations-unattended-cancel = Cancel
admin-integrations-unattended-saving = Saving…
admin-integrations-unattended-approved-done = Unattended send approved for key { $prefix }. It is in effect now.
admin-integrations-unattended-withdrawn-done = Approval withdrawn for key { $prefix }. This key no longer sends unattended.
admin-integrations-unattended-error-gone = Key { $prefix } no longer exists or was revoked. Nothing was changed.
admin-integrations-unattended-error-conflict = Key { $prefix } is revoked or its owner did not request unattended send. Nothing was approved.
admin-integrations-unattended-error = Could not change the approval for key { $prefix }.

# -- Observability -----------------------------------------------------------
admin-obs-title = Observability
admin-obs-load-error = Could not load observability data
admin-obs-export-error = Could not export the audit log
admin-obs-ban-add-error = Could not add the ban
admin-obs-unban-error = Could not remove the ban
admin-obs-telemetry-note = Operational telemetry is off by default. Configure its collector, signals and limits on the dedicated telemetry page. MW_OTLP_ENDPOINT and MW_METRICS_TOKEN remain deployment overrides; changes require a server restart. MW_LOG controls the startup log filter.
admin-obs-telemetry-open = Configure operational telemetry
admin-obs-audit = Audit log
admin-obs-export = Export JSONL
admin-obs-audit-empty = No audit entries.
admin-obs-col-time = Time
admin-obs-col-actor = Actor
admin-obs-col-action = Action
admin-obs-col-target = Target
admin-obs-bans = IP ban list
admin-obs-bans-note = Active entries block application requests from the trusted client IP, including existing sessions. Expired entries no longer block. Health checks remain available. Forwarded addresses are trusted only from configured proxies; banning a proxy or loopback address can block everyone using that address. Fail2ban and firewall rules operate independently.
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
admin-allowlist-col-component = Component
admin-allowlist-col-state = State
admin-allowlist-col-approved = Approved
admin-allowlist-approve-done = The digest of { $id } is approved.
admin-allowlist-revoke-done = The pin of { $id } is revoked and the plugin is disabled.
admin-allowlist-uninstall-done = { $id } was uninstalled.

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

# Rebuilt list + detail (f33, t32-cfg §4.12). `name` and `id` are isolated values.
admin-sso-col-name = Name
admin-sso-col-state = State
admin-sso-search = Search providers
admin-sso-no-match = No provider matches the search.
admin-sso-scope-for-domain = Domain { $domain }
admin-sso-config-unreadable = Unreadable configuration
admin-sso-config-unreadable-note = The stored configuration of this provider could not be read. Fill in the form and save to replace it.
admin-sso-not-found = There is no sign-in provider with the id { $id }.
admin-sso-edit-title = Edit sign-in provider
admin-sso-enabled-done = { $name } is on.
admin-sso-disabled-done = { $name } is off.
admin-sso-deleted-done = { $name } was deleted.
admin-sso-disable-title = Turn off { $name }?
admin-sso-disable-body = Its button leaves the sign-in page and nobody can sign in with it until it is turned on again. Accounts are kept.
admin-sso-delete-title = Delete { $name }?
admin-sso-delete-body = The provider and its stored secret are removed and nobody can sign in with it. Accounts made through it are kept.
admin-sso-err-required = { $field } is required.
admin-sso-on = On

admin-sso-group-basics = Provider
admin-sso-id-help = A short id in lower-case letters, digits and hyphens; it becomes part of this server's addresses for the provider.
admin-sso-id-fixed = The id cannot change because the provider knows this server's addresses by it; delete the provider and add it again to use another id.
admin-sso-id-pattern = Use lower-case letters, digits and hyphens, starting with a letter or digit.
admin-sso-id-taken = Another provider already uses the id { $id }.
admin-sso-display-name-help = The text of the button on the sign-in page.
admin-sso-kind-help = OpenID Connect for providers such as Keycloak, Entra ID or Google; SAML 2.0 for providers that publish SAML metadata.
admin-sso-scope-help = Whole deployment shows the button to everyone; one domain shows it only for addresses in that domain.
admin-sso-domain-help = The mail domain whose sign-in page shows this provider.
admin-sso-domain-choose = Choose a domain
admin-sso-domain-invalid = Enter a domain name such as example.org.
admin-sso-first-login-help = What happens when someone signs in for the first time: refused unless their account exists, or an account is made for them.
admin-sso-enabled-help = Off keeps the settings but takes the button off the sign-in page.

admin-sso-group-oidc = OpenID Connect
admin-sso-issuer-help = The provider's issuer address; this server reads its discovery document from there.
admin-sso-client-id-help = The client id this server was registered with at the provider.
admin-sso-client-secret-help = The client secret from the provider; it is stored sealed and never shown again.
admin-sso-redirect-help = Leave empty to use this server's address /api/sso/<id>/callback; register the same address at the provider.
admin-sso-scopes-help = The scopes asked for at sign-in; openid is required.
admin-sso-scopes-openid = The scopes must include openid.
admin-sso-scopes-invalid = A scope is one word without spaces or quotes.

admin-sso-group-saml-idp = Identity provider (SAML)
admin-sso-group-saml-idp-help = Read the provider's metadata file to fill these fields, or enter them by hand.
admin-sso-meta = Provider metadata
admin-sso-meta-help = Upload or paste the provider's metadata to see what it contains and fill the fields below from it.
admin-sso-meta-none-stored = No metadata is stored with this provider.
admin-sso-meta-stored = Metadata from { $entity } is stored with this provider.
admin-sso-meta-stored-unnamed = Metadata is stored with this provider.
admin-sso-meta-file = Metadata file
admin-sso-meta-paste = Or paste the metadata
admin-sso-meta-not-xml = This is not an XML document.
admin-sso-meta-no-idp = This metadata does not describe an identity provider (no IDPSSODescriptor).
admin-sso-meta-preview = What the metadata says
admin-sso-meta-entity = Provider entity id
admin-sso-meta-missing = Not in the metadata
admin-sso-meta-sso-binding = Listed only for a binding other than HTTP-Redirect, which this server uses
admin-sso-meta-no-certs = No signing certificate in the metadata
admin-sso-meta-use = Use these values
admin-sso-meta-used = The sign-in address, sign-out address and certificates were filled in from the metadata. Save to keep them.
admin-sso-idp-sso-url-help = Where people are sent to sign in (HTTP-Redirect binding).
admin-sso-idp-slo-url-help = Where people are sent to sign out; leave empty when the provider has none.
admin-sso-idp-certs-help = The certificates the provider signs with; a response signed with any other certificate is refused.
admin-sso-cert-required = Add at least one signing certificate.
admin-sso-idp-metadata-url-help = Kept for reference only: this server does not fetch metadata from it.

admin-sso-cert-valid = Valid until { $date }
admin-sso-cert-expiring =
    { $days ->
        [0] Expires today
        [one] Expires in 1 day
       *[other] Expires in { $days } days
    }
admin-sso-cert-expired = Expired on { $date }
admin-sso-cert-not-yet = Valid from { $date }
admin-sso-cert-unreadable = Unreadable certificate
admin-sso-cert-none-found = No PEM certificate found. A certificate starts with -----BEGIN CERTIFICATE-----.
admin-sso-cert-empty = No certificate yet.
admin-sso-cert-no-subject = Certificate without a name
admin-sso-cert-remove = Remove the certificate { $subject }
admin-sso-cert-file = Add from a file
admin-sso-cert-paste = Or paste a certificate
admin-sso-cert-add = Add certificate

admin-sso-group-saml-sp = This server (SAML)
admin-sso-sp-entity-id-help = The name this server uses towards the provider; leave empty to use its metadata address /api/sso/<id>/metadata.
admin-sso-acs-url-help = Where the provider posts its answer; leave empty to use /api/sso/<id>/acs on this server.
admin-sso-nameid-format-help = The kind of user identifier asked from the provider.
admin-sso-want-signed-help = Refuse assertions that are not signed by one of the certificates above.
admin-sso-want-encrypted-help = Refuse assertions that are not encrypted to this server; needs the service provider key below.
admin-sso-sp-key = Service provider private key
admin-sso-sp-key-help = The PEM private key this server decrypts assertions with; it is stored sealed and never shown again.

admin-sso-claims-help = Which claim or attribute of the provider's answer fills each part of the account.
admin-sso-claim-email-help = The email address of the person.
admin-sso-claim-username-help = The name the account is matched or made with.
admin-sso-claim-display-help = The name shown to others.
admin-sso-claim-groups-help = Group membership; leave empty when not used.

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
admin-servermeta-add-user = Add a user
admin-servermeta-account-help = The annotations are read and written on this account's mail server, which decides what the account may set.

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
admin-nav-sessions = Sessions
admin-sessions-domain = Domain filter
admin-sessions-account = Account ID filter
admin-sessions-client = Client type filter
admin-sessions-preview = Preview session revocation
admin-sessions-count = Sessions selected: { $count }
admin-sessions-confirm = Revoke these sessions
admin-sessions-truncated = Showing at most 1,000 sessions. Narrow the filters to inspect further sessions.
admin-security-center-title = Security centre
admin-security-center-policy-help = Set authentication and session policies globally or for a domain, group, profile or account. Deployment timeout limits still apply.
admin-security-center-posture = Security checklist
admin-security-center-coverage = Accounts with an authentication factor: { $enrolled } of { $total } known accounts
admin-security-center-admins = Administrators without a second factor: { $count }
admin-security-center-keys = Active application keys unused for over 90 days: { $count }
admin-security-center-allowlist-on = An upstream allowlist or default is configured
admin-security-center-allowlist-off = No upstream allowlist or default is configured
admin-security-center-env-on = Environment administrator sign-in is enabled
admin-security-center-env-off = Environment administrator sign-in is disabled

admin-users-back = Back to users
admin-users-identity = User identity
admin-users-id = Account identifier
admin-users-last-signin = Last sign-in

admin-sso-filter-scope = Filter by scope
admin-sso-all-scopes = All scopes
admin-sso-back = Back to sign-in providers
admin-sso-editor-intro = Configure this sign-in provider, then save or cancel to return to the provider table.

admin-session-mailbox-title = Mailbox sessions
admin-session-console-title = Administrator sessions
admin-session-empty = No sessions match these filters.
admin-session-search = Search account, device or IP
admin-session-account = Account
admin-session-device = Device and client
admin-session-ip = IP address
admin-session-activity = Last activity
admin-session-created = Created
admin-session-expiry = Expiry
admin-session-status = Status
admin-session-active = Active
admin-session-idle-expiry = Idle expiry
admin-session-revoke-account = Revoke session for { $account }
admin-session-revoke-help = This ends this session. The account can sign in again.
admin-session-console-metadata = Administrator device, IP and location metadata are not recorded for these sessions.

admin-session-current = This session
admin-session-revoke = Revoke session

admin-egress-scope = Route scope
admin-egress-scope-help = Use global or domain:example.org for an existing configured domain. Domain aliases use their canonical domain. Only selected traffic classes use this route.
admin-egress-classes = Traffic carried by this route
admin-egress-classes-help = Select at least one class. Other classes keep their own route decision. Push remains HTTPS and public-address only; selecting a proxy never permits private push targets.
admin-egress-class-remote_content = Remote message images and files
admin-egress-class-ics_feeds = Calendar subscription feeds
admin-egress-class-keyservers = Public key servers
admin-egress-class-webhooks = Outgoing webhooks
admin-egress-class-admin_detect = Administrator discovery and probes
admin-egress-class-pki = Certificate infrastructure
admin-egress-class-dns_auth = Mail authentication DNS requests
admin-egress-class-unsubscribe = List unsubscribe requests
admin-egress-class-siem = Security event collectors
admin-egress-class-analytics = Optional usage analytics
admin-egress-class-nextcloud = Nextcloud files and DAV requests
admin-egress-class-oauth = OAuth provider requests
admin-egress-class-update_check = Update checks
admin-egress-class-sieve = ManageSieve rules transport
admin-egress-class-push = Browser push delivery

admin-obs-ban-expiry = Expiry (this device's local time)
admin-obs-ban-expiry-help = Leave empty for a permanent ban. Expiry must be in the future. Health checks remain available, but this can block your own active session.
admin-obs-ban-expired = Expired
admin-obs-ban-until = Active until
admin-obs-ban-permanent = Active until removed
admin-obs-ban-save = Save IP ban
admin-obs-ban-cancel = Cancel
admin-obs-ban-invalid-expiry = Choose a future expiry or leave it empty.
admin-obs-ban-status = Status and expiry
admin-obs-ban-search = Search IP bans
admin-obs-ban-active-only = Active bans only

admin-egress-class-package_tracking = Optional package tracking provider

admin-obs-locks-title = Temporary sign-in locks
admin-obs-locks-note = Failed credentials can temporarily lock sign-in from one source IP. Administrator and mailbox counters are separate. These locks expire automatically; repeated attempts do not extend them. Manual global IP bans are managed separately.
admin-obs-locks-back = Back to observability
admin-obs-locks-refresh = Refresh locks
admin-obs-locks-configure = Configure sign-in limits
admin-obs-lock-surface = Sign-in surface
admin-obs-lock-surface-all = All sign-in surfaces
admin-obs-lock-surface-admin = Administrator
admin-obs-lock-surface-webmail = Mailbox
admin-obs-lock-failures = Rejected credentials
admin-obs-lock-until = Locked until
admin-obs-locks-empty = No temporary sign-in locks.
admin-obs-locks-search = Search temporary sign-in locks
admin-obs-lock-unlock = Unlock
admin-obs-lock-unlock-target = Unlock { $surface } sign-in for { $ip }
admin-obs-lock-confirm = Reset the failed-credential counter and permit another sign-in attempt for this source. Manual global IP bans still apply. A fresh security check is required.
admin-obs-lock-unlocked = Unlocked { $surface } sign-in for { $ip }.

admin-obs-geo-title = Sign-in country policy
admin-obs-geo-note = This diagnostic evaluates your trusted source address. Country policy affects new sign-ins only; existing sessions are not revoked. Review the result before enabling a policy, including the administrator sign-in decision.
admin-obs-geo-unavailable = The local country database is missing or unreadable. Active country policies refuse sign-in in this state, even when unmapped addresses are allowed. Configure MW_GEOIP_DB and restart after replacing or repairing the database.
admin-obs-geo-refresh = Refresh diagnostic
admin-obs-geo-configure = Configure country policy
admin-obs-geo-field = Policy detail
admin-obs-geo-value = Current value
admin-obs-geo-empty = No diagnostic available.
admin-obs-geo-field-source = Trusted source IP
admin-obs-geo-field-country = Resolved country
admin-obs-geo-field-database = Local country database
admin-obs-geo-field-scope = Sign-in scope
admin-obs-geo-field-mode = Country list mode
admin-obs-geo-field-countries = Country codes
admin-obs-geo-field-unknown = Unmapped address policy
admin-obs-geo-field-admin = Your administrator sign-in decision
admin-obs-geo-field-mailbox = Your mailbox sign-in decision
admin-obs-geo-value-off = Off
admin-obs-geo-value-allow = Allow
admin-obs-geo-value-deny = Deny
admin-obs-geo-value-unavailable = Unavailable — active policies refuse sign-in
admin-obs-geo-value-ready = Ready
admin-obs-geo-value-unknown = Unknown
admin-obs-geo-value-empty = Empty list
admin-obs-geo-value-admin = Administrator sign-in
admin-obs-geo-value-mailbox = Mailbox sign-in
admin-obs-geo-value-all = All sign-ins

admin-obs-geo-field-sessions = Existing authenticated requests
admin-obs-geo-sessions-on = Country checks enabled
admin-obs-geo-sessions-off = Sign-in checks only
admin-obs-geo-session-note = The configured country policy also checks authenticated requests, native bearer access and signed reader content. Blocked clients can sign out. This does not erase sessions or terminate already-open streams; streams are checked again when they reconnect. Review your administrator decision before enabling it.
admin-obs-geo-configure-sessions = Configure existing-session checks
admin-egress-class-plugins = Approved plugin and provider requests
admin-egress-destinations-title = Internal destination permissions
admin-egress-destinations-help = Permit one traffic class to contact an approved private service. Route selection and plugin hostname permissions still apply. Other traffic classes receive no access.
admin-egress-destinations-back = Back to outbound routes
admin-egress-destinations-add = Add destination permission
admin-egress-destinations-empty = No private destination permissions are configured.
admin-egress-destination-class = Traffic class
admin-egress-destination-choose = Choose a traffic class
admin-egress-destination-class-help = Select the single service allowed to use this permission. Analytics permissions never authorize plugins or mail content.
admin-egress-destination-target = Destination
admin-egress-destination-target-help = Enter an exact hostname or IP address, optionally with a port (ews.example.test:443), or a bounded private network such as 10.20.0.0/16. URLs, metadata, loopback and link-local addresses are refused.
admin-egress-destination-note = Administrative note
admin-egress-destination-actions = Actions
admin-egress-destination-revoke = Revoke permission
admin-egress-destination-confirm = New requests to this private destination will be refused unless another explicit permission covers it.
admin-egress-destination-confirm-button = Confirm revocation

admin-deployment-title = Deployment security assessment
admin-deployment-back = Back to diagnostics
admin-deployment-refresh = Refresh assessment
admin-deployment-help = Read-only observations from this server process and this request. These checks do not certify the deployment or contact external services.
admin-deployment-search = Find a check
admin-deployment-status = Status
admin-deployment-status-all = All checks
admin-deployment-status-pass = Present
admin-deployment-status-warning = Review needed
admin-deployment-status-info = Information
admin-deployment-empty = No checks match your filters.
admin-deployment-check = Check
admin-deployment-observed = Observed value
admin-deployment-settings = Configuration
admin-deployment-limits = What this assessment does not test
admin-deployment-snapshot = Snapshot taken at { $time }.
admin-deployment-yes = Yes
admin-deployment-no = No
admin-deployment-check-https = HTTPS request
admin-deployment-check-secureCookies = Secure cookies
admin-deployment-check-hsts = Strict transport security
admin-deployment-check-csrf = Strict CSRF protection
admin-deployment-check-adminSessions = Administrator session limits
admin-deployment-check-adminFactors = Enabled administrators without a second factor
admin-deployment-check-mailboxFactors = Mailbox factor enrollment
admin-deployment-check-signinPacing = Sign-in retry delay (seconds)
admin-deployment-check-credentialLocks = Temporary credential-failure locks
admin-deployment-check-geography = Country policy
admin-deployment-check-renderIsolation = Document preview isolation
admin-deployment-value-https = Effective HTTPS: { $effectiveRequestHttps }; configured external HTTPS: { $configuredHttps }.
admin-deployment-value-adminSessions = Idle limit: { $idleSeconds } seconds; absolute limit: { $absoluteSeconds } seconds.
admin-deployment-value-mailboxFactors = { $enrolled } of { $knownAccounts } known accounts have a factor.
admin-deployment-value-credentialLocks = Administrator limit: { $administratorLimit }; mailbox limit: { $mailboxLimit }; window: { $windowSeconds } seconds; lock: { $lockSeconds } seconds.
admin-deployment-value-geography = Scope: { $scope }; existing-request checks: { $existingRequests }; local database ready: { $localDatabaseReady }.
admin-deployment-value-renderIsolation = Isolated renderer required: { $jailRequired }; worker configured: { $workerConfigured }.
admin-deployment-limit-no-external-tls-or-firewall-probe = Public TLS certificates, reverse-proxy configuration and firewall exposure are not tested.
admin-deployment-limit-no-upstream-authentication = Upstream authentication and mailbox access are not tested.
admin-deployment-limit-configured-worker-not-executed = The configured document worker is not executed by this assessment.
admin-deployment-limit-not-a-security-certification = A reported setting is evidence of configuration, not a guarantee of complete protection.
admin-egress-destination-note-help = Describe why this private service is approved. This note is for administrators and is never sent upstream.

admin-deployment-check-connectionNegotiation = Inbound connection negotiation limits
admin-deployment-value-connectionNegotiation = Built-in listener startup configuration: TLS deadline { $tlsTimeoutMs } ms, up to { $tlsMaximum } handshakes; PROXY header deadline { $proxyTimeoutMs } ms, up to { $proxyMaximum } negotiations. Changes require restart. External proxy limits are not measured here.

admin-personal-dashboard-keep-checklist = Keep the completed setup checklist prominent
admin-personal-dashboard-keep-checklist-help = Otherwise, completing every item collapses the checklist below the other dashboard widgets. An incomplete item restores its usual position.

admin-session-select = Select
admin-session-select-account = Select session for { $account }
admin-session-select-visible = Select all visible sessions
admin-session-revoke-selected = Revoke selected sessions ({ $count })
admin-session-batch-confirm = Revoke only the selected sessions. You may be asked to confirm your identity. Failed or skipped sessions will be reported.
admin-session-batch-current = Your current administrator session is selected. It will be revoked last, signing you out, only if the other selected sessions are handled successfully. If another request fails, this session is kept so you can review the result.
admin-session-batch-confirm-button = Confirm selected revocations
admin-session-batch-result = Revoked: { $revoked }. Already ended: { $unchanged }. Could not confirm: { $failed }. Not attempted: { $skipped }.
admin-session-refresh-failed = The session list could not be refreshed. Reload it before selecting more sessions.
admin-deployment-check-httpAdmission = HTTP admission and request limits
admin-deployment-value-httpAdmission = Built-in listener startup configuration: { $connections } connections, { $requests } requests through response completion, { $headerBytes } header bytes and { $headers } headers. HTTP opening and HTTP/1 header deadline { $headerTimeoutMs } ms; request-body deadline { $bodyTimeoutMs } ms; { $http2Streams } HTTP/2 streams per connection; graceful drain { $shutdownTimeoutMs } ms. Fixed limits require restart. Response streams are not subject to the request-body deadline.

admin-protection-title = Aggregate and adaptive protection
admin-protection-back = Back to observability
admin-protection-refresh = Refresh
admin-protection-explanation = Optional, deterministic protection compares typed unusual requests with normal requests over short and long windows. Independent anomaly counts resist dilution by normal traffic. Both modes are off by default; protocol safety limits remain active.
admin-protection-current-policy = Current policy
admin-protection-modes = Request budgets: { $rate }. Adaptive protection: { $adaptive }.
admin-protection-mode-off = Off
admin-protection-mode-observe = Observe only
admin-protection-mode-enforce = Enforce
admin-protection-retention = At most { $subjects } subject records and { $events } decisions; decisions expire after { $days } days. Changes propagate to other replicas within { $seconds } seconds.
admin-protection-coverage = Client counters use the trusted source address. Account counters require a verified session, native account key, or signed content authority. Proxy accounts are separated by service realm; legacy proxy keys without that realm cannot pass account-level enforcement. Earlier HTTP parser and connection-capacity refusals are separate safety limits, not adaptive observations. Existing streams are checked on their next request.
admin-protection-configure = Configure modes, windows, thresholds, and exemptions
admin-protection-subjects = Current subject state
admin-protection-subject = Subject fingerprint
admin-protection-kind-client = Client address
admin-protection-kind-account = Mailbox account
admin-protection-kind-admin = Administrator
admin-protection-short = Short: unusual / normal
admin-protection-long = Long: unusual / normal
admin-protection-action-until = Current action deadline
admin-protection-monitoring = No active deadline
admin-protection-observed-at = Last observed
admin-protection-counts-note = Counts show unusual / normal observations at the listed observation time. Fingerprints are purpose-bound hashes; raw addresses and account IDs are not stored in this history. A fixed deadline does not slide when blocked requests repeat.
admin-protection-empty = No subject state is currently retained.
admin-protection-next = Next page
admin-protection-history = Protection decisions
admin-protection-history-empty = No protection decisions are retained.
admin-protection-reason = Trigger
admin-protection-action = Decision
admin-protection-evidence = Evidence
admin-protection-evidence-counts = Short window: { $shortBad } unusual, { $shortNormal } normal. Long window: { $longBad } unusual, { $longNormal } normal.
admin-protection-evidence-thresholds = Applied short window: { $shortSecs } seconds, absolute { $shortAbsolute }, ratio { $shortRatio }% after { $shortMinimum } samples. Applied long window: { $longSecs } seconds, absolute { $longAbsolute }, ratio { $longRatio }% after { $longMinimum } samples. Either enabled threshold can trigger.
admin-protection-no-signal = No anomaly signal
admin-protection-reason-request-rate = Aggregate request budget
admin-protection-reason-short-absolute = Short-window anomaly count
admin-protection-reason-short-ratio = Short-window unusual-request proportion
admin-protection-reason-long-absolute = Long-window anomaly count
admin-protection-reason-long-ratio = Long-window unusual-request proportion
admin-protection-reason-manual-clear = Administrator cleared state
admin-protection-action-throttle = Temporary throttle
admin-protection-action-block = Temporary block
admin-protection-action-cleared = State cleared
admin-protection-enforced = Enforced
admin-protection-observed = Observed only
admin-protection-clear = Clear state
admin-protection-clear-target = Clear protection state for { $subject }
admin-protection-clear-body = Clear this subject's counters and temporary action after confirming your identity. This does not change policy or remove manual bans. New observations may trigger protection again. If the state changes first, refresh before retrying.
admin-protection-cleared = Cleared protection state for { $subject }.

admin-accounts-last-super-protected = This is the last enabled Super admin. Actions that remove this access are disabled. Grant another account unrestricted Super admin access first.
admin-accounts-last-super-allowed = This is the last enabled Super admin. The explicit deployment policy permits losing this access; local deployment administration would be required to recover.
admin-accounts-last-super-confirm = This action can leave the deployment without any Super admin access. Recovery requires local deployment administration. Confirm only if you intend to lose this access.

admin-history-title = Request history
admin-history-period = Period
admin-history-default = Administrator default
admin-history-custom = Custom period
admin-history-period-3600 = Last 60 minutes
admin-history-period-21600 = Last 6 hours
admin-history-period-86400 = Last 24 hours
admin-history-period-604800 = Last 7 days
admin-history-period-2592000 = Last 30 days
admin-history-from = From (local time)
admin-history-to = To (local time)
admin-history-help = Counts include requests handled by this server. Latency measures when the response is ready, not when a streamed download finishes. Gaps mean no recorded history; partial buckets include only observed traffic.
admin-history-range = { $from } to { $to } · { $seconds } seconds per bucket
admin-history-partial = History covers { $observed } of { $requested } requested minutes. Missing periods are not counted as zero.
admin-history-requests = Requests per bucket
admin-history-load-error = Request history could not be loaded. Refresh to try again.
admin-history-error-invalid = Enter valid start and end dates.
admin-history-error-order = The end must be later than the start.
admin-history-error-future = The period cannot end in the future.
admin-history-error-limit = This period exceeds the administrator's configured limit.
admin-history-refresh = Refresh request history
admin-history-errors = Server errors (5xx) per bucket

admin-users-legacy-alias = This account signs in through a domain alias. Existing administrative record: { $address }. Its settings and history are preserved; the records have not been merged or renamed.

admin-history-latency-resolution = New latency samples retain integer milliseconds through 1 second; slower timings use wider bounded bins. Percentiles describe response-ready timing, not the full transfer.
admin-history-latency-legacy = This range includes older or mixed coarse latency estimates. Their original measurement precision cannot be recovered.

admin-users-access-title = Mailwoman access
admin-users-access-help = These controls affect local sign-in only. Upstream mail, settings and history are preserved. Other policy or account-flag restrictions still apply.
admin-users-block-for = Block login for { $account }
admin-users-unblock-for = Unblock login for { $account }
admin-users-delete-for = Remove Mailwoman access for { $account }
admin-users-block-help = Block new sign-ins and revoke current Mailwoman sessions.
admin-users-unblock-help = Remove this directory access hold. Other restrictions still apply.
admin-users-delete-help = Delete local access while preserving the upstream mailbox and local history.
admin-users-deleted = Local access deleted
admin-users-lifecycle-block-title = Block Mailwoman login
admin-users-lifecycle-unblock-title = Unblock Mailwoman login
admin-users-lifecycle-delete-title = Delete local Mailwoman access
admin-users-lifecycle-block-body = This changes the directory sign-in hold. Blocking revokes current sessions. It does not change the upstream mailbox or override other account restrictions.
admin-users-lifecycle-delete-body = This keeps a deleted access record to prevent automatic re-enrollment. Mailwoman settings, history and the upstream mailbox remain stored. This action cannot be undone from this page.
admin-users-lifecycle-block-confirm = Block login
admin-users-lifecycle-unblock-confirm = Unblock login
admin-users-lifecycle-delete-confirm = Delete local access
admin-users-lifecycle-unblock-body = Remove the local directory login block. Other security restrictions still apply; this does not restore deleted access.
admin-users-legacy-clear = Clear legacy login block
admin-users-legacy-block-help = A legacy login flag also blocks this user. Clear it explicitly before changing the directory block. Other flags and security restrictions remain in effect.
admin-users-lifecycle-legacy-title = Clear legacy login block?
admin-users-lifecycle-legacy-body = Clear only the legacy disabled flag. Preserve all other flags and the directory access status. This alone does not guarantee that login is allowed.
admin-users-lifecycle-legacy-confirm = Clear legacy block

admin-transfer-title = Configuration & backups
admin-transfer-scope = Portable files contain registered global setting values and locks. They exclude domains, user data, credentials, environment variables and dedicated integration configuration. Full recovery backups are separate.
admin-transfer-export = Export settings
admin-transfer-import = Import settings
admin-transfer-backups = Backup destinations
admin-transfer-export-help = Download the stored global policies as a versioned JSON file. Unset policies remain unset; dedicated security confirmations are excluded.
admin-transfer-download = Download settings
admin-transfer-exported = Exported { $count } policies; { $excluded } dedicated policies were excluded.
admin-transfer-import-help = Choose a settings file to validate every entry and review changes before applying. Missing keys are left unchanged. Changes use individual revision checks; a concurrent change can stop the import after earlier entries were applied.
admin-transfer-file = Settings JSON file
admin-transfer-too-large = Choose a settings file smaller than 2 MiB.
admin-transfer-review = Review settings changes
admin-transfer-key = Setting
admin-transfer-before = Current policy
admin-transfer-after = Imported policy
admin-transfer-apply = Apply reviewed changes
admin-transfer-confirm = Apply the reviewed policies? This can change defaults and locks for users. Reauthentication is required. If a concurrent change stops the import, the result identifies how many earlier policies were applied.
admin-transfer-imported = Applied { $count } policies.
admin-transfer-partial = Applied { $count } policies, then stopped at { $key } (HTTP { $status }). Reload and review the file again before continuing.
admin-transfer-backup-help = Full deployment snapshots include the database, uploads, configuration and server key. Destination copies require archive encryption and checksum verification. Configure destinations on the deployment host; the web server does not receive host filesystem or Docker privileges.
admin-transfer-destinations = Supported backup destinations
admin-transfer-method = Method
admin-transfer-setup = Setup
admin-transfer-method-local = Local directory
admin-transfer-method-s3 = S3-compatible object storage
admin-transfer-method-sftp = SFTP / SSH tunnel
admin-transfer-method-webdav = WebDAV
admin-transfer-method-smb = SMB / CIFS
admin-transfer-method-nfs = NFS
admin-transfer-setup-local = Existing absolute directory on the deployment host.
admin-transfer-setup-s3 = Named S3 rclone remote with bucket/path and host-managed credentials.
admin-transfer-setup-sftp = Named SFTP remote with pinned host keys; an operator-managed SSH tunnel can provide the connection.
admin-transfer-setup-webdav = Named HTTPS WebDAV remote, including compatible Nextcloud storage.
admin-transfer-setup-smb = Named SMB remote with a share/path and host-managed credentials.
admin-transfer-setup-nfs = Existing NFS mount. An absent mount is refused rather than writing to local disk.
admin-transfer-backup-template = Download destination template
admin-transfer-backup-host = Enable and configure destinations in the template, then set BACKUP_TARGETS_FILE and BACKUP_RECIPIENT in deploy.env. The existing deploy.sh backup command and systemd timer use them. All template destinations start disabled; downloading the template does not change a running deployment.
