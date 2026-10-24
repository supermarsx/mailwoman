# Admin panel (V6)

The admin panel is an operator surface for domains, users, quotas, security policy,
integrations, and observability. It is **separate from the mailbox**: a distinct route
(`/admin`), a distinct session domain (its own cookie, `mw_admin_session` unless
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

An invalid value in any of these stops the server at start with an error naming the
variable. Since 26.20 the `MW_ADMIN_*` variables that nothing reads —
`MW_ADMIN_PORT`, `MW_ADMIN_LOG_LEVEL`, `MW_ADMIN_OTLP_DSN`, `MW_ADMIN_METRICS`,
`MW_ADMIN_THEME`, `MW_ADMIN_BRAND`, `MW_ADMIN_ACCENT` — also stop the start, with
what to use instead. Before 26.20 they logged a warning and the server started.

Set `admin.enabled = false` (or `MW_ADMIN_ENABLED=0`) for a GitOps-only deployment
where all administration is done via the CLI + config and no HTTP panel is exposed.

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
admin surface without a bespoke parser.

## Session domain separation (why it matters)

The admin session is a different cookie on a different path than the mailbox session.
Compromising (or simply holding) a mailbox session grants **no** admin capability, and
the admin panel can be bound/scoped independently. Keep `MW_COOKIE_SECURE=true` behind
TLS so the admin cookie is HTTPS-only, and keep `MW_ADMIN_PASSWORD` out of any file
that isn't `0600`.
