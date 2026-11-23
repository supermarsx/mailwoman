# Example reverse-proxy configurations

This directory holds a working configuration for each reverse proxy Mailwoman is
tested behind, and the TLS material generator they share. The same files are booted
by `docker-compose.proxy.yml` at the repository root, which the `proxy-conformance`
workflow uses to test them. For what the server needs from a proxy, and the
environment variables to set, read [Running behind a reverse
proxy](../reverse-proxy.md) first.

## Files

```
nginx/       nginx.conf (complete, for the container) + mailwoman.conf (drop-in vhost)
apache/      httpd.conf (complete, for the container) + mailwoman.conf (drop-in vhosts)
caddy/       Caddyfile
haproxy-l7/  haproxy.cfg: mode http, TLS terminated by HAProxy
haproxy-l4/  haproxy.cfg: mode tcp, send-proxy-v2, TLS passed through to Mailwoman
traefik/     traefik.yml (static) + dynamic.yml (routers, service, TLS)
envoy/       envoy.yaml
iis/         web.config + README.md with the server-level steps; not tested
tls/         gen-certs.sh; certs/ is generated and not committed
```

Each file uses the test ports (`8080` plain, `8443` TLS), the upstream name `app`
from the compose file, and a self-signed certificate from `tls/gen-certs.sh`. For a
real deployment, change the listen ports to 80/443, point the upstream at your
Mailwoman address, and use a CA-issued certificate or the proxy's own ACME support.

## What every configuration does

All of these except `haproxy-l4`, which does not parse HTTP:

- **Overwrites `X-Forwarded-For`** with the address the proxy saw, so Mailwoman gets
  one entry the client could not influence. Envoy is the exception: it appends, and
  Mailwoman's right-to-left walk takes the entry Envoy added. Each file shows the
  append form to use in a proxy chain; that form is safe only when every hop is
  listed in `MW_TRUSTED_PROXIES`.
- **Sets `X-Forwarded-Proto`** to the scheme the client used. On the TLS listener this
  is what gives the session cookie its `Secure` flag and turns on HSTS.
- **Removes a client-supplied `Forwarded` header** (all except Envoy), because none of
  them use RFC 7239.
- **Keeps `Host`** as the browser sent it, port included. Mailwoman compares `Origin`
  against `Host` and derives the passkey origin from the host.
- **Passes WebSocket upgrades** on `/jmap/ws`, with an idle timeout of an hour or
  none at all.
- **Streams** `/jmap/eventsource` and `/api/assist/invoke` without buffering or
  compression, with a read timeout of an hour or none at all.
- **Accepts request bodies of at least 100 MB**, above Mailwoman's own limits, so an
  oversized upload is refused by Mailwoman with its JSON `413`.
- **Leaves HSTS to Mailwoman**, which sends it on HTTPS responses.

None of them strips `X-Remote-User`. Add that yourself before turning on header
authentication; see [Header authentication](../reverse-proxy.md#header-authentication).

## Notes for each proxy

**nginx.** Three stock defaults are corrected: `proxy_buffering` (off for the stream
locations), `client_max_body_size` (raised from 1 MB) and `proxy_http_version`
(1.1, needed for WebSocket). The `map $http_upgrade $connection_upgrade` block must
sit at `http{}` scope; copy it from `nginx.conf` if you use only the vhost. The vhost
sends `$mw_host` as `Host`, a `map` that rebuilds the host for HTTP/3 requests, which
carry no `Host` header. The TLS server also listens for HTTP/3 (QUIC). The location
`/` sets `Connection ""` for upstream keepalive, so `/jmap/actions` is not upgraded
and the web client sends those requests over HTTP instead.

**Apache httpd.** Needs `mod_proxy`, `mod_proxy_http`, `mod_proxy_wstunnel`,
`mod_headers`, and `mod_env` for `SetEnv`. `ProxyPreserveHost On` is required.
`mod_proxy` appends to `X-Forwarded-For`, so the file unsets the client's copy and
lets `mod_proxy` add one entry. Streaming paths get their own `ProxyPass` with
`timeout=3600`, because `ProxyTimeout` cannot be set per location. Upload and import
paths use `proxy-nokeepalive`, so that an early answer from Mailwoman (a `401` before
the body is read) reaches the client instead of an Apache `500`. `mod_deflate` and
`mod_cache` are not loaded; if your server loads them, keep them off the stream
paths. Apache ignores `X-Accel-Buffering`. `/jmap/actions` goes through the
catch-all `ProxyPass /`, which has no `upgrade=websocket`, so the web client sends
those requests over HTTP.

**Caddy.** Passes WebSocket upgrades without a directive, does not buffer responses,
has no body limit until one is set, and serves HTTP/3 on TLS sites. The file sets the
forwarded headers, a 100 MB body limit and the timeouts. For a real site, replace the
two site blocks with one named after your host and remove `auto_https off`; Caddy then
obtains the certificate itself.

**HAProxy, HTTP mode (`haproxy-l7`).** `timeout tunnel 3600s` applies once a
WebSocket is upgraded; `timeout client` and `timeout server` stop applying. The file
deletes the client's forwarded headers and sets its own from `%[src]` instead of
using `option forwardfor`, which appends. The stream paths use a backend with a
one-hour server timeout.

**HAProxy, TCP mode (`haproxy-l4`).** `mode tcp` with `send-proxy-v2`: HAProxy does
not terminate TLS, and the client address travels in a PROXY protocol header. Run
Mailwoman with its own TLS (`MW_TLS_CERT`/`MW_TLS_KEY` or `MW_ACME`),
`MW_PROXY_PROTOCOL=require`, and the balancer's address in `MW_TRUSTED_PROXIES`.
`check-send-proxy` is required, because Mailwoman drops a health check that arrives
without the header.

**Traefik.** Entry points and their timeouts are in the static file
(`traefik.yml`); routers and middlewares are in the dynamic file (`dynamic.yml`). A
timeout in the wrong file is ignored. `forwardedHeaders.trustedIPs` is left unset, so
Traefik discards client-supplied `X-Forwarded-*` headers and writes its own. A
middleware removes `Forwarded` and `X-Real-IP`. The `websecure` entry point enables
HTTP/3.

**Envoy.** `use_remote_address: true` with `xff_num_trusted_hops: 0` appends the
client address to `X-Forwarded-For`. `stream_idle_timeout: 0s` and `timeout: 0s` on
the stream and WebSocket routes keep long-lived connections open; Envoy's defaults
would close them.

**IIS.** Several settings live at server level and cannot be set in `web.config`.
Follow [iis/README.md](iis/README.md).

## Test harness

`docker-compose.proxy.yml` has one profile per proxy. Each profile starts the JMAP
mock, a Mailwoman server and one proxy on fixed host ports:

| Profile | HTTP | HTTPS |
|---|---|---|
| `nginx` | 8601 | 8641 |
| `apache` | 8602 | 8642 |
| `caddy` | 8603 | 8643 |
| `haproxy-l7` | 8604 | 8644 |
| `haproxy-l4` | none | 8645 |
| `traefik` | 8605 | 8646 |
| `envoy` | 8606 | 8647 |
| Mailwoman without a proxy | 8607 | none |

`nginx`, `caddy` and `traefik` also publish their HTTPS port over UDP for HTTP/3. The
CI workflow and the test suite address the cells by these ports.

The network is pinned so that the trusted range is narrower than the network:

```
172.28.10.0/24   proxy containers   <- MW_TRUSTED_PROXIES
172.28.20.0/24   Mailwoman and the JMAP mock
172.28.0.1       bridge gateway     <- host traffic arrives from here; not trusted
```

A request from the host reaches the proxy from `172.28.0.1`, so that is the client
address the tests expect. The server runs with `MW_FORWARDED_MODE=xff` and
`MW_CSRF_STRICT=true`.

To run one cell:

```sh
# Once: generate the self-signed TLS pair the HTTPS listeners need.
sh docs/deploy/proxy/tls/gen-certs.sh

# Start the cell, then wait for its health checks. Two commands, because the
# haproxy-l4 profile has a service with no health check, which some Compose
# versions refuse in the --wait set.
docker compose -f docker-compose.proxy.yml --profile nginx up -d
docker compose -f docker-compose.proxy.yml --profile nginx up -d \
  --no-deps --wait --wait-timeout 300 mock app nginx

MW_T20_PROXY=1 \
MW_T20_PROXY_BASE=http://127.0.0.1:8601 \
MW_T20_PROXY_KIND=nginx \
  cargo test -p mw-server --test t20_proxy_conformance -- --nocapture --test-threads=1

# Always with -v: the app volume holds a seeded database.
docker compose -f docker-compose.proxy.yml --profile nginx down -v
```

For HTTP/3, `scripts/proxy-h3-check.sh` drives a booted `caddy`, `nginx` or `traefik`
cell with a curl that speaks HTTP/3, from `172.28.30.5` on the compose network:

```sh
bash scripts/proxy-h3-check.sh caddy --published 8643
```

It checks for HTTP/2 with an `Alt-Svc: h3=` header over TCP and for HTTP/3 over UDP,
and, over both, that Mailwoman sees the right host (sign-in with the proxy's
`Origin` passes and a foreign one is refused), scheme (`Secure` cookie and HSTS) and
client address (an API key allowed for `172.28.30.5` answers `200`, one allowed for
another address `403`). `--published PORT` adds a request from the host to the
published UDP port.

Which cells the CI workflow runs, and which are blocking, is listed under [Tested
configurations](../reverse-proxy.md#tested-configurations).
