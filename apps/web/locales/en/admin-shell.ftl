# Mailwoman — admin console shell and shared admin components (source locale: en).
# t32-admin.md §2.1, t32-ext.md §K/§M. Ids are `admin-shell-*`. Section labels that
# existed before t32 stay in admin.ftl (`admin-nav-*`).

# -- Navigation ---------------------------------------------------------------
admin-shell-nav-dashboard = Dashboard
admin-shell-nav-settings = Settings
admin-shell-group-overview = Overview
admin-shell-group-people = People
admin-shell-group-security = Security
admin-shell-group-configuration = Configuration
admin-shell-group-operations = Operations
admin-shell-menu = Menu
admin-shell-close-menu = Close menu
admin-shell-search = Search
admin-shell-search-key = Ctrl+K
admin-shell-signed-in-as = Signed in as { $name }
admin-shell-language-label = Language
admin-shell-breadcrumb = Breadcrumb
admin-shell-unknown-section = This section does not exist, or your role does not include it.
admin-shell-back-to-dashboard = Back to the dashboard
admin-shell-cap-missing = Your role does not include { $capability }.

# -- Shared states and actions -------------------------------------------------
admin-shell-loading = Loading…
admin-shell-retry = Try again
admin-shell-cancel = Cancel
admin-shell-save = Save
admin-shell-not-available = Not available

# -- Leaving a form with unsaved changes ---------------------------------------
admin-shell-leave-title = Leave this page?
admin-shell-leave-body = Your changes on this page have not been saved. Leaving discards them.
admin-shell-leave-confirm = Leave and discard changes

# -- Confirmation dialog -------------------------------------------------------
admin-shell-confirm-type = Type { $name } to confirm

# -- Command palette -----------------------------------------------------------
admin-shell-palette-title = Search the admin console
admin-shell-palette-label = Section, user, domain or setting
admin-shell-palette-results = Results
admin-shell-palette-none = Nothing matches.
admin-shell-palette-searching = Searching users, domains and settings…
admin-shell-palette-sections-only = This server cannot search users, domains and settings yet, so only sections are listed.
admin-shell-palette-error = The search of users, domains and settings failed; only sections are listed.
admin-shell-palette-kind-section = Section
admin-shell-palette-kind-user = User
admin-shell-palette-kind-domain = Domain
admin-shell-palette-kind-setting = Setting

# -- Data table ------------------------------------------------------------------
admin-shell-table-filters = Filters
admin-shell-table-actions = Actions
admin-shell-table-error = The list could not be loaded.
admin-shell-table-paging = Pages of { $table }
admin-shell-table-prev = Previous page
admin-shell-table-next = Next page

# -- Charts ----------------------------------------------------------------------
admin-shell-chart-point = { $time }: { $value }
admin-shell-chart-summary = Latest { $latest }, highest { $max }
admin-shell-chart-col-time = Time
admin-shell-chart-col-item = Item
admin-shell-chart-col-value = Value

# -- Pinned value, lock switch, restore ------------------------------------------
admin-shell-pinned-prefix = Set by
admin-shell-pinned-sr = This value comes from the server's environment and cannot be changed here.
admin-shell-lock-text = Lock for users
admin-shell-lock-setting = { $setting }
admin-shell-restore = Restore previous value
admin-shell-restore-note = Changed by { $actor } on { $time }. Previous value: { $value }
admin-shell-restore-done = The previous value is restored.

# -- Wizard ----------------------------------------------------------------------
admin-shell-wizard-steps = Steps
admin-shell-wizard-step-n = { $n }. { $title }
admin-shell-wizard-step-of = Step { $n } of { $total }: { $title }
admin-shell-wizard-review = Review
admin-shell-wizard-back = Back
admin-shell-wizard-next = Next
admin-shell-wizard-saved = Saved.

# -- Errors: what failed and what to do ------------------------------------------
admin-shell-err-network = The server could not be reached. Check the connection and try again.
admin-shell-err-session = Your admin session has ended. Sign in again.
admin-shell-err-forbidden = Your role does not allow this action. Ask an admin with the right role.
admin-shell-err-out-of-scope = This object belongs to a domain your role does not cover.
admin-shell-err-not-found = It no longer exists. Reload the list.
admin-shell-err-conflict = Someone changed it in the meantime. Reload and try again.
admin-shell-err-validation = The server refused a value. Check the fields and try again.
admin-shell-err-rate-limited = Too many requests. Wait a minute and try again.
admin-shell-err-server = The server answered with an error ({ $status }). Try again; if it repeats, check the server log.
admin-shell-err-upstream-dns = The host name could not be resolved. Check the spelling of the host.
admin-shell-err-upstream-connect = Nothing answered on that host and port. Check the port and any firewall in between.
admin-shell-err-upstream-tls = The server answered but the TLS handshake failed. Check the port and the connection security setting.
admin-shell-err-upstream-auth = The server refused the credentials. Check the username and password.

# -- Sign-in: throttling and the second step --------------------------------------
admin-shell-login-throttled = Too many sign-in attempts. Wait a few minutes and try again.
admin-shell-totp-form = Admin sign in, second step
admin-shell-totp-note = Enter the 6-digit code from the authenticator app set up for { $name }.
admin-shell-totp-code = Authentication code
admin-shell-totp-verify = Verify
admin-shell-totp-start-over = Start again
admin-shell-totp-invalid = That code is not valid. Enter the current code from your authenticator app.
admin-shell-totp-expired = The sign-in took too long. Start again and enter your password.
