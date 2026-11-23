# Operational telemetry

Mailwoman can export request traces and metrics to an OpenTelemetry collector
(OTLP over gRPC), and can serve Prometheus metrics at `/metrics`. Export is off
by default. This page covers the admin console screen, the environment variables
that override it, and exactly what is exported.

![The Configure operational telemetry tab of Observability, with the Collector and signals section: Enable OTLP export, Export traces, Export metrics, OTLP gRPC endpoint and Service name.](/screenshots/deploy/telemetry-editor.webp){.mw-shot}

## Configure export in the admin console

1. Open **Observability** and select the **Configure operational telemetry** tab.
2. Under **Collector and signals**, enter the **OTLP gRPC endpoint**, for example
   `https://otel.example.org:4317`. Choose **Export traces** and
   **Export metrics**, and set **Service name** and, optionally, **Environment**.
3. Select **Enable OTLP export**. Entering an endpoint alone does not enable it.
4. Adjust **Sampling and export limits** if needed (see the table below).
5. Select **Save telemetry configuration**, then restart the server.

Saving asks you to confirm your identity and is refused if someone else
changed the configuration since you loaded it. Until you restart, the page shows
`The saved configuration differs from the running server. A restart is required.`

The endpoint must be an `http://` or `https://` URL without user name, password,
query or fragment. It is a gRPC endpoint (usually port 4317), not the OTLP HTTP
port 4318. HTTPS verifies the collector's certificate. HTTP sends telemetry
unencrypted, so use it only on a network you trust.

| Field | What it does | Default | Range |
|---|---|---|---|
| **Trace sampling ratio** | Share of new root requests that are traced; child spans follow the parent | 0.1 | 0-1 |
| **Export timeout (milliseconds)** | Longest export request | 5000 | 100-30000 |
| **Metrics interval (seconds)** | Time between metric exports | 60 | 10-3600 |
| **Span queue size** | Spans waiting for export; when full, spans are dropped rather than slowing mail requests | 2048 | 128-32768 |
| **Span batch size** | Spans per export, no larger than the queue | 512 | 1-4096 |
| **Span batch delay (milliseconds)** | Normal wait between batches | 5000 | 100-60000 |
| **Enable Prometheus scraping** | Opens `/metrics` to requests with the `MW_METRICS_TOKEN` bearer token | off | |

These settings apply to the whole server; users cannot change them.

## Environment variables

Environment variables override the saved configuration. The fields they control
are shown disabled in the editor.

| Variable | What it does | Default |
|---|---|---|
| `MW_OTLP_ENDPOINT` | Sets the collector endpoint and turns export on | unset |
| `MW_OTEL_SERVICE` | Sets the service name | `mailwoman` |
| `MW_METRICS_TOKEN` | Bearer token for `/metrics`; also turns on **Enable Prometheus scraping**. The value is never shown in the console. Without it, `/metrics` always answers `401`. | unset |
| `MW_LOG` | Log filter at start-up. `RUST_LOG`, if set, takes precedence. | `info` |

## What is exported

- **Metrics:** `mailwoman.http.requests` (a counter) and
  `mailwoman.http.duration` (a histogram, in seconds).
- **Traces:** only the `mailwoman.http.request` span. All other log events stay in
  the local logs.
- **Attributes:** `route.class` and `status.class`, which are fixed route and
  status categories. URL paths, query strings, addresses, account ids and message
  content are not exported.
- The service name and environment labels are sent as you enter them; do not put
  personal information in them.

Signals you turn off do not start an exporter. CLI health checks and admin
commands never start exporters.

## Try it with a local collector

Run the OpenTelemetry Collector with an OTLP gRPC receiver and the debug
exporter, following the
[Collector Docker instructions](https://opentelemetry.io/docs/collector/install/docker/).
Publish the receiver port on `127.0.0.1` only. A collector can forward to other
self-hosted systems, such as Prometheus or a trace store.

## Related

- [Monitoring a deploy.sh installation](monitoring.md)
- [Observability (admin)](../admin/observability.md)
- [Observability and privacy](../security/observability.md)
