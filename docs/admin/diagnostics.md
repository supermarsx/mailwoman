---
title: Diagnostics
screen: true
---
# Diagnostics

**Diagnostics** (under **Operations** in the console navigation) runs checks on this server and on the mail servers saved for your domains, keeps the results, and builds a report you can send to someone who is helping you. The checks are read-only: they send no credentials, deliver no mail and repair nothing.

![The Overview tab of Diagnostics after Run all checks: one tile per group with counts of OK, warning, failed and not run checks, and the Needs attention list.](/screenshots/admin-ops/opsa-diagnostics-overview.webp){.mw-shot}

## Before you start

- Reading results needs the `diagnostics.read` permission, and running checks needs `diagnostics.run`. Both must not be limited to some domains.
- **Mail backends** checks also need the permission to change domain connections (`domains.upstream_write`). Without it they are skipped, and **Run all checks** runs the server checks only.
- An administrator session can start a limited number of runs: the burst is 10 runs and the session regains 10 runs per minute. Starting a check that is already running returns that run instead of counting again. The limits are the settings in the table under [Options](#options).

## Run all checks

1. Open **Diagnostics**. The **Overview** tab shows one tile per group (**Server**, **Connectivity**, **Mail backends**, **Security posture**) with the count of OK, warning, failed and not run checks.
2. Choose **Run all checks**. It runs the server checks, then checks every saved backend. A progress line shows **Step 1 of 2: server checks.** and then **Step 2 of 2: saved backends.**, and a message **All checks finished. The results are saved in History.** appears at the end.
3. Read **Needs attention**. It lists every check whose latest run was a warning or a failure; choose one to open its tab.
4. **Last server checks** and **Last backend checks** give the time of the latest run of each kind.

Open a group tile, or one of the tabs, to run only that part.

![The eight tabs of Diagnostics with markers 1 to 8: Overview, Server, Connectivity, Mail backends, Security posture, Tools, History and Report.](/screenshots/admin-ops/opsa-diagnostics-tabs.webp){.mw-crop}

1. **Overview**: the latest result of every check.
2. **Server**: the database and the server key.
3. **Connectivity**: the public name and port, backend ports, and the private-address rule, plus the domain tools.
4. **Mail backends**: a handshake with every saved mail, submission, rules and DAV endpoint.
5. **Security posture**: sign-in and session protections.
6. **Tools**: links to other places that check the service.
7. **History**: saved runs.
8. **Report**: the troubleshooting report.
{.mw-callouts}

Each check has a title and a one-line purpose, a status chip (**OK**, **Warning**, **Failed**, **Skipped**, **Not run** or **Running**), a one-line purpose, **Run** or **Run again**, and, when it did not pass, **How to fix**. **Details** shows the raw result and **Copy details** copies it. A check that says **The last server run did not report this check; it does not apply to this deployment.** is not relevant to your installation.

A run is **Failed** if any check failed, otherwise **Warning** if any check warned. Each server check has a five-second deadline, and a check that exceeds it is a failure.

## The checks

### Server

| Check | What it tests |
|---|---|
| **Database connection** | The database answers a simple query. |
| **Database writes** | A scratch row can be written, then it is rolled back. |
| **Database migrations** | The database has every migration this server version expects. |
| **Server key** | The server key seals and opens a test value. |
| **PDF viewer files** | The PDF viewer's files are all present, correctly typed and match this server build. |

### Connectivity

| Check | What it tests |
|---|---|
| **Public name** | The host name in `MW_PUBLIC_URL` resolves from this server. It is skipped when `MW_PUBLIC_URL` is not set. |
| **Public port** | The port of the public URL accepts a connection from this server. A failure is a warning, because a router that cannot connect to its own public address (NAT hairpinning) can block it while outside clients still connect. |
| **Backend ports** | The host and port of each saved, enabled backend (up to 32) accept a direct connection, under the same address policy as a sign-in. Nothing is written to the connection. A failure is a warning, because an outbound route may be the only way to reach a backend. |
| **Private address protection** | By default, outgoing connections to loopback and private addresses are refused. |

The **Connectivity** tab also has **Domain tools** for one saved domain: **DNS records** (MX, SPF, DMARC and service records), **TLS and handshake** (TLS version, certificate expiry and time taken for each saved endpoint) and **Discover settings** (published autoconfig, autodiscover and service records; nothing is saved). They do not sign in or send mail.

### Mail backends

A connection and security handshake with every saved mail, submission, rules and DAV endpoint, including backends that are not the preferred one. No credentials are sent. Disabled entries are reported as skipped. The tab also shows, for each domain, whether it may switch to another backend when the first cannot be reached (**Failover**). If the list of saved backends cannot be read, the card says so and asks you to save the domain's connections again.

These checks run as one job with at most 128 checks, four connections at a time, 15 seconds for each check including its time in the queue, and 60 seconds for the whole run. A run that exceeds the inventory limit says **The inventory exceeded the bounded check budget. This run is partial.** **Cancel checks** stops queued and running work; a cancelled run is saved as a warning. Only the administrator session that started a job can poll or cancel it, and a running job is forgotten after 30 minutes or when the server restarts (the saved report stays in **History**).

### Security posture

![The Security posture tab of Diagnostics after a run: rows for HTTPS, secure cookies, HSTS, CSRF protection and the other posture checks, each with a status chip, its result and, for a warning, how to fix it.](/screenshots/admin-ops/opsa-diagnostics-posture.webp){.mw-shot}

| Check | Passes when | Fix it with |
|---|---|---|
| **HTTPS** | Requests reach this server over HTTPS. | Serve the console over HTTPS, or set `MW_PUBLIC_URL` to https and `MW_TRUSTED_PROXIES` when a reverse proxy ends TLS. |
| **Secure cookies** | Session cookies are only sent over HTTPS. | Set `MW_COOKIE_SECURE` to true. |
| **HSTS** | Browsers are told to use HTTPS for this site. | Turn on Strict-Transport-Security in the security settings, or send it from the reverse proxy. |
| **CSRF protection** | Requests that change data must carry the CSRF token. | Set `MW_CSRF_STRICT` to true. |
| **Administrator second factor** | Every administrator has a second sign-in factor. | Ask each administrator without one to add a passkey or an authenticator app in **My preferences**, or require it in the **Security centre**. |
| **Sign-in retry delay** | A short wait follows a failed sign-in. | Turn on sign-in pacing in the security policy. |
| **Credential locks** | Repeated failed sign-ins lock the credential for a while. | Turn on automatic locks in the **Security centre**. |
| **Sign-in geography data** | The sign-in geography policy has the country data it needs. | Install or update the country database, or turn off the geography policy. |
| **Isolated message rendering** | Messages are rendered in the separate, isolated worker. | Check that `MW_RENDER_BIN` points at the `mw-render` program and that it starts. |
| **Deployment settings** | The deployment security settings can be read. | Check the database connection, then open the deployment security assessment. |
| **Passwords over unencrypted connections** | No saved backend accepts passwords without TLS. It fails for a backend on a public network and warns for one on a private network. | Edit the domain's connections and choose TLS or STARTTLS. |
| **fail2ban client address** | The address a fail2ban log line would name is the real client's address. It fails when requests carry no client address and warns for a loopback or private address. | Behind a reverse proxy, set `MW_TRUSTED_PROXIES` to the proxy's address, or fail2ban bans the proxy. A loopback address is expected when you open the console on the server itself. See [fail2ban](/deploy/fail2ban). |

The tab has a **Deployment security** button that opens the full assessment.

## Use History and the report

- **History** lists saved runs, newest first (50 at a time; **Older runs** loads more). Choose a run to see its results and **Changes**, what differs from the run of the same kind before it.
- **Report** builds the troubleshooting report as JSON. Choose the run to include, then **Copy troubleshooting report**, **Export troubleshooting report** (a file named `mailwoman-diagnostics.json`) or **Preview**.
- **Replace domain names and backend IDs in the report** is ticked at first. It replaces every saved domain name with `domain-N` and every backend ID with `backend-N`, numbered in order of first appearance. With it ticked, a selected run that is not among the latest ten saved runs is left out of the report.
- The report contains the build version, platform, mode and uptime, a fixed list of runtime security flags and timeouts, and the latest ten stored runs. It never contains raw configuration, credentials, tokens, cookies, keys, mail or certificate contents. Read it before you share it outside your organization.

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| [`diagnostics.retentionDays`](/admin/settings#diagnostics.retentiondays) | Days a saved run is kept (1 to 365). Expired runs are removed when a run is saved and when history is read, not by a timer. | 180 | Administrators with settings access (global) |
| [`rateLimit.diagnosticsBurst`](/admin/settings#ratelimit.diagnosticsburst) (**Diagnostic runs at once**) | Runs and backend checks an administrator session may start in a row. | 10 | Administrators with the security policy permission |
| [`rateLimit.diagnosticsPerMinute`](/admin/settings#ratelimit.diagnosticsperminute) (**Diagnostic runs per minute**) | Runs and backend checks per minute the session regains. | 10 | Administrators with the security policy permission |

## What the checks do not prove

They do not establish that mail is delivered, that a mailbox sign-in works, that a certificate will renew, that the whole database is intact, or that all stored data can be decrypted. Arbitrary local services and pipes have no check; the catalogue under **Tools** (**Supported checks and limits**) lists them as unsupported. There is no shell command and no automatic repair.

## Troubleshooting

- **Run all checks is disabled.** Another run is in progress, or your role lacks `diagnostics.run`.
- **The tab says "Results are shown, but history or audit storage failed. Check database availability before leaving this page."** The checks ran, but the result could not be saved. Fix the **Database writes** check first.
- **The message "Results were saved, but expired diagnostic history could not be removed."** appears after a run: the cleanup of old runs failed; the run itself is saved.
- **Database connection fails.** Check that the database is running and that `MW_DB_PATH` or `MW_DATABASE_URL` points at it. The server log names the connection error.
- **Database migrations fails.** Restart the server so that it applies the pending migrations, and read its log for a migration that failed. Do not run an older server on a newer database.
- **Server key fails.** `MW_SERVER_KEY` (or the key file) is not the key this database was created with. Stored secrets cannot be opened with another key.
- **Public port is a warning but users connect fine.** This is the NAT hairpinning case described above. Test from outside your network.
- **Mail backends says the saved backends have not been checked, or fails for one host.** Check the host name, port and TLS mode saved for that backend, that DNS resolves, and that the certificate is valid for the host name, then run again.
- **fail2ban client address warns on every run.** The address is loopback or private; behind a reverse proxy that means `MW_TRUSTED_PROXIES` is missing.

## Related

- [Logs](/admin/logs)
- [Observability](/admin/observability)
- [fail2ban](/deploy/fail2ban)
- [Settings reference](/admin/settings)
