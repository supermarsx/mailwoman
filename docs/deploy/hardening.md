# Web hardening

The server adds security headers to its responses, checks the origin of every request that changes something, and expires sessions on a timer. Most of this is always on. This page lists what is on by default, the environment variables that change it, and what to set for a production deployment. The console's **Security centre** > **Deployment checks** tab shows the values a running server uses (see [Deployment security](../admin/deployment-security.md)).

## Response headers

Always sent:

- `Content-Security-Policy` (`default-src 'none'`, scripts and styles only from the server itself, `frame-ancestors 'none'`), `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Cross-Origin-Opener-Policy: same-origin`;
- `Cross-Origin-Resource-Policy: same-origin`;
- `Permissions-Policy`, which turns off camera, microphone, geolocation, USB, payment and other browser features the webmail does not use.

Configurable:

| Header | Default | Variable |
|---|---|---|
| `Cross-Origin-Embedder-Policy: require-corp` | on | `MW_COEP` (or `--coep`); `false` turns it off |
| `Strict-Transport-Security` | `max-age=31536000`, sent only on requests that reached the server over HTTPS | `MW_HSTS=off` turns it off; `MW_HSTS_MAX_AGE`, `MW_HSTS_INCLUDE_SUBDOMAINS`, `MW_HSTS_PRELOAD` |

## Cross-site request checks

Every `POST`, `PUT`, `PATCH` and `DELETE` passes an origin check, which cannot be turned off (the exceptions are listed below):

- a `Sec-Fetch-Site` header that is `cross-site`, or that has any value other than `same-origin`, `same-site` or `none`, is refused;
- a request that repeats the `Host`, `Origin`, `Referer` or `Sec-Fetch-Site` header is refused;
- otherwise the `Origin` header, or else the `Referer` header, must name the same host as the request; a request with neither is allowed (native clients send neither), except that `Sec-Fetch-Site: same-site` without either is refused;
- a request without a `Host` (or HTTP/2 authority) is refused.

A refused request gets `403` `{"error":"cross-origin request rejected"}`. Session cookies are `HttpOnly` and `SameSite=Strict`.

On top of that, **strict CSRF** (`MW_CSRF_STRICT=true` or `--csrf-strict`) makes every write that a session cookie authorises repeat a token: the value of the `mw_csrf` cookie in an `X-CSRF-Token` header. The server's default is **off**; `deploy.sh` writes `MW_CSRF_STRICT=true` for a new installation, and the example `docker-compose.proxy.yml` sets it. With it on, a write without the right token gets `403` `{"error":"missing or invalid CSRF token"}`.

- The token is derived from the session with the server key, so a token from one session is not valid for another, and a mailbox token is not valid for the console. A new token is issued when a session starts or is rotated.
- The web app sends it on every write. A script that signs in with a cookie must read `mw_csrf` and send the header itself.
- Not asked for a token (they are still origin-checked): the sign-in routes of webmail and the console, which run before a session exists, and routes no cookie authorises, such as `/oauth/token`, `/oauth/revoke`, `/oauth/register`, `/api/webhooks/inbound` and `/mcp`.
- A request authorised only by an `Authorization: Bearer` header (the desktop and mobile apps, API keys, OAuth clients) skips both checks. If it also carries a session cookie, both checks apply.
- The single sign-on callback routes (`/api/sso/<id>/callback`, `/api/sso/<id>/acs`) skip both checks; they carry state the server bound to the sign-in when it started. The identity provider's `/oidc/token` and `/oidc/revoke` also skip both checks; no cookie authorises them.
- A desktop or Android app signing in (`/api/login`, `/api/login/2fa...`, `/api/discover`) with its app origin (or one listed in `MW_NATIVE_ORIGINS`) and no session cookie skips both checks.

## Sessions

| Variable | Default | Meaning |
|---|---|---|
| `MW_SESSION_IDLE_SECS` (`--session-idle-secs`) | `604800` (7 days) | A mailbox session ends after this long without use in the foreground. |
| `MW_SESSION_ABSOLUTE_SECS` (`--session-absolute-secs`) | `0` (no fixed end) | A nonzero value ends every mailbox session this long after sign-in, even while in use. |

A new session id is issued at every sign-in, and the web app can rotate it (`POST /api/session/rotate`) without moving the session's start time. Session times are stored in the database, so a restart does not extend a session. Console sessions have their own, shorter timeouts. See [Sessions](../admin/sessions.md).

## Cookies over HTTPS

Cookies get the `Secure` flag when the server knows the browser reached it over HTTPS: `MW_PUBLIC_URL` starts with `https://`, a trusted proxy (`MW_TRUSTED_PROXIES`) sends `X-Forwarded-Proto: https`, or the server terminates TLS itself ([ACME or its own certificate](./acme.md)). `MW_COOKIE_SECURE=true` forces the flag on and `false` forces it off, whatever the scheme.

## Recommended production settings

| Variable | Default | Production |
|---|---|---|
| `MW_PUBLIC_URL` | not set | the `https://` address users open |
| `MW_TRUSTED_PROXIES` | not set | the reverse proxy's address, so client addresses and the HTTPS scheme are known |
| `MW_CSRF_STRICT` | `false` | `true` (set by `deploy.sh` for a new installation) |
| `MW_COEP` | `true` | `true` |
| `MW_SESSION_IDLE_SECS` | `604800` | according to your policy |
| `MW_SESSION_ABSOLUTE_SECS` | `0` | a nonzero limit if your policy requires one |

If a proxy signs users in and passes their name in a header (`MW_HEADER_AUTH`), the proxy must remove that header from client requests; none of the example proxy configurations does this. See [Reverse proxy](./reverse-proxy.md#header-authentication).

## Related

- [Reverse proxy](./reverse-proxy.md)
- [HTTP protocol guard](../admin/protocol-guard.md)
- [Aggregate and adaptive protection](../admin/request-protection.md)
- [Sessions](../admin/sessions.md)
- [fail2ban](./fail2ban.md)
