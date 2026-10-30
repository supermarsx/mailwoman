# t105: admin console Diagnostics, grouped tabs and check cards.
# The older admin-diag-* ids (buttons, history table, report) stay in admin-shell.ftl.

## Tabs
admin-diagx-tabs = Diagnostics pages
admin-diagx-tab-overview = Overview
admin-diagx-tab-server = Server
admin-diagx-tab-connectivity = Connectivity
admin-diagx-tab-backends = Mail backends
admin-diagx-tab-posture = Security posture
admin-diagx-tab-tools = Tools
admin-diagx-tab-history = History
admin-diagx-tab-report = Report
admin-diagx-tab-server-intro = Checks of this server's own parts: the database, its migrations and the server key. They run with the server checks.
admin-diagx-tab-connectivity-intro = Whether this server is reachable under its public name, can reach the saved backends, and keeps private addresses out of reach. The domain tools below look up DNS records, test TLS and discover settings for one domain.
admin-diagx-tab-backends-intro = A connection and security handshake with every saved mail, submission, rules and DAV endpoint. No credentials are sent and no mail is delivered.
admin-diagx-tab-posture-intro = Settings that protect sign-ins and sessions, and how fail2ban would see the address of a request.
admin-diagx-tab-tools-intro = Other places to check the service, and the limits that apply to diagnostic runs.
admin-diagx-tab-history-intro = Saved runs, newest first. Open a run to see its results and what changed since the run before it.

## Status chips
admin-diagx-status-ok = OK
admin-diagx-status-warn = Warning
admin-diagx-status-fail = Failed
admin-diagx-status-skipped = Skipped
admin-diagx-status-not-run = Not run
admin-diagx-status-running = Running

## Cards
admin-diagx-run = Run
admin-diagx-run-again = Run again
admin-diagx-running = Running…
admin-diagx-run-server-note = Runs the server checks again. Every card from that run updates.
admin-diagx-run-backends-note = Checks all saved backends again. Every backend card updates.
admin-diagx-how-to-fix = How to fix
admin-diagx-details = Details
admin-diagx-copy-details = Copy details
admin-diagx-copy-value = Copy { $label }
admin-diagx-copied = Copied.
admin-diagx-copy-failed = The browser did not allow copying. Select the text and copy it instead.
admin-diagx-not-run-yet = Not run yet.
admin-diagx-not-reported = The last server run did not report this check; it does not apply to this deployment.
admin-diagx-posture-met = Met
admin-diagx-posture-not-met = Not met
admin-diagx-loading = Loading…
admin-diagx-none = None
admin-diagx-yes = Yes
admin-diagx-no = No
admin-diagx-open = Open
admin-diagx-row-result = Result
admin-diagx-row-duration = Duration
admin-diagx-row-checked = Checked
admin-diagx-row-id = Check ID
admin-diagx-row-domain = Domain
admin-diagx-row-backend = Backend
admin-diagx-row-protocol = Protocol
admin-diagx-row-tls = TLS
admin-diagx-row-capabilities = Capabilities
admin-diagx-row-credentials = Credentials sent
admin-diagx-row-source = Found in
admin-diagx-row-oauth = Needs OAuth

## Overview
admin-diagx-overview-intro = The latest result of every check, by group. Run all runs the server checks, then checks every saved backend.
admin-diagx-run-all = Run all checks
admin-diagx-run-all-step = Step { $step } of { $steps }: { $suite ->
    [backends] saved backends
   *[server] server checks
}.
admin-diagx-run-all-done = All checks finished. The results are saved in History.
admin-diagx-backends-progress = { $completed } of { $total } backends checked.
admin-diagx-counts = { $ok } OK · { $warn } warnings · { $fail } failed · { $notRun } not run
admin-diagx-attention = Needs attention
admin-diagx-attention-none = No check reported a warning or a failure in its latest run.
admin-diagx-never-run = No checks have been run yet. Run all checks to get a first result.
admin-diagx-last-server = Last server checks
admin-diagx-last-backends = Last backend checks
admin-diagx-never = Never
admin-diagx-open-health = Service health

## Server checks
admin-diagx-check-database-connectivity = Database connection
admin-diagx-check-database-connectivity-purpose = The database answers a simple query.
admin-diagx-check-database-connectivity-fix = Check that the database is running and that MW_DB_PATH or MW_DATABASE_URL points at it. The server log names the connection error.
admin-diagx-check-database-write = Database writes
admin-diagx-check-database-write-purpose = A scratch row can be written, then it is rolled back.
admin-diagx-check-database-write-fix = Check free disk space and that the database file or user may be written. A read-only volume or a full disk stops writes.
admin-diagx-check-database-migrations = Database migrations
admin-diagx-check-database-migrations-purpose = The database has every migration this server version expects.
admin-diagx-check-database-migrations-fix = Restart the server so it applies pending migrations, and check its log for a migration that failed. Do not run an older server on a newer database.
admin-diagx-check-server-key-roundtrip = Server key
admin-diagx-check-server-key-roundtrip-purpose = The server key seals and opens a test value.
admin-diagx-check-server-key-roundtrip-fix = Check that MW_SERVER_KEY (or the key file) is the key this database was created with. Stored secrets cannot be opened with another key.
admin-diagx-check-other-purpose = A check reported by this server.
admin-diagx-check-other-fix = Open Details for the raw result and check the server log at the time of the run.

## Connectivity checks
admin-diagx-check-network-public-dns = Public name
admin-diagx-check-network-public-dns-purpose = The host name in MW_PUBLIC_URL resolves from this server.
admin-diagx-check-network-public-dns-fix = Check the DNS record of the public name, or set MW_PUBLIC_URL to the address people use to reach this server.
admin-diagx-check-network-public-port = Public port
admin-diagx-check-network-public-port-purpose = The port of the public URL accepts a connection from this server.
admin-diagx-check-network-public-port-fix = Check the firewall, the reverse proxy and port forwarding. Some routers cannot connect to their own public address (NAT hairpinning); test from outside your network as well.
admin-diagx-check-network-backend-ports = Backend ports
admin-diagx-check-network-backend-ports-purpose = The host and port of each saved backend accept a direct connection.
admin-diagx-check-network-backend-ports-fix = Check that the backend hosts are up and that a firewall allows this server to reach them. Mail backends shows which endpoint fails its handshake.
admin-diagx-check-egress-private-policy = Private address protection
admin-diagx-check-egress-private-policy-purpose = By default, outgoing connections to loopback and private addresses are refused.
admin-diagx-check-egress-private-policy-fix = Review the egress settings under Privacy. Allow private addresses only for named backends that need them.

## Security posture checks
admin-diagx-check-security-https = HTTPS
admin-diagx-check-security-https-purpose = Requests reach this server over HTTPS.
admin-diagx-check-security-https-fix = Serve the console over HTTPS, or set MW_PUBLIC_URL to https and MW_TRUSTED_PROXIES when a reverse proxy ends TLS.
admin-diagx-check-security-secure-cookies = Secure cookies
admin-diagx-check-security-secure-cookies-purpose = Session cookies are only sent over HTTPS.
admin-diagx-check-security-secure-cookies-fix = Set MW_COOKIE_SECURE to true once the server is reached over HTTPS.
admin-diagx-check-security-hsts = HSTS
admin-diagx-check-security-hsts-purpose = Browsers are told to use HTTPS for this site.
admin-diagx-check-security-hsts-fix = Turn on Strict-Transport-Security in the security settings, or send it from the reverse proxy.
admin-diagx-check-security-csrf = CSRF protection
admin-diagx-check-security-csrf-purpose = Requests that change data must carry the CSRF token.
admin-diagx-check-security-csrf-fix = Set MW_CSRF_STRICT to true.
admin-diagx-check-security-admin-factors = Administrator second factor
admin-diagx-check-security-admin-factors-purpose = Every administrator has a second sign-in factor.
admin-diagx-check-security-admin-factors-fix = Ask each administrator without one to add a passkey or an authenticator app in My preferences, or require it in the Security centre.
admin-diagx-check-security-signin-pacing = Sign-in retry delay
admin-diagx-check-security-signin-pacing-purpose = A short wait follows a failed sign-in.
admin-diagx-check-security-signin-pacing-fix = Turn on sign-in pacing in the security policy.
admin-diagx-check-security-credential-locks = Credential locks
admin-diagx-check-security-credential-locks-purpose = Repeated failed sign-ins lock the credential for a while.
admin-diagx-check-security-credential-locks-fix = Turn on automatic locks in the Security centre.
admin-diagx-check-security-geography = Sign-in geography data
admin-diagx-check-security-geography-purpose = The sign-in geography policy has the country data it needs.
admin-diagx-check-security-geography-fix = Install or update the country database, or turn off the geography policy.
admin-diagx-check-security-render-isolation = Isolated message rendering
admin-diagx-check-security-render-isolation-purpose = Messages are rendered in the separate, isolated worker.
admin-diagx-check-security-render-isolation-fix = Check that MW_RENDER_BIN points at the mw-render program and that it starts. Service health on the Dashboard shows the worker's state.
admin-diagx-check-security-deployment = Deployment settings
admin-diagx-check-security-deployment-purpose = The deployment security settings can be read.
admin-diagx-check-security-deployment-fix = Check the database connection, then open the deployment security assessment.
admin-diagx-check-security-plaintext-backends = Passwords over unencrypted connections
admin-diagx-check-security-plaintext-backends-purpose = No saved backend accepts passwords without TLS.
admin-diagx-check-security-plaintext-backends-fix = Edit the domain's connections and choose TLS or STARTTLS. Allow plain connections only on a private network you control.
admin-diagx-check-fail2ban-client-address = fail2ban client address
admin-diagx-check-fail2ban-client-address-purpose = The address a fail2ban log line would name is the real client's address.
admin-diagx-check-fail2ban-client-address-fix = Behind a reverse proxy, set MW_TRUSTED_PROXIES to the proxy's address, or fail2ban bans the proxy instead of the client. A loopback address is expected when you open the console on the server itself.

## Mail backends
admin-diagx-backend-title = { $domain } · { $backend }
admin-diagx-backend-purpose = { $protocol } endpoint: connection and security handshake, without signing in.
admin-diagx-backend-other-purpose = A note from the backends run.
admin-diagx-backend-fix = Check the host name, port and TLS mode saved for this backend, that DNS resolves, and that the certificate is valid for the host name. Then run again.
admin-diagx-backend-config-fix = Open the domain's connections and save them again; the saved backend list could not be read.
admin-diagx-backend-results = Backend checks
admin-diagx-backends-not-run = The saved backends have not been checked yet.
admin-diagx-backends-run-at = Last checked { $date }: { $count } results.
admin-diagx-failover-title = Failover
admin-diagx-failover-purpose = Whether a domain may switch to another mail backend when its first choice cannot be reached.
admin-diagx-failover-state = Failover { $state ->
    [on] on
   *[off] off
} · { $backends } mail backends{ $enabled ->
    [no] {" "}· domain disabled
   *[yes] {""}
}
admin-diagx-no-domains = No saved domains.

## Connectivity tools
admin-diagx-domain-tools = Domain tools
admin-diagx-domain-tools-purpose = One-off checks for a saved domain. They do not sign in or send mail.
admin-diagx-domain = Domain
admin-diagx-tool-dns = DNS records
admin-diagx-tool-dns-purpose = Looks up the domain's MX, SPF, DMARC and service (SRV) records.
admin-diagx-tool-dns-fix = Add the missing records at your DNS provider. Without MX the domain receives no mail; without SPF and DMARC other servers may reject its mail.
admin-diagx-srv-found = { $found } of { $total } service records found
admin-diagx-tool-tls = TLS and handshake
admin-diagx-tool-tls-purpose = Connects to each saved endpoint of the domain and reports TLS version, certificate expiry and time taken.
admin-diagx-tool-tls-fix = For an endpoint that cannot be reached, check host, port and firewall. Renew a certificate that expires within 14 days.
admin-diagx-probe-none = The domain has no saved endpoints to test.
admin-diagx-probe-line = { $host } · reachable: { $reachable ->
    [yes] yes
   *[no] no
} · TLS { $tls } · certificate { $days ->
    [none] not reported
   *[other] expires in { $days } days
} · { $time }{ $error ->
    [none] {""}
   *[other] {" "}· { $error }
}
admin-diagx-tool-detect = Discover settings
admin-diagx-tool-detect-purpose = Looks up published mail settings (autoconfig, autodiscover, SRV) for a domain name. Nothing is saved.
admin-diagx-tool-detect-fix = Publish autoconfig or SRV records for the domain, or enter the settings by hand under Domains.
admin-diagx-detect-name = Domain name to look up

## Tools
admin-diagx-tool-health = Service health
admin-diagx-tool-health-purpose = Mail sync, the outbox, the search index, the rendering worker and plugins, on the Dashboard.
admin-diagx-tool-deployment-purpose = Every deployment security setting this server can observe, with the setting that controls it.
admin-diagx-tool-ssh = SSH tunnel tests
admin-diagx-tool-ssh-purpose = Test the SSH tunnels used to reach backends on another network.
admin-diagx-tool-geo = Sign-in geography
admin-diagx-tool-geo-purpose = Where sign-ins come from, and the country database behind the geography policy.
admin-diagx-catalogue-purpose = What each diagnostic suite covers, and the limits each run keeps to.
admin-diagx-suite-unsupported = Not available: { $reason }
admin-diagx-suite-server = Server checks
admin-diagx-suite-backends = Saved backends

## History
admin-diagx-run-title = { $suite ->
    [backends] Saved backends
   *[server] Server checks
}, { $date }
admin-diagx-changes = Changes
admin-diagx-changes-first = No earlier run of this kind to compare with.
admin-diagx-changes-none = Nothing changed since the run of { $date }.
admin-diagx-changes-since = Changed since the run of { $date }:
admin-diagx-change = { $before } → { $after }

## Report
admin-diagx-report-title = Troubleshooting report
admin-diagx-report-purpose = A JSON report for support: runtime flags, the latest results and the run you choose. Passwords, tokens, cookies, keys, raw configuration and mail are never included.
admin-diagx-report-run = Run to include
admin-diagx-report-preview = Preview

## Breadcrumb words for the sub-tabs (the trail resolves admin-crumb-<route word>)
admin-crumb-server = Server
admin-crumb-connectivity = Connectivity
admin-crumb-posture = Security posture
admin-crumb-tools = Tools
admin-crumb-history = History
admin-crumb-report = Report
