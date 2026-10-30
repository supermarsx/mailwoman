# Server configuration reference

<!-- Generated from crates/mw-server/src/knobs.rs and settings_registry.rs.
     Do not edit by hand: run `BLESS=1 cargo test -p mw-server --test t32_cfg_knob_coverage`. -->

Catalogue of declared server, deployment and build inputs and their configuration locations.
The console's **Server configuration** page shows the same list with each value and
its source. Values marked unavailable are not introspectable from this running process;
configuredInput is an environment input, not an assertion about effective runtime state.
Registry values describe resolved policy, not proof that every subsystem consumes that key.
Secret values are never shown, only whether they are set. URLs are
shown without user name, password and query.

Precedence: **pin** = the variable wins and a console write is refused
(`409 pinnedByEnv`); **fallback** = the stored value wins, the variable is used when
nothing is stored; **boot** = read once at start, a change needs a restart.

## Listener, TLS and HTTP headers

| Name | Class | Value | Default | Precedence | What | Set in / reason |
|---|---|---|---|---|---|---|
| <a id="mw-bind"></a>`MW_BIND` | readOnly | plain | 0.0.0.0:8080 | boot | listen address | needed before the database opens or to reach the console |
| <a id="mw-base-path"></a>`MW_BASE_PATH` | readOnly | plain |  | boot | URL prefix of every route, the console included | needed before the database opens or to reach the console |
| <a id="mw-public-url"></a>`MW_PUBLIC_URL` | readOnly | url | the request's Host header | boot | canonical external origin (cookies, WebAuthn, OAuth) | needed before the database opens or to reach the console |
| <a id="mw-cookie-secure"></a>`MW_COOKIE_SECURE` | readOnly | plain | derived from the scheme | boot | the Secure attribute on cookies | needed before the database opens or to reach the console |
| <a id="mw-mode"></a>`MW_MODE` | readOnly | plain | proxy | boot | proxy or engine mode, for the whole process | needed before the database opens or to reach the console |
| <a id="mw-acme"></a>`MW_ACME` | readOnly | plain | false | boot | built-in ACME certificates | needed before the database opens or to reach the console |
| <a id="mw-acme-contact"></a>`MW_ACME_CONTACT` | readOnly | plain |  | boot | ACME account contact | needed before the database opens or to reach the console |
| <a id="mw-acme-cache"></a>`MW_ACME_CACHE` | readOnly | path | acme-cache | boot | ACME account and certificate cache directory | needed before the database opens or to reach the console |
| <a id="mw-acme-staging"></a>`MW_ACME_STAGING` | readOnly | plain | false | boot | use the ACME staging directory | needed before the database opens or to reach the console |
| <a id="mw-tls-cert"></a>`MW_TLS_CERT` | readOnly | path |  | boot | certificate file (reloaded on SIGHUP) | needed before the database opens or to reach the console |
| <a id="mw-tls-key"></a>`MW_TLS_KEY` | readOnly | secret |  | boot | private key file (reloaded on SIGHUP) | needed before the database opens or to reach the console |
| <a id="mw-http-max-connections"></a>`MW_HTTP_MAX_CONNECTIONS` | readOnly | plain | 1024 | boot | concurrent HTTP connections including upgraded WebSockets (16..8192; restart required) | needed before the database opens or to reach the console |
| <a id="mw-http-max-requests"></a>`MW_HTTP_MAX_REQUESTS` | readOnly | plain | 256 | boot | concurrent HTTP requests through response-body completion (1..4096; restart required) | needed before the database opens or to reach the console |
| <a id="mw-http-max-header-bytes"></a>`MW_HTTP_MAX_HEADER_BYTES` | readOnly | plain | 32768 | boot | HTTP request header budget in bytes (8192..65536; restart required) | needed before the database opens or to reach the console |
| <a id="mw-http-max-headers"></a>`MW_HTTP_MAX_HEADERS` | readOnly | plain | 100 | boot | HTTP request header count (16..256; restart required) | needed before the database opens or to reach the console |
| <a id="mw-http-header-timeout-ms"></a>`MW_HTTP_HEADER_TIMEOUT_MS` | readOnly | plain | 10000 | boot | HTTP opening and HTTP1 header deadline in milliseconds (1000..60000; restart required) | needed before the database opens or to reach the console |
| <a id="mw-http-body-timeout-ms"></a>`MW_HTTP_BODY_TIMEOUT_MS` | readOnly | plain | 60000 | boot | request-body ingress deadline in milliseconds (1000..300000; response streams are not timed out) | needed before the database opens or to reach the console |
| <a id="mw-http2-max-streams"></a>`MW_HTTP2_MAX_STREAMS` | readOnly | plain | 32 | boot | maximum HTTP2 streams per connection (1..128; restart required) | needed before the database opens or to reach the console |
| <a id="mw-http-shutdown-timeout-ms"></a>`MW_HTTP_SHUTDOWN_TIMEOUT_MS` | readOnly | plain | 30000 | boot | graceful HTTP drain deadline in milliseconds (1000..120000; restart required) | needed before the database opens or to reach the console |
| <a id="mw-tls-handshake-timeout-ms"></a>`MW_TLS_HANDSHAKE_TIMEOUT_MS` | readOnly | plain | 10000 | boot | complete inbound TLS handshake deadline in milliseconds (1000..60000; restart required) | needed before the database opens or to reach the console |
| <a id="mw-tls-max-handshakes"></a>`MW_TLS_MAX_HANDSHAKES` | readOnly | plain | 128 | boot | maximum simultaneous inbound TLS handshakes (1..1024; restart required) | needed before the database opens or to reach the console |
| <a id="mw-proxy-header-timeout-ms"></a>`MW_PROXY_HEADER_TIMEOUT_MS` | readOnly | plain | 5000 | boot | complete PROXY header negotiation deadline in milliseconds (1000..30000; restart required) | needed before the database opens or to reach the console |
| <a id="mw-proxy-max-handshakes"></a>`MW_PROXY_MAX_HANDSHAKES` | readOnly | plain | 128 | boot | maximum simultaneous PROXY header negotiations (1..1024; restart required) | needed before the database opens or to reach the console |
| <a id="mw-proxy-protocol"></a>`MW_PROXY_PROTOCOL` | readOnly | plain | off | boot | accept the PROXY protocol from trusted peers | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-trusted-proxies"></a>`MW_TRUSTED_PROXIES` | readOnly | plain |  | boot | peers that may set the client address and scheme | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-forwarded-mode"></a>`MW_FORWARDED_MODE` | readOnly | plain | off | boot | which forwarding headers are read from trusted peers | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-hsts"></a>`MW_HSTS` | readOnly | plain | false | boot | send Strict-Transport-Security | a wrong value leaves state in clients that cannot be undone |
| <a id="mw-hsts-max-age"></a>`MW_HSTS_MAX_AGE` | readOnly | plain |  | boot | HSTS max-age in seconds | a wrong value leaves state in clients that cannot be undone |
| <a id="mw-hsts-include-subdomains"></a>`MW_HSTS_INCLUDE_SUBDOMAINS` | readOnly | plain | false | boot | HSTS includeSubDomains | a wrong value leaves state in clients that cannot be undone |
| <a id="mw-hsts-preload"></a>`MW_HSTS_PRELOAD` | readOnly | plain | false | boot | HSTS preload | a wrong value leaves state in clients that cannot be undone |
| <a id="mw-coep"></a>`MW_COEP` | readOnly | plain | true | boot | Cross-Origin-Embedder-Policy require-corp | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-csrf-strict"></a>`MW_CSRF_STRICT` | readOnly | plain | false (deploy.sh: true) | boot | double-submit CSRF enforcement | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-native-origins"></a>`MW_NATIVE_ORIGINS` | readOnly | url |  | boot | CORS origins of the desktop and mobile shells | a root of trust, or a protection a stolen admin session could switch off |

## Storage and host paths

| Name | Class | Value | Default | Precedence | What | Set in / reason |
|---|---|---|---|---|---|---|
| <a id="mw-server-key"></a>`MW_SERVER_KEY` | readOnly | secret | generated at first start | boot | 32-byte key that seals every stored secret | needed before the database opens or to reach the console |
| <a id="mw-db-path"></a>`MW_DB_PATH` | readOnly | dsn | mailwoman.db | boot | SQLite path or PostgreSQL URL | needed before the database opens or to reach the console |
| <a id="mw-web-dir"></a>`MW_WEB_DIR` | readOnly | path | embedded | boot | serve the web client from this directory | needed before the database opens or to reach the console |
| <a id="mw-retained-assets-dir"></a>`MW_RETAINED_ASSETS_DIR` | readOnly | path | retained-assets beside the SQLite database | boot | keep the hashed web files of the last 5 builds (14 days after replacement) for tabs still running one; off disables | needed before the database opens or to reach the console |
| <a id="mw-redis-url"></a>`MW_REDIS_URL` | readOnly | url |  | boot | shared cache | needed before the database opens or to reach the console |
| <a id="mw-upload-dir"></a>`MW_UPLOAD_DIR` | readOnly | path | the database | boot | sealed upload directory | needed before the database opens or to reach the console |
| <a id="mw-render-bin"></a>`MW_RENDER_BIN` | readOnly | path |  | boot | render sandbox binary | needed before the database opens or to reach the console |
| <a id="mw-render-jail"></a>`MW_RENDER_JAIL` | readOnly | plain |  | boot | require the render sandbox jail | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-plugin-dir"></a>`MW_PLUGIN_DIR` | readOnly | path |  | boot | plugin directory | needed before the database opens or to reach the console |
| <a id="mw-thirdparty-plugin-dir"></a>`MW_THIRDPARTY_PLUGIN_DIR` | readOnly | path |  | boot | third-party plugin directory | needed before the database opens or to reach the console |
| <a id="mw-search-dir"></a>`MW_SEARCH_DIR` | readOnly | path | next to the database | boot | search index location (memory or a path) | needed before the database opens or to reach the console |
| <a id="mw-wkd-dir"></a>`MW_WKD_DIR` | readOnly | path |  | boot | WKD files directory | needed before the database opens or to reach the console |
| <a id="mw-abuse-spool"></a>`MW_ABUSE_SPOOL` | readOnly | path |  | boot | abuse report (ARF) spool directory | needed before the database opens or to reach the console |
| <a id="mw-geoip-db"></a>`MW_GEOIP_DB` | readOnly | path |  | boot | local country database file; when set it overrides the console-managed one | needed before the database opens or to reach the console |
| <a id="mw-geoip-dir"></a>`MW_GEOIP_DIR` | readOnly | path | geoip/ next to a SQLite database | boot | directory for the console-managed country database | needed before the database opens or to reach the console |
| <a id="mw-geoip-seed"></a>`MW_GEOIP_SEED` | readOnly | path |  | boot | country database installed when the data directory has none | needed before the database opens or to reach the console |

## Admin console bootstrap

| Name | Class | Value | Default | Precedence | What | Set in / reason |
|---|---|---|---|---|---|---|
| <a id="mw-admin-enabled"></a>`MW_ADMIN_ENABLED` | readOnly | plain | true when user and password are set | boot | admin console on or off | needed before the database opens or to reach the console |
| <a id="mw-admin-user"></a>`MW_ADMIN_USER` | readOnly | plain |  | boot | environment admin user name (break-glass) | needed before the database opens or to reach the console |
| <a id="mw-admin-password"></a>`MW_ADMIN_PASSWORD` | readOnly | secret |  | boot | environment admin password (break-glass) | needed before the database opens or to reach the console |
| <a id="mw-admin-path"></a>`MW_ADMIN_PATH` | readOnly | plain | /admin | boot | console and admin API path; unset, the stored path (generated on a new install, /admin kept on an upgrade) | needed before the database opens or to reach the console |
| <a id="mw-admin-session-cookie"></a>`MW_ADMIN_SESSION_COOKIE` | readOnly | plain | mw_admin_session | boot | admin session cookie name | needed before the database opens or to reach the console |
| <a id="mw-admin-env-login"></a>`MW_ADMIN_ENV_LOGIN` | readOnly | plain | on | boot | whether the environment admin may sign in (off turns it off) | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-admin-min-tls"></a>`MW_ADMIN_MIN_TLS` | ui | plain |  | pin | pins the stored admin TLS floor | Admin → Security policy |
| <a id="mw-admin-require-2fa"></a>`MW_ADMIN_REQUIRE_2FA` | ui | plain |  | pin | pins the admin two-factor requirement | Admin → Security policy |
| <a id="mw-admin-port"></a>`MW_ADMIN_PORT` | refused | plain |  | boot | refused at start (see the reason) | the admin panel is served on the main listener; there is no separate admin port |
| <a id="mw-admin-log-level"></a>`MW_ADMIN_LOG_LEVEL` | refused | plain |  | boot | refused at start (see the reason) | the log filter is set with MW_LOG at start |
| <a id="mw-admin-otlp-dsn"></a>`MW_ADMIN_OTLP_DSN` | refused | plain |  | boot | refused at start (see the reason) | the OTLP collector is set with MW_OTLP_ENDPOINT at start |
| <a id="mw-admin-metrics"></a>`MW_ADMIN_METRICS` | refused | plain |  | boot | refused at start (see the reason) | /metrics is served when MW_METRICS_TOKEN is set at start |
| <a id="mw-admin-theme"></a>`MW_ADMIN_THEME` | refused | plain |  | boot | refused at start (see the reason) | no deployment-default theme is read from the environment |
| <a id="mw-admin-brand"></a>`MW_ADMIN_BRAND` | refused | plain |  | boot | refused at start (see the reason) | no brand name is read from the environment |
| <a id="mw-admin-accent"></a>`MW_ADMIN_ACCENT` | refused | plain |  | boot | refused at start (see the reason) | no accent colour is read from the environment |

## Sign-in, identity providers and sessions

| Name | Class | Value | Default | Precedence | What | Set in / reason |
|---|---|---|---|---|---|---|
| <a id="mw-header-auth"></a>`MW_HEADER_AUTH` | readOnly | plain | false | boot | trust an identity header from listed addresses | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-header-auth-header"></a>`MW_HEADER_AUTH_HEADER` | readOnly | plain |  | boot | the identity header name | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-header-auth-password"></a>`MW_HEADER_AUTH_PASSWORD` | readOnly | secret |  | boot | service password for header-authenticated mail access | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-header-auth-jmap-url"></a>`MW_HEADER_AUTH_JMAP_URL` | readOnly | url |  | boot | JMAP upstream for header-authenticated sessions | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-header-auth-trusted-ips"></a>`MW_HEADER_AUTH_TRUSTED_IPS` | readOnly | plain |  | boot | addresses allowed to send the identity header | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-directory-bind-dn"></a>`MW_DIRECTORY_BIND_DN` | ui | plain |  | pin | directory (GAL/LDAP) service bind DN | Admin → Integrations → Directory |
| <a id="mw-directory-bind-pw"></a>`MW_DIRECTORY_BIND_PW` | ui | secret |  | pin | directory (GAL/LDAP) service bind password | Admin → Integrations → Directory |
| <a id="mw-webauthn-origin"></a>`MW_WEBAUTHN_ORIGIN` | readOnly | url | derived from the public URL | boot | WebAuthn relying-party origin | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-webauthn-rp-id"></a>`MW_WEBAUTHN_RP_ID` | readOnly | plain | derived from the public URL | boot | WebAuthn relying-party id | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-session-idle-secs"></a>`MW_SESSION_IDLE_SECS` | ui | plain | 1800 | boot | mailbox session idle limit (upper bound for the stored policy) | Admin → Sessions |
| <a id="mw-session-absolute-secs"></a>`MW_SESSION_ABSOLUTE_SECS` | ui | plain | 1296000 | boot | mailbox session absolute limit (upper bound for the stored policy) | Admin → Sessions |

## Integrations

| Name | Class | Value | Default | Precedence | What | Set in / reason |
|---|---|---|---|---|---|---|
| <a id="mw-passwd-backend"></a>`MW_PASSWD_BACKEND` | ui | plain |  | pin | password-change backend; when set the environment integration is in force | Admin → Integrations → Password change |
| <a id="mw-passwd-ldap-url"></a>`MW_PASSWD_LDAP_URL` | ui | url |  | pin | LDAP password-modify server | Admin → Integrations → Password change |
| <a id="mw-passwd-ldap-starttls"></a>`MW_PASSWD_LDAP_STARTTLS` | ui | plain |  | pin | LDAP password-modify STARTTLS | Admin → Integrations → Password change |
| <a id="mw-passwd-ldap-bind-dn"></a>`MW_PASSWD_LDAP_BIND_DN` | ui | plain |  | pin | LDAP password-modify bind DN | Admin → Integrations → Password change |
| <a id="mw-passwd-ldap-bind-pw"></a>`MW_PASSWD_LDAP_BIND_PW` | ui | secret |  | pin | LDAP password-modify bind password | Admin → Integrations → Password change |
| <a id="mw-passwd-dovecot-url"></a>`MW_PASSWD_DOVECOT_URL` | ui | url |  | pin | doveadm HTTP API | Admin → Integrations → Password change |
| <a id="mw-passwd-dovecot-api-key"></a>`MW_PASSWD_DOVECOT_API_KEY` | ui | secret |  | pin | doveadm HTTP API key | Admin → Integrations → Password change |
| <a id="mw-passwd-dovecot-command"></a>`MW_PASSWD_DOVECOT_COMMAND` | ui | plain | pw | pin | doveadm API command name sent in the request | Admin → Integrations → Password change |
| <a id="mw-passwd-poppassd-host"></a>`MW_PASSWD_POPPASSD_HOST` | ui | plain |  | pin | poppassd host | Admin → Integrations → Password change |
| <a id="mw-passwd-poppassd-port"></a>`MW_PASSWD_POPPASSD_PORT` | ui | plain | 106 | pin | poppassd port | Admin → Integrations → Password change |
| <a id="mw-passwd-webhook-url"></a>`MW_PASSWD_WEBHOOK_URL` | ui | url |  | pin | password-change webhook | Admin → Integrations → Password change |
| <a id="mw-passwd-webhook-secret"></a>`MW_PASSWD_WEBHOOK_SECRET` | ui | secret |  | pin | password-change webhook signing secret | Admin → Integrations → Password change |
| <a id="mw-nextcloud-url"></a>`MW_NEXTCLOUD_URL` | readOnly | url |  |  | deprecated shared Nextcloud URL; ignored by account-owned file operations | Admin → Nextcloud; User settings → Nextcloud; a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-nextcloud-user"></a>`MW_NEXTCLOUD_USER` | readOnly | plain |  |  | deprecated shared Nextcloud user; ignored, each mailbox must verify its own link | Admin → Nextcloud; User settings → Nextcloud; a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-nextcloud-app-password"></a>`MW_NEXTCLOUD_APP_PASSWORD` | readOnly | secret |  |  | deprecated shared Nextcloud password; ignored, never grants mailbox access | Admin → Nextcloud; User settings → Nextcloud; a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-bridge-oauth-client-secret"></a>`MW_BRIDGE_OAUTH_CLIENT_SECRET` | ui | secret |  | pin | bridge OAuth client secret | Admin → OAuth providers |
| <a id="mw-dcr-initial-access-token"></a>`MW_DCR_INITIAL_ACCESS_TOKEN` | ui | secret |  | pin | OAuth dynamic client registration initial access token | Admin → Integrations → Client registration |
| <a id="mw-webhook-inbound-secret"></a>`MW_WEBHOOK_INBOUND_SECRET` | ui | secret |  | pin | inbound webhook HMAC secret | Admin → Integrations → Inbound webhooks |
| <a id="mw-ui-plugin-trust-keys"></a>`MW_UI_PLUGIN_TRUST_KEYS` | readOnly | plain |  | boot | public keys UI plugin signatures are checked against | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-assist-local-programs"></a>`MW_ASSIST_LOCAL_PROGRAMS` | readOnly | plain |  | boot | programs a local AI adapter may run on the host | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-mcp-resource"></a>`MW_MCP_RESOURCE` | readOnly | url | derived from the public origin | boot | MCP OAuth resource identifier; changing it invalidates issued tokens | a wrong value leaves state in clients that cannot be undone |

## Mail, upstreams and content

| Name | Class | Value | Default | Precedence | What | Set in / reason |
|---|---|---|---|---|---|---|
| <a id="mw-jmap-upstreams"></a>`MW_JMAP_UPSTREAMS` | ui | url |  | fallback | allowed JMAP proxy origins (seeds the default upstream) | Admin → Domains → Upstreams |
| <a id="mw-smtp-host"></a>`MW_SMTP_HOST` | ui | plain | the IMAP host | fallback | engine submission host when a domain has no SMTP upstream | Admin → Domains → Upstreams |
| <a id="mw-smtp-port"></a>`MW_SMTP_PORT` | ui | plain | 587 | fallback | engine submission port | Admin → Domains → Upstreams |
| <a id="mw-smtp-security"></a>`MW_SMTP_SECURITY` | ui | plain | starttls | fallback | engine submission security | Admin → Domains → Upstreams |
| <a id="mw-engine-tls"></a>`MW_ENGINE_TLS` | readOnly | plain |  | boot | forces the engine's TLS mode (test override; the dashboard warns while it is set) | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-autoconfig-allow-private"></a>`MW_AUTOCONFIG_ALLOW_PRIVATE` | readOnly | plain | false | boot | allow private addresses in autoconfig lookups | a root of trust, or a protection a stolen admin session could switch off |
| <a id="mw-allowed-froms"></a>`MW_ALLOWED_FROMS` | ui | plain |  | boot | server-advertised sender identities | Admin → Identities |
| <a id="mw-dlp-rules"></a>`MW_DLP_RULES` | ui | plain |  | boot | outbound DLP rules from the environment (applied beside the stored rules) | Admin → Security policy |

## Logs, traces and metrics

| Name | Class | Value | Default | Precedence | What | Set in / reason |
|---|---|---|---|---|---|---|
| <a id="mw-log"></a>`MW_LOG` | readOnly | plain | info | boot | log filter at start (the live level is set in the console) | needed before the database opens or to reach the console |
| <a id="mw-otlp-endpoint"></a>`MW_OTLP_ENDPOINT` | readOnly | url |  | boot | OTLP trace collector | needed before the database opens or to reach the console |
| <a id="mw-otel-service"></a>`MW_OTEL_SERVICE` | readOnly | plain | mailwoman | boot | OpenTelemetry service name | needed before the database opens or to reach the console |
| <a id="mw-metrics-token"></a>`MW_METRICS_TOKEN` | ui | secret |  | pin | bearer token for /metrics | Admin → Observability |
| <a id="mw-sentry-dsn"></a>`MW_SENTRY_DSN` | readOnly | secret |  | boot | Sentry error export | needed before the database opens or to reach the console |
| <a id="rust-log"></a>`RUST_LOG` | readOnly | plain |  | boot | log filter; wins over MW_LOG | needed before the database opens or to reach the console |

## Built-in constants

| Name | Class | Value | Default | Precedence | What | Set in / reason |
|---|---|---|---|---|---|---|
| <a id="const-refresh-every"></a>`const:REFRESH_EVERY` | readOnly | plain | 30 s |  | how often each process re-reads admin changes | a CLI, build or test input, or a protocol or safety constant |
| <a id="const-boot-register-concurrency"></a>`const:BOOT_REGISTER_CONCURRENCY` | readOnly | plain | 4 |  | engine accounts registered in parallel at start | a CLI, build or test input, or a protocol or safety constant |
| <a id="const-boot-register-timeout"></a>`const:BOOT_REGISTER_TIMEOUT` | readOnly | plain | 600 s |  | time limit for registering engine accounts at start | a CLI, build or test input, or a protocol or safety constant |
| <a id="const-materialize-back-days"></a>`const:MATERIALIZE_BACK_DAYS` | readOnly | plain | 2 years |  | recurring events expanded into the past | a CLI, build or test input, or a protocol or safety constant |
| <a id="const-materialize-fwd-days"></a>`const:MATERIALIZE_FWD_DAYS` | readOnly | plain | 5 years |  | recurring events expanded into the future | a CLI, build or test input, or a protocol or safety constant |
| <a id="const-rerank-top-n"></a>`const:RERANK_TOP_N` | readOnly | plain | built-in |  | semantic search rerank depth | a CLI, build or test input, or a protocol or safety constant |
| <a id="const-rest-default-limit"></a>`const:REST_DEFAULT_LIMIT` | readOnly | plain | 50 |  | REST API page size | a CLI, build or test input, or a protocol or safety constant |
| <a id="const-rest-max-limit"></a>`const:REST_MAX_LIMIT` | readOnly | plain | 200 |  | REST API largest page | a CLI, build or test input, or a protocol or safety constant |
| <a id="const-plugin-max-memory-mb"></a>`const:PLUGIN_MAX_MEMORY_MB` | readOnly | plain | 1024 |  | ceiling for a plugin's memory limit | a CLI, build or test input, or a protocol or safety constant |
| <a id="const-plugin-max-deadline-ms"></a>`const:PLUGIN_MAX_DEADLINE_MS` | readOnly | plain | 120000 |  | ceiling for a plugin's time limit | a CLI, build or test input, or a protocol or safety constant |
| <a id="const-open-tracking-limits"></a>`const:OPEN_TRACKING_LIMITS` | readOnly | plain | built-in |  | open-tracking abuse ceilings | a CLI, build or test input, or a protocol or safety constant |
| <a id="const-push-wake-ttl"></a>`const:PUSH_WAKE_TTL` | readOnly | plain | built-in |  | push wake time to live | a CLI, build or test input, or a protocol or safety constant |
| <a id="const-realtime-heartbeat"></a>`const:REALTIME_HEARTBEAT` | readOnly | plain | built-in |  | event stream heartbeat | a CLI, build or test input, or a protocol or safety constant |
| <a id="const-discover-limits"></a>`const:DISCOVER_LIMITS` | readOnly | plain | built-in |  | discovery route limits (the route answers 410) | a CLI, build or test input, or a protocol or safety constant |
| <a id="const-default-max-query-changes"></a>`const:DEFAULT_MAX_QUERY_CHANGES` | readOnly | plain | 500 |  | JMAP queryChanges limit | a CLI, build or test input, or a protocol or safety constant |
| <a id="const-max-appearance-bytes"></a>`const:MAX_APPEARANCE_BYTES` | readOnly | plain | 8 KiB |  | largest stored appearance record | a CLI, build or test input, or a protocol or safety constant |
| <a id="const-sso-pending-ttl"></a>`const:SSO_PENDING_TTL` | readOnly | plain | 600 s |  | single sign-on round-trip life | a CLI, build or test input, or a protocol or safety constant |

## Stored settings keys, tables, CLI actions and files

| Name | Class | Value | Default | Precedence | What | Set in / reason |
|---|---|---|---|---|---|---|
| <a id="const-max-upload-bytes"></a>`const:MAX_UPLOAD_BYTES` | ui | plain |  |  | largest attachment upload (the limit.* keys set it per user) | Admin → Limits |
| <a id="const-image-proxy-limits"></a>`const:IMAGE_PROXY_LIMITS` | ui | plain |  |  | remote-content cache and rate limits | Admin → Privacy → Remote content |
| <a id="const-totp-issuer"></a>`const:TOTP_ISSUER` | ui | plain |  |  | issuer label in authenticator apps (the product name) | Admin → Branding |
| <a id="key-v6-admin-security-policy"></a>`key:v6:admin:security_policy` | ui | plain |  |  | security policy record | Admin → Security policy |
| <a id="key-v6-admin-observability"></a>`key:v6:admin:observability` | ui | plain |  |  | metrics toggle and live log level | Admin → Observability |
| <a id="key-v6-admin-bans"></a>`key:v6:admin:bans` | ui | plain |  |  | IP bans written by the admin API, the CLI and failed admin sign-ins | Admin → Access |
| <a id="key-v6-admin-flags"></a>`key:v6:admin:flags` | ui | plain |  |  | per-account flags (disabled, features) | Admin → Users |
| <a id="key-appearance"></a>`key:appearance` | ui | plain |  |  | a user's appearance record | Settings → Appearance; Admin → Settings |
| <a id="key-mdn-policy"></a>`key:mdn_policy` | ui | plain |  |  | a user's read-receipt policy | Settings → Mail; Admin → Settings |
| <a id="key-push.quiet-hours"></a>`key:push.quiet_hours` | ui | plain |  |  | old quiet-hours key, read once as the fallback of push.quietHours | Admin → Notifications |
| <a id="table-domains"></a>`table:domains` | ui | plain |  |  | domains, upstreams and allow/block lists | Admin → Domains |
| <a id="table-quotas"></a>`table:quotas` | ui | plain |  |  | per-user quotas | Admin → Users |
| <a id="table-admin-users"></a>`table:admin_users` | ui | plain |  |  | admin accounts | Admin → Admins |
| <a id="table-twofa-policy"></a>`table:twofa_policy` | ui | plain |  |  | two-factor policy | Admin → Two-factor |
| <a id="table-assist-config"></a>`table:assist_config` | ui | plain |  |  | AI assistant configuration | Admin → AI assistant |
| <a id="table-sso-config"></a>`table:sso_config` | ui | plain |  |  | single sign-on providers | Admin → Single sign-on |
| <a id="table-egress-proxy"></a>`table:egress_proxy` | ui | plain |  |  | outbound proxies | Admin → Outbound |
| <a id="table-plugins"></a>`table:plugins` | ui | plain |  |  | server plugins and their limits | Admin → Plugins |
| <a id="table-ui-plugins"></a>`table:ui_plugins` | ui | plain |  |  | UI plugins | Admin → UI plugins |
| <a id="table-oauth-dcr"></a>`table:oauth_dcr` | ui | plain |  |  | dynamically registered OAuth clients | Admin → OAuth clients |
| <a id="table-webhooks"></a>`table:webhooks` | ui | plain |  |  | outbound webhooks | Admin → Integrations |
| <a id="table-api-keys"></a>`table:api_keys` | ui | plain |  |  | API keys | Admin → Integrations |
| <a id="table-cache-scope"></a>`table:cache_scope` | ui | plain |  |  | cache layers and lifetimes per data class | Admin → Cache scope |
| <a id="table-directory-config"></a>`table:directory_config` | ui | plain |  |  | directory (LDAP/GAL) endpoints | Admin → Integrations → Directory |
| <a id="table-push-config"></a>`table:push_config` | ui | plain |  |  | VAPID key pair | Admin → Notifications |
| <a id="table-integration-config"></a>`table:integration_config` | ui | plain |  |  | password change, Nextcloud, client registration and inbound webhook integrations | Admin → Integrations |
| <a id="cli-admin"></a>`cli:admin` | ui | plain |  |  | mailwoman admin … (every console write keeps CLI parity where it exists) | mailwoman admin |
| <a id="cli-plugin"></a>`cli:plugin` | ui | plain |  |  | mailwoman plugin list\|approve | Admin → Plugins |
| <a id="cli-maintenance-rethread"></a>`cli:maintenance rethread` | ui | plain |  |  | re-thread mailboxes | Admin → Maintenance |
| <a id="cli-doctor"></a>`cli:doctor` | ui | plain |  |  | cache and sandbox posture | Admin → Diagnostics |
| <a id="cli-maintenance-gc-uploads"></a>`cli:maintenance gc-uploads` | ui | plain |  |  | remove unreferenced uploads | Admin → Scheduled tasks |
| <a id="cli-maintenance-vacuum"></a>`cli:maintenance vacuum` | ui | plain |  |  | compact the database | Admin → Scheduled tasks |
| <a id="cli-fonts-pull"></a>`cli:fonts pull` | readOnly | plain |  |  | a build step that writes the self-hosted font files into the web build | a CLI, build or test input, or a protocol or safety constant |
| <a id="cli-healthcheck"></a>`cli:healthcheck` | readOnly | plain |  |  | container health probe | a CLI, build or test input, or a protocol or safety constant |
| <a id="cli-migrate-store"></a>`cli:migrate-store` | readOnly | plain |  |  | offline copy between SQLite and PostgreSQL (server stopped) | a CLI, build or test input, or a protocol or safety constant |
| <a id="cli-mcp-stdio"></a>`cli:mcp-stdio` | readOnly | plain |  |  | MCP client proxy | a CLI, build or test input, or a protocol or safety constant |
| <a id="cli-password"></a>`cli:password` | readOnly | plain |  |  | change one account's password from the CLI | a CLI, build or test input, or a protocol or safety constant |
| <a id="file-admin.toml"></a>`file:admin.toml` | readOnly | plain |  |  | AdminConfig TOML; nothing loads it (the configuration export is the file form) | a CLI, build or test input, or a protocol or safety constant |
| <a id="file-server.json"></a>`file:server.json` | readOnly | plain |  |  | desktop provisioning file, written by the OS administrator | a CLI, build or test input, or a protocol or safety constant |

## CLI inputs

| Name | Class | Value | Default | Precedence | What | Set in / reason |
|---|---|---|---|---|---|---|
| <a id="mw-account-id"></a>`MW_ACCOUNT_ID` | cliOnly | plain |  |  | account whose password the CLI changes | mailwoman password; a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-old-password"></a>`MW_OLD_PASSWORD` | cliOnly | secret |  |  | current password for the CLI change | mailwoman password; a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-new-password"></a>`MW_NEW_PASSWORD` | cliOnly | secret |  |  | new password for the CLI change | mailwoman password; a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-migrate-from"></a>`MW_MIGRATE_FROM` | cliOnly | dsn |  |  | source store of an offline copy | mailwoman migrate-store; a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-migrate-to"></a>`MW_MIGRATE_TO` | cliOnly | dsn |  |  | target store of an offline copy | mailwoman migrate-store; a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-mcp-server"></a>`MW_MCP_SERVER` | cliOnly | url |  |  | server the MCP client proxy talks to | mailwoman mcp-stdio; a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-mcp-token"></a>`MW_MCP_TOKEN` | cliOnly | secret |  |  | token of the MCP client proxy | mailwoman mcp-stdio; a CLI, build or test input, or a protocol or safety constant |

## Build inputs

| Name | Class | Value | Default | Precedence | What | Set in / reason |
|---|---|---|---|---|---|---|
| <a id="mw-build-sha"></a>`MW_BUILD_SHA` | build | plain |  |  | optional source revision embedded in the dashboard server metadata | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-shell-server-url"></a>`MW_SHELL_SERVER_URL` | build | url |  |  | server URL baked into a shell build | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-shell-product-name"></a>`MW_SHELL_PRODUCT_NAME` | build | plain |  |  | product name of a shell build | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-shell-identifier"></a>`MW_SHELL_IDENTIFIER` | build | plain |  |  | bundle identifier of a shell build | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-shell-updater-url"></a>`MW_SHELL_UPDATER_URL` | build | url |  |  | desktop update feed of a shell build | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-shell-updater-pubkey"></a>`MW_SHELL_UPDATER_PUBKEY` | build | secret |  |  | desktop update signing public key of a shell build | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-i18n-pseudo"></a>`MW_I18N_PSEUDO` | build | plain |  |  | build the web client with the pseudo-locale | a CLI, build or test input, or a protocol or safety constant |

## Deployment tooling

| Name | Class | Value | Default | Precedence | What | Set in / reason |
|---|---|---|---|---|---|---|
| <a id="mw-image-digest"></a>`MW_IMAGE_DIGEST` | deploy | plain |  |  | image digest deploy.sh writes into the app environment | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-caddyfile"></a>`MW_CADDYFILE` | deploy | plain |  |  | Caddyfile deploy.sh passes to the proxy container | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-acme-ca-root"></a>`MW_ACME_CA_ROOT` | deploy | plain |  |  | private ACME CA root deploy.sh passes to the proxy container | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-proxy-app-image"></a>`MW_PROXY_APP_IMAGE` | deploy | plain |  |  | app image of the proxy-conformance compose file | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-proxy-mock-image"></a>`MW_PROXY_MOCK_IMAGE` | deploy | plain |  |  | mock image of the proxy-conformance compose file | a CLI, build or test input, or a protocol or safety constant |

## Test inputs and markers

| Name | Class | Value | Default | Precedence | What | Set in / reason |
|---|---|---|---|---|---|---|
| <a id="mw-test-pg"></a>`MW_TEST_PG` | testOnly | dsn |  |  | PostgreSQL for the store tests | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-test-redis"></a>`MW_TEST_REDIS` | testOnly | plain |  |  | run the Redis tests | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-test-redis-url"></a>`MW_TEST_REDIS_URL` | testOnly | url |  |  | Redis for the cache tests | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-test-skip-log"></a>`MW_TEST_SKIP_LOG` | testOnly | plain |  |  | quiet test logging | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-require-live"></a>`MW_REQUIRE_LIVE` | testOnly | plain |  |  | fail instead of skipping when a live test service is missing | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-e14-pg-dsn"></a>`MW_E14_PG_DSN` | testOnly | secret |  |  | PostgreSQL for the e14 tests | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-mock-port"></a>`MW_MOCK_PORT` | testOnly | plain |  |  | port of the mock mail server | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-e2e-admin-path"></a>`MW_E2E_ADMIN_PATH` | testOnly | plain |  |  | console path the e2e suites use | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-imap-live-host"></a>`MW_IMAP_LIVE_HOST` | testOnly | plain |  |  | live IMAP test server | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-pop3-live-host"></a>`MW_POP3_LIVE_HOST` | testOnly | plain |  |  | live POP3 test server | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-sso-live"></a>`MW_SSO_LIVE` | testOnly | plain |  |  | run the live single sign-on tests | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-t14-live"></a>`MW_T14_LIVE` | testOnly | plain |  |  | run the t14 live tests | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-t14-tls-live"></a>`MW_T14_TLS_LIVE` | testOnly | plain |  |  | run the t14 TLS live tests | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-t20-proxy"></a>`MW_T20_PROXY` | testOnly | plain |  |  | t20 forwarding test proxy | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-t20-proxy-base"></a>`MW_T20_PROXY_BASE` | testOnly | plain |  |  | t20 forwarding test base path | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-t20-proxy-kind"></a>`MW_T20_PROXY_KIND` | testOnly | plain |  |  | t20 forwarding test proxy kind | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-t22-squid"></a>`MW_T22_SQUID` | testOnly | plain |  |  | t22 egress test proxy | a CLI, build or test input, or a protocol or safety constant |
| <a id="mw-actions"></a>`MW_ACTIONS` | marker | plain |  |  | a marker in the AI assistant's tool prompt, not a variable | a CLI, build or test input, or a protocol or safety constant |

## Settings registry keys

Set under **Settings** in the console or through
the routes under `/settings`. Each key's details are in [settings.md](settings.md).

| Key | Category | Scopes | Default | Value | Environment | Hot | Read by |
|---|---|---|---|---|---|---|---|
| <a id="accounts.linkingenabled"></a>`accounts.linkingEnabled` | accounts | global+domain+admin_user | false | plain |  | yes | Allow verified linked mailbox accounts; global false is a hard ceiling |
| <a id="accounts.uselinked"></a>`accounts.useLinked` | accounts | global+domain+user | true | plain |  | yes | Use linked mailboxes when permitted by the administrator |
| <a id="accounts.maxlinked"></a>`accounts.maxLinked` | accounts | global | 5 | plain |  | yes | Maximum verified linked mailboxes per primary identity |
| <a id="accounts.storelinkedcredentials"></a>`accounts.storeLinkedCredentials` | accounts | global+domain+admin_user | true | plain |  | yes | Keep a sealed copy of each linked mailbox's password so its access renews without asking; off keeps sign-in-only links |
| <a id="privacy.stripmailerheaders"></a>`privacy.stripMailerHeaders` | privacy | global+domain+user | true | plain |  | yes | remove client fingerprint headers from native outgoing email; preserve message bodies and attached messages |
| <a id="privacy.remoteimagedownloads"></a>`privacy.remoteImageDownloads` | privacy | global+domain+user | false | plain |  | yes | allow deliberate message image downloads through the guarded privacy proxy |
| <a id="privacy.remoteimagestoast"></a>`privacy.remoteImagesToast` | privacy | global+domain+user | true | plain |  | yes | show one notification while a message's remote images load through the privacy proxy; presentation only, the image policy is unchanged |
| <a id="appearance.customcssenabled"></a>`appearance.customCssEnabled` | appearance | global | false | plain |  | yes | Allow owner-scoped appearance CSS; disabled by default and paused under enforced mailbox appearance locks |
| <a id="appearance.customjsenabled"></a>`appearance.customJsEnabled` | appearance | global | false | plain |  | yes | Allow owner-written scripts in an opaque appearance-only worker sandbox; disabled by default |
| <a id="mail.folderrole.inbox"></a>`mail.folderRole.inbox` | mail | global+domain+user | null | plain |  | yes | inbox folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder |
| <a id="mail.folderrole.drafts"></a>`mail.folderRole.drafts` | mail | global+domain+user | null | plain |  | yes | drafts folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder |
| <a id="mail.folderrole.sent"></a>`mail.folderRole.sent` | mail | global+domain+user | null | plain |  | yes | sent folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder |
| <a id="mail.folderrole.trash"></a>`mail.folderRole.trash` | mail | global+domain+user | null | plain |  | yes | trash folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder |
| <a id="mail.folderrole.junk"></a>`mail.folderRole.junk` | mail | global+domain+user | null | plain |  | yes | junk folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder |
| <a id="mail.folderrole.archive"></a>`mail.folderRole.archive` | mail | global+domain+user | null | plain |  | yes | archive folder role override; null keeps automatic discovery, an exact path or owned ID selects the folder |
| <a id="nextcloud.syncintervalsecs"></a>`nextcloud.syncIntervalSecs` | integrations | global+domain+user | 300 | plain |  | yes | Nextcloud collection refresh interval; zero is manual only, positive values have a sixty-second minimum |
| <a id="packages.allowed"></a>`packages.allowed` | integrations | global+domain+admin_user | true | plain |  | yes | Allow package tracking; provider activation and separate account opt-in are still required |
| <a id="cap.integration.nextcloud"></a>`cap.integration.nextcloud` | integrations | global+domain+admin_user | true | plain |  | yes | Nextcloud integration |
| <a id="cap.integration.google"></a>`cap.integration.google` | integrations | global+domain+admin_user | true | plain |  | yes | Google OAuth bridge |
| <a id="cap.integration.microsoft"></a>`cap.integration.microsoft` | integrations | global+domain+admin_user | true | plain |  | yes | Microsoft OAuth bridge |
| <a id="cap.integration."></a>`cap.integration.*` | integrations | global+domain+admin_user | true | plain |  | yes | Individual integration permission |
| <a id="cap.ui-plugin."></a>`cap.ui_plugin.*` | integrations | global+domain+admin_user | true | plain |  | yes | Individual UI plugin permission |
| <a id="cap.ai-assistant"></a>`cap.ai_assistant` | features | global+domain+admin_user | true | plain |  | yes | AI assistant |
| <a id="cap.plugins"></a>`cap.plugins` | features | global+domain+admin_user | true | plain |  | yes | Plugins |
| <a id="cap.pim.calendar"></a>`cap.pim.calendar` | features | global+domain+admin_user | true | plain |  | yes | Calendar |
| <a id="cap.pim.contacts"></a>`cap.pim.contacts` | features | global+domain+admin_user | true | plain |  | yes | Contacts |
| <a id="cap.pim.tasks"></a>`cap.pim.tasks` | features | global+domain+admin_user | true | plain |  | yes | Tasks |
| <a id="cap.pim.notes"></a>`cap.pim.notes` | features | global+domain+admin_user | true | plain |  | yes | Notes |
| <a id="cap.export"></a>`cap.export` | features | global+domain+admin_user | true | plain |  | yes | Export |
| <a id="cap.import"></a>`cap.import` | features | global+domain+admin_user | true | plain |  | yes | Import |
| <a id="cap.api-keys"></a>`cap.api_keys` | features | global+domain+admin_user | true | plain |  | yes | API keys |
| <a id="cap.oauth-apps"></a>`cap.oauth_apps` | features | global+domain+admin_user | true | plain |  | yes | OAuth applications |
| <a id="cap.rules"></a>`cap.rules` | features | global+domain+admin_user | true | plain |  | yes | Mail rules |
| <a id="cap.rules.forward"></a>`cap.rules.forward` | features | global+domain+admin_user | true | plain |  | yes | Rule forwarding |
| <a id="cap.remote-images"></a>`cap.remote_images` | features | global+domain+admin_user | true | plain |  | yes | Remote images |
| <a id="cap.user-background"></a>`cap.user_background` | features | global+domain+admin_user | false | plain |  | yes | Custom background |
| <a id="cap.user-css"></a>`cap.user_css` | features | global+domain+admin_user | false | plain |  | yes | Custom CSS |
| <a id="cap.compose.rich-text"></a>`cap.compose.rich_text` | features | global+domain+admin_user | true | plain |  | yes | JMAP Email/set create with an HTML body (feature_gate.rs); composer plain-text only |
| <a id="cap.compose.attachments"></a>`cap.compose.attachments` | features | global+domain+admin_user | true | plain |  | yes | JMAP Email/set create with attachment parts (feature_gate.rs); composer attach button |
| <a id="cap.compose.share-links"></a>`cap.compose.share_links` | features | global+domain+admin_user | true | plain |  | yes | POST /api/nextcloud/share-link (feature_gate.rs) |
| <a id="cap.compose.scheduled-send"></a>`cap.compose.scheduled_send` | features | global+domain+admin_user | true | plain |  | yes | JMAP EmailSubmission/set with sendAt (feature_gate.rs) |
| <a id="cap.compose.read-receipts"></a>`cap.compose.read_receipts` | features | global+domain+admin_user | true | plain |  | yes | JMAP Email/set create with mailwomanRequestReadReceipt (feature_gate.rs) |
| <a id="cap.signatures"></a>`cap.signatures` | features | global+domain+admin_user | true | plain |  | yes | writes under /api/account/signatures (feature_gate.rs) |
| <a id="cap.identities"></a>`cap.identities` | features | global+domain+admin_user | true | plain |  | yes | writes under /api/account/identities (feature_gate.rs) |
| <a id="cap.keys"></a>`cap.keys` | features | global+domain+admin_user | true | plain |  | yes | JMAP CryptoKey/* (feature_gate.rs); Keys module, compose sign and encrypt |
| <a id="cap.links.trusted"></a>`cap.links.trusted` | features | global+domain+admin_user | true | plain |  | yes | PUT /api/settings/links.* (feature_gate.rs) |
| <a id="cap.zoom"></a>`cap.zoom` | features | global+domain+admin_user | true | plain |  | yes | PUT /api/settings/reader.zoom (feature_gate.rs); reader zoom controls |
| <a id="cap.mail.focused-inbox"></a>`cap.mail.focused_inbox` | features | global+domain+admin_user | true | plain |  | yes | PUT /api/settings/mail.focusedInbox (feature_gate.rs); Focused and Other tabs |
| <a id="cap.views.events"></a>`cap.views.events` | features | global+domain+admin_user | true | plain |  | yes | JMAP Email/query on mwViewEvents (feature_gate.rs); Events view |
| <a id="cap.views.attachments"></a>`cap.views.attachments` | features | global+domain+admin_user | true | plain |  | yes | Attachments view in the sidebar (web only: the same messages stay reachable by search) |
| <a id="cap.search.operators"></a>`cap.search.operators` | features | global+domain+admin_user | true | plain |  | yes | JMAP Email/query text with field operators (feature_gate.rs); search filter chips |
| <a id="cap.saved-searches"></a>`cap.saved_searches` | features | global+domain+admin_user | true | plain |  | yes | writes under /api/account/saved-searches (feature_gate.rs) |
| <a id="cap.vacation"></a>`cap.vacation` | features | global+domain+admin_user | true | plain |  | yes | JMAP VacationResponse/* (feature_gate.rs, besides cap.rules) |
| <a id="cap.labels"></a>`cap.labels` | features | global+domain+admin_user | true | plain |  | yes | JMAP Email/set adding a non-system keyword (feature_gate.rs) |
| <a id="cap.snooze"></a>`cap.snooze` | features | global+domain+admin_user | true | plain |  | yes | JMAP Email/set snoozedUntil (feature_gate.rs) |
| <a id="cap.follow-up"></a>`cap.follow_up` | features | global+domain+admin_user | true | plain |  | yes | JMAP Email/set followUpAt (feature_gate.rs) |
| <a id="cap.masked"></a>`cap.masked` | features | global+domain+admin_user | true | plain |  | yes | /api/masked (feature_gate.rs) |
| <a id="cap.directory"></a>`cap.directory` | features | global+domain+admin_user | true | plain |  | yes | /api/directory (feature_gate.rs) |
| <a id="cap.mcp"></a>`cap.mcp` | features | global+domain+admin_user | true | plain |  | yes | /mcp with a key or token (scope_mw.rs) |
| <a id="cap.appearance.custom"></a>`cap.appearance.custom` | features | global+domain+admin_user | true | plain |  | yes | PUT /api/account/appearance (feature_gate.rs); theme, accent, custom and badge colours, density, font |
| <a id="cap.appearance.custom-code"></a>`cap.appearance.custom_code` | features | global+domain+admin_user | true | plain |  | yes | PUT /api/account/appearance/customization (feature_gate.rs), below appearance.customCssEnabled/customJsEnabled |
| <a id="cap.avatars"></a>`cap.avatars` | features | global+domain+admin_user | true | plain |  | yes | PUT /api/accounts/layout with avatars (feature_gate.rs) |
| <a id="cap.shortcuts"></a>`cap.shortcuts` | features | global+domain+admin_user | true | plain |  | yes | PUT /api/settings/shortcuts.* (feature_gate.rs); shortcut editor |
| <a id="cap.analytics"></a>`cap.analytics` | features | global+domain+admin_user | true | plain |  | yes | POST /api/analytics/events (feature_gate.rs); consent prompt |
| <a id="cap.apps.native"></a>`cap.apps.native` | features | global+domain+admin_user | true | plain |  | yes | native bearer sessions (feature_gate.rs) |
| <a id="access.protocols"></a>`access.protocols` | features | global+domain+admin_user | ["imap","pop3","ews"] | plain |  | yes | mail protocols a native account may use (imap, pop3, ews); /jmap and /api/v1 refuse the others (feature_gate.rs) |
| <a id="privacy.remotecontent"></a>`privacy.remoteContent` | privacy | global+domain+user | "ask" | plain |  | yes | Reader remote-content mode |
| <a id="privacy.trackers"></a>`privacy.trackers` | privacy | global+domain+user | true | plain |  | yes | Block email trackers |
| <a id="privacy.cleanlinks"></a>`privacy.cleanLinks` | privacy | global+domain+user | true | plain |  | yes | Remove tracking parameters from links |
| <a id="links.trusteddomains"></a>`links.trustedDomains` | privacy | global+domain+user | [] | plain |  | yes | exact trusted link domains that open from Review links without the destination step |
| <a id="links.trustedsubdomains"></a>`links.trustedSubdomains` | privacy | global+domain+user | false | plain |  | yes | whether trusted link domains also trust their subdomains |
| <a id="privacy.maxbytes"></a>`privacy.maxBytes` | privacy | global | 8388608 | plain |  | yes | Largest fetched remote image in bytes |
| <a id="privacy.cachebytes"></a>`privacy.cacheBytes` | privacy | global | 33554432 | plain |  | yes | Remote image cache size in bytes |
| <a id="privacy.ratepermin"></a>`privacy.ratePerMin` | privacy | global | 120 | plain |  | yes | Remote fetches per account per minute |
| <a id="privacy.fetchconnecttimeoutms"></a>`privacy.fetchConnectTimeoutMs` | privacy | global | 5000 | plain |  | yes | Remote image connect and TLS handshake timeout in milliseconds |
| <a id="privacy.fetchheaderstimeoutms"></a>`privacy.fetchHeadersTimeoutMs` | privacy | global | 10000 | plain |  | yes | Remote image time to response headers in milliseconds |
| <a id="privacy.fetchidletimeoutms"></a>`privacy.fetchIdleTimeoutMs` | privacy | global | 10000 | plain |  | yes | Remote image longest pause between body chunks in milliseconds |
| <a id="privacy.fetchtotaltimeoutms"></a>`privacy.fetchTotalTimeoutMs` | privacy | global | 20000 | plain |  | yes | Remote image total time per image in milliseconds |
| <a id="privacy.maxredirects"></a>`privacy.maxRedirects` | privacy | global | 3 | plain |  | yes | Redirects followed per remote image |
| <a id="privacy.maxconcurrentpermessage"></a>`privacy.maxConcurrentPerMessage` | privacy | global | 6 | plain |  | yes | Remote image fetches at a time per message |
| <a id="privacy.maxconcurrentperaccount"></a>`privacy.maxConcurrentPerAccount` | privacy | global | 12 | plain |  | yes | Remote image fetches at a time per account |
| <a id="privacy.allowdirect"></a>`privacy.allowDirect` | privacy | global | false | plain |  | yes | Permit direct remote-content mode with the direct-content capability |
| <a id="cap.remote-content.direct"></a>`cap.remote_content.direct` | privacy | global+domain+admin_user | false | plain |  | yes | Permit this account to load remote images directly |
| <a id="session.maxconcurrent"></a>`session.maxConcurrent` | signin | global+domain+admin_user | 10 | plain |  | yes | Maximum concurrent mailbox sessions |
| <a id="session.idlesecs"></a>`session.idleSecs` | signin | global+domain+admin_user | 1800 | plain |  | yes | Mailbox session idle timeout, bounded by the deployment cap |
| <a id="session.absolutesecs"></a>`session.absoluteSecs` | signin | global+domain+admin_user | 1296000 | plain |  | yes | Mailbox session absolute timeout, bounded by the deployment cap |
| <a id="twofa.rememberdays"></a>`twofa.rememberDays` | signin | global+domain+admin_user | 0 | plain |  | yes | Remembered-device lifetime; zero disables remembered devices |
| <a id="twofa.passkeysenabled"></a>`twofa.passkeysEnabled` | signin | global+domain+admin_user | true | plain |  | yes | Allow passkey authentication |
| <a id="twofa.required"></a>`twofa.required` | signin | global+domain+admin_user | false | plain |  | yes | Require a second authentication factor |
| <a id="twofa.passkeyattachment"></a>`twofa.passkeyAttachment` | signin | global+domain+admin_user | "any" | plain |  | yes | Which authenticators can hold a mailbox passkey |
| <a id="twofa.passkeyuserverification"></a>`twofa.passkeyUserVerification` | signin | global+domain+admin_user | "preferred" | plain |  | yes | Whether a passkey must verify the user (PIN or biometric) |
| <a id="twofa.passkeyresidentkey"></a>`twofa.passkeyResidentKey` | signin | global+domain+admin_user | "discouraged" | plain |  | yes | Whether a passkey must be a discoverable credential |
| <a id="twofa.passkeylegacy"></a>`twofa.passkeyLegacy` | signin | global+domain+admin_user | "allow" | plain |  | yes | Passkeys registered before a stricter passkey policy |
| <a id="twofa.lastfactorremoval"></a>`twofa.lastFactorRemoval` | signin | global+domain+admin_user | "allow" | plain |  | yes | Removing the last second factor of an account |
| <a id="sieve.maxscriptbytes"></a>`sieve.maxScriptBytes` | sieve | global+domain+admin_user | 65536 | plain |  | yes | Sieve script byte limit |
| <a id="sieve.maxscripts"></a>`sieve.maxScripts` | sieve | global+domain+admin_user | 5 | plain |  | yes | Sieve script count limit |
| <a id="sieve.maxredirects"></a>`sieve.maxRedirects` | sieve | global+domain+admin_user | 4 | plain |  | yes | Sieve redirect count limit |
| <a id="sieve.vacationallowed"></a>`sieve.vacationAllowed` | sieve | global+domain+admin_user | false | plain |  | yes | Sieve vacation actions |
| <a id="sieve.allowedextensions"></a>`sieve.allowedExtensions` | sieve | global+domain+admin_user | null | plain |  | yes | Allowed Sieve extensions |
| <a id="cap.sieve.raw-edit"></a>`cap.sieve.raw_edit` | sieve | global+domain+admin_user | false | plain |  | yes | Raw Sieve editing permission |
| <a id="cap.sieve.forward-external"></a>`cap.sieve.forward_external` | sieve | global+domain+admin_user | false | plain |  | yes | Sieve forwarding outside the account domain |
| <a id="mail.externalrecipientwarning"></a>`mail.externalRecipientWarning` | mail | global+domain+user | "warn" | plain |  | yes | external recipient composer warning |
| <a id="mail.trustedrecipientdomains"></a>`mail.trustedRecipientDomains` | mail | global+domain+user | [] | plain |  | yes | exact trusted recipient domains for composer warnings |
| <a id="time.displayzone"></a>`time.displayZone` | time | global+domain+user | "local" | plain |  | yes | preferred message display time zone |
| <a id="time.comparisonzone"></a>`time.comparisonZone` | time | global+domain+user | "" | plain |  | yes | optional second time zone for message and event helpers |
| <a id="time.showhelpers"></a>`time.showHelpers` | time | global+domain+user | true | plain |  | yes | show message and event time zone comparisons |
| <a id="shortcuts.enabled"></a>`shortcuts.enabled` | shortcuts | global+domain+user | true | plain |  | yes | enable application keyboard shortcuts |
| <a id="shortcuts.compose"></a>`shortcuts.compose` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for compose |
| <a id="shortcuts.archive"></a>`shortcuts.archive` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for archive |
| <a id="shortcuts.reply"></a>`shortcuts.reply` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for reply |
| <a id="shortcuts.replyall"></a>`shortcuts.replyAll` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for replyAll |
| <a id="shortcuts.forward"></a>`shortcuts.forward` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for forward |
| <a id="shortcuts.next"></a>`shortcuts.next` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for next |
| <a id="shortcuts.previous"></a>`shortcuts.previous` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for previous |
| <a id="shortcuts.search"></a>`shortcuts.search` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for search |
| <a id="shortcuts.refresh"></a>`shortcuts.refresh` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for refresh |
| <a id="shortcuts.settings"></a>`shortcuts.settings` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for settings |
| <a id="shortcuts.trash"></a>`shortcuts.trash` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for trash |
| <a id="shortcuts.markread"></a>`shortcuts.markRead` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for markRead |
| <a id="shortcuts.markunread"></a>`shortcuts.markUnread` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for markUnread |
| <a id="shortcuts.spam"></a>`shortcuts.spam` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for spam |
| <a id="shortcuts.zoomin"></a>`shortcuts.zoomIn` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for zoomIn |
| <a id="shortcuts.zoomout"></a>`shortcuts.zoomOut` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for zoomOut |
| <a id="shortcuts.zoomreset"></a>`shortcuts.zoomReset` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for zoomReset |
| <a id="shortcuts.openintab"></a>`shortcuts.openInTab` | shortcuts | global+domain+user | "inherit" | plain |  | yes | keyboard binding for openInTab |
| <a id="mail.quotestyle"></a>`mail.quoteStyle` | mail | global+domain+user | "classic" | plain |  | yes | reply and forward header style |
| <a id="mail.quickreply"></a>`mail.quickReply` | mail | global+domain+user | false | plain |  | yes | enable the compact message reply composer |
| <a id="mail.requestreadreceipt"></a>`mail.requestReadReceipt` | mail | global+domain+user | false | plain |  | yes | request a read receipt by default when composing supported mail |
| <a id="mail.requestdeliveryreceipt"></a>`mail.requestDeliveryReceipt` | mail | global+domain+user | false | plain |  | yes | request delivery status notifications by default when the account supports DSN |
| <a id="mail.showfrom"></a>`mail.showFrom` | mail | global+domain+user | false | plain |  | yes | show the From field by default when composing; sending authority is unchanged |
| <a id="mail.modifierselection"></a>`mail.modifierSelection` | mail | global+domain+user | true | plain |  | yes | select messages with Ctrl, Command, Shift and arrow keys |
| <a id="mail.draftsavemode"></a>`mail.draftSaveMode` | mail | global+domain+user | "realtime" | plain |  | yes | account draft save scheduling; manual disables automatic upstream writes |
| <a id="mail.draftdebouncems"></a>`mail.draftDebounceMs` | mail | global+domain+user | 1000 | plain |  | yes | idle milliseconds before saving an edited account draft |
| <a id="mail.draftintervalsecs"></a>`mail.draftIntervalSecs` | mail | global+domain+user | 30 | plain |  | yes | periodic account draft save interval in seconds |
| <a id="mail.resumetabs"></a>`mail.resumeTabs` | mail | global+domain+user | true | plain |  | yes | restore account-owned workspace tab references after reload |
| <a id="mail.dragtofolder"></a>`mail.dragToFolder` | mail | global+domain+user | true | plain |  | yes | move selected messages by dragging them to a folder |
| <a id="mail.foldercounts"></a>`mail.folderCounts` | mail | global+domain+user | "both" | plain |  | yes | counts displayed beside mail folders (unreadIfAny: unread count, hidden at 0); account wide across linked mailboxes |
| <a id="mail.foldercountscope"></a>`mail.folderCountScope` | mail | global+domain+user | "all" | plain |  | yes | which folders show counts: all, or only folders with a role (Inbox, Drafts, Sent, Archive, Junk, Trash, Outbox, other special-use); account wide |
| <a id="mail.compactfolderbadges"></a>`mail.compactFolderBadges` | mail | global+domain+user | true | plain |  | yes | smaller count badges for folders other than Inbox |
| <a id="mail.focusedinbox"></a>`mail.focusedInbox` | mail | global+domain+user | false | plain |  | yes | split Inbox into Focused and Other tabs (newsletters and automated senders go to Other); off by default |
| <a id="mail.accountunreadbadge"></a>`mail.accountUnreadBadge` | mail | global+domain+user | false | plain |  | yes | Inbox unread count beside the mailbox name in the sidebar head; the mailbox menu always shows per-mailbox counts; account wide |
| <a id="contacts.avatarmode"></a>`contacts.avatarMode` | contacts | global+domain+user | "initials" | plain |  | yes | local generated avatars or embedded photos from this account contacts; never remote photo requests |
| <a id="contacts.sendericons"></a>`contacts.senderIcons` | contacts | global+domain+user | true | plain |  | yes | bundled sender-domain icons with generated initials fallback; domain hints are not authentication evidence |
| <a id="reader.actionstyle"></a>`reader.actionStyle` | reader | global+domain+user | "icons" | plain |  | yes | compact reader actions or icons with visible labels |
| <a id="reader.colormode"></a>`reader.colorMode` | reader | global+domain+user | "auto" | plain |  | yes | message color adaptation inside the isolated reader |
| <a id="reader.fontsize"></a>`reader.fontSize` | reader | global+domain+user | 14 | plain |  | yes | message reading font size in pixels |
| <a id="reader.lineheight"></a>`reader.lineHeight` | reader | global+domain+user | 1.6 | plain |  | yes | message reading line height |
| <a id="reader.contentwidth"></a>`reader.contentWidth` | reader | global+domain+user | "wide" | plain |  | yes | message content width |
| <a id="reader.loading"></a>`reader.loading` | reader | global+domain+user | "skeleton" | plain |  | yes | message loading appearance |
| <a id="reader.loadingdelayms"></a>`reader.loadingDelayMs` | reader | global+domain+user | 120 | plain |  | yes | delay before showing message loading feedback |
| <a id="reader.zoom"></a>`reader.zoom` | reader | global+domain+user | 100 | plain |  | yes | message body zoom in percent; the reader snaps it to its zoom steps |
| <a id="reader.attachmentscollapsed"></a>`reader.attachmentsCollapsed` | reader | global+domain+user | false | plain |  | yes | start the reader's attachments bar collapsed (t110) |
| <a id="ui.language"></a>`ui.language` | interface | global+domain+user | "browser" | plain |  | yes | web client locale (cached in localStorage mw.locale) |
| <a id="ui.bar.mailtopbar"></a>`ui.bar.mailTopBar` | interface | global+domain+user | "pinned" | plain |  | yes | mail top bar: always shown, or auto-hidden behind a slim handle |
| <a id="ui.bar.workspacetabs"></a>`ui.bar.workspaceTabs` | interface | global+domain+user | "pinned" | plain |  | yes | workspace tab strip: always shown, or auto-hidden behind a slim handle |
| <a id="ui.bar.mailribbon"></a>`ui.bar.mailRibbon` | interface | global+domain+user | "pinned" | plain |  | yes | mail ribbon (ribbon layout): always shown, or only its tab names |
| <a id="ui.bar.composerribbon"></a>`ui.bar.composerRibbon` | interface | global+domain+user | "pinned" | plain |  | yes | composer ribbon: always shown, or only its tab names |
| <a id="appearance.theme"></a>`appearance.theme` | appearance | global+domain+user | "dark" | plain |  | yes | appearance blob theme/lightTheme/darkTheme; deployment default |
| <a id="appearance.accent"></a>`appearance.accent` | appearance | global+domain+user | "" | plain |  | yes | appearance blob accentId (empty = the theme's own) |
| <a id="appearance.badgecolor"></a>`appearance.badgeColor` | appearance | global+domain+user | "default" | plain |  | yes | appearance blob badgeColor: unread-count badge colour (default = the theme's signal colour; accent-lighter/accent-darker = the accent 0.2 OKLCH L lighter/darker) |
| <a id="appearance.density"></a>`appearance.density` | appearance | global+domain+user | "cozy" | plain |  | yes | appearance blob density |
| <a id="appearance.font"></a>`appearance.font` | appearance | global+domain+user | "default" | plain |  | yes | self-hosted interface font; account choice or administrator default/lock |
| <a id="appearance.brandname"></a>`appearance.brandName` | appearance | global | "Mailwoman" | plain |  | yes | public config brand name |
| <a id="login.showdomainhint"></a>`login.showDomainHint` | appearance | global | false | plain |  | yes | public config preferred-domain hint visibility |
| <a id="login.remembersignins"></a>`login.rememberSignins` | appearance | global | true | plain |  | yes | allow this browser to remember successful sign-in addresses; never credentials |
| <a id="login.showrememberedhint"></a>`login.showRememberedHint` | appearance | global | false | plain |  | yes | show explanatory text for remembered sign-in addresses |
| <a id="login.notice"></a>`login.notice` | appearance | global | null | plain |  | yes | public config sign-in notice |
| <a id="receipts.dsnallowed"></a>`receipts.dsnAllowed` | mail | global+domain+admin_user | true | plain |  | yes | Allow delivery status notification requests at SMTP submission |
| <a id="mail.readreceipts"></a>`mail.readReceipts` | mail | global+domain+user | "ask" | plain |  | yes | mdn_policy:<account> (MDN/send) |
| <a id="notifications.pushallowed"></a>`notifications.pushAllowed` | notifications | global+domain | true | plain |  | yes | account-authorized push registration and delivery |
| <a id="offline.persistentstorageallowed"></a>`offline.persistentStorageAllowed` | offline | global+domain | false | plain |  | yes | explicit browser persistent-storage permission request |
| <a id="notifications.enabled"></a>`notifications.enabled` | notifications | global+domain+user | false | plain |  | yes | notification rules enabled flag |
| <a id="offline.retentiondays"></a>`offline.retentionDays` | offline | global+domain+user | 30 | plain |  | yes | web device prefs offlineRetentionDays (client clamps) |
| <a id="offline.budgetmb"></a>`offline.budgetMb` | offline | global+domain+user | 250 | plain |  | yes | web device prefs offlineBudgetMb (client clamps) |
| <a id="assistant.enabled"></a>`assistant.enabled` | assistant | global+domain+user | true | plain |  | yes | below the assist_config deployment ceiling and kill switch |
| <a id="assist.profile"></a>`assist.profile` | assistant | global+domain+admin_user | "" | plain |  | yes | AI provider profile id; empty = the deployment endpoint (assist_profiles.rs) |
| <a id="upstream.default"></a>`upstream.default` | upstream | global | null | url | `MW_UPSTREAM_DEFAULT` (fallback) | yes | deployment default upstream (domain config wins) |
| <a id="upstream.proxytimeoutsecs"></a>`upstream.proxyTimeoutSecs` | upstream | global | 300 | plain |  | yes | proxy per-request timeout (lib.rs) |
| <a id="signin.antibotmode"></a>`signin.antiBotMode` | signin | global | "off" | plain |  | yes | initial sign-in proof-of-work policy |
| <a id="signin.antibotdifficulty"></a>`signin.antiBotDifficulty` | signin | global | 2000 | plain |  | yes | maximum proof-of-work solver counter |
| <a id="signin.antibotexpirysecs"></a>`signin.antiBotExpirySecs` | signin | global | 180 | plain |  | yes | single-use challenge lifetime |
| <a id="security.protection.ratemode"></a>`security.protection.rateMode` | security | global | "off" | plain |  | yes | Off by default. Observe records budget exhaustion without refusal; enforce returns a temporary 429. Existing protocol concurrency limits remain independent. |
| <a id="security.protection.adaptivemode"></a>`security.protection.adaptiveMode` | security | global | "off" | plain |  | yes | Off by default. Observe records explainable typed anomalies and proposed actions; enforce applies bounded temporary throttles or blocks. This is a deterministic counter policy, not an AI classifier. |
| <a id="security.protection.clientrequests"></a>`security.protection.clientRequests` | security | global | 600 | plain |  | yes | Requests per window from one source address that is not signed in, and every sign-in or identity-confirmation request, across replicas. A request with a valid mailbox or admin session uses its own allowance instead. Zero disables this dimension; the source address comes only from the trusted proxy resolver. |
| <a id="security.protection.accountrequests"></a>`security.protection.accountRequests` | security | global | 1800 | plain |  | yes | Requests per window for one signed-in mailbox, across its sessions, devices and addresses and across replicas. Kept above the anonymous allowance so signed-in use has more room than an unknown address. Zero disables this dimension. |
| <a id="security.protection.adminrequests"></a>`security.protection.adminRequests` | security | global | 1200 | plain |  | yes | Requests per window for one signed-in administrator, across replicas. Kept above the anonymous allowance. Zero disables this dimension. |
| <a id="security.protection.ratewindowsecs"></a>`security.protection.rateWindowSecs` | security | global | 60 | plain |  | yes | Fixed request-budget interval. Capacity load shedding and ordinary not-found responses are not adaptive evidence. |
| <a id="security.protection.adaptiveclients"></a>`security.protection.adaptiveClients` | security | global | true | plain |  | yes | Enable this subject dimension when adaptive mode is active. Accounts and administrators are counted only after their identity is verified. |
| <a id="security.protection.adaptiveaccounts"></a>`security.protection.adaptiveAccounts` | security | global | true | plain |  | yes | Enable this subject dimension when adaptive mode is active. Accounts and administrators are counted only after their identity is verified. |
| <a id="security.protection.adaptiveadmins"></a>`security.protection.adaptiveAdmins` | security | global | true | plain |  | yes | Enable this subject dimension when adaptive mode is active. Accounts and administrators are counted only after their identity is verified. |
| <a id="security.protection.shortwindowsecs"></a>`security.protection.shortWindowSecs` | security | global | 300 | plain |  | yes | Rolling observation window. Long windows support hours through thirty days; bucket rounding can retain at most one extra sixty-fourth of a window. |
| <a id="security.protection.shortminimumsamples"></a>`security.protection.shortMinimumSamples` | security | global | 30 | plain |  | yes | Minimum observed normal plus anomalous requests before the ratio trigger applies. The absolute anomaly threshold is independent of this minimum. |
| <a id="security.protection.shortabsoluteanomalies"></a>`security.protection.shortAbsoluteAnomalies` | security | global | 20 | plain |  | yes | Trigger when this many selected anomalies accumulate, even when normal requests dilute the ratio. Zero disables this trigger. |
| <a id="security.protection.shortratiopercent"></a>`security.protection.shortRatioPercent` | security | global | 50 | plain |  | yes | Trigger when selected anomalies reach this percentage after the minimum sample count. Either the absolute or ratio trigger suffices. Zero disables this trigger. |
| <a id="security.protection.longwindowsecs"></a>`security.protection.longWindowSecs` | security | global | 604800 | plain |  | yes | Rolling observation window. Long windows support hours through thirty days; bucket rounding can retain at most one extra sixty-fourth of a window. |
| <a id="security.protection.longminimumsamples"></a>`security.protection.longMinimumSamples` | security | global | 100 | plain |  | yes | Minimum observed normal plus anomalous requests before the ratio trigger applies. The absolute anomaly threshold is independent of this minimum. |
| <a id="security.protection.longabsoluteanomalies"></a>`security.protection.longAbsoluteAnomalies` | security | global | 100 | plain |  | yes | Trigger when this many selected anomalies accumulate, even when normal requests dilute the ratio. Zero disables this trigger. |
| <a id="security.protection.longratiopercent"></a>`security.protection.longRatioPercent` | security | global | 25 | plain |  | yes | Trigger when selected anomalies reach this percentage after the minimum sample count. Either the absolute or ratio trigger suffices. Zero disables this trigger. |
| <a id="security.protection.normalcapperbucket"></a>`security.protection.normalCapPerBucket` | security | global | 1000 | plain |  | yes | Bound the normal baseline learned in each rolling-window bucket. Independent absolute anomaly budgets prevent unlimited valid-request padding from hiding abuse. |
| <a id="security.protection.signals"></a>`security.protection.signals` | security | global | ["malformed","oversized","credential-rejected"] | plain |  | yes | Select authoritative malformed, oversized, credential-rejected, permission-denied, body-deadline or opt-in admin-route-miss observations. Admin route probing excludes entity-not-found responses and static assets; generic status codes, capacity refusals and user-agent strings do not establish abuse. |
| <a id="security.protection.throttlesecs"></a>`security.protection.throttleSecs` | security | global | 3 | plain |  | yes | First threshold crossing starts a fixed throttle deadline. Requests during that deadline do not extend it. |
| <a id="security.protection.blocksecs"></a>`security.protection.blockSecs` | security | global | 300 | plain |  | yes | A later threshold crossing before recovery may escalate to this bounded block. No permanent account lock is created. |
| <a id="security.protection.decaysecs"></a>`security.protection.decaySecs` | security | global | 86400 | plain |  | yes | After this anomaly-free interval and expiry of the current action, escalation returns to its initial stage. Rolling evidence expires according to its configured windows. |
| <a id="security.protection.exemptclients"></a>`security.protection.exemptClients` | security | global | [] | plain |  | yes | Exact IP addresses or CIDR networks exempt only the client dimension. Verified account limits still apply. Use deliberately for shared NAT networks; forwarded addresses are trusted only through configured proxies. |
| <a id="security.protection.exemptaccounts"></a>`security.protection.exemptAccounts` | security | global | [] | plain |  | yes | Exact server-verified mailbox account IDs exempt only their account dimension, never the source-client dimension. Email addresses and caller-supplied account parameters are not identity evidence. |
| <a id="security.protection.exemptadmins"></a>`security.protection.exemptAdmins` | security | global | [] | plain |  | yes | Exact authenticated administrator IDs exempt only their administrator dimension. Existing sign-in failure locks and strict administrator session deadlines remain unchanged. |
| <a id="security.protection.storagefailure"></a>`security.protection.storageFailure` | security | global | "closed" | plain |  | yes | Closed refuses protected work when an enabled enforcement budget cannot be checked. Open temporarily permits it with a bounded operational warning. Observe-only failures never impose an enforcement action. |
| <a id="security.adminprobe.enabled"></a>`security.adminProbe.enabled` | security | global | true | plain | `MW_ADMIN_PROBE_ENABLED` (pin) | yes | Count requests that look for the admin console per client network (IPv4 address, IPv6 /64) and slow, then block, a source that keeps going. The grace allowance absorbs typos and stale bookmarks. |
| <a id="security.adminprobe.graceallowance"></a>`security.adminProbe.graceAllowance` | security | global | 10 | plain |  | yes | Misses per window that are ignored. The score drains continuously at this many points per window. |
| <a id="security.adminprobe.windowsecs"></a>`security.adminProbe.windowSecs` | security | global | 600 | plain |  | yes | The window the grace allowance applies to. |
| <a id="security.adminprobe.blockafter"></a>`security.adminProbe.blockAfter` | security | global | 10 | plain |  | yes | Misses beyond the grace allowance that start a block. Misses in between are answered after the tarpit delay. |
| <a id="security.adminprobe.tarpitdelayms"></a>`security.adminProbe.tarpitDelayMs` | security | global | 1500 | plain |  | yes | Delay added to each counted miss above the grace allowance. Zero turns the tarpit off. Requests that are not misses are never delayed. |
| <a id="security.adminprobe.blocksecs"></a>`security.adminProbe.blockSecs` | security | global | 900 | plain |  | yes | Length of the first block. Every request from a blocked source gets the same answer, whatever its path. |
| <a id="security.adminprobe.maxblocksecs"></a>`security.adminProbe.maxBlockSecs` | security | global | 86400 | plain |  | yes | A repeat block doubles the previous length up to this cap. |
| <a id="security.adminprobe.repeatwindowsecs"></a>`security.adminProbe.repeatWindowSecs` | security | global | 86400 | plain |  | yes | A block that starts within this long after the previous one ended counts as a repeat. After a quiet period this long, the next block is a first block again. |
| <a id="security.adminprobe.blockstatus"></a>`security.adminProbe.blockStatus` | security | global | "429" | plain |  | yes | HTTP status of the blocked answer, sent with Retry-After. |
| <a id="security.adminprobe.signals"></a>`security.adminProbe.signals` | security | global | ["probe-path","slug-guess","admin-signin-failed","admin-unauthenticated"] | plain |  | yes | Which misses count: probe-path (a path on the probe list), slug-guess (a 404 at the console's path depth), admin-signin-failed (a refused admin sign-in) and admin-unauthenticated (an admin API call without a valid admin session). |
| <a id="security.adminprobe.probepaths"></a>`security.adminProbe.probePaths` | security | global | ["/admin","/administrator","/wp-admin","/wp-login.php","/xmlrpc.php","/phpmyadmin","/pma","/myadmin","/adminer.php","/.env","/.git","/manager/html","/admin.php","/cpanel","/webadmin","/server-status","/actuator","/vendor/phpunit","/cgi-bin","/boaform"] | plain |  | yes | Paths scanners ask for. A request whose path contains one of these as whole segments (any case) counts as a miss; the console's own path never does. |
| <a id="security.adminprobe.allowlist"></a>`security.adminProbe.allowlist` | security | global | [] | plain |  | yes | IP addresses or CIDR networks (office, VPN) that are never counted or blocked. Forwarded addresses are trusted only through configured proxies. |
| <a id="security.adminprobe.auditip"></a>`security.adminProbe.auditIp` | security | global | "masked" | plain |  | yes | How a blocked source appears in the audit log and the Security centre: masked (IPv4 /24, IPv6 /48) or full. The fail2ban line always carries the full address. |
| <a id="security.adminprobe.fail2banlog"></a>`security.adminProbe.fail2banLog` | security | global | true | plain |  | yes | Write one fail2ban-readable line (mailwoman[admin-probe]: source blocked; ... rhost=<address>) when a source is blocked. |
| <a id="security.signin.geoenforcesessions"></a>`security.signin.geoEnforceSessions` | security | global | false | plain |  | yes | also check current source country on authenticated requests |
| <a id="security.signin.geoscope"></a>`security.signin.geoScope` | security | global | "off" | plain |  | yes | country policy applies only to new sign-ins |
| <a id="security.signin.geomode"></a>`security.signin.geoMode` | security | global | "allow" | plain |  | yes | allow only listed countries or deny listed countries |
| <a id="security.signin.geocountries"></a>`security.signin.geoCountries` | security | global | [] | plain |  | yes | uppercase two-letter country codes |
| <a id="security.signin.geounknown"></a>`security.signin.geoUnknown` | security | global | "deny" | plain |  | yes | decision for unmapped addresses in a readable local database |
| <a id="security.signin.mailboxfailurelimit"></a>`security.signin.mailboxFailureLimit` | security | global | 10 | plain |  | yes | rejected mailbox credentials per source before a temporary lock |
| <a id="security.signin.adminfailurelimit"></a>`security.signin.adminFailureLimit` | security | global | 5 | plain |  | yes | rejected admin credentials per source before a temporary lock |
| <a id="security.signin.failurewindowsecs"></a>`security.signin.failureWindowSecs` | security | global | 900 | plain |  | yes | rolling failed-credential counting window |
| <a id="security.signin.locksecs"></a>`security.signin.lockSecs` | security | global | 900 | plain |  | yes | fixed temporary source lock duration; rejected retries never extend it |
| <a id="signin.retrycooldownsecs"></a>`signin.retryCooldownSecs` | signin | global | 3 | plain |  | yes | wait after a sign-in attempt that does not end in a session (per source network) |
| <a id="ratelimit.diagnosticsburst"></a>`rateLimit.diagnosticsBurst` | security | global | 10 | plain |  | yes | diagnostic runs and backend checks an admin session may start in a row (keyed by session); repeating a job the session has running returns it uncharged |
| <a id="ratelimit.diagnosticsperminute"></a>`rateLimit.diagnosticsPerMinute` | security | global | 10 | plain |  | yes | diagnostic runs and jobs per minute an admin session regains |
| <a id="signin.newusers"></a>`signin.newUsers` | signin | global+domain | "open" | plain |  | yes | signin_policy::admit new-user rule |
| <a id="signin.domainmode"></a>`signin.domainMode` | signin | global | "any" | plain |  | yes | domain gate mode (e12) |
| <a id="signin.refusaldetail"></a>`signin.refusalDetail` | signin | global | "explain" | plain |  | yes | refusal answers: explain (403 codes) or uniform (401) |
| <a id="signin.ldapbindprecheck"></a>`signin.ldapBindPrecheck` | signin | global | false | plain | `MW_LDAP_BIND_AUTH` (pin) | yes | LDAP bind pre-check before the mail credential |
| <a id="twofa.pendingttlsecs"></a>`twofa.pendingTtlSecs` | signin | global | 300 | plain |  | yes | pending 2FA token life (twofa_routes.rs) |
| <a id="twofa.maxattempts"></a>`twofa.maxAttempts` | signin | global | 5 | plain |  | yes | pending 2FA attempts (twofa_routes.rs) |
| <a id="admin.allowlastsuperadminloss"></a>`admin.allowLastSuperadminLoss` | admin | global | false | plain |  | yes | Explicitly allow removing the final Super admin access; requires privileged confirmation |
| <a id="admin.session.idlesecs"></a>`admin.session.idleSecs` | admin | global | 1800 | plain |  | yes | admin session idle timeout |
| <a id="admin.session.absolutesecs"></a>`admin.session.absoluteSecs` | admin | global | 43200 | plain |  | yes | admin session absolute cap |
| <a id="passwd.policy.minlength"></a>`passwd.policy.minLength` | passwords | global+domain | 12 | plain |  | yes | password policy minimum length (every backend) |
| <a id="passwd.policy.requireupper"></a>`passwd.policy.requireUpper` | passwords | global+domain | false | plain |  | yes | password policy: an upper-case letter |
| <a id="passwd.policy.requirelower"></a>`passwd.policy.requireLower` | passwords | global+domain | false | plain |  | yes | password policy: a lower-case letter |
| <a id="passwd.policy.requiredigit"></a>`passwd.policy.requireDigit` | passwords | global+domain | false | plain |  | yes | password policy: a digit |
| <a id="passwd.policy.requiresymbol"></a>`passwd.policy.requireSymbol` | passwords | global+domain | false | plain |  | yes | password policy: a symbol |
| <a id="attachments.previewallowed"></a>`attachments.previewAllowed` | attachments | global+domain | true | plain |  | yes | allow attachment previews |
| <a id="attachments.downloadallowed"></a>`attachments.downloadAllowed` | attachments | global+domain | true | plain |  | yes | allow original attachment delivery; disabling also prevents raw previews |
| <a id="attachments.maxbytes"></a>`attachments.maxBytes` | attachments | global+domain | 0 | plain |  | yes | maximum delivered attachment bytes; zero is unlimited |
| <a id="attachments.allowedextensions"></a>`attachments.allowedExtensions` | attachments | global+domain | [] | plain |  | yes | attachment allowed Extensions; blocked rules take precedence |
| <a id="attachments.allowedmimetypes"></a>`attachments.allowedMimeTypes` | attachments | global+domain | [] | plain |  | yes | attachment allowed MimeTypes; blocked rules take precedence |
| <a id="attachments.blockedextensions"></a>`attachments.blockedExtensions` | attachments | global+domain | [] | plain |  | yes | attachment blocked Extensions; blocked rules take precedence |
| <a id="attachments.blockedmimetypes"></a>`attachments.blockedMimeTypes` | attachments | global+domain | [] | plain |  | yes | attachment blocked MimeTypes; blocked rules take precedence |
| <a id="attachments.warnextensions"></a>`attachments.warnExtensions` | attachments | global+domain | [] | plain |  | yes | attachment warn Extensions; blocked rules take precedence |
| <a id="attachments.warnmimetypes"></a>`attachments.warnMimeTypes` | attachments | global+domain | [] | plain |  | yes | attachment warn MimeTypes; blocked rules take precedence |
| <a id="limit.attachment-bytes"></a>`limit.attachment_bytes` | limits | global+domain+admin_user | 50000000 | plain |  | yes | Attachment bytes; 0 = no limit |
| <a id="limit.send-per-hour"></a>`limit.send_per_hour` | limits | global+domain+admin_user | 0 | plain |  | yes | Submissions per hour; 0 = no limit |
| <a id="limit.recipients-per-message"></a>`limit.recipients_per_message` | limits | global+domain+admin_user | 0 | plain |  | yes | Recipients per message; 0 = no limit |
| <a id="limit.storage-bytes"></a>`limit.storage_bytes` | limits | global+domain+admin_user | 0 | plain |  | yes | blob reservations; 0 = no limit |
| <a id="limit.send-per-day"></a>`limit.send_per_day` | limits | global+domain+admin_user | 0 | plain |  | yes | EmailSubmission/set creates per day; 0 = no limit |
| <a id="limit.ai-calls-per-day"></a>`limit.ai_calls_per_day` | limits | global+domain+admin_user | 0 | plain |  | yes | assistant adapter calls per day; 0 = no limit |
| <a id="limit.linked-mailboxes"></a>`limit.linked_mailboxes` | limits | global+domain+admin_user | 0 | plain |  | yes | linked mailboxes per user, below accounts.maxLinked; 0 = no limit beyond it (feature_gate.rs) |
| <a id="quota.warnthresholds"></a>`quota.warnThresholds` | limits | global+domain | [80,95] | plain |  | yes | usage warning thresholds (percent) |
| <a id="quota.warnenabled"></a>`quota.warnEnabled` | limits | global+domain | true | plain |  | yes | usage warnings on |
| <a id="usage.retentiondays"></a>`usage.retentionDays` | limits | global | 400 | plain |  | yes | usage counter pruning |
| <a id="usage.meterapi"></a>`usage.meterApi` | limits | global | true | plain |  | yes | API usage counters |
| <a id="usage.measureintervalhours"></a>`usage.measureIntervalHours` | limits | global | 6 | plain |  | yes | storage measurer interval |
| <a id="assistant.ratelimitpermin"></a>`assistant.rateLimitPerMin` | assistant | global | 0 | plain | `MW_ASSIST_RATE_LIMIT_PER_MIN` (pin) | yes | assistant rate limit; 0 = none (v7_mount.rs) |
| <a id="plugins.kv.maxkeybytes"></a>`plugins.kv.maxKeyBytes` | plugins | global | 256 | plain | `MW_PLUGIN_KV_MAX_KEY_BYTES` (pin) | yes | plugin KV key size (v7_mount.rs) |
| <a id="plugins.kv.maxvaluebytes"></a>`plugins.kv.maxValueBytes` | plugins | global | 65536 | plain | `MW_PLUGIN_KV_MAX_VALUE_BYTES` (pin) | yes | plugin KV value size |
| <a id="plugins.kv.maxtotalbytes"></a>`plugins.kv.maxTotalBytes` | plugins | global | 5242880 | plain | `MW_PLUGIN_KV_MAX_TOTAL_BYTES` (pin) | yes | plugin KV total size |
| <a id="plugins.kv.maxkeys"></a>`plugins.kv.maxKeys` | plugins | global | 1000 | plain | `MW_PLUGIN_KV_MAX_KEYS` (pin) | yes | plugin KV key count |
| <a id="import.maxmessages"></a>`import.maxMessages` | import | global+domain | 20000 | plain |  | yes | import message count (import_routes.rs) |
| <a id="import.maxmessagebytes"></a>`import.maxMessageBytes` | import | global+domain | 67108864 | plain |  | yes | import message size (import_routes.rs) |
| <a id="webhooks.maxattempts"></a>`webhooks.maxAttempts` | webhooks | global | 3 | plain |  | yes | webhook delivery attempts |
| <a id="webhooks.timeoutsecs"></a>`webhooks.timeoutSecs` | webhooks | global | 10 | plain |  | yes | webhook delivery timeout |
| <a id="outbox.maxattempts"></a>`outbox.maxAttempts` | mail | global | 8 | plain |  | yes | submission attempts (engine dispatcher) |
| <a id="realtime.rechecksecs"></a>`realtime.recheckSecs` | realtime | global | 30 | plain | `MW_REALTIME_RECHECK_SECS` (pin) | yes | push stream re-check period (push.rs) |
| <a id="privacy.watermark.enabled"></a>`privacy.watermark.enabled` | privacy | global+domain | false | plain | `MW_WATERMARK` (pin) | yes | reader watermark |
| <a id="privacy.watermark.opacity"></a>`privacy.watermark.opacity` | privacy | global+domain | 0.08 | plain | `MW_WATERMARK_OPACITY` (pin) | yes | reader watermark opacity |
| <a id="privacy.opentracking.retentiondays"></a>`privacy.openTracking.retentionDays` | privacy | global | 90 | plain |  | yes | open-tracking token retention |
| <a id="privacy.opentracking.unboundttlhours"></a>`privacy.openTracking.unboundTtlHours` | privacy | global | 24 | plain |  | yes | open-tracking unbound token life |
| <a id="maskedemail.domain"></a>`maskedEmail.domain` | mail | global | null | plain | `MW_MASKED_EMAIL_DOMAIN` (pin) | yes | masked-address domain (masked.rs) |
| <a id="abuse.reportaddress"></a>`abuse.reportAddress` | mail | global+domain | null | plain | `MW_ABUSE_ADDRESS` (pin) | yes | ARF report address |
| <a id="dlp.adminaddress"></a>`dlp.adminAddress` | mail | global | null | plain | `MW_DLP_ADMIN` (pin) | yes | DLP notice address |
| <a id="observability.errorforwardurl"></a>`observability.errorForwardUrl` | observability | global | null | url | `MW_ERROR_FORWARD_URL` (pin) | yes | client error forwarding (errors.rs) |
| <a id="egress.probeurl"></a>`egress.probeUrl` | egress | global | "https://example.com/" | url | `MW_EGRESS_PROBE_URL` (pin) | yes | egress proxy test target |
| <a id="push.vapidcontact"></a>`push.vapidContact` | notifications | global | null | url | `MW_VAPID_CONTACT` (pin) | yes | VAPID contact (push_relay.rs) |
| <a id="push.quiethours"></a>`push.quietHours` | notifications | global | null | plain |  | yes | push quiet hours, UTC |
| <a id="updates.check"></a>`updates.check` | updates | global | "off" | plain |  | yes | daily update check |
| <a id="updates.feedurl"></a>`updates.feedUrl` | updates | global | "https://api.github.com/repos/supermarsx/mailwoman/releases" | url |  | yes | release feed |
| <a id="updates.apply"></a>`updates.apply` | updates | global | "disabled" | plain |  | yes | update apply through the host agent |
| <a id="backup.retentioncount"></a>`backup.retentionCount` | backups | global | 7 | plain |  | yes | SQLite backups kept |
| <a id="about.securitycontact"></a>`about.securityContact` | about | global | null | url |  | yes | security.txt Contact |
| <a id="about.securitypolicyurl"></a>`about.securityPolicyUrl` | about | global | null | url |  | yes | security.txt Policy |
| <a id="about.projectlinks"></a>`about.projectLinks` | about | global | [] | url |  | yes | About screen links |
| <a id="config.snapshotskept"></a>`config.snapshotsKept` | config | global | 20 | plain |  | yes | configuration snapshots kept |
| <a id="jobs.historydays"></a>`jobs.historyDays` | jobs | global | 30 | plain |  | yes | job run history |
| <a id="maintenance.announceminutesdefault"></a>`maintenance.announceMinutesDefault` | maintenance | global | 15 | plain |  | yes | maintenance announcement lead time |
| <a id="maintenance.defaultmessage"></a>`maintenance.defaultMessage` | maintenance | global | {} | plain |  | yes | maintenance banner text per locale |
| <a id="health.diskfreeminpercent"></a>`health.diskFreeMinPercent` | health | global | 5 | plain |  | yes | /readyz disk free floor |
| <a id="observability.otlpmetrics"></a>`observability.otlpMetrics` | observability | global | false | plain |  | yes | OTLP metrics export |
| <a id="metrics.drilldowntopn"></a>`metrics.drilldownTopN` | observability | global | 200 | plain |  | yes | metrics drill-down rows |
| <a id="metrics.defaultrangeseconds"></a>`metrics.defaultRangeSeconds` | observability | global | 3600 | plain |  | yes | default request history range |
| <a id="metrics.maxrangeseconds"></a>`metrics.maxRangeSeconds` | observability | global | 2592000 | plain |  | yes | maximum request history range |
| <a id="metrics.rollupdays"></a>`metrics.rollupDays` | observability | global | 30 | plain |  | yes | hourly metric rollups kept |
| <a id="diagnostics.retentiondays"></a>`diagnostics.retentionDays` | diagnostics | global | 30 | plain |  | yes | diagnostic runs kept |
| <a id="dataexport.linkhours"></a>`dataExport.linkHours` | users | global | 24 | plain |  | yes | data export download window |
| <a id="logs.auth.retentiondays"></a>`logs.auth.retentionDays` | observability | global | 7 | plain |  | yes | authentication history days kept |
| <a id="logs.auth.maxentries"></a>`logs.auth.maxEntries` | observability | global | 50000 | plain |  | yes | maximum authentication history entries |
| <a id="servertext."></a>`serverText.*` | serverText | global+domain | {} | plain |  | yes | server_text(id, locale, domain) overrides |
