# Mailwoman — admin console shell and shared admin components (source locale: en).
# t32-admin.md §2.1, t32-ext.md §K/§M. Ids are `admin-shell-*`. Section labels that
# existed before t32 stay in admin.ftl (`admin-nav-*`).

# -- Navigation ---------------------------------------------------------------
admin-shell-nav-dashboard = Dashboard
admin-shell-nav-settings = Settings
admin-config-title = Effective configuration
admin-diag-title = Diagnostics
admin-diag-description = Run server checks or connection checks for saved mail and DAV endpoints. These checks do not authenticate to providers, deliver mail, or test arbitrary services.
admin-diag-run = Run server checks
admin-diag-refresh = Refresh history
admin-diag-result = Check result
admin-diag-checks = Check results
admin-diag-check = Check
admin-diag-status = Status
admin-diag-duration = Duration
admin-diag-history = Previous runs
admin-diag-empty = No saved diagnostic runs.
admin-diag-more = Older runs
admin-diag-not-saved = Results are shown, but history or audit storage failed. Check database availability before leaving this page.
admin-diag-unavailable = This server does not provide the diagnostics suite yet.
admin-config-description = Inspect the server's redacted configuration report. Global policy values can have domain, group and user overrides. Change settings in their dedicated editors.
admin-config-refresh = Refresh configuration
admin-config-search = Search configuration
admin-config-group = Area
admin-config-all = All areas
admin-config-all-sources = All sources
admin-config-clear = Clear filters
admin-config-no-results = No configuration entries match these filters.
admin-config-count = { $count } matching entries
admin-config-table = Configuration visible to your administrator role
admin-config-key = Setting
admin-config-value = Reported value
admin-config-source = Source
admin-config-change = How changes apply
admin-config-unknown = Runtime value unavailable
admin-config-empty = Not set
admin-config-secret-set = Secret configured (hidden)
admin-config-secret-unset = Secret not configured
admin-config-input = Configured input
admin-config-input-note = This input is not proof of the value currently used by the subsystem.
admin-config-locked = Policy locked
admin-config-restart = Restart or rebuild required
admin-config-live = See setting behavior
admin-config-open-setting = Open settings editor
admin-config-more = Show more entries
admin-config-unavailable = This server does not provide a configuration report.
admin-config-source-default = Default
admin-config-source-stored = Stored policy
admin-config-source-env-pin = Environment override
admin-config-source-env-fallback = Environment fallback
admin-config-source-boot = Captured at startup
admin-config-source-build = Build value
admin-config-source-refused = Unsupported
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
admin-diag-retention-failed = Results were saved, but expired diagnostic history could not be removed.
admin-auth-title = Authentication logs
admin-auth-description = Authentication metadata kept for up to { $days } days, subject to the entry limit. Repeated matching failures are grouped within one minute. Unknown details are not inferred.
admin-auth-unavailable = Authentication history is unavailable on this server.
admin-auth-unknown = Unavailable
admin-auth-search = Account, attempted username or IP
admin-auth-surface = Sign-in area
admin-auth-all = All
admin-auth-admin = Administration
admin-auth-result = Result
admin-auth-success = Success
admin-auth-failure = Failure
admin-auth-blocked = Blocked
admin-auth-backend = Backend (exact identifier)
admin-auth-reason = Reason (exact code)
admin-auth-apply = Apply filters
admin-auth-time = First attempt
admin-auth-account = Account or attempted username
admin-auth-method = Method
admin-auth-ip = Source IP
admin-auth-count = Attempts
admin-auth-last = Last:
admin-auth-device = Device details
admin-auth-empty = No matching attempts.
admin-auth-more = Older attempts
admin-logs-title = Logs
admin-app-logs-title = Application logs
admin-app-logs-description = Warnings and errors captured by this server process, subject to its log filter. Up to 1,000 entries from the last 24 hours; cleared on restart. Matching events within one minute are grouped. Sensitive fields are redacted.
admin-app-logs-discarded = Entries discarded from this process buffer: { $count }
admin-app-logs-unavailable = Application logs are unavailable on this server.
admin-app-logs-search = Search redacted details
admin-app-logs-target = Component
admin-app-logs-level = Severity
admin-app-logs-details = Redacted details
admin-app-logs-empty = No matching warnings or errors in this process buffer.
admin-config-filters = Find configuration

admin-config-entries = Configuration entries
admin-shell-sidebar-resize = Resize navigation sidebar
admin-shell-sidebar-show = Show navigation sidebar
admin-shell-sidebar-hide = Hide navigation sidebar
admin-diag-backends = Check all saved backends
admin-diag-cancel = Cancel checks
admin-diag-cancelling = Cancellation requested. In-flight checks are being stopped.
admin-diag-copy = Copy troubleshooting report
admin-diag-export = Export troubleshooting report
admin-diag-copied = Redacted troubleshooting report copied.
admin-diag-limited = The inventory exceeded the bounded check budget. This run is partial.
admin-diag-redaction = Checks use saved targets and send no credentials. Reports include runtime security flags, domain names, backend identifiers and results; never passwords, tokens, raw configuration or mail. Authenticated provider actions, arbitrary services and pipes are not tested.
admin-diag-filter = Filter checks, domains or backends
admin-diag-all = All statuses
admin-diag-catalogue = Supported checks and limits
admin-diag-cancelled = Checks cancelled. Unfinished connections were stopped.
admin-diag-backends-permission = Requires global diagnostics-run and domain-upstream configuration permissions.
admin-diag-no-checks = No checks were recorded for this run. No endpoint success is implied.
admin-diag-no-match = No checks match this filter.
admin-diag-limits = Up to { $checks } checks, { $concurrency } concurrent connections, { $check } seconds per check including queue time, and { $run } seconds for probe work.

admin-login-retry-countdown = Try again in { $seconds } seconds.
admin-analytics-title = Usage analytics
