# Admin console security

The admin console is where operators manage domains, users, security policy,
integrations and monitoring. This page describes how it is separated from the
mailbox, how administrators sign in, how their sessions and permissions work, what
is recorded, and what the hidden console path and probing protection do and do not
achieve.

<!-- screenshot: id=security/admin-signin; screen=Admin console sign-in page; state=empty form; element=Admin username, Password, Sign in -->

## Separation from the mailbox

- **Own accounts.** Administrator accounts are stored apart from mailbox accounts.
  Being signed in to a mailbox gives no administrator rights; an administrator
  session is created only by signing in at the console.
- **Own session cookie.** The console uses its own cookie (`mw_admin_session`
  unless `MW_ADMIN_SESSION_COOKIE` names another; `mw_session` and `mw_csrf` are
  refused). Its `Path` is the console path, so the browser does not send it to the
  mail app.
- **Own factors.** Administrator passkeys, authenticator secrets and recovery
  codes are separate from mailbox ones; a mailbox passkey cannot authorise an
  administrator session.
- **Same origin.** The console is served from the same origin as the mail app. A
  script running in the mail app (for example through a cross-site scripting bug)
  that learns the console path can send requests to the console API from a browser
  that holds an administrator session. The separation above protects against a
  stolen mailbox session, not against code running in the page.

## Turning the console on and the environment credential

| Variable | Default | Meaning |
|---|---|---|
| `MW_ADMIN_ENABLED` | `true` | `false` or `0` makes every console sign-in and console API request answer `401`. Administration then happens only through the `mailwoman admin` CLI. |
| `MW_ADMIN_USER`, `MW_ADMIN_PASSWORD` | unset | The environment (break-glass) credential. |
| `MW_ADMIN_ENV_LOGIN` | `on` | `off` stops the environment credential from signing in. |
| `MW_ADMIN_SESSION_COOKIE` | `mw_admin_session` | Cookie name: 1 to 64 characters from `A-Z a-z 0-9 _ - .`. |
| `MW_ADMIN_PATH` | stored path | Pins the console path (see [Console path](#console-path)). |
| `MW_ADMIN_MIN_TLS`, `MW_ADMIN_REQUIRE_2FA` | unset | Pin two fields of the [security policy](#security-policy). |

An invalid value stops the server at start with an error naming the variable. So
do `MW_ADMIN_PORT`, `MW_ADMIN_LOG_LEVEL`, `MW_ADMIN_OTLP_DSN`, `MW_ADMIN_METRICS`,
`MW_ADMIN_THEME`, `MW_ADMIN_BRAND` and `MW_ADMIN_ACCENT`, which the server does
not support; the error says what to use instead.

How the environment credential behaves:

- On a server with no administrator accounts, the first successful sign-in with
  `MW_ADMIN_USER` / `MW_ADMIN_PASSWORD` creates that account in the database with
  an Argon2id password hash and the Super admin role.
- Once a database account with that username has its own password, only that
  password signs in. Changing `MW_ADMIN_PASSWORD` afterwards does not change it.
- The comparison with the environment values does not stop early on a
  mismatching character, but a length difference is detected immediately.
- Keep the variables out of world-readable files; with `MW_ADMIN_ENV_LOGIN=off`
  the environment password is not accepted at all.

## Sign-in

1. **Source checks.** A source on the deployment ban list, or under a temporary
   sign-in lock, is refused before the password is checked (see
   [Sign-in locks](/admin/signin-locks) and [IP bans](/admin/ip-bans)).
2. **Pacing.** Each attempt from a client network (IPv4 address, IPv6 /64) is
   charged against [`signin.retryCooldownSecs`](/admin/settings#signin.retrycooldownsecs);
   only an attempt that ends in a session gets its charge back.
3. **Challenge.** If the sign-in challenge is enabled for the console, a valid
   ALTCHA proof of work (computed in the browser, verified by the server) is
   required.
4. **Password.** Database accounts are checked against their Argon2id hash. A
   disabled account is refused like a wrong password.
5. **Second factor.** If the account has an authenticator app or a passkey, the
   server returns a one-time pending token (32 random bytes, valid five minutes,
   five wrong codes) and no cookie. The session starts only after a valid code,
   passkey or recovery code, and only if the account is still enabled and its
   password hash unchanged.

Each failed sign-in writes a `login-failed` audit entry and, when the source
address is known, one log line in a fail2ban-readable format:

```
<RFC 3339 time> mailwoman[auth]: authentication failure; logname=<user> rhost=<ip>
```

See [fail2ban](/deploy/fail2ban) for a jail that reads it.

Administrator passwords set in the console need at least 12 characters (at most
1024 bytes).

## Sessions, cookies and CSRF

- **Token.** A random session token is set in the cookie; the database stores only
  its SHA-256 hash.
- **Cookie attributes.** `HttpOnly; SameSite=Strict; Path=<base path><console
  path>; Max-Age=<absolute limit>`. `Secure` is added when `MW_COOKIE_SECURE=true`
  or `MW_PUBLIC_URL` starts with `https://`. Behind TLS, set one of them.
- **Lifetime.** [`admin.session.idleSecs`](/admin/settings#admin.session.idlesecs)
  (default 1800 s, clamped to 300 to 7200) and
  [`admin.session.absoluteSecs`](/admin/settings#admin.session.absolutesecs)
  (default 43200 s, clamped to 3600 to 86400). Every console request moves the
  idle deadline forward, never past the absolute deadline fixed at sign-in. Both
  deadlines are stored in the database, so a restart does not extend a session.
  The console's session bar reads them with `GET {console}/session/status`, which
  does not move the idle deadline.
- **Ending sessions.** Changing your own password signs out all of your console
  sessions. **Admin accounts** lists sessions and can revoke them.
- **Cross-site requests.** Every state-changing request must pass an Origin check:
  a request whose `Sec-Fetch-Site` is `cross-site`, whose `Origin`/`Referer` names
  another host, or that repeats `Host`, `Origin`, `Referer` or `Sec-Fetch-Site`
  is refused with `403`. When `MW_CSRF_STRICT=true` (the default of the
  install script; the server's own default is `false`), a cookie-authenticated
  write must also send `X-CSRF-Token` equal to the `mw_csrf` cookie, and that
  token must be the HMAC-SHA256 of the console session id under the server key. A
  mailbox session's token does not authorise a console request, or the other way
  round.

<!-- screenshot: id=security/admin-session-bar; screen=Admin console header; state=signed in, session about to end; element=session countdown bar -->

## Roles and capabilities

Every console API route names one capability, and a request is refused with
`403` unless the signed-in administrator holds it. Capabilities come from roles;
the built-in roles are **Super admin**, **Domain admin**, **Helpdesk**, **Auditor**
and **Security officer**, and administrators can define their own. A role is
granted for all domains or for a list of domains; capabilities that act on objects
outside any domain (security policy, plugins, logs, roles and similar) are carried
only by a grant for all domains.

An administrator can manage another administrator account only if they could
grant every capability that account holds. Anyone who may manage an account can
reset its password and replace its authenticator, so `admins.manage` is effectively
the power to take over the accounts it covers. The server refuses to remove or
disable the last enabled Super admin.

<!-- screenshot: id=security/admin-accounts-roles; screen=Admin console > Admin accounts > account detail; state=account with Domain admin role limited to one domain; element=roles and domain fields -->

## Confirmation of sensitive actions

Some actions need a fresh confirmation that the person at the keyboard is the
account holder, not only someone holding the session: changing roles, removing
or adding a factor, creating an API key, changing branding CSS, importing a
configuration, managing integrations or bans, moving the console path, exporting
or erasing user data, and others. Without a live confirmation the server answers
`403` with `step_up_required` and the methods this session can use.

- The administrator confirms with password plus an enrolled factor, or with a
  user-verified passkey alone. A recovery code can stand in for a lost factor.
- A confirmation lasts five minutes, is bound to the session that made it, and
  covers any sensitive action in that time. It is cleared when the account's
  factors change.
- Five wrong attempts in 15 minutes lock confirmation for that session for the
  rest of the window (`429`).
- Confirmations live in the memory of the server process: a restart drops them,
  and several processes need session affinity.

## Disabling a mailbox account

Setting `disabled` on a user (`PUT {console}/users/{id}/flags`) refuses that
account in Mailwoman: password and header-authenticated sign-in, SSO, second-factor
completion, existing sessions, native bearer sessions, API keys and OAuth tokens.
It does not disable the mailbox on the mail server; a client that talks to the mail
server directly is not affected.

The flag is stored under the address in lower case. Because a mail server can
accept other spellings of the same address, a sign-in is also refused when it
uses:

- the address with a `+tag` in the local part (`alice+x@example.com` for
  `alice@example.com`);
- the other form of an internationalised domain (`alice@xn--bcher-kva.example` for
  `alice@bücher.example`, either way round);
- the bare local part (`alice`), because a mail server with a default realm
  accepts it. A bare name is refused while **any** `alice@<domain>` is disabled.

The bare-name rule fails closed: on a server with several domains, a bare `alice`
that the mail server maps to `alice@example.org` is refused while
`alice@example.com` is disabled. That user can still sign in with the full
address. Tag delimiters other than `+`, and aliases only the mail server knows,
are not recognised until someone signs in with them.

`forcePasswordChange` holds a sign-in under the same spellings. A password change
clears it under the names that session was opened with, their `+tag`-less form and
the other form of the domain, but not under an address reached only from a bare
name. Such a user clears the hold by signing in with the full address and changing
the password there, or an administrator clears the flag.

## Security policy

**Security policy** in the console (`GET`/`PUT {console}/security-policy`, CLI
`mailwoman admin security-policy show|set`) holds deployment-wide fields.
`PUT` takes all of them; it answers `422` with the reason for an out-of-range
value or DLP rules that do not parse, and `409` naming the variable for a field an
environment variable pins.

<!-- screenshot: id=security/admin-security-policy; screen=Admin console > Security policy; state=defaults; element=minimum TLS, Argon2 cost fields, rendering floor, capture policy -->

| Field | Values (default) | What it does |
|---|---|---|
| `minTls` | `1.2` / `1.3` (`1.2`) | Lowest TLS version for the HTTPS listener and for outbound IMAP, SMTP, POP3, ManageSieve and egress-proxy tunnel connections opened afterwards. Not applied to LDAP, Redis, Postgres or HTTP fetches. Pinned by `MW_ADMIN_MIN_TLS`. |
| `argon2MCost`, `argon2TCost`, `argon2PCost` | memory 19456 to 1048576 KiB, passes 1 to 10 (one pass needs at least 47104 KiB), lanes 1 to 16 (19456 / 2 / 1) | Argon2id cost of new hashes of locally stored passwords and recovery codes. An existing password hash is re-hashed at the next successful sign-in; recovery codes keep the cost they were made with. API-key secrets are not affected. |
| `dlpRulesJson` | JSON array (`[]`) | Outbound DLP rules for mail sent through engine-mode accounts, applied beside `MW_DLP_RULES`; an environment rule wins on the same `id`. See [Outbound DLP](./dlp.md). |
| `maxSecurityFloor` | `null` / `full-sanitized` / `sanitized-no-media` / `plain-text` (`null`) | The least restrictive rendering mode the Mailwoman reader offers. Served at `GET /api/policy`; other mail clients do not see it. See [Message opening mode](./max-security.md). |
| `capturePolicy` | `off` / `protect` (`off`) | With `protect`, the desktop and Android apps ask the operating system to keep their window out of screenshots and recordings. A browser cannot be asked. See [Screen capture](./screen-capture.md). |
| `require2fa` | `true` / `false` | The global rule of the **Require two-factor** screen (the same stored row). Pinned by `MW_ADMIN_REQUIRE_2FA`. |

The server applies the policy at start and after each `PUT`, and re-reads the
stored policy every 30 seconds, so a change made through the CLI or another server
process takes effect within that time.

## Audit log

Console writes, sign-in successes and failures, confirmations, API-key and OAuth
checks and other security events are written to the `audit_log` table with time,
actor and actor kind, action, target, a detail object (console writes also record
the administrator's roles, the capability used, and redacted before and after
values) and, where the writer knows it, the source address. It is shown under
**Observability** → **Audit log** and can be exported.

<!-- screenshot: id=security/admin-audit-log; screen=Admin console > Observability > Audit log; state=a few entries; element=audit table with actor, action, target, address -->

What protects the log, precisely:

- **No edit path in the application.** The console and CLI can append and read
  entries; they have no route that edits or deletes a single entry.
- **Retention.** Entries are deleted when an administrator sets an audit retention
  period (30 to 3650 days per category). With no retention set, nothing is
  deleted. A legal hold stops deletion. See [Retention](/admin/retention).
- **Erasure.** When a person's data is erased, their identifiers in audit entries
  are replaced with keyed pseudonyms (HMAC-SHA256) rather than the entries being
  removed.
- **Hash chain.** Entries are linked by an unkeyed SHA-256 hash chain, and pruning
  writes a checkpoint naming where the retained log starts. The chain shows
  accidental damage and edits by someone who does not recompute it. It does not
  stop a person with write access to the database from rewriting entries and
  recomputing the chain; for that, export the log to a system they cannot write
  to.
- An entry is written after the action it records (API-key and OAuth checks in
  the background). If the write fails, the failure is logged and the action is not
  undone.

## Console path

The console page, its sign-in and every console API route are served at one path,
relative to `MW_BASE_PATH`. With `/ops-7f3k` as the path, the server answers
`POST /ops-7f3k/login`, `GET /ops-7f3k/users` and so on.

<!-- screenshot: id=security/admin-console-address; screen=Admin console > Security centre > Console address; state=generated path; element=current path and change controls -->

- **Where it comes from:** `MW_ADMIN_PATH` when set (it pins the path); otherwise
  the path stored in the database; on the first start without either, a generated
  path of 20 random characters (about 100 bits) on a new installation, or `/admin`
  on an installation that already has data, with a warning at every start.
  `mailwoman admin console-path show` prints it.
- **Changing it:** **Security centre** → **Console address** (capability
  `security_policy.write` and a fresh confirmation) or
  `mailwoman admin console-path set|generate`. A chosen path needs at least 12
  characters after the `/` and may not be `/admin`. The change applies at the next
  start, which ends every console session. The audit entries record a fingerprint
  of the old and new path, not the path.
- **Accepted values:** `/` followed by one to four segments of
  `A-Z a-z 0-9 . _ ~ -`, with no `.` or `..` segment and no trailing `/`. The first
  segment may not be one the server already uses (`api`, `jmap`, `oauth`,
  `.well-known`, `dav`, `caldav`, `carddav`, `mcp`, `metrics`, `healthz`, `errors`,
  `t`, `assets`, `fonts`, `icons`, `themes`, the files at the root of the web build)
  or the first segment of `MW_BASE_PATH`.
- **Other paths:** `/admin` (once moved) and every other path get exactly the
  answer an unknown path gets. There is no alias.
- **Not published:** the path is not in the web build, the service worker, the
  mail app, `/api/policy`, discovery documents or headers outside the console, and
  the application log store shows it as `[admin-path]`. The console page is sent
  with `Content-Security-Policy: frame-ancestors 'none'`, `X-Frame-Options: DENY`,
  `X-Robots-Tag: noindex, nofollow` and `Cache-Control: private, no-cache`.

What it is worth: a path nobody can guess keeps scanners and drive-by sign-in
attempts off the console. It does not isolate the console (see
[Separation](#separation-from-the-mailbox)), it does not replace the password,
second factor, sign-in limits or a network restriction, and it is visible to anyone
who can read your proxy logs or an administrator's browser history.

<a id="admin-probe-protection"></a>
## Probing protection

Probing protection slows and then blocks a source that keeps looking for the
console. It is on by default
([`security.adminProbe.enabled`](/admin/settings#security.adminprobe.enabled)) and
leaves room for honest mistakes. It is managed under **Security centre** →
**Probing protection**; see [Security centre](/admin/security-center) for every
control.

<!-- screenshot: id=security/admin-probe-card; screen=Admin console > Security centre > Probing protection; state=one blocked source; element=policy summary, blocked sources with Unblock -->

**What counts as a miss** (one point each, per IPv4 address or IPv6 /64):

| Signal | When |
|---|---|
| `probe-path` | The path contains an entry of `security.adminProbe.probePaths` (for example `/wp-admin`, `/.env`, and `/admin` once the console has moved), outside the console. |
| `slug-guess` | A path with as many segments as the console path answered 404. File names browsers and crawlers request on their own (`favicon.ico`, `robots.txt`, images) and the server's own prefixes do not count. |
| `admin-signin-failed` | A console sign-in was refused (`401`, or `429` from the sign-in limit). |
| `admin-unauthenticated` | A console API call answered `401`; a request carrying a session cookie counts once per cookie, so an expired console tab costs one point. |

**Defaults and escalation:**

- The score drains at `graceAllowance` points per `windowSecs` (10 per 600 s), so
  the first 10 misses in a window do nothing.
- Above that, each further miss is answered after `tarpitDelayMs` (1500 ms).
  Requests that are not misses are never delayed.
- At `graceAllowance + blockAfter` misses (10 + 10) the source is blocked for
  `blockSecs` (900 s). A block starting within `repeatWindowSecs` (86400 s) of the
  previous one doubles its length, up to `maxBlockSecs` (86400 s). Every block
  expires.
- A successful console sign-in clears its source's score.
- Never counted: `security.adminProbe.allowlist` networks, a configured trusted
  proxy itself, and requests without a peer address.

**While blocked**, every request from the source, on any path, gets `429` (or
`403`, per `blockStatus`) with `Retry-After`, `Cache-Control: no-store` and
`{"code":"sourceUnavailable","error":"Request unavailable."}`, the same answer an
IP ban gives. The console path is answered the same way, so a block reveals
nothing about where the console is. A blocked source also cannot use the mail app;
users behind one shared address (an office NAT) share a score, so add such
networks to **Never-blocked networks**.

**Records:** each block writes the security event `admin-probe-blocked` with the
source masked by default ([`security.adminProbe.auditIp`](/admin/settings#security.adminprobe.auditip)
`masked`: IPv4 /24, IPv6 /48); with
[`security.adminProbe.fail2banLog`](/admin/settings#security.adminprobe.fail2banlog)
on (default) it also writes one fail2ban line with the full address. Unblocking
and policy changes are audited with the administrator and the before and after
values.

**Locked out?** Wait for `Retry-After`, sign in from another network and unblock
your address, or set `MW_ADMIN_PROBE_ENABLED=false` and restart.

**Limits:**

- State is per process: a table of 10 000 sources (idle sources are evicted
  first), and active blocks saved to the database and read back at start (up to
  1 000).
- With several instances behind a load balancer, each counts only the requests it
  receives and enforces only its own blocks; a block reaches other instances only
  when they restart. Use the fail2ban jail or a block at the load balancer for a
  ban that covers every instance.
- Without `MW_TRUSTED_PROXIES` behind a reverse proxy, every request appears to
  come from the proxy, which is never counted, so the protection does nothing.

Separately from this, request protection blocks a source on its first request to a
recognised scanner path (`/.env`, `/.git/config`, `/wp-admin` and similar) while
[`security.protection.probePathMode`](/admin/settings#security.protection.probepathmode)
is `enforce` (the default). See [Request limits](/admin/request-protection).

## Related

- [Administrator authenticator app](./admin-totp.md)
- [Passkeys](./passkeys.md)
- [Security centre](/admin/security-center)
- [Sessions](/admin/sessions)
- [fail2ban](/deploy/fail2ban)
