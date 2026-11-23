# Realtime push behind a reverse proxy

The web client learns about new mail and other changes through a long-lived
connection to the server. This page lists those connections and what a reverse proxy
must allow for them to stay open. For the rest of the proxy setup, see [Running
behind a reverse proxy](reverse-proxy.md).

## The endpoints

| Path | Transport | Purpose |
|---|---|---|
| `/jmap/ws` | WebSocket (RFC 8887) | Push channel. Sends `StateChange` messages; the client then fetches what changed. Negotiates the `jmap` subprotocol when the client offers it. |
| `/jmap/eventsource` | Server-sent events | Push fallback when the WebSocket fails. Sends the same messages as `data:` events. |
| `/api/assist/invoke` | `POST` answered with server-sent events | Streams AI assistant replies. |
| `/jmap/actions` | WebSocket | Carries the web client's API requests. If it cannot connect, the client sends the same requests over ordinary HTTP. The admin console uses `<console path>/actions` the same way. |

All of them are same-origin and authenticated by the session cookie, so the
application's Content Security Policy needs no change.

## How the client connects

The web client tries `/jmap/ws` first. It retries a failed transport once, after
about half a second, then moves on: to `/jmap/eventsource`, and from there to
polling. From polling it tries the push
transports again after 30, 60 and 120 seconds, then stays on polling until the user
reconnects or signs in again.

On an idle push connection the server sends a heartbeat every 30 seconds: a
`{"@type":"ping"}` message on both transports, plus a WebSocket ping frame on
`/jmap/ws`. Every 30 seconds it also checks that the session is still valid, and
closes the stream (WebSocket close code `1008`) when the session was signed out,
revoked or expired. `MW_REALTIME_RECHECK_SECS` changes that interval.

The event-stream responses carry `X-Accel-Buffering: no` and `Cache-Control:
no-cache, no-transform`. nginx reads the first and turns off buffering for that
response; other proxies need buffering turned off in their configuration.

## What the proxy must do

- Forward the WebSocket upgrade (`Upgrade` and `Connection` headers) on `/jmap/ws`
  and `/jmap/actions`.
- Turn off response buffering and compression on `/jmap/eventsource` and
  `/api/assist/invoke`.
- Keep idle connections open for well over 30 seconds. The example configurations
  use one hour, or no timeout.

The complete files are in [`proxy/`](proxy/README.md). The relevant parts:

### nginx

A `map` at `http{}` scope translates the `Connection` header:

```nginx
map $http_upgrade $connection_upgrade {
    default upgrade;
    ''      close;
}
```

The WebSocket location forwards the upgrade and uses long timeouts:

```nginx
location /jmap/ws {
    proxy_pass http://mailwoman;
    proxy_http_version 1.1;
    proxy_set_header Host       $mw_host;
    proxy_set_header Upgrade    $http_upgrade;
    proxy_set_header Connection $connection_upgrade;
    proxy_read_timeout 3600s;
    proxy_send_timeout 3600s;
}
```

The event-stream location turns buffering and compression off:

```nginx
location /jmap/eventsource {
    proxy_pass http://mailwoman;
    proxy_http_version 1.1;
    proxy_set_header Host            $mw_host;
    proxy_set_header Connection      "";
    proxy_set_header Accept-Encoding "";
    proxy_buffering off;
    proxy_cache off;
    gzip off;
    proxy_read_timeout 3600s;
}
```

The example adds the forwarded headers to each location, and a third location for
`/api/assist/invoke` with `proxy_request_buffering off` as well. `/jmap/actions` is
not given its own location there, so it uses HTTP; copy the `/jmap/ws` block for it
if you want it carried over WebSocket.

### Caddy

`reverse_proxy` passes WebSocket upgrades without a directive. The example sets
`flush_interval -1` and no read or write timeout for the two stream paths, and an
`idle 1h` server timeout so idle WebSockets stay open.

### Traefik

Traefik passes WebSocket upgrades without configuration and does not buffer
responses unless a buffering middleware is added. The timeouts are set on the entry
points in the static configuration: `readTimeout: 0s`, `writeTimeout: 0s` and
`idleTimeout: 1h`.

### HAProxy

In HTTP mode, `timeout tunnel 3600s` governs a WebSocket once it is upgraded. The
example sends the two stream paths to a backend with `timeout server 3600s`.

### Apache httpd

`mod_proxy_wstunnel` handles the upgrade:
`ProxyPass /jmap/ws ws://app:8080/jmap/ws upgrade=websocket timeout=3600`. The stream
paths get their own `ProxyPass` with `timeout=3600` and `SetEnv no-gzip 1`.

## Troubleshooting

**New mail shows up late or only after a refresh; the WebSocket never opens.** The
proxy is not forwarding the upgrade on `/jmap/ws`. Check that the
`Upgrade` and `Connection` headers reach the server; in nginx the location needs
`proxy_http_version 1.1` and the `$connection_upgrade` map.

**Events arrive in batches or only when the connection closes.** The proxy buffers
or compresses `/jmap/eventsource`. Turn buffering and compression off for that path
(and for `/api/assist/invoke`).

**Push connections drop and reconnect at a fixed interval.** The proxy's idle or read
timeout is shorter than the gap between messages. Raise it well above 30 seconds;
the examples use one hour.

**Push stops working behind a CDN.** Some CDNs buffer event streams or block
WebSockets. The client falls back to polling, so mail still arrives. Turn off edge
buffering for `/jmap/*`, or allow WebSockets at the edge.

## Related

- [Running behind a reverse proxy](reverse-proxy.md)
- [Example proxy configurations](proxy/README.md)
- [Connection limits](connection-limits.md)
