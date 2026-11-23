# Mailwoman behind IIS with ARR

This page explains how to run Mailwoman behind IIS 10 with Application Request
Routing (ARR) 3.0 and URL Rewrite 2.1, using the `web.config` in this directory.

::: warning
This configuration is documented, not CI-tested. IIS has no cell in the
`proxy-conformance` workflow, and the `web.config` has not been run against a live
IIS server. Treat it as a starting point and work through the checklist below
before you rely on it.
:::

## Before you start

Several settings Mailwoman needs are server-level and cannot be set in a
`web.config`. With the file alone, the site loads but large uploads and event
streams fail. Run the commands below from an elevated prompt.

**Install the features.** ARR 3.0 and URL Rewrite 2.1 are separate downloads. The
WebSocket Protocol feature is part of IIS:

```
dism /online /enable-feature /featurename:IIS-WebSockets /all
```

**Turn on the ARR proxy.** It is installed disabled; without it the rewrite rule to
`http://127.0.0.1:8080/` returns `404` instead of proxying:

```
appcmd set config -section:system.webServer/proxy /enabled:"True" /commit:apphost
```

**Preserve the Host header.** ARR rewrites `Host` to the backend address by default.
Mailwoman compares `Origin` against `Host`, so sign-in fails with `403`:

```
appcmd set config -section:system.webServer/proxy /preserveHostHeader:"True" /commit:apphost
```

**Raise the ARR proxy timeout.** The default of 120 seconds closes idle WebSockets
and assistant replies that take longer than two minutes:

```
appcmd set config -section:system.webServer/proxy /timeout:"00:60:00" /commit:apphost
```

**Allow the rewrite rules to set server variables.** URL Rewrite refuses to set a
request header that is not on the server-level list, and every request then returns
`HTTP 500.50`:

```
appcmd set config -section:system.webServer/rewrite/allowedServerVariables /+"[name='HTTP_X_FORWARDED_FOR']"   /commit:apphost
appcmd set config -section:system.webServer/rewrite/allowedServerVariables /+"[name='HTTP_X_FORWARDED_PROTO']" /commit:apphost
appcmd set config -section:system.webServer/rewrite/allowedServerVariables /+"[name='HTTP_X_FORWARDED_HOST']"  /commit:apphost
appcmd set config -section:system.webServer/rewrite/allowedServerVariables /+"[name='HTTP_X_REAL_IP']"         /commit:apphost
appcmd set config -section:system.webServer/rewrite/allowedServerVariables /+"[name='HTTP_FORWARDED']"         /commit:apphost
appcmd set config -section:system.webServer/rewrite/allowedServerVariables /+"[name='RESPONSE_BUFFER_LIMIT']"  /commit:apphost
```

**Raise the request read-ahead** so a large upload is passed on as it arrives:

```
appcmd set config -section:system.webServer/serverRuntime /uploadReadAheadSize:"104857600" /commit:apphost
```

## Set up the site

1. Copy `web.config` into the site's root folder.
2. In the last rewrite rule, change `http://127.0.0.1:8080/` to the address Mailwoman
   listens on.
3. Tell Mailwoman to trust the forwarded headers from IIS. With IIS and Mailwoman on
   the same host:

   ```
   MW_TRUSTED_PROXIES=127.0.0.1/32
   MW_FORWARDED_MODE=xff
   ```

   If they are on different hosts, use the IIS host's address, and firewall
   Mailwoman's port so only IIS can reach it.
4. Restart Mailwoman and work through the checklist.

## What the web.config does

- Raises `maxAllowedContentLength` to 100 MB. The IIS default of 30,000,000 bytes is
  below Mailwoman's 50 MB upload limit.
- Enables WebSockets for `/jmap/ws`.
- Sets `X-Forwarded-For` and `X-Real-IP` to the connection address,
  `X-Forwarded-Host` to the `Host` header and `X-Forwarded-Proto` to `https` or
  `http`, and removes a client-supplied `Forwarded` header.
- Turns off response buffering (`RESPONSE_BUFFER_LIMIT` 0) for `/jmap/eventsource` and
  `/api/assist/invoke`.
- Removes `text/event-stream` from dynamic compression. That list is server-level,
  so check that the per-site override takes effect on your server.
- Passes Mailwoman's own error responses through (`existingResponse="PassThrough"`).

## Verification checklist

| # | Check | How | Expected |
|---|---|---|---|
| 1 | Site serves | `GET /` | The web client, `200` |
| 2 | Health | `GET /healthz` from the IIS host | `200` |
| 3 | Host kept | Sign in through `https://mail.example.org` | Sign-in succeeds; no `403` |
| 4 | Client address | Any request | Mailwoman's logs show the client's address, not the IIS host's |
| 5 | Forged header | A request with `X-Forwarded-For: 1.2.3.4` | Mailwoman still records the real client address |
| 6 | Event stream | `GET /jmap/eventsource` while signed in | Events arrive as they happen, not when the connection closes |
| 7 | Assistant stream | `POST /api/assist/invoke` with the assistant configured | Same as 6 |
| 8 | WebSocket | Connect to `/jmap/ws` while signed in | The upgrade succeeds |
| 9 | Upload | Attach a 20 MB file | Accepted |
| 10 | Oversized upload | Attach a 60 MB file | Mailwoman's JSON `413`, not an IIS error page |
| 11 | TLS | Over HTTPS | The session cookie carries `Secure`; responses carry `Strict-Transport-Security` |

Checks 6, 7 and 10 are the ones that fail when only `web.config` is applied and the
server-level steps are skipped.

## Not covered

- **Sub-path hosting** (`/mail`): the rules assume the site root. See [Sub-path
  hosting](../../reverse-proxy.md#sub-path-hosting).
- **PROXY protocol**: IIS has no equivalent. For a TCP pass-through balancer, see the
  `haproxy-l4` example.
- **HTTP/3**: IIS can accept HTTP/3 from browsers and speak HTTP/1.1 to Mailwoman;
  nothing in this file changes for it.

## Related

- [Running behind a reverse proxy](../../reverse-proxy.md)
- [Example proxy configurations](../README.md)
