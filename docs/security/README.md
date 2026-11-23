# Security model

This section describes how Mailwoman protects mail and accounts, and where that
protection ends. Each page states what the code does, the defaults, and the
threats it does and does not address.

## The model in brief

Mailwoman is a server that signs in to mail servers on its users' behalf and
serves a web app to their browsers. That shapes everything else:

- **The server sees mail in clear text.** It fetches, sanitizes, indexes and caches
  mail, and it stores each account's mail-server password sealed with its own key
  so it can keep signing in. Anyone who controls the running server process can
  read what it reads. [Zero-access storage](./zero-access.md) reduces what is kept
  at rest for an account; it does not change this.
- **The server also serves the code that runs in the browser.** Browser-side
  protections (OpenPGP decryption, passphrase handling, view modes) trust that
  code. An operator who changes it can defeat them.
- **End-to-end encryption is the protection that does not depend on the
  server.** OpenPGP message bodies are decrypted in the browser with a key unlocked
  by the user's passphrase. See [OpenPGP and S/MIME](./crypto.md) for what the
  server still stores and can attempt.

## Who can do what

| Party | Can | Cannot (by default) |
|---|---|---|
| Someone with a copy of the database file | Read everything not sealed if SQLCipher is off; read sealed values too if the server key file was copied with it (the default location is next to the database). Use any unexpired session id: mailbox session ids are stored as-is in the `sessions` table. | Read the file if SQLCipher is on and the database key is stored elsewhere. Read sealed columns without the server key. Decrypt OpenPGP mail without the user's passphrase. See [Encryption at rest](./encryption-at-rest.md). |
| The operator of the running server | Read mail, sealed credentials and keys held in memory; change the web app. | Read OpenPGP-encrypted mail without the user's passphrase unless they change the web app to capture it. |
| A network attacker between browser and server | Little, when HTTPS is used throughout (see below). | Read session cookies sent over HTTPS: cookies carry `Secure` when the request arrived over HTTPS. |
| The sender of a malicious email | Get HTML rendered in a sandboxed frame; learn that a message was opened if remote images load under your [remote-image policy](./egress.md#remote-images-in-messages); make the server fetch their WKD key when a user decrypts their message without having their key. | Run script in the app's origin, read cookies or storage, submit forms. See [Message view modes](./max-security.md). |
| A holder of a stolen API key or OAuth token | Act within the key's scopes until it expires or is revoked. | Exceed its scopes. See [API keys and OAuth](./api-keys-oauth.md). |

## Transport

- Mailwoman normally runs behind a reverse proxy that terminates TLS, or terminates
  TLS itself with ACME (`MW_ACME`). See [Reverse proxy](/deploy/reverse-proxy) and
  [ACME](/deploy/acme).
- Cookies get the `Secure` attribute when the server sees the request as HTTPS
  (from `MW_PUBLIC_URL` or trusted proxy headers). `MW_COOKIE_SECURE` overrides this
  in either direction.
- Over HTTPS the server sends `Strict-Transport-Security: max-age=31536000` by
  default. `MW_HSTS=off` disables it; `MW_HSTS_MAX_AGE`,
  `MW_HSTS_INCLUDE_SUBDOMAINS` and `MW_HSTS_PRELOAD` change it.

## Mailbox sessions

- Signing in creates a session id of 32 random bytes (hex-encoded). It is sent as an
  `HttpOnly; SameSite=Strict; Path=/` cookie and stored in the `sessions` table
  together with the account's mail credentials, sealed with the server key.
- Cross-site request forgery is blocked by the `SameSite=Strict` cookie and by an
  origin check on every cookie-authenticated `POST`, `PUT`, `PATCH` and `DELETE`:
  the server refuses a request whose `Sec-Fetch-Site` is `cross-site`, or whose
  `Origin` or `Referer` names a different host. A request with neither header (a
  non-browser client) passes this check. Setting `MW_CSRF_STRICT=true` adds a
  double-submit token check (the readable `mw_csrf` cookie repeated in the
  `X-CSRF-Token` header); it is off by default.
- Sessions end after a period of inactivity (`session.idleSecs`, default 604800
  seconds, 7 days). A fixed maximum lifetime (`session.absoluteSecs`) is off by
  default (`0`). Disabling an account deletes its sessions on their next use.
- Sign-in protection (rate limits, lockouts, two-factor, passkeys, SSO) is covered
  in [Sign-in locks](/admin/signin-locks), [Sessions](/admin/sessions),
  [Passkeys](./passkeys.md), [SSO](/admin/sso) and
  [Request protection](/admin/request-protection).

The admin console has its own sessions and cookies; see
[Admin console security](./admin-panel.md).

## The web app

- The app shell is served with this Content Security Policy:
  `default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self';
  img-src 'self' blob: data:; font-src 'self'; manifest-src 'self';
  connect-src 'self' blob:; frame-src 'self'; worker-src 'self' blob:;
  base-uri 'none'; form-action 'none'; require-trusted-types-for 'script'`.
  Enabling a browser analytics tag adds that provider's origins.
- The app shell is sent with `X-Frame-Options: DENY`.
- Message HTML is sanitized on the server (`mw-sanitize`, built on ammonia).
  Decrypted OpenPGP HTML never goes back to the server: the same sanitizer,
  compiled to WebAssembly, runs in the browser's crypto worker. The browser makes
  links inert until they are allowed, and renders the result in sandboxed frames
  without `allow-same-origin`. See
  [Message view modes and the reading floor](./max-security.md).
- Sender authentication results (DKIM, SPF, DMARC, ARC) are computed on the server
  with the `mail-auth` library.

## Pages in this section

**Data at rest**

- [Encryption at rest](./encryption-at-rest.md): SQLCipher whole-file encryption,
  sealed columns, where the keys live.
- [Zero-access storage](./zero-access.md): what the per-account mode removes from
  the server, and what it does not.
- [Personal settings and verified identities](./personal-settings-authority.md)
  and [Browser storage authority](./browser-storage-authority.md): how per-person
  data is kept apart.

**Mail content**

- [OpenPGP and S/MIME](./crypto.md): key handling, decryption in the browser, WKD.
- [Message view modes and the reading floor](./max-security.md).
- [PDF attachment preview](./pdf-preview.md).
- [Data-loss prevention](./dlp.md): outbound rules.
- [Screen-capture protection](./screen-capture.md).

**Accounts and access**

- [Admin console security](./admin-panel.md) and
  [Administrator authenticator app](./admin-totp.md).
- [Passkeys](./passkeys.md).
- [Password change](./password-change.md).
- [API keys and OAuth](./api-keys-oauth.md).
- [MCP server](./mcp.md).

**Outbound connections and extensions**

- [Outbound connections (egress)](./egress.md): the address policy behind every
  server-side fetch, operator proxies, remote images.
- [Engine plugins](./plugins.md): trust, capabilities and limits.
- [AI assistant: data flows and controls](../assist.md).

**Operations**

- [Logging and monitoring](./observability.md): what is logged and redacted,
  metrics, error reports, the audit log.

Material prepared for an external security review is in
[audit-prep](./audit-prep/README.md). It describes the code at the time it was
written; where it disagrees with these pages, these pages are kept current.
