# Upgrade note: forwarded headers and public address (26.19)

Applies to every deployment behind a reverse proxy or load balancer.

## What changes

- **Forwarded headers are trusted only from listed proxies.** The server believes forwarded headers only when the connecting peer is in `MW_TRUSTED_PROXIES`. From such a peer, `X-Forwarded-Proto` and `X-Forwarded-Host` are read whatever `MW_FORWARDED_MODE` says. The client address is read only when `MW_FORWARDED_MODE` also names its header (`xff` for `X-Forwarded-For`, `forwarded` for `Forwarded`). Without `MW_TRUSTED_PROXIES`, the server does not see that the original request was HTTPS; without both variables, every client address it records (audit log, per-key IP allowlists, rate limits, the ban list) is the proxy's address. Nothing fails loudly.
- **`MW_PUBLIC_URL`** sets the canonical external address (`https://mail.example.com`) and takes precedence over the request's headers. It also sets the WebAuthn origin: setting an `https` value on a deployment whose passkeys were registered under another origin makes those passkeys stop working, and their users register them again.
- **`MW_BASE_PATH`** serves the application under a path prefix (`/mail`). Releases before 26.19 accepted the variable but did nothing with it.
- **`MW_PROXY_PROTOCOL`** (`off`, `accept`, `require`) accepts the PROXY protocol header from an L4 load balancer.

## What to do

1. Set `MW_TRUSTED_PROXIES` to your proxy's address as the server sees it (an address or CIDR, comma-separated) and `MW_FORWARDED_MODE` to the header your proxy sends. With `deploy.sh --proxy-network`, the script sets both unless `app.env` already does.
2. Before setting an `https` `MW_PUBLIC_URL` on a server that already has passkeys, tell your users they will need to register their passkeys again.
3. See [Reverse proxy](../reverse-proxy.md) for the full configuration.

## Example proxy configurations (26.20)

- The nginx example (`docs/deploy/proxy/nginx/mailwoman.conf`) passes `Host $mw_host` instead of `$http_host`, which fixes `403` on sign-in over HTTP/3. If you copied the earlier server block, update it.
- With `deploy.sh --domain`, the Caddy service could fail to start with "cannot create config … in read-only service caddy". The Caddyfile is now passed as an environment value; run `bash deploy.sh deploy` again on an affected installation.
