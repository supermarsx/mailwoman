# Realtime push behind a reverse proxy (WebSocket + SSE)

V2 adds realtime JMAP push so the SPA reacts to server-side changes instead of
polling. Two same-origin, cookie-authenticated endpoints are served by
`mw-server`:

| Endpoint | Transport | Purpose |
|----------|-----------|---------|
| `/jmap/ws` | WebSocket (RFC 8887) | Primary push channel. Streams `StateChange` frames; the client then calls the relevant `*/changes` and refetches. |
| `/jmap/eventsource` | Server-Sent Events | Fallback when WebSocket is blocked. Same `StateChange` JSON as `data:` frames. |

The client ladders **WS → SSE → poll**, reconnecting with backoff. The server
sends a **30 s heartbeat** (WS ping / SSE keep-alive comment) so idle
connections stay open.

Because both are same-origin, the app's `connect-src 'self'` CSP already permits
them — no CSP change is needed. The reverse proxy, however, must be configured
for streaming.

## nginx

See [`nginx.conf`](./nginx.conf) for the full snippet. The essentials:

1. A `map` at `http{}` scope to translate the `Connection` header:

   ```nginx
   map $http_upgrade $connection_upgrade {
       default upgrade;
       ''      close;
   }
   ```

2. A `location /jmap/ws` that forwards the upgrade and uses a long read timeout:

   ```nginx
   location /jmap/ws {
       proxy_pass http://127.0.0.1:8080;
       proxy_http_version 1.1;
       proxy_set_header Upgrade    $http_upgrade;
       proxy_set_header Connection $connection_upgrade;
       proxy_read_timeout 3600s;
       proxy_send_timeout 3600s;
   }
   ```

3. A `location /jmap/eventsource` with **buffering disabled** (otherwise SSE
   events are batched and arrive late):

   ```nginx
   location /jmap/eventsource {
       proxy_pass http://127.0.0.1:8080;
       proxy_http_version 1.1;
       proxy_buffering off;
       proxy_cache off;
       proxy_read_timeout 3600s;
   }
   ```

## Traefik

Traefik proxies WebSockets automatically — no special middleware is needed for
`/jmap/ws`; just route the host to the Mailwoman service. For SSE, ensure any
compression/buffering middleware is not applied to `/jmap/eventsource`
(Traefik does not buffer responses by default, so the default is already
correct). Give the router/entrypoint a generous `respondingTimeouts.idleTimeout`
(e.g. `3600s`) so long-lived streams are not cut.

## Caddy

Caddy reverse-proxies WebSockets and SSE transparently with a plain
`reverse_proxy 127.0.0.1:8080`; `flush_interval -1` on the eventsource path
disables buffering if you have tuned it on elsewhere.

## Troubleshooting

- **Push never arrives, polling works** — the proxy is dropping the upgrade
  (WS) or buffering (SSE). Confirm the `Upgrade`/`Connection` headers reach the
  backend and that buffering is off for `/jmap/eventsource`.
- **Connections drop every ~60 s** — the proxy idle/read timeout is shorter than
  the 30 s heartbeat cadence; raise it (the snippets above use 3600 s).
- **Works locally, fails behind a CDN** — some CDNs buffer or block WS; the
  client falls back to SSE and then polling automatically, but for realtime you
  want WS or SSE to survive the edge. Disable edge buffering for `/jmap/*`.
