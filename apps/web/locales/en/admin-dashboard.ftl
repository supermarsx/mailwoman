# Mailwoman — admin dashboard (source locale: en). t32-admin.md §2.2, t32-ext.md §K.
# Ids are `admin-dash-*`. Every figure on the page is one the server read; these
# strings never stand in for a number.

admin-dash-refresh = Refresh
admin-dash-unavailable = The dashboard figures are not available yet.
admin-dash-unavailable-detail = This server does not provide the dashboard data (GET /admin/dashboard answers 404). It arrives with a later server update. The other sections work as before.
admin-dash-error-network = The dashboard could not be loaded: the server did not answer. Try again.
admin-dash-error-status = The dashboard could not be loaded: the server answered { $status }. Try again; if it repeats, check the server log.

# -- Warnings --------------------------------------------------------------------
admin-dash-warnings = Needs attention
admin-dash-warn-fix = Open the section that fixes this
admin-dash-warn-blocked-accounts = { $count ->
    [one] One account cannot sign in because its domain has no configuration.
   *[other] { $count } accounts cannot sign in because their domains have no configuration.
}
admin-dash-warn-no-trusted-proxy = IP rules are active but no trusted proxy is configured, so every client may appear to come from the proxy's address.
admin-dash-warn-allowlist-bypass = An environment variable turns the IP allowlist off.
admin-dash-warn-metrics-token = The metrics endpoint has no token set.
admin-dash-warn-geoip-missing = A country policy is active but no usable country database is loaded, so sign-ins it covers are refused.
admin-dash-warn-geoip-stale = A country policy is active and the country database is out of date.
admin-dash-warn-pending-migrations = { $count ->
    [one] One database migration has not been applied. Restart the server to apply it.
   *[other] { $count } database migrations have not been applied. Restart the server to apply them.
}
admin-dash-warn-cert-expiring = { $count ->
    [one] One upstream certificate expires soon (from the last connection test).
   *[other] { $count } upstream certificates expire soon (from the last connection test).
}
admin-dash-warn-unknown = Server warning { $code } ({ $count })

# -- Blocked accounts (DA3) ---------------------------------------------------------
admin-dash-blocked-title = { $count ->
    [one] One account blocked: no domain configuration
   *[other] { $count } accounts blocked: no domain configuration
}
admin-dash-blocked-note = These accounts signed in before this update, but their domain has no configuration in the console, so they cannot sign in now. Configuring the domain lets them sign in again.
admin-dash-blocked-chart = Blocked accounts per domain
admin-dash-blocked-row = { $count ->
    [one] one account
   *[other] { $count } accounts
}
admin-dash-blocked-seen = (last used mail host { $host })
admin-dash-configure-domain = Configure { $domain }
admin-dash-blocked-no-access = Your role cannot configure domains.

# -- First-run checklist -------------------------------------------------------------
admin-dash-checklist = { $open ->
    [0] Setup checklist: nothing open
    [one] Setup checklist: one item open
   *[other] Setup checklist: { $open } items open
}
admin-dash-check-done = Done
admin-dash-check-open = Open
admin-dash-check-optional = (optional)
admin-dash-check-go = Open
admin-dash-check-hide = Hide
admin-dash-check-unhide = Show again
admin-dash-check-hidden = { $count ->
    [one] One item is hidden. Hiding does not mark it done.
   *[other] { $count } items are hidden. Hiding does not mark them done.
}
admin-dash-check-show-hidden = Show hidden items
admin-dash-check-hide-hidden = Leave hidden items out
admin-dash-check-domain-exists = Add a mail domain
admin-dash-check-domain-upstream = Give the domain a mail server, or set a default upstream
admin-dash-check-test-connection = Run a connection test that succeeds (within the last 7 days)
admin-dash-check-preferred-domain = Choose a preferred domain, so users can sign in without typing it
admin-dash-check-second-admin = Create a second admin account, or turn off the environment sign-in
admin-dash-check-trusted-proxies = Configure the trusted proxies in front of this server
admin-dash-check-security-policy = Save the security policy
admin-dash-check-metrics-token = Set a token for the metrics endpoint
admin-dash-check-branding = Review the branding

# -- Tiles ---------------------------------------------------------------------------
admin-dash-overview = Accounts and sessions
admin-dash-users = Provisioned users
admin-dash-signed-in-24h = Accounts signed in, last 24 hours
admin-dash-signed-in-7d = Accounts signed in, last 7 days
admin-dash-domains = Domains
admin-dash-domains-detail = { $upstream } with a mail server, { $disabled } disabled
admin-dash-sessions = Active webmail sessions
admin-dash-admin-sessions = Active admin sessions
admin-dash-security = Sign-in security
admin-dash-failed-webmail = Failed webmail sign-ins, last 24 hours
admin-dash-failed-admin = Failed admin sign-ins, last 24 hours
admin-dash-active-blocks = Blocked addresses
admin-dash-allowlists = IP allowlists
admin-dash-allowlists-value = Admin { $admin }, webmail { $webmail }
admin-dash-on = on
admin-dash-off = off
admin-dash-trusted-proxies = Trusted proxies
admin-dash-configured = Configured
admin-dash-not-configured = Not configured
admin-dash-untrusted-forwarded = { $count ->
    [one] One request in 24 hours carried forwarding headers from an untrusted peer
   *[other] { $count } requests in 24 hours carried forwarding headers from an untrusted peer
}

# -- Traffic ---------------------------------------------------------------------------
admin-dash-traffic = Traffic, last { $mins } minutes
admin-dash-traffic-note = Counted by this server process since it started. Other processes or replicas are not included.
admin-dash-traffic-empty = No requests were counted in this window.
admin-dash-requests = Requests per minute
admin-dash-errors = Server errors (5xx) per minute
admin-dash-p95 = Response time, 95th percentile

# -- Health ----------------------------------------------------------------------------
admin-dash-health = Health
admin-dash-h-ok = OK
admin-dash-h-attention = Needs attention
admin-dash-h-db = Database
admin-dash-h-db-ok = Answered in { $ms }
admin-dash-h-migrations = Migrations
admin-dash-h-migrations-ok = Up to date ({ $n } applied)
admin-dash-h-migrations-behind = { $applied } of { $expected } applied
admin-dash-h-engine = Mail sync
admin-dash-h-engine-value = { $accounts } accounts registered, { $errors } with a sync error
admin-dash-h-outbox = Outbox
admin-dash-h-outbox-value = { $pending } waiting (oldest { $oldest }), { $failed } failed in 24 hours
admin-dash-h-outbox-none = none
admin-dash-h-index = Search index
admin-dash-h-index-value = { $n } documents
admin-dash-h-index-rebuilding = Rebuilding ({ $n } documents so far)
admin-dash-h-sandbox = Render sandbox
admin-dash-h-sandbox-ok = Available: untrusted documents are rendered in a separate process.
admin-dash-h-sandbox-missing = Missing: untrusted documents are rendered inside the server process.
admin-dash-h-sandbox-required = Missing, and this server requires it: rendering untrusted documents fails.
admin-dash-h-plugins = Plugin runtime
admin-dash-h-plugins-ok = Available
admin-dash-h-plugins-off = Turned off
admin-dash-na-proxy = this server runs in proxy mode and does not sync mail itself.
admin-dash-na-unreported = the server did not report it.

# -- Storage ---------------------------------------------------------------------------
admin-dash-storage = Storage
admin-dash-db-size = Database file
admin-dash-wal-size = Write-ahead log
admin-dash-index-size = Search index
admin-dash-volume-free = Free space on the data volume
admin-dash-volume-of = of { $total }
admin-dash-na-wal = This database does not use a write-ahead log file, or the server did not report it.
admin-dash-na-index = This server has no search index.
admin-dash-na-volume = The server cannot read free space on this platform.

# -- Recent audit, server ----------------------------------------------------------------
admin-dash-recent-audit = Recent admin actions
admin-dash-audit-empty = No admin actions are recorded yet.
admin-dash-audit-when = When
admin-dash-audit-actor = Who
admin-dash-audit-action = Action
admin-dash-audit-target = Target
admin-dash-audit-all = Open the full audit log
admin-dash-server = Server
admin-dash-version = Version
admin-dash-build = Build
admin-dash-mode = Mode
admin-dash-mode-engine = Engine: this server syncs and stores mail
admin-dash-mode-proxy = Proxy: this server passes requests to the mail server
admin-dash-uptime = Running for
admin-dash-base-path = Served at
admin-dash-base-root = the site root
admin-dash-duration-dh = { $d } d { $h } h
admin-dash-duration-hm = { $h } h { $m } min
admin-dash-duration-m = { $m } min
admin-dash-duration-s = { $s } s
admin-dash-customize = Customize dashboard
admin-dash-limitations = Data availability
