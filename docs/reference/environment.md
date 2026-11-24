# Environment variables

Mailwoman reads its start-up configuration from environment variables, from the options of [`mailwoman serve`](cli.md#serve), and from [settings](../admin/settings.md) stored in the database. This page groups the variables by what you are trying to do and links each one to its entry in the [server configuration reference](../admin/configuration.md), which lists every variable with its default, its precedence and where it can also be set. The console's **Server configuration** page shows the same list with each value and its source.

Precedence, where a variable and a stored setting meet: a **pin** wins and the console refuses to change the stored value; a **fallback** is used only when nothing is stored; **boot** variables are read once at start. Secret values are never shown, only whether they are set.

## Network and listener

Where the server listens, what it advertises, and how it treats the connection in front of it.

| Variable | Purpose |
|---|---|
| [`MW_BIND`](../admin/configuration.md#mw-bind) | listen address |
| [`MW_BASE_PATH`](../admin/configuration.md#mw-base-path) | URL prefix of every route, the console included |
| [`MW_PUBLIC_URL`](../admin/configuration.md#mw-public-url) | canonical external origin (cookies, WebAuthn, OAuth) |
| [`MW_TRUSTED_PROXIES`](../admin/configuration.md#mw-trusted-proxies) | peers that may set the client address and scheme |
| [`MW_FORWARDED_MODE`](../admin/configuration.md#mw-forwarded-mode) | which forwarding headers are read from trusted peers |
| [`MW_PROXY_PROTOCOL`](../admin/configuration.md#mw-proxy-protocol) | accept the PROXY protocol from trusted peers |
| [`MW_PROXY_HEADER_TIMEOUT_MS`](../admin/configuration.md#mw-proxy-header-timeout-ms) | complete PROXY header negotiation deadline in milliseconds (1000..30000; restart required) |
| [`MW_PROXY_MAX_HANDSHAKES`](../admin/configuration.md#mw-proxy-max-handshakes) | maximum simultaneous PROXY header negotiations (1..1024; restart required) |
| [`MW_NATIVE_ORIGINS`](../admin/configuration.md#mw-native-origins) | CORS origins of the desktop and mobile shells |
| [`MW_HEALTH_TOKEN`](../admin/configuration.md#mw-health-token) | token that lets an external monitor reach /healthz from any address (header X-Health-Token or query token) |

## TLS and certificates

Built-in certificates, your own certificate files, and transport headers.

| Variable | Purpose |
|---|---|
| [`MW_ACME`](../admin/configuration.md#mw-acme) | built-in ACME certificates |
| [`MW_ACME_CONTACT`](../admin/configuration.md#mw-acme-contact) | ACME account contact |
| [`MW_ACME_CACHE`](../admin/configuration.md#mw-acme-cache) | ACME account and certificate cache directory |
| [`MW_ACME_STAGING`](../admin/configuration.md#mw-acme-staging) | use the ACME staging directory |
| [`MW_TLS_CERT`](../admin/configuration.md#mw-tls-cert) | certificate file (reloaded on SIGHUP) |
| [`MW_TLS_KEY`](../admin/configuration.md#mw-tls-key) | private key file (reloaded on SIGHUP) |
| [`MW_TLS_HANDSHAKE_TIMEOUT_MS`](../admin/configuration.md#mw-tls-handshake-timeout-ms) | complete inbound TLS handshake deadline in milliseconds (1000..60000; restart required) |
| [`MW_TLS_MAX_HANDSHAKES`](../admin/configuration.md#mw-tls-max-handshakes) | maximum simultaneous inbound TLS handshakes (1..1024; restart required) |
| [`MW_HSTS`](../admin/configuration.md#mw-hsts) | send Strict-Transport-Security |
| [`MW_HSTS_MAX_AGE`](../admin/configuration.md#mw-hsts-max-age) | HSTS max-age in seconds |
| [`MW_HSTS_INCLUDE_SUBDOMAINS`](../admin/configuration.md#mw-hsts-include-subdomains) | HSTS includeSubDomains |
| [`MW_HSTS_PRELOAD`](../admin/configuration.md#mw-hsts-preload) | HSTS preload |
| [`MW_DB_REQUIRE_TLS`](../admin/configuration.md#mw-db-require-tls) | 1, true, yes or on: refuse a PostgreSQL URL that does not force TLS (sslmode=require, verify-ca or verify-full) |

## Database, keys and storage

The database, the key that seals stored secrets, encryption at rest, and the directories the server writes to.

| Variable | Purpose |
|---|---|
| [`MW_DB_PATH`](../admin/configuration.md#mw-db-path) | SQLite path or PostgreSQL URL |
| [`MW_SERVER_KEY`](../admin/configuration.md#mw-server-key) | 32-byte key that seals every stored secret |
| [`MW_SERVER_KEY_FILE`](../admin/configuration.md#mw-server-key-file) | persistent sealing key file; PostgreSQL requires this or MW_SERVER_KEY |
| [`MW_DB_ENCRYPTION`](../admin/configuration.md#mw-db-encryption) | database encryption at rest: off, optional (encrypt when a key is given) or required (refuse to start without one) |
| [`MW_DB_KEY`](../admin/configuration.md#mw-db-key) | database encryption key, inline; MW_DB_KEY_FILE and MW_DB_KEY_COMMAND are the alternatives |
| [`MW_DB_KEY_FILE`](../admin/configuration.md#mw-db-key-file) | read the database encryption key from this file |
| [`MW_DB_KEY_COMMAND`](../admin/configuration.md#mw-db-key-command) | run this command and use its output as the database encryption key |
| [`MW_REDIS_URL`](../admin/configuration.md#mw-redis-url) | shared cache |
| [`MW_UPLOAD_DIR`](../admin/configuration.md#mw-upload-dir) | sealed upload directory |
| [`MW_SEARCH_DIR`](../admin/configuration.md#mw-search-dir) | search index location (memory or a path) |
| [`MW_WEB_DIR`](../admin/configuration.md#mw-web-dir) | serve the web client from this directory |
| [`MW_RETAINED_ASSETS_DIR`](../admin/configuration.md#mw-retained-assets-dir) | keep the hashed web files of the last 5 builds (14 days after replacement) for tabs still running one; off disables |
| [`MW_WKD_DIR`](../admin/configuration.md#mw-wkd-dir) | WKD files directory |
| [`MW_ABUSE_SPOOL`](../admin/configuration.md#mw-abuse-spool) | abuse report (ARF) spool directory |
| [`MW_GEOIP_DB`](../admin/configuration.md#mw-geoip-db) | local country database file; when set it overrides the console-managed one |
| [`MW_GEOIP_DIR`](../admin/configuration.md#mw-geoip-dir) | directory for the console-managed country database |
| [`MW_GEOIP_SEED`](../admin/configuration.md#mw-geoip-seed) | country database installed when the data directory has none |
| [`MW_PLUGIN_DIR`](../admin/configuration.md#mw-plugin-dir) | plugin directory |
| [`MW_THIRDPARTY_PLUGIN_DIR`](../admin/configuration.md#mw-thirdparty-plugin-dir) | third-party plugin directory |

## Mail upstream

How the server reaches your mail servers.

| Variable | Purpose |
|---|---|
| [`MW_MODE`](../admin/configuration.md#mw-mode) | proxy or engine mode, for the whole process |
| [`MW_JMAP_UPSTREAMS`](../admin/configuration.md#mw-jmap-upstreams) | allowed JMAP proxy origins (seeds the default upstream) |
| [`MW_SMTP_HOST`](../admin/configuration.md#mw-smtp-host) | engine submission host when a domain has no SMTP upstream |
| [`MW_SMTP_PORT`](../admin/configuration.md#mw-smtp-port) | engine submission port |
| [`MW_SMTP_SECURITY`](../admin/configuration.md#mw-smtp-security) | engine submission security |
| [`MW_SMTP_POLICY_DOH_URL`](../admin/configuration.md#mw-smtp-policy-doh-url) | DNS wire resolver for enabled SMTP transport policies; DNSSEC validates locally through the dns_auth route |
| [`MW_ENGINE_TLS`](../admin/configuration.md#mw-engine-tls) | forces the transport (implicit, starttls or plaintext) of engine connections for a domain that has no mail configuration of its own; for fronting a plaintext test server |
| [`MW_AUTOCONFIG_ALLOW_PRIVATE`](../admin/configuration.md#mw-autoconfig-allow-private) | allow private addresses in autoconfig lookups |
| [`MW_ALLOWED_FROMS`](../admin/configuration.md#mw-allowed-froms) | server-advertised sender identities |

## Sign-in and identity

Mailbox sessions, header authentication, WebAuthn, password change and directory binds.

| Variable | Purpose |
|---|---|
| [`MW_SESSION_IDLE_SECS`](../admin/configuration.md#mw-session-idle-secs) | mailbox session idle limit (upper bound for the stored policy) |
| [`MW_SESSION_ABSOLUTE_SECS`](../admin/configuration.md#mw-session-absolute-secs) | optional mailbox absolute limit; 0 leaves the stored policy in control |
| [`MW_HEADER_AUTH`](../admin/configuration.md#mw-header-auth) | trust an identity header from listed addresses |
| [`MW_HEADER_AUTH_HEADER`](../admin/configuration.md#mw-header-auth-header) | the identity header name |
| [`MW_HEADER_AUTH_PASSWORD`](../admin/configuration.md#mw-header-auth-password) | service password for header-authenticated mail access |
| [`MW_HEADER_AUTH_JMAP_URL`](../admin/configuration.md#mw-header-auth-jmap-url) | JMAP upstream for header-authenticated sessions |
| [`MW_HEADER_AUTH_TRUSTED_IPS`](../admin/configuration.md#mw-header-auth-trusted-ips) | addresses allowed to send the identity header |
| [`MW_WEBAUTHN_ORIGIN`](../admin/configuration.md#mw-webauthn-origin) | WebAuthn relying-party origin |
| [`MW_WEBAUTHN_RP_ID`](../admin/configuration.md#mw-webauthn-rp-id) | WebAuthn relying-party id |
| [`MW_PASSWD_BACKEND`](../admin/configuration.md#mw-passwd-backend) | password-change backend; when set the environment integration is in force |
| [`MW_PASSWD_LDAP_URL`](../admin/configuration.md#mw-passwd-ldap-url) | LDAP password-modify server |
| [`MW_PASSWD_LDAP_STARTTLS`](../admin/configuration.md#mw-passwd-ldap-starttls) | LDAP password-modify STARTTLS |
| [`MW_PASSWD_LDAP_BIND_DN`](../admin/configuration.md#mw-passwd-ldap-bind-dn) | LDAP password-modify bind DN |
| [`MW_PASSWD_LDAP_BIND_PW`](../admin/configuration.md#mw-passwd-ldap-bind-pw) | LDAP password-modify bind password |
| [`MW_PASSWD_DOVECOT_URL`](../admin/configuration.md#mw-passwd-dovecot-url) | doveadm HTTP API |
| [`MW_PASSWD_DOVECOT_API_KEY`](../admin/configuration.md#mw-passwd-dovecot-api-key) | doveadm HTTP API key |
| [`MW_PASSWD_DOVECOT_COMMAND`](../admin/configuration.md#mw-passwd-dovecot-command) | doveadm API command name sent in the request |
| [`MW_PASSWD_POPPASSD_HOST`](../admin/configuration.md#mw-passwd-poppassd-host) | poppassd host |
| [`MW_PASSWD_POPPASSD_PORT`](../admin/configuration.md#mw-passwd-poppassd-port) | poppassd port |
| [`MW_PASSWD_WEBHOOK_URL`](../admin/configuration.md#mw-passwd-webhook-url) | password-change webhook |
| [`MW_PASSWD_WEBHOOK_SECRET`](../admin/configuration.md#mw-passwd-webhook-secret) | password-change webhook signing secret |
| [`MW_DIRECTORY_BIND_DN`](../admin/configuration.md#mw-directory-bind-dn) | directory (GAL/LDAP) service bind DN |
| [`MW_DIRECTORY_BIND_PW`](../admin/configuration.md#mw-directory-bind-pw) | directory (GAL/LDAP) service bind password |
| [`MW_MCP_RESOURCE`](../admin/configuration.md#mw-mcp-resource) | MCP OAuth resource identifier; changing it invalidates issued tokens |
| [`MW_BRIDGE_OAUTH_CLIENT_SECRET`](../admin/configuration.md#mw-bridge-oauth-client-secret) | bridge OAuth client secret |
| [`MW_DCR_INITIAL_ACCESS_TOKEN`](../admin/configuration.md#mw-dcr-initial-access-token) | OAuth dynamic client registration initial access token |

## Admin console

The console path, the break-glass administrator and the console session.

| Variable | Purpose |
|---|---|
| [`MW_ADMIN_ENABLED`](../admin/configuration.md#mw-admin-enabled) | admin console on or off |
| [`MW_ADMIN_PATH`](../admin/configuration.md#mw-admin-path) | console and admin API path; unset, the stored path (generated on a new install, /admin kept on an upgrade) |
| [`MW_ADMIN_USER`](../admin/configuration.md#mw-admin-user) | environment admin user name (break-glass) |
| [`MW_ADMIN_PASSWORD`](../admin/configuration.md#mw-admin-password) | environment admin password (break-glass) |
| [`MW_ADMIN_ENV_LOGIN`](../admin/configuration.md#mw-admin-env-login) | whether the environment admin may sign in (off turns it off) |
| [`MW_ADMIN_SESSION_COOKIE`](../admin/configuration.md#mw-admin-session-cookie) | admin session cookie name |
| [`MW_ADMIN_MIN_TLS`](../admin/configuration.md#mw-admin-min-tls) | pins the stored admin TLS floor |
| [`MW_ADMIN_REQUIRE_2FA`](../admin/configuration.md#mw-admin-require-2fa) | pins the admin two-factor requirement |

## Security hardening

Browser isolation, request forgery protection, the render sandbox and signed extensions.

| Variable | Purpose |
|---|---|
| [`MW_COEP`](../admin/configuration.md#mw-coep) | Cross-Origin-Embedder-Policy require-corp |
| [`MW_CSRF_STRICT`](../admin/configuration.md#mw-csrf-strict) | double-submit CSRF enforcement |
| [`MW_COOKIE_SECURE`](../admin/configuration.md#mw-cookie-secure) | the Secure attribute on cookies |
| [`MW_RENDER_BIN`](../admin/configuration.md#mw-render-bin) | render sandbox binary |
| [`MW_RENDER_JAIL`](../admin/configuration.md#mw-render-jail) | require the render sandbox jail |
| [`MW_DLP_RULES`](../admin/configuration.md#mw-dlp-rules) | outbound DLP rules from the environment (applied beside the stored rules) |
| [`MW_WATERMARK`](../admin/configuration.md#mw-watermark) | watermark switch returned by `GET /api/security/watermark`; the web app does not draw a watermark |
| [`MW_WATERMARK_OPACITY`](../admin/configuration.md#mw-watermark-opacity) | watermark opacity returned by `GET /api/security/watermark` |
| [`MW_UI_PLUGIN_TRUST_KEYS`](../admin/configuration.md#mw-ui-plugin-trust-keys) | public keys UI plugin signatures are checked against |
| [`MW_WEBHOOK_INBOUND_SECRET`](../admin/configuration.md#mw-webhook-inbound-secret) | inbound webhook HMAC secret |
| [`MW_COMPLIANCE_SMTP_PASSWORD`](../admin/configuration.md#mw-compliance-smtp-password) | Password for the compliance SMTP relay; never exposed in settings |
| [`MW_COMPLIANCE_SMTP_PASSWORD_FILE`](../admin/configuration.md#mw-compliance-smtp-password-file) | File containing the compliance SMTP password |
| [`MW_COMPLIANCE_JMAP_PASSWORD`](../admin/configuration.md#mw-compliance-jmap-password) | Password for JMAP compliance submission; never exposed in settings |
| [`MW_COMPLIANCE_JMAP_PASSWORD_FILE`](../admin/configuration.md#mw-compliance-jmap-password-file) | File containing the JMAP compliance submission password |

## Limits

Connection, request and handshake ceilings. Each is read when the server starts.

| Variable | Purpose |
|---|---|
| [`MW_HTTP_MAX_CONNECTIONS`](../admin/configuration.md#server.http.maxconnections) | concurrent HTTP connections including upgraded WebSockets (16..65536; unset = Auto, derived from the open-file limit and memory, at least 1024; pins `server.http.maxConnections`) |
| [`MW_HTTP_MAX_REQUESTS`](../admin/configuration.md#server.http.maxrequests) | concurrent HTTP requests through response-body completion (1..16384; unset = Auto, from CPUs, the database pool and memory, at least 256; pins `server.http.maxRequests`) |
| [`MW_HTTP_MAX_HEADER_BYTES`](../admin/configuration.md#mw-http-max-header-bytes) | HTTP request header budget in bytes (8192..65536; restart required) |
| [`MW_HTTP_MAX_HEADERS`](../admin/configuration.md#mw-http-max-headers) | HTTP request header count (16..256; restart required) |
| [`MW_HTTP_HEADER_TIMEOUT_MS`](../admin/configuration.md#mw-http-header-timeout-ms) | HTTP opening and HTTP1 header deadline in milliseconds (1000..60000; restart required) |
| [`MW_HTTP_BODY_TIMEOUT_MS`](../admin/configuration.md#mw-http-body-timeout-ms) | request-body ingress deadline in milliseconds (1000..300000; response streams are not timed out) |
| [`MW_HTTP2_MAX_STREAMS`](../admin/configuration.md#server.http.http2maxstreams) | maximum HTTP2 streams per connection (1..128; unset = Auto, an eighth of the request limit, 32..100; pins `server.http.http2MaxStreams`) |
| [`MW_HTTP_SHUTDOWN_TIMEOUT_MS`](../admin/configuration.md#mw-http-shutdown-timeout-ms) | graceful HTTP drain deadline in milliseconds (1000..120000; restart required) |
| [`MW_HTTP_REQUEST_QUEUE_MS`](../admin/configuration.md#server.http.requestqueuems) | wait for a free request slot before 503, in milliseconds (0..2000, default 200; 0 = 503 at once; pins `server.http.requestQueueMs`) |
| [`MW_HTTP_REQUEST_QUEUE_MAX`](../admin/configuration.md#server.http.requestqueuemax) | requests that may wait for a slot at once (1..65536; unset = the request limit; pins `server.http.requestQueueMax`) |
| [`MW_HTTP_ADAPTIVE_CONCURRENCY`](../admin/configuration.md#mw-http-adaptive-concurrency) | `off` (default) or `on`: lower the request limit while requests slow down; wins over `server.http.adaptiveConcurrency` |
| [`MW_HTTP_RAISE_OPEN_FILES`](../admin/configuration.md#mw-http-raise-open-files) | `on` (default) or `off`: raise the soft open-file limit at start when the connection limit is Auto; wins over `server.http.raiseOpenFileLimit` |

## Features and integrations

Optional integrations that are configured from the environment.

| Variable | Purpose |
|---|---|
| [`MW_ASSIST_LOCAL_PROGRAMS`](../admin/configuration.md#mw-assist-local-programs) | programs a local AI adapter may run on the host |
| [`MW_NEXTCLOUD_URL`](../admin/configuration.md#mw-nextcloud-url) | deprecated shared Nextcloud URL; ignored by account-owned file operations |
| [`MW_NEXTCLOUD_USER`](../admin/configuration.md#mw-nextcloud-user) | deprecated shared Nextcloud user; ignored, each mailbox must verify its own link |
| [`MW_NEXTCLOUD_APP_PASSWORD`](../admin/configuration.md#mw-nextcloud-app-password) | deprecated shared Nextcloud password; ignored, never grants mailbox access |
| [`MW_EGRESS_PROBE_URL`](../admin/configuration.md#mw-egress-probe-url) | the URL an egress route test fetches (default `https://example.com/`); environment only |

## Logs, traces and metrics

What the server logs and where telemetry goes.

| Variable | Purpose |
|---|---|
| [`MW_LOG`](../admin/configuration.md#mw-log) | log filter at start (the live level is set in the console) |
| [`RUST_LOG`](../admin/configuration.md#rust-log) | log filter; wins over MW_LOG |
| [`MW_OTLP_ENDPOINT`](../admin/configuration.md#mw-otlp-endpoint) | OTLP trace collector |
| [`MW_ERROR_FORWARD_URL`](../admin/configuration.md#mw-error-forward-url) | where scrubbed browser error reports are also sent; environment only |
| [`MW_OTEL_SERVICE`](../admin/configuration.md#mw-otel-service) | OpenTelemetry service name |
| [`MW_METRICS_TOKEN`](../admin/configuration.md#mw-metrics-token) | bearer token for /metrics |
| [`MW_SENTRY_DSN`](../admin/configuration.md#mw-sentry-dsn) | Sentry error export |

## Command-line inputs

Variables that only the command-line tools read. See the [command-line reference](cli.md).

| Variable | Purpose |
|---|---|
| [`MW_ACCOUNT_ID`](../admin/configuration.md#mw-account-id) | account whose password the CLI changes |
| [`MW_OLD_PASSWORD`](../admin/configuration.md#mw-old-password) | current password for the CLI change |
| [`MW_NEW_PASSWORD`](../admin/configuration.md#mw-new-password) | new password for the CLI change |
| [`MW_MIGRATE_FROM`](../admin/configuration.md#mw-migrate-from) | source store of an offline copy |
| [`MW_MIGRATE_TO`](../admin/configuration.md#mw-migrate-to) | target store of an offline copy |
| [`MW_MCP_SERVER`](../admin/configuration.md#mw-mcp-server) | server the MCP client proxy talks to |
| [`MW_MCP_TOKEN`](../admin/configuration.md#mw-mcp-token) | token of the MCP client proxy |

## Defaults and switches that are easy to misread

How the server reads these variables. Where this differs from the default shown in the generated reference, this page describes what the program does.

| Variable | Behaviour |
|---|---|
| [`MW_ADMIN_ENABLED`](../admin/configuration.md#mw-admin-enabled) | The console is on unless the variable is `0` or `false` (any case). The default is on whether or not `MW_ADMIN_USER` and `MW_ADMIN_PASSWORD` are set. |
| [`MW_ACME`](../admin/configuration.md#mw-acme) | A comma-separated list of domain names, not a switch. Unset means no ACME. Do not set it to `true` or `false`: either would be read as a domain name. |
| [`MW_COOKIE_SECURE`](../admin/configuration.md#mw-cookie-secure) | No fixed default. Without it, cookies are `Secure` on requests the server treats as HTTPS (from the scheme of `MW_PUBLIC_URL` or from trusted proxy headers). `true` forces `Secure` on and `false` forces it off. `mailwoman serve` accepts only the literal values `true` and `false` for this variable; any other value stops it with an invalid-value error. |
| [`MW_CSRF_STRICT`](../admin/configuration.md#mw-csrf-strict) | Off unless set. The `deploy.sh` script sets it to `true`. Only `true` and `false` are accepted. |
| [`MW_COEP`](../admin/configuration.md#mw-coep) | On. The command-line flag cannot turn it off; set the variable to `false`. Only `true` and `false` are accepted. |

## Variables tied to a setting

These variables are tied to a registry setting. A **pin** wins over the stored value and the console refuses to change the setting while the variable is set; a **fallback** is used when nothing is stored. The last column says whether the setting is read when saved: for the keys marked **no**, the variable is the only control that has an effect.

| Variable | Setting | Precedence | Saved value read |
|---|---|---|---|
| `MW_UPSTREAM_DEFAULT` | [`upstream.default`](../admin/settings.md#upstream.default) | fallback | yes |
| `MW_ADMIN_PROBE_ENABLED` | [`security.adminProbe.enabled`](../admin/settings.md#security.adminprobe.enabled) | pin | yes |
| `MW_ADMIN_GEO_ENABLED` | [`security.trust.adminGeoEnabled`](../admin/settings.md#security.trust.admingeoenabled) | pin | yes |
| `MW_LDAP_BIND_AUTH` | [`signin.ldapBindPrecheck`](../admin/settings.md#signin.ldapbindprecheck) | pin | no |
| `MW_ASSIST_RATE_LIMIT_PER_MIN` | [`assistant.rateLimitPerMin`](../admin/settings.md#assistant.ratelimitpermin) | pin | no |
| `MW_PLUGIN_KV_MAX_KEY_BYTES` | [`plugins.kv.maxKeyBytes`](../admin/settings.md#plugins.kv.maxkeybytes) | pin | no |
| `MW_PLUGIN_KV_MAX_VALUE_BYTES` | [`plugins.kv.maxValueBytes`](../admin/settings.md#plugins.kv.maxvaluebytes) | pin | no |
| `MW_PLUGIN_KV_MAX_TOTAL_BYTES` | [`plugins.kv.maxTotalBytes`](../admin/settings.md#plugins.kv.maxtotalbytes) | pin | no |
| `MW_PLUGIN_KV_MAX_KEYS` | [`plugins.kv.maxKeys`](../admin/settings.md#plugins.kv.maxkeys) | pin | no |
| `MW_REALTIME_RECHECK_SECS` | [`realtime.recheckSecs`](../admin/settings.md#realtime.rechecksecs) | pin | no |
| `MW_MASKED_EMAIL_DOMAIN` | [`maskedEmail.domain`](../admin/settings.md#maskedemail.domain) | pin | no |
| `MW_ABUSE_ADDRESS` | [`abuse.reportAddress`](../admin/settings.md#abuse.reportaddress) | pin | no |
| `MW_DLP_ADMIN` | [`dlp.adminAddress`](../admin/settings.md#dlp.adminaddress) | pin | no |
| `MW_VAPID_CONTACT` | [`push.vapidContact`](../admin/settings.md#push.vapidcontact) | pin | no |
| `MW_DB_POOL` | [`database.pool`](../admin/settings.md#database.pool) | pin | yes |

Where the last column says **no**, see the effect column of the setting in the [key reference](../admin/settings.md#key-reference).

## Other inputs in the generated reference

The configuration reference also lists inputs that are not start-up configuration:

- [Built-in constants](../admin/configuration.md#built-in-constants): values fixed in the program, with the interval or size they set.
- [Stored settings keys, tables, CLI actions and files](../admin/configuration.md#stored-settings-keys-tables-cli-actions-and-files): where the console keeps what it changes.
- [Settings registry keys](../admin/configuration.md#settings-registry-keys): every setting with its scopes and default; explained in [Settings](../admin/settings.md).
- [Build inputs](../admin/configuration.md#build-inputs) and [Deployment tooling](../admin/configuration.md#deployment-tooling): variables read when building or deploying, not by the running server.
- Variables that start with `MW_ADMIN_` and name a console option (`MW_ADMIN_PORT`, `MW_ADMIN_LOG_LEVEL`, `MW_ADMIN_OTLP_DSN`, `MW_ADMIN_METRICS`, `MW_ADMIN_THEME`, `MW_ADMIN_BRAND`, `MW_ADMIN_ACCENT`) are refused at start; the reference gives the reason for each.
