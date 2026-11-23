# Logging and monitoring

This page describes what the server writes to logs, traces and metrics, what it
removes before writing, and which parts send data off the server. Everything that
sends data elsewhere is off until an operator configures it.

## Server log

The server always logs to standard output. With
[`logging.file.enabled`](/admin/settings#logging.file.enabled) it also writes
rotating files under [`logging.file.directory`](/admin/settings#logging.file.directory);
each record in those files is encrypted and authenticated under the server key
(`MW_SERVER_KEY`), so the files cannot be read without it. Standard output is not encrypted; whatever
collects it (Docker, systemd, a log shipper) receives plain text.

The level is `MW_LOG` at start (default `info`), then
[`logging.level`](/admin/settings#logging.level) and
[`logging.componentLevels`](/admin/settings#logging.componentlevels). The
redaction below applies to standard output and to the files.

What is removed from log lines:

| What | Default | Setting |
|---|---|---|
| Bearer and Basic credentials, `password=`-style pairs, session cookies, JWT-shaped strings, `mwk_` API keys, URL user-info | removed | [`logging.redaction.tokens`](/admin/settings#logging.redaction.tokens) (`true`) |
| Email addresses | **kept in full** | [`logging.redaction.addresses`](/admin/settings#logging.redaction.addresses): `full`, `truncated` (first letter and domain), `hashed`, `none` (removed) |
| IP addresses | **kept in full** | [`logging.redaction.ips`](/admin/settings#logging.redaction.ips): `full`, `truncated` (IPv4 /24, IPv6 /48), `hashed`, `none` |

::: warning Addresses and IPs are logged by default
Sign-in records, fail2ban lines and many operational messages include the account
name (usually an email address) and the client address. Set the two redaction
settings above if your logs leave the server or are kept long.
:::

<!-- screenshot: id=security/observability-log-redaction; screen=Admin console > Settings > Observability; state=defaults; element=logging.redaction.addresses, logging.redaction.ips, logging.redaction.tokens -->

## Application log store

The console's **Logs** screen shows records the server process keeps in memory.
Before a record is kept, it passes a second, broader redaction:

- a field named like message content (`subject`, `body`, `snippet`, `html`,
  `preview`, `content`, …) keeps only its length;
- a field named like a credential (`password`, `token`, `secret`, `cookie`,
  `api_key`, `otp`, `code`, …) is replaced;
- the values of secret environment variables the process started with (names
  ending in `PASSWORD`, `_PW`, `SECRET`, `TOKEN`, `_KEY`, `API_KEY`, `DSN`) are cut
  out wherever they appear;
- the console path is shown as `[admin-path]`.

Reading the store needs the `logs.read` capability. See [Logs](/admin/logs).

### Message content in logs

Message bodies and subjects are not logged by the server's own code paths that
handle mail, and the redaction above removes fields named like message content.
This is a convention backed by a wrapper type (`Redacted`) and tests, not a proof:
a log statement that puts message text into a field with another name, or a
dependency that logs at `debug` or `trace`, can still write it. Keep production
levels at `info` or above.

## Metrics

`GET /metrics` serves Prometheus text format only with
`Authorization: Bearer <MW_METRICS_TOKEN>`, compared in constant time. Without a
token configured, every request gets `401`. The metrics describe counts, timings
and outcomes; message content is not used as a metric name or label.

## Traces and metrics export (OTLP)

With `MW_OTLP_ENDPOINT` (or telemetry configured in the console under
[Telemetry](/deploy/telemetry)), the server exports OpenTelemetry traces and
metrics to that collector over gRPC, using rustls for TLS. Service name:
`MW_OTEL_SERVICE` (default `mailwoman`). The console's telemetry settings sample
10 % of traces by default. Spans carry what the server's tracing records, which can
include account names and addresses; treat the collector as holding personal data.

## Browser error reports

The web app reports its own errors (section failures, uncaught errors, failed
chunk loads, Content-Security-Policy violations) to
`POST /api/observability/client-errors`. The browser scrubs each report, and the
server scrubs it again: query strings and fragments are removed from URLs,
addresses and long opaque tokens from messages, and every field is checked
against a short alphabet and length. No user id, address, IP, token, attachment or
file name is stored. Administrators read them under **Observability** → **Client
errors**.

| Setting | Default | Effect |
|---|---|---|
| [`observability.client.enabled`](/admin/settings#observability.client.enabled) | `true` | Collect reports. |
| [`observability.client.includeAccountHash`](/admin/settings#observability.client.includeaccounthash) | `false` | Add a keyed one-way hash of the account to signed-in reports. |
| [`observability.client.captureConsole`](/admin/settings#observability.client.captureconsole) | `false` | Also report `console.error` calls. |
| [`privacy.errorReports`](/admin/settings#privacy.errorreports) | `on` | Per user: send a report when a section fails to load. |

The older `POST /errors` endpoint also accepts a report, removes fields named like
mail content (`subject`, `body`, `from`, `to`, `email`, …) and anything that looks
like an address, and then forwards it to `MW_ERROR_FORWARD_URL` if set, or logs
only its top-level field names. The bundled web app does not send to it.

## Sentry or GlitchTip

`MW_SENTRY_DSN` configures a relay to a Sentry-compatible ingest endpoint (plain
HTTPS over rustls, no Sentry SDK). No server error path sends events through it,
so setting it has no effect beyond a start-up log line. Use the client error
reports, logs and OTLP instead.

## Audit log

The audit log is a separate record of who did what in the console and of security
events. See [Audit log](./admin-panel.md#audit-log).

## Related

- [Logs](/admin/logs)
- [Observability](/admin/observability)
- [Telemetry](/deploy/telemetry)
- [Admin console security](./admin-panel.md)
