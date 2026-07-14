# Observability & logging (V6)

Mailwoman's observability is **opt-in and privacy-preserving**: nothing is exported
unless you configure an endpoint, the metrics surface is never open, and no mail body,
subject, or address is ever written to a log or a trace. OTLP + Prometheus are the
committed paths; Sentry is off by default and not linked.

## Configuration

| Env | Default | Meaning |
|---|---|---|
| `MW_LOG` | `info` | Per-subsystem `tracing` directives (e.g. `mw_engine=debug,info`). Hot-reloadable on `SIGHUP` — re-read at runtime without a restart. |
| `MW_OTLP_ENDPOINT` | *(unset)* | OTLP collector, e.g. `http://otel:4317`. Unset → OTLP export disabled. Traces + metrics ship over `tonic` with **rustls** (no OpenSSL). |
| `MW_OTEL_SERVICE` | `mailwoman` | Service name reported to the collector. |
| `MW_METRICS_TOKEN` | *(unset)* | Bearer token guarding `GET /metrics`. **Unset → `/metrics` is unreachable** (metrics are never exposed unauthenticated). |
| `MW_ERROR_FORWARD_URL` | *(unset)* | Where the server POSTs a **scrubbed** browser-error report. Unset → reports are dropped after scrubbing. |

## Traces & metrics (OTLP)

When `MW_OTLP_ENDPOINT` is set, the server exports OpenTelemetry traces and metrics to
your collector. The transport is verified free of OpenSSL/native-TLS — it uses the
rustls `tls-roots` path, consistent with the rest of the tree's TLS floor
(`deny.toml`).

## Prometheus `/metrics` (auth-gated)

`GET /metrics` emits the Prometheus text exposition format, but **only** with a valid
`Authorization: Bearer <MW_METRICS_TOKEN>`. If `MW_METRICS_TOKEN` is unset the endpoint
does not serve — there is no unauthenticated metrics scrape. Point Prometheus at it with
a bearer-token scrape config.

## The `/errors` scrubber tunnel

Browser-side error reports are POSTed to `/errors`, scrubbed **server-side**, and only
then optionally forwarded to `MW_ERROR_FORWARD_URL`. Scrubbing strips mail content and
addresses before anything leaves the server, which is why the browser's CSP can stay
`connect-src 'self'` — the browser never talks to a third-party error service directly.

## The rule: no mail content in telemetry

Mail bodies, subjects, and addresses **never** enter a log line, a span, or a metric
label. This is enforced with a typed redaction wrapper (`Redacted`) rather than review
discipline — the fields that carry mail content cannot be formatted into a log by
accident. Combined with server-side scrubbing on `/errors`, telemetry is safe to ship
to a shared collector without leaking user data.

## Audit log

Operator actions are recorded in the append-only **audit log** (see
[`admin-panel.md`](./admin-panel.md)), viewable and exportable from the admin panel and
`mailwoman admin`. It is a security record of *who did what*, separate from application
tracing.

## Sentry

Sentry is **off by default and not linked** into the build. OTLP + Prometheus are the
committed observability paths; Sentry would only be enabled after its dependency tree is
vetted for a rustls transport with no OpenSSL (recorded in `deny.toml` as
vet-before-enable). Do not assume Sentry delivery in 26.7.
