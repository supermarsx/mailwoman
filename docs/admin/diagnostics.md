# Server diagnostics

Open **Operations → Diagnostics** to run the server suite and review saved runs.
Reading history requires the unscoped `diagnostics.read` grant; running checks
requires unscoped `diagnostics.run`. A session can start two runs per minute.

The server suite performs these checks:

- Query the configured database.
- Insert a synthetic settings row in a transaction, then roll it back.
- Compare the applied migration count with this server build.
- Seal and open a synthetic sample with the server key.
- Verify that the default address policy rejects representative private and
  loopback addresses.

Network and ports:

- `network-public-dns`: the host name in `MW_PUBLIC_URL` resolves from this
  server (skipped when `MW_PUBLIC_URL` is not set).
- `network-public-port`: the port of the public URL accepts a TCP connection
  from this server. A failure is a warning, because NAT hairpinning can block
  it while outside clients still connect.
- `network-backend-ports`: how many saved, enabled backend host:port pairs (up to
  32) accept a direct TCP connection under the same address policy as a sign-in.
  Nothing is written to the socket. It needs `domains.upstream_write` and is
  skipped without it. A failure is a warning, because a configured egress route
  may be the only way to reach a backend; **Check all saved backends** tests the
  route and TLS.

Security posture:

- The deployment checks from **Diagnostics → Deployment security** that report
  pass or warning: HTTPS, Secure cookies, HSTS, strict CSRF, administrator
  second factors, sign-in pacing, credential locks, sign-in geography data and
  render isolation.
- `security-plaintext-backends`: fails when a saved backend allows passwords
  without TLS on a non-private network, warns when only private-network
  backends do.
- `fail2ban-client-address`: classifies the address a fail2ban line would name
  for the admin request that ran the suite. It fails when requests carry no
  client address, and warns when the address is loopback or private, which
  behind a reverse proxy means `MW_TRUSTED_PROXIES` is missing and fail2ban
  would ban the proxy. See [fail2ban](../deploy/fail2ban.md).

Statuses are `pass`, `warn`, `fail` and `skipped`. A run with a failure is
`fail`; otherwise a run with a warning is `warn`. Summaries contain counts and
fixed text, never host names, addresses or upstream answers.

Each step has a five-second deadline. A failed check or deadline reports a
failure. Reports contain fixed descriptions, status and elapsed time, without
database errors, credentials or sampled user data. Audit and history failures
are reported separately: a successful probe does not imply its report was saved.

History is paginated in groups of 50. `diagnostics.retentionDays` sets the global
retention period (1–365 days, default 30). Expired completed runs are removed when
history is read and after a suite runs; cleanup is not a background timer. A
cleanup failure is reported rather than silently claiming retention succeeded.

The API is `POST /admin/diag/run` with `{"suite":"server"}`, and
`GET /admin/diag/runs` with an optional opaque `cursor` from `nextCursor`.
Responses are marked private and no-store. Unsupported suite names are rejected.

**Check all saved backends** starts an asynchronous job. It requires both
unscoped `diagnostics.run` and `domains.upstream_write`, matching the existing
domain-probe authority. It includes every saved priority backend, not just the
preferred selection, plus saved CalDAV/CardDAV URLs. Disabled entries are
reported as skipped. These are connection/security handshakes and guarded HTTP
requests, not authenticated mailbox or delivery tests. DNS resolution and the
configured address/route protections remain enforced. A changed or disabled
domain is refused when its queued check reaches the network stage.

Jobs are bounded to 128 checks, four concurrent connections across jobs, eight
active jobs globally and two per requesting session. Each check has a 15-second
budget including queue time; the run has a 60-second budget. Cancel stops queued
and in-flight work. Polling and cancellation belong to the admin session that
started the job. Live job state expires after 30 minutes and does not survive a
server restart; completed reports use the existing persisted history.

Cancellation is stored as a warning with an explicit termination result. A run
deadline is an error. Truncated inventories/reports are warnings rather than
claims that all targets passed. Stored result JSON is capped below 64 KiB;
omitted results are counted explicitly.

The additional APIs are `GET /admin/diag/catalogue`, `POST /admin/diag/jobs`
with `{"suite":"backends"}`, `GET /admin/diag/jobs/{id}`, and
`POST /admin/diag/jobs/{id}/cancel`. Filter results by status, domain, backend or
description; domain links lead back to configuration for remediation.

**Copy/Export troubleshooting report** uses `GET /admin/diag/report` and adds
the currently selected run. It includes build version/platform/mode/uptime,
allowlisted runtime security flags and timeouts, and the latest ten stored runs.
Domain/backend identifiers and administrative actor names may be present.
Tick **Replace domain names and backend IDs in the report** (API:
`?omitDomains=true`) to replace every saved domain name, wherever it appears in
the text, with `domain-N`, and backend IDs with `backend-N`, numbered in order of
first appearance. With `?selected=<run id>` the server attaches that run from
saved history, with the same placeholders. A selected run that is not in the
latest ten saved runs is left out when names are replaced
(`"selectedRunOmitted": true`).
It excludes raw configuration, credentials, tokens, cookies, keys, mail,
certificate blobs and upstream greeting/error text. Review a report before
sharing it outside your organization.

These checks do not establish SMTP delivery, authenticated provider operations,
certificate renewal, full database integrity, or decryption of all stored data.
Arbitrary local services and pipes have no diagnostic adapter and are explicitly
listed as unsupported. No shell command execution or automated repair is offered.
