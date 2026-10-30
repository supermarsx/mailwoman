# Admin panel (V6)

The admin panel is an operator surface for domains, users, quotas, security policy,
integrations, and observability. It is **separate from the mailbox**: a distinct route
(the console path: `MW_ADMIN_PATH`, or the path stored at the first start), a distinct session domain (its own cookie, `mw_admin_session` unless
`MW_ADMIN_SESSION_COOKIE` names another), and a
distinct credential. A logged-in mailbox user is **not** an admin; an admin session is
obtained only by authenticating at the admin surface.

## Enabling & disabling

| Env | Default | Meaning |
|---|---|---|
| `MW_ADMIN_ENABLED` | `true` | When `false`/`0`, the `/admin/*` routes return `401` — the panel is unreachable and unmounted from the operator's perspective. |
| `MW_ADMIN_USER` | *(unset)* | Admin operator username. Unset → admin login always fails. |
| `MW_ADMIN_PASSWORD` | *(unset)* | Admin operator password. Compared in constant time. |
| `MW_ADMIN_SESSION_COOKIE` | `mw_admin_session` | Name of the admin session cookie: 1 to 64 characters from `A-Z a-z 0-9 _ - .`; `mw_session` and `mw_csrf` are refused. |
| `MW_ADMIN_MIN_TLS` | *(unset)* | `1.2` or `1.3`. Pins the policy's minimum TLS version (see [Security policy](#security-policy)). |
| `MW_ADMIN_REQUIRE_2FA` | *(unset)* | `true` or `false`. Pins the global require-two-factor rule. |
| `MW_ADMIN_PATH` | *(stored path)* | URL path of the console, its sign-in and its API; pins it while set (see [Console path](#console-path)). |

An invalid value in any of these stops the server at start with an error naming the
variable. Since 26.20 the `MW_ADMIN_*` variables that nothing reads —
`MW_ADMIN_PORT`, `MW_ADMIN_LOG_LEVEL`, `MW_ADMIN_OTLP_DSN`, `MW_ADMIN_METRICS`,
`MW_ADMIN_THEME`, `MW_ADMIN_BRAND`, `MW_ADMIN_ACCENT` — also stop the start, with
what to use instead. Before 26.20 they logged a warning and the server started.

Set `admin.enabled = false` (or `MW_ADMIN_ENABLED=0`) for a GitOps-only deployment
where all administration is done via the CLI + config and no HTTP panel is exposed.

## Console path

The console page, the admin sign-in and every `/admin/*` route below are served
at one path only, relative to `MW_BASE_PATH`. With `/ops-7f3k` as that path the
server answers `GET /ops-7f3k/users`, `POST /ops-7f3k/login` and so on, and the
page itself at `/ops-7f3k`. The `/admin/...` routes in this document are written
with the old default path; read them as `{console}/...`.

* **Where it comes from (26.20):** `MW_ADMIN_PATH` when set (it pins the path);
  otherwise the stored path (`admin.consolePath`); on the first start without
  either, a generated path on a new installation (20 random characters, logged
  once) or `/admin` kept on an upgrade, with a warning at every start.
  `mailwoman admin console-path show` prints it.
* **Changing it:** Security > Console address in the console
  (`PUT {console}/console-path`, capability `security_policy.write`, step-up
  action `consolePath`, audit entry `admin-console-path-changed` with a
  fingerprint of the old and new path rather than the path itself), or
  `mailwoman admin console-path set|generate`. A chosen path needs at least 12
  characters after the `/` and may not be `/admin`. The next start applies it and
  ends every admin session (audit entry `admin-console-path-applied`).
* **Accepted values:** `/` followed by one to four segments of
  `A-Z a-z 0-9 . _ ~ -`, no `.` or `..` segment and no trailing `/`. The first
  segment must not be one the server already uses (`api`, `jmap`, `oauth`,
  `.well-known`, `dav`, `caldav`, `carddav`, `mcp`, `metrics`, `healthz`,
  `errors`, `t`, `assets`, `fonts`, `icons`, `themes`, the files at the root of the
  web build) or the first segment of `MW_BASE_PATH`. Anything else stops the start
  with an error naming the variable.
* **Every other path:** `/admin` (once moved) and every other path get exactly
  the answer an unknown path gets (status, headers and body), so probing does not
  lead anywhere or say where the console is. There is no alias.
* **No leak:** the path is not in the web build, the service worker, the mail
  client's page, `/api/policy`, discovery documents, error pages for other paths
  or any response header outside the console, and the application log store
  shows it as `[admin-path]`. The mail app has no link to the console. The
  console page is marked as the console (`<meta name="mw-console" content="admin">`)
  and the web app uses its own address as the console's location.
* **Cookie and headers:** the admin session cookie has
  `Path={MW_BASE_PATH}{console}`, so the browser does not send it to the mail
  app (sign-out also clears a `Path=/` cookie from before 26.20). The console
  page carries `Content-Security-Policy: frame-ancestors 'none'` next to the
  shell policy, `X-Frame-Options: DENY` and `X-Robots-Tag: noindex, nofollow`.
* **What it is worth:** a hidden path reduces scanning and drive-by sign-in
  attempts. It hides the console but does not isolate it: the console shares
  the mail app's origin, so a script running in the mail app that learns the
  path can reach the console API from a browser with an admin session. It does
  not replace the admin password, two-factor sign-in, the sign-in rate limit or
  a network restriction, and it is visible to anyone who can read your proxy
  logs or an admin's browser history.

The sign-in rate limit and the CSRF rules are the same at any path.

## Everything is also CLI + config

Every panel action has a `mailwoman admin <noun> <verb>` CLI equivalent and a
TOML/env binding, so the panel is a convenience over a fully scriptable surface — not
the only way in. This keeps deployments reproducible (GitOps) and lets you disable the
HTTP panel entirely while still administering the server.

## Surface

The `/admin/*` HTTP endpoints (all under the separate admin session):

- **Domains** — `GET/PUT /admin/domains[/{name}]`: managed domains, upstream config,
  allow/blocklists.
- **Users** — `GET/POST /admin/users`, `PUT /admin/users/{id}/quota`,
  `PUT /admin/users/{id}/flags`, `PUT /admin/users/{id}/zero-access`,
  `POST /admin/users/{id}/revoke-sessions`: provisioning, quotas, feature flags
  (including the per-account zero-access toggle), and session revocation.
- **Security policy** — `GET/PUT /admin/security-policy`: see
  [Security policy](#security-policy) below.
- **Integrations** — `GET /admin/integrations`, `GET /admin/webhooks`,
  `GET /admin/api-keys` + `POST /admin/api-keys/{id}/revoke`: webhook and
  API-key/MCP-key oversight. LDAP / Nextcloud entries are shown **inert** ("coming
  soon") — the config surface exists but there is no live directory glue in V6 (that
  is V7).
- **Observability** — `GET/PUT /admin/observability`, `GET /admin/audit`: log level,
  OTLP DSN, the audit-log viewer + export, the login monitor + ban list.
- **Appearance** — branding/theme configuration.

## Disabling an account

`PUT /admin/users/{id}/flags` with `disabled: true` refuses the account here:
password and header-auth logins, SSO, second-factor completion, existing
sessions, native bearer sessions, API keys and OAuth tokens. It does not
disable the mailbox on the mail server; a client that talks to the mail
server directly is not affected.

The flag is stored under the address folded to lower case. A mail server can
accept other spellings of that address, so a sign-in is also refused when it
uses:

- the address with a `+tag` in the local part (`alice+x@example.com` for
  `alice@example.com`);
- the other form of an internationalised domain (`alice@xn--bcher-kva.example`
  for `alice@bücher.example`, either way round);
- the bare local part (`alice`), as a mail server with a default realm accepts
  it. The bare name is refused while **any** `alice@<domain>` is disabled.

The bare-name rule fails closed. On a server with more than one domain, a
bare `alice` that the mail server maps to `alice@example.org` is refused while
`alice@example.com` is disabled, though they are two people. That user can
still sign in with the full address. A tag delimiter other than `+` and an
alias the mail server knows are not recognised: a flag set under one of those
names applies only once someone has signed in with it.

`forcePasswordChange` set under an address holds a sign-in under any of the
spellings above. A password change clears it under the names that session
was opened with, their `+tag`-less form and the other form of the domain. It
does not clear it under an address found only from a bare name, because that
address may be another person's. A user held that way clears the hold by
signing in with the full address and changing the password there, or the
admin clears the flag.

## Security policy

`GET /admin/security-policy` returns every field below, the DLP rules that come from
`MW_DLP_RULES` (`envDlpRules`, read-only here), and `pinnedByEnv`, the fields an
environment variable sets. `PUT` takes all eight fields; it answers `422` with the
reason when a value is out of range or the DLP rules do not parse, `409` naming the
variable when it would change a pinned field, and otherwise `200` with the policy now
in force. The CLI equivalent is `mailwoman admin security-policy show|set`, with the
same checks.

| Field | Values | What it does |
|---|---|---|
| `minTls` | `1.2` / `1.3` | Lowest TLS version for the HTTPS listener and for outbound IMAP, SMTP, POP3, ManageSieve and egress-proxy tunnel connections. New connections only; established ones keep their version. Not applied to LDAP, Redis, Postgres or HTTP fetches. Pinned by `MW_ADMIN_MIN_TLS`. |
| `argon2MCost`, `argon2TCost`, `argon2PCost` | memory 19456..=1048576 KiB, passes 1..=10 (1 needs at least 47104 KiB), lanes 1..=16 | Argon2id cost of new hashes of locally stored account passwords and of recovery codes. An existing password hash is re-hashed at the next successful sign-in. API-key secrets are not affected. |
| `dlpRulesJson` | JSON array of DLP rules | Applied beside `MW_DLP_RULES` to mail sent through engine-mode accounts. An environment rule wins when both use the same `id`. See [`dlp.md`](./dlp.md). |
| `maxSecurityFloor` | `null` / `full-sanitized` / `sanitized-no-media` / `plain-text` | The least restrictive rendering mode the Mailwoman reader offers. Served to signed-in clients at `GET /api/policy`; the server cannot make another mail client honour it. |
| `capturePolicy` | `off` / `protect` | With `protect`, the desktop and Android apps ask the operating system to keep their window out of screenshots and screen recordings. A browser has no such switch. Served at `GET /api/policy`. |
| `require2fa` | `true` / `false` | The global rule of the "Require two-factor" screen (the same stored row, not a copy). Pinned by `MW_ADMIN_REQUIRE_2FA`; while pinned, that screen also answers `409` for the global rule. |

The server applies the policy at start and right after a `PUT`. It also re-reads the
stored policy every 30 seconds, so a change made with the CLI or through another
server process takes effect within that time.

## Audit log (append-only)

Every admin action writes an entry to the **append-only** audit log. There is no update
or delete path in the writer — the invariant is structural, asserted by a test, not a
policy an operator has to uphold. Entries record the timestamp, actor + actor kind,
action, target, a detail payload, and the source IP. The log is viewable and
exportable from the Observability section (and via `mailwoman admin`).

## Login monitor & ban list

Failed admin logins feed a login monitor and a ban list emitted in **fail2ban-compatible
log format**, so an operator can wire the existing fail2ban tooling against Mailwoman's
admin surface without a bespoke parser. Sources that search for the console are
handled by [admin-probe protection](#admin-probe-protection).

## Admin-probe protection

A hidden console path stops scanners from finding the console. Admin-probe
protection (26.20, `security.adminProbe.*`) also stops them from searching for
it: a source that keeps looking is slowed and then blocked. It is on by default
and leaves room for honest mistakes.

**What counts as a miss.** Each of these costs the client's source one point:

| Signal | When |
|---|---|
| `probe-path` | The path contains an entry of `security.adminProbe.probePaths` as whole segments, any case (`/wp-admin`, `/blog/wp-login.php`, `/.env`, `/manager/html`, and `/admin` once the console has moved). The console's own path and the server's own prefixes (`/api`, `/jmap`, `/.well-known`, …) never count. |
| `slug-guess` | A path with as many segments as the console path answered 404. File names a browser or crawler asks for on its own (`favicon.ico`, `robots.txt`, `*.png`, `*.xml`, …) do not count. |
| `admin-signin-failed` | `POST {console}/login…` refused (401, or 429 from the sign-in limit). |
| `admin-unauthenticated` | An admin API call answered 401. A request that carries a session cookie counts once per cookie, so an expired console tab that keeps polling costs one point. The console page, `{console}/session` and sign-out do not count. |

**Room for innocent failures.**

* The score drains continuously at `graceAllowance` points per `windowSecs`
  (default 10 per 10 minutes). The first 10 misses in a window do nothing, so a
  mistyped URL or a stale `/admin` bookmark never triggers anything.
* A successful admin sign-in clears its source's score and repeat count.
* `security.adminProbe.allowlist` networks (office, VPN) are never counted.
* A configured trusted proxy is never counted itself. Behind one, the source is
  the forwarded client address (`MW_FORWARDED_MODE` + `MW_TRUSTED_PROXIES`); a
  forwarded header from any other peer is ignored, so rotating it does not
  reset the score.
* IPv6 sources are counted per /64, IPv4 per address.

**Escalation.**

1. Above the grace allowance, each further miss is answered after
   `tarpitDelayMs` (1.5 s). Requests that are not misses are never delayed.
2. At `graceAllowance + blockAfter` misses (20 by default) the source is
   blocked for `blockSecs` (15 minutes).
3. A block that starts within `repeatWindowSecs` (24 h) of the previous one
   ending lasts twice as long as the previous one, up to `maxBlockSecs` (24 h).
   After a quiet `repeatWindowSecs`, the next block is a first block again.
4. Every block expires.

**The blocked answer.** While blocked, every request from the source, on any
path and with any method, gets the same answer: `429` (or `403`, per
`blockStatus`) with `Retry-After`, `Cache-Control: no-store` and
`{"code":"sourceUnavailable","error":"Request unavailable."}`, the same body
a deployment IP ban gives. The console path is answered the same way as every
other path, so a block says nothing about where the console is or whether it
exists. A blocked source cannot use the mail app either; that is deliberate,
as a block is only reached after 20 misses in a few minutes. Users who share
one public address (a company NAT) share its score: put such networks on the
allowlist if one scanner there must not lock the others out.

**Audit and logs.**

* Each block writes the security event `admin-probe-blocked` with the source,
  masked by default (`auditIp = masked`: IPv4 /24, IPv6 /48; `full` records the
  address), an opaque source id, the signal, the repeat count and the length.
* Unblocking writes `admin-probe-unblocked` and a policy change writes
  `admin-probe-policy-changed`, both with the admin and the before/after.
* With `fail2banLog` on (default) each block also writes one line for fail2ban
  with the full address; see [fail2ban](../deploy/fail2ban.md#admin-probe-blocks).

**API (for the Security centre).** All under the console path; reads need
`bans.read`, writes `bans.write` for all domains and a step-up (`banManage`).

| Route | |
|---|---|
| `GET {console}/security/admin-probe` | `policy` (every key), `stats` (`blocksTotal`, `blocksLast24h`, `activeBlocks`, `refusedRequests`, `tarpittedRequests`, `clearedBySignin`, `signals` by kind, `trackedSources`, `capacity`, `evictedSources`, `since`), `topOffenders` (10 rows: `id`, `source`, `score`, `signals`, `lastSignal`, `strikes`, `blocked`, `blockedUntil`, `retryAfterSecs`), `scope: "instance"`. Sends an `ETag`. |
| `PUT {console}/security/admin-probe/policy` | `{"values":{"graceAllowance":20,…}}`, keys without the `security.adminProbe.` prefix, `null` restores a default. Validated like the registry (422), `If-Match` (412), 409 for a key an environment variable pins. Answers like the `GET`. |
| `GET {console}/security/admin-probe/blocks` | `{"blocks":[…]}`, the active blocks. |
| `DELETE {console}/security/admin-probe/blocks/{id}` | Lifts one block (204, 404 when not blocked). The repeat count is kept. |

The generic settings route refuses these keys with `useDedicatedRoute`.

**Locked out?** An admin whose own address is blocked can wait for
`Retry-After`, sign in from another network and unblock it, or set
`MW_ADMIN_PROBE_ENABLED=false` and restart (the pin turns the protection off
and active blocks are not applied while it is off).

**Limits.**

* State is per process. The table holds 10 000 sources; when it is full, idle
  sources go first, then the least recently seen unblocked one. Active blocks
  are also saved to the database and read back at start (up to 1 000).
* With several instances behind a load balancer, each counts only the requests
  it receives and enforces only its own blocks, so a scanner spread across
  instances gets up to that many times the allowance before every instance has
  blocked it. A block made on one instance reaches the others only when they
  restart. Use the fail2ban jail (or a block at the load balancer) for a ban
  that covers every instance at once. Policy changes reach other instances
  within 30 seconds.
* The source address is only as good as the proxy configuration: without
  `MW_TRUSTED_PROXIES` behind a reverse proxy, every request comes from the
  proxy, which is never counted, so the protection does nothing.

## Session domain separation (why it matters)

The admin session is a different cookie on a different path than the mailbox session.
Compromising (or simply holding) a mailbox session grants **no** admin capability, and
the admin panel can be bound/scoped independently. Keep `MW_COOKIE_SECURE=true` behind
TLS so the admin cookie is HTTPS-only, and keep `MW_ADMIN_PASSWORD` out of any file
that isn't `0600`.
