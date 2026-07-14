# Admin panel (V6)

The admin panel is an operator surface for domains, users, quotas, security policy,
integrations, and observability. It is **separate from the mailbox**: a distinct route
(`/admin`), a distinct session domain (its own `mw_admin_session` cookie), and a
distinct credential. A logged-in mailbox user is **not** an admin; an admin session is
obtained only by authenticating at the admin surface.

## Enabling & disabling

| Env | Default | Meaning |
|---|---|---|
| `MW_ADMIN_ENABLED` | `true` | When `false`/`0`, the `/admin/*` routes return `401` — the panel is unreachable and unmounted from the operator's perspective. |
| `MW_ADMIN_USER` | *(unset)* | Admin operator username. Unset → admin login always fails. |
| `MW_ADMIN_PASSWORD` | *(unset)* | Admin operator password. Compared in constant time. |

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
- **Security policy** — `GET/PUT /admin/security-policy`: min-TLS, 2FA, Argon2
  parameters, DLP rules, the max-security floor, capture policy.
- **Integrations** — `GET /admin/integrations`, `GET /admin/webhooks`,
  `GET /admin/api-keys` + `POST /admin/api-keys/{id}/revoke`: webhook and
  API-key/MCP-key oversight. LDAP / Nextcloud entries are shown **inert** ("coming
  soon") — the config surface exists but there is no live directory glue in V6 (that
  is V7).
- **Observability** — `GET/PUT /admin/observability`, `GET /admin/audit`: log level,
  OTLP DSN, the audit-log viewer + export, the login monitor + ban list.
- **Appearance** — branding/theme configuration.

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
