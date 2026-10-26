# Operational telemetry

Open **Admin → Observability → Configure operational telemetry** to configure the collector, signals, root sampling ratio, export timeout, metric interval and bounded span queue/batches. Changes use revision checks and administrator step-up; they apply at server restart. The page distinguishes saved configuration from the active process configuration. Export is disabled by default.

Mailwoman exports OTLP over **gRPC**, normally port 4317. Use an OpenTelemetry Collector or another compatible receiver. HTTPS verifies certificates; HTTP is intended for an explicitly trusted deployment network and does not encrypt telemetry. Collector addresses cannot contain credentials, query parameters or fragments. The collector can forward to self-hosted systems such as Prometheus or a trace backend; Mailwoman does not load remote browser scripts for operational telemetry.

`MW_OTLP_ENDPOINT` explicitly enables export and pins the endpoint. `MW_OTEL_SERVICE` pins the service name. `MW_METRICS_TOKEN` enables the separately authenticated Prometheus endpoint; the token is never returned to the admin page. These operator overrides are shown as disabled fields. `MW_LOG` remains the startup log-filter configuration. Merely entering an endpoint in the editor does not enable export.

The OTLP metric series are `mailwoman.http.requests` and `mailwoman.http.duration` (seconds). Attributes are bounded route/status classes, not URL paths, queries, addresses, account IDs or message content. Exported tracing spans are restricted to the fixed `mailwoman.http.request` span, with route class and status. Other diagnostic tracing events stay in local logs. The service/environment labels are operator-provided; avoid putting personal information in them. Existing Prometheus instrumentation remains available independently.

The span queue and export timeout are bounded. A full queue can drop spans instead of blocking mail requests. Sampling applies to root requests. Disabled signals do not create their exporters. CLI health checks and administrative commands do not initialize exporters. Saved settings are server-wide; users cannot change the collector destination.

For a local receiver example, use the official [Collector Docker configuration](https://opentelemetry.io/docs/collector/install/docker/) with an OTLP gRPC receiver and debug exporter. Bind published test ports to loopback. Collector ingestion and deployment network access must be checked in the actual deployment.
