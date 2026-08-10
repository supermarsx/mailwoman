# Mailwoman behind IIS + ARR

## Status: documented, not CI-tested, not yet verified on a live host

Two separate things are true and both matter:

1. **IIS is not in the proxy-conformance CI matrix and cannot be.** Windows
   Server Core containers are multi-GB, and Windows-container jobs on hosted
   runners are slow and flaky enough that a red cell would carry no information.
   Every other proxy in `docs/deploy/proxy/` that is called supported has a CI
   cell that boots it and runs the conformance suite against it. IIS does not.

2. **The shipped `web.config` has not been run against a real IIS install
   either.** It was written against the IIS/ARR/URL-Rewrite documentation and
   against the prior art already in this repository (`docs/deploy/kerberos.md`
   covers IIS + ARR for the EWS-SPNEGO case), and it is reviewable — but nobody
   has yet executed the checklist below on IIS 10 with ARR 3.0. Until somebody
   does, treat it as a starting point.

Do not write "supports IIS" anywhere on the strength of this directory. The
accurate phrasing, and the one `docs/deploy/reverse-proxy.md` uses, is
"documented, not CI-tested".

## What must be configured outside `web.config`

This is the part that catches people. Several of the settings Mailwoman needs
are server-level and a `web.config` cannot set them. Dropping in the file alone
produces a site that appears to work and then fails on exactly the two things
this application does that a plain website does not: large uploads and long
-lived streams.

Run these from an elevated prompt, replacing `Mailwoman` with your site name.

**Install the features.** ARR 3.0 and URL Rewrite 2.1 are separate downloads
(Web Platform Installer or the standalone MSIs). The WebSocket Protocol feature
is part of IIS itself:

```
dism /online /enable-feature /featurename:IIS-WebSockets /all
```

**Turn on the ARR proxy.** It is installed disabled; without this the rewrite
rule that points at `http://127.0.0.1:8080/` will 404 rather than proxy:

```
appcmd set config -section:system.webServer/proxy /enabled:"True" /commit:apphost
```

**Preserve the Host header.** ARR rewrites `Host` to the backend address by
default. Mailwoman derives the OAuth dynamic-client-registration issuer URL and
the WebAuthn RP ID from `Host`, and compares `Origin` against it for CSRF — so
with the default, registered passkeys stop validating and the failure looks like
a client bug:

```
appcmd set config -section:system.webServer/proxy /preserveHostHeader:"True" /commit:apphost
```

**Raise the ARR proxy timeout.** The default is 120 seconds, which cuts an idle
WebSocket and any assistant turn that takes longer than two minutes:

```
appcmd set config -section:system.webServer/proxy /timeout:"00:60:00" /commit:apphost
```

**Allow the rewrite rules to set server variables.** URL Rewrite refuses to set
request headers unless the variable is allow-listed at server level. If you skip
this, every request returns `HTTP 500.50` and the rules never run:

```
appcmd set config -section:system.webServer/rewrite/allowedServerVariables /+"[name='HTTP_X_FORWARDED_FOR']"   /commit:apphost
appcmd set config -section:system.webServer/rewrite/allowedServerVariables /+"[name='HTTP_X_FORWARDED_PROTO']" /commit:apphost
appcmd set config -section:system.webServer/rewrite/allowedServerVariables /+"[name='HTTP_X_FORWARDED_HOST']"  /commit:apphost
appcmd set config -section:system.webServer/rewrite/allowedServerVariables /+"[name='HTTP_X_REAL_IP']"         /commit:apphost
appcmd set config -section:system.webServer/rewrite/allowedServerVariables /+"[name='HTTP_FORWARDED']"         /commit:apphost
appcmd set config -section:system.webServer/rewrite/allowedServerVariables /+"[name='RESPONSE_BUFFER_LIMIT']"  /commit:apphost
```

**Raise the request read-ahead** so a 50 MB upload is not spooled in 48 KB
chunks against a backend that is streaming it anyway:

```
appcmd set config -section:system.webServer/serverRuntime /uploadReadAheadSize:"104857600" /commit:apphost
```

## Application-side configuration

IIS is the front end, so tell the application which addresses to trust. Without
`MW_TRUSTED_PROXIES` the forwarded headers this config sets are ignored by
design — the application defaults to ignoring them, because trusting them from
anyone is worse than not having them at all.

```
MW_TRUSTED_PROXIES=127.0.0.1/32
MW_FORWARDED_MODE=xff
```

If IIS and the application are on different hosts, use the IIS host's address
and firewall the application's port so only IIS can reach it. A forwarded header
is only as trustworthy as the network path that carries it.

## Verification checklist

Run this on a real IIS 10 + ARR 3.0 host and record the result. Until each line
has been observed, the corresponding claim is not earned.

| # | Check | How | Expected |
|---|---|---|---|
| 1 | Site serves | `GET /` | the SPA shell, HTTP 200 |
| 2 | Health | `GET /healthz` | 200 |
| 3 | Host preserved | `GET /jmap/session` with `Host: mail.example.org` | session document, and the app's log shows that Host, not `127.0.0.1:8080` |
| 4 | Client IP | any request | the app's audit record shows the real client address, not the IIS host's |
| 5 | Spoof resistance | request with `X-Forwarded-For: 1.2.3.4` | the app still records the real client address |
| 6 | SSE (EventSource) | `GET /jmap/eventsource` | first event within ~2 s, not on connection close |
| 7 | SSE (assistant) | `POST /api/assist/invoke` | same; this is the one ARR buffering usually still breaks |
| 8 | WebSocket | connect `/jmap/ws` | upgrade succeeds, ping/pong round trips |
| 9 | Upload | `POST /jmap/upload/*` with 20 MB | 200 |
| 10 | Oversized upload | same with 60 MB | the **app's JSON** 413, not an IIS HTML error page |
| 11 | TLS posture | over HTTPS | session cookie carries `Secure`, response carries `Strict-Transport-Security` |

Checks 6, 7 and 10 are the ones that fail when only `web.config` was applied and
the server-level steps above were skipped.

## Known gaps

- **Sub-path hosting** (`/mail`) is not covered here. It has its own status
  across the whole application; see `docs/deploy/reverse-proxy.md`.
- **PROXY protocol** has no IIS equivalent. If you need the L4/TLS-passthrough
  shape, that is the `haproxy-l4` configuration, not this one.
- **HTTP/3** is a front-end concern. IIS terminating h3 and speaking HTTP/1.1 to
  the application is fine and needs nothing here.
