---
title: Observability
screen: true
---
# Observability

**Observability** (under **Operations** in the console navigation) shows how the server and the web app are behaving: request counts and response times, how long browsers take to load, errors that browsers report, the administrator audit log, database size and server resources. This page also covers how to watch an installation from outside the console, with Prometheus and with the health check of `deploy.sh`.

![The Overview tab of Observability: the tab strip, the period control, notes about what the figures mean, and the first key figures for traffic and client load times.](/screenshots/admin-ops/opsa-observability-overview.webp){.mw-shot}

## Before you start

- **Overview**, **Configure operational telemetry**, **Client errors**, **Database** and **Resources** need the `observability.read` permission. **Audit log** needs `audit.read`. A tab you may not open is not shown.
- The account, session, sign-in and storage figures of **Overview** also need `dashboard.read`.

![The six tabs of Observability with markers 1 to 6: Overview, Audit log, Configure operational telemetry, Client errors, Database and Resources.](/screenshots/admin-ops/opsa-observability-tabs.webp){.mw-crop}

1. **Overview**: request statistics, browser load times and saved statistics.
2. **Audit log**: what administrators did.
3. **Configure operational telemetry**: the page for exporting traces and metrics to a collector. See [Operational telemetry](/deploy/telemetry).
4. **Client errors**: errors the web app caught in browsers.
5. **Database**: file size, growth and write origins. See [Database growth and maintenance](/admin/database-maintenance).
6. **Resources**: CPU, memory and disk of the server. See [Database growth and maintenance](/admin/database-maintenance#resources).
{.mw-callouts}

## Read the Overview

1. Choose the **Period**: **Administrator default**, **Last 60 minutes**, **Last 6 hours**, **Last 24 hours**, **Last 7 days**, **Last 30 days** or **Custom period**. The refresh button next to it (**Refresh statistics**) reloads every figure. **Administrator default** is the range set by `metrics.defaultRangeSeconds`, and a custom period cannot be longer than `metrics.maxRangeSeconds`.
2. Read the notes under the period. They say which range is shown, how much of it has recorded history (**History covers 1 of 61 requested minutes. Missing periods are not counted as zero.**), and what the figures measure: response time is when the response is ready, not when a streamed download finishes.
3. **Key figures** has these groups:
   - **Traffic in this period**: **Requests**, **Server errors (5xx)**, **Not found (404)**, **Slowest 95th percentile** and the median, 95th and 99th percentile of **Response time**.
   - **Client load times**, **Folder load times** and **Message load times**: browser-measured figures, described below.
   - **Accounts and sessions**: **Provisioned users**, **Domains**, **Active webmail sessions**, **Active admin sessions** and accounts signed in over the last 24 hours and 7 days. These are the situation now, not for the period.
   - **Sign-in security**, and **Storage and delivery, now**: **Database file**, **Write-ahead log**, **Search index**, **Database query time**, **Pending outbox messages** and **Failed sends, last hour**.
4. **Charts** plots requests, server errors, 404 answers and response time over the period, the browser charts, and a bar chart of blocked accounts per domain. **View as table** shows the same numbers as a table.
5. **Tables** lists the **Most frequent not-found paths** and **Client load times by device**.

A figure that says **Not available** has no recorded data for the period. A tile for a percentile says why: **No samples in this period.** or **Only 3 samples; at least 5 are needed.**

How request statistics are measured and kept is on [Request latency and 404 paths](/admin/latency-metrics) and [Statistics that survive a restart](/admin/metrics-persistence).

## Client load times

The web app measures how long it takes to load and reports the result to the server. The report is sent once per page load, after the message list has shown its first rows, and folder and message loads are sent in batches every few seconds.

![The Client load times group of key figures: median, 95th and 99th percentile for time to usable, first contentful paint, largest contentful paint and page load, with the number of samples under each.](/screenshots/admin-ops/opsa-observability-client-timing.webp){.mw-crop}

The measures are:

| Label | What it measures |
|---|---|
| **Time to usable** | From the start of navigation until the message list first shows rows. |
| **First byte** | Until the first byte of the page arrived. |
| **DOM ready** | Until the page's document was ready. |
| **Page load** | Until the page finished loading. |
| **First contentful paint**, **Largest contentful paint** | The browser's own paint timings. |
| **Folder load, from the network**, **Folder load, from cache** | From choosing a folder until its message list is on screen. "From cache" means rows were shown at once from the browser's read cache. |
| **Message load, from the network**, **Message load, from cache or prefetch** | From opening a message until its headers and body are on screen. "From cache or prefetch" covers a message that was already prefetched or read. |

What a report contains: a device class (**Desktop**, **Mobile** or **Unknown**) and up to 16 pairs of a metric name and a whole number of milliseconds. It contains no URL, folder or message name, subject, address, account, session, IP address or user agent. The server adds each value to a counter and keeps, per hour, metric and device class, a count, a sum and a histogram of 48 bins; nothing is stored per person or per session.

The browser sends nothing when Do Not Track or Global Privacy Control is on, when the server says collection is off, when the session is outside the sampled share, or when the browser has no performance interface. Only signed-in browsers report; the sign-in page and other anonymous pages do not. A report is answered with an empty success even when collection is off. An unknown metric name, an unknown field or a malformed body is refused, and a single session that reports too often is refused with a retry delay.

A percentile is read from a histogram, so it is an estimate: one bin under 10 ms, then 47 bins from 10 ms to 120,000 ms, each about 22% wider than the one before. A percentile is therefore at most about 22% off and usually under 10%. Counts and means are exact. A value above 120 seconds is counted as 120 seconds. Slow loads that a person abandoned before the browser reported are not in the figures. A percentile is shown only when the period has at least `observability.clientTiming.minSamples` samples.

Each person can switch off folder and message load times for themselves in **Settings → Privacy** (**Load times**), unless the administrator locked the setting. When either is off for a person, that browser sends nothing for it and the server drops what it still receives from them. When the instance default is off, the page says **Folder load times are switched off by default. Only people who turned them on are measured.** (or the same for messages).

To turn collection off, switch off [`observability.clientTiming.enabled`](/admin/settings#observability.clienttiming.enabled). The server stops recording at once, and browsers learn of it at their next page load. Stored figures stay until they age out. The page then says **Collection of client load times is switched off. The figures below are from before it was switched off.**

## Client errors

When part of the web app fails in a browser, the app can report it, and the report appears under **Client errors** with a stack trace. This covers the "Something went wrong" panels, uncaught errors, unhandled promise rejections, failed lazy-loaded files, errors in the helper workers, Content-Security-Policy violations and, if switched on, `console.error` calls.

![The Client errors tab: filters for status, build, location and age, buttons to export CSV or JSON, and a table of two error groups.](/screenshots/admin-ops/opsa-observability-client-errors.webp){.mw-shot}

1. Open **Client errors**. Reports are grouped by cause, one row per group, with **Occurrences**, **Sessions (estimate)**, **Trend**, **First seen**, **Last seen**, **Build**, **Where** and **Status**.
2. Filter by **Status** (**All**, **Open**, **Resolved** or **Ignored**), **Build**, **Where** and **Last seen within** (1h, 24h, 7d, 30d or 90d). **Export CSV** and **Export JSON** download the list.
3. Open a group to see **Where** the error happened, **First seen** and **Last seen**, **Occurrences over time**, breakdowns by **Browser**, **Operating system**, **Build** and **Route**, the **Stack trace** and **Recent occurrences**.
4. Choose **Mark resolved**, **Ignore**, **Reopen** or **Delete group**. A group marked resolved reopens, with a **Regressed** label, when a build that never saw it reports it again. A group that is ignored still counts new occurrences but does not store them. **Copy as text** and **Copy as issue** copy the report.

What a report contains: the error class and a scrubbed message (addresses, long tokens, file names, URL queries and hashes removed, at most 300 characters); the stack frames of the bundle as paths without origin, query or hash; the part of the app (for example `Reader/AttachmentViewer/text`) and the route with ids replaced by `:id`; the build id, browser family and major version, operating system family, a viewport class (`narrow`, `medium` or `wide`), the language, whether the browser was online, reduced motion and the theme. A fingerprint, a hash of the error name, the normalised message, the location and the chunk names, groups reports of one defect across builds. A random id for the page load, held in memory only, lets the console estimate how many sessions a defect affects. A report never contains a user id, address, IP address, token, message content, attachment or file name, or a URL with a query, and the server scrubs again on arrival. A keyed one-way hash of the account is added only when [`observability.client.includeAccountHash`](/admin/settings#observability.client.includeaccounthash) is on and only to signed-in reports.

Limits: a signed-in session sends batches of at most 20 reports; the anonymous route, used by the sign-in page, takes at most 5 reports of 16 KB per request and four requests per address with a refill of one every 20 seconds. The browser does not retry a refused batch, stops sending after three failures in a row, sends one report per fingerprint every 60 seconds (repeats are folded into one follow-up with a count), and stops at [`observability.client.maxPerSession`](/admin/settings#observability.client.maxpersession) reports for a page load. A group stores details for its first 20 occurrences in an hour and only counts after that. Each person can switch reporting off for themselves in **Settings → Privacy** (**Error reports**), unless it is locked.

Each stored report writes one `info` log line with the target `client_error`. The metrics endpoint exposes `mw_client_errors_total{kind,boundary}` and `mw_client_error_groups_open`. With [`observability.client.forward`](/admin/settings#observability.client.forward) on and `MW_ERROR_FORWARD_URL` set, each scrubbed report is also posted to that address. There is no built-in OpenTelemetry or Sentry exporter for browser errors.

### Stack traces

Production stack frames point into minified files. The Dockerfile builds the web app with `MW_SOURCEMAP=hidden` unless you pass `--build-arg MW_SOURCEMAP=`: source maps are written next to the files without a reference in them, and the server answers every request for a `.map` file with 404, so browsers never receive them. The server reads the map of the exact file named in a frame and stores the resolved stack beside the raw one. With no matching map the group shows **No source map matched this stack: showing the raw frames.**, and when `observability.client.sourceMaps` is off the tab says **Source maps are switched off (observability.client.sourceMaps): stack traces show bundle positions.**

## Audit log

The **Audit log** tab lists what administrators and the system did (**Time**, **Actor**, **Action**, **Target**); it loads the latest 100 entries. **Export JSONL** downloads up to 1,000 entries as `audit-log.jsonl`. Below the list, **Audit log retention** sets how long entries are kept for each category, with an optional legal hold; **Prune now** applies the policy at once. See [Retention](/admin/retention).

## Watch the server from outside

### Prometheus metrics

`GET /metrics` serves the Prometheus text format and requires `Authorization: Bearer <MW_METRICS_TOKEN>`. Without the token, or with a wrong one, it answers `401`, and with `MW_METRICS_TOKEN` unset it answers `401` to everyone, so metrics are never open. No mail content is a metric name or a label ([Observability and privacy](/security/observability)).

`bash deploy.sh init`, which the first deploy also runs, writes a random 64-digit hexadecimal `MW_METRICS_TOKEN` into `app.env`. `bash deploy.sh status` prints a `Metrics:` line saying whether the applied `app.env` has a token. To add a token to an installation that has none, then apply it:

```sh
printf 'MW_METRICS_TOKEN=%s\n' "$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')" >> .deploy.local/app.env
bash deploy.sh deploy --no-build
bash deploy.sh status
```

To turn metrics off, delete the line and run `deploy --no-build` again. Give Prometheus the token as a file that only Prometheus can read:

```yaml
scrape_configs:
  - job_name: mailwoman
    scheme: https                     # http for a local installation
    metrics_path: /metrics
    authorization:
      type: Bearer
      credentials_file: /etc/prometheus/mailwoman-metrics-token
    static_configs:
      - targets: ['mail.example.org']  # localhost:8080 for a local installation
```

With `--proxy-network`, a Prometheus container on the same Docker network can scrape `http://<project>:8080/metrics` directly.

### Health check

```sh
bash deploy.sh check
```

exits 0 only when, on a full-stack installation, the `postgres` and `valkey` containers are running and Docker reports them `healthy`; the app container is running and Docker reports it `healthy`; and `/healthz` answers at the public URL from `deploy.env`. For `network` exposure, whose public URL a proxy that `deploy.sh` does not manage serves, it checks `http://<project>:8080/healthz` on the proxy network instead. Otherwise it prints `CHECK FAILED (<check>): <message>` with `<check>` one of `postgres`, `valkey`, `container`, `public-url` or `proxy-network`, runs `ALERT_HOOK`, and exits 1.

By default `/healthz` answers only to loopback, private, link-local and unique-local client addresses; other clients get the same `404` as for an unknown path. The setting is [`security.health.mode`](/admin/settings#security.health.mode) (**internal-only**, **allowlist** or **open**). A request that carries the token in `MW_HEALTH_TOKEN`, as the `X-Health-Token` header or a `token` query parameter, is always admitted, and `check` sends the token when `app.env` has one. If a trusted proxy hides the real client address, the server may see the check as a public address; set `MW_HEALTH_TOKEN` then. See [the health endpoint behind a proxy](/deploy/reverse-proxy#the-health-endpoint-behind-a-proxy).

`check` only reads, takes no lock and changes nothing, so it can run during a backup or deploy; the app is briefly stopped during those, and a check at that moment fails. It proves that the app answers over HTTP(S). It does not sign in to a mailbox or send mail. [Backups](/deploy/backups#scheduling-timers) shows how to run it every five minutes with systemd or cron.

`/readyz` is the readiness check. It answers `200` with `ok`, or `503` with a line naming the directories that are short of free space: the one holding the database, the one named by `MW_UPLOAD_DIR`, and the search index directory when it is kept on disk. A directory is short when less than [`health.diskFreeMinPercent`](/admin/settings#health.diskFreeMinPercent) percent of its filesystem is free (default 5). A directory that cannot be measured, or a measurement that takes more than 1.5 seconds, is skipped, so a measurement error never makes the server report not ready. `/readyz` is reachable under the same rules as `/healthz`. The Helm chart and the Compose files still probe `/healthz`; use `/readyz` only for readiness and load-balancer checks, never as a liveness probe, because a full disk should take the server out of rotation, not restart it.

### `ALERT_HOOK`

`ALERT_HOOK` in `deploy.env` is the absolute path of an executable that `check` runs when a check fails, with two arguments, the name of the check and the message, as the user that runs `deploy.sh`. Its exit status is logged but does not change the exit status of `check` (always 1 on failure). The hook runs on every failing check, every five minutes with the timer, so it must remove duplicates itself if that matters. `deploy.sh` accepts only an absolute path made of letters, digits and `._/+@-`. If the file is not an executable, `check` logs `ALERT_HOOK <path> is not an executable file; no alert was sent`.

```sh
#!/bin/sh
# Send the failure to an ntfy topic (or any webhook).
exec curl -fsS -m 10 -d "mailwoman: $1: $2" https://ntfy.example.org/mailwoman-alerts
```

### Logs and traces

`RUST_LOG` sets the log filter at start-up. `deploy.sh` writes `RUST_LOG=info` into `app.env`, and when it is set it takes precedence over `MW_LOG`, whose default is `info`. `MW_OTLP_ENDPOINT` turns on the export of traces and metrics to an OpenTelemetry collector. See [Operational telemetry](/deploy/telemetry). Recent warnings and errors of the running server are in [Logs](/admin/logs).

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Browser load times** ([`observability.clientTiming.enabled`](/admin/settings#observability.clienttiming.enabled)) | Collect client load times. | On | Administrators (global) |
| **Browser load time sample rate** ([`observability.clientTiming.sampleRatePercent`](/admin/settings#observability.clienttiming.sampleratepercent)) | Share of sessions that report, 1 to 100. | 100 | Administrators (global) |
| **Load time minimum samples** ([`observability.clientTiming.minSamples`](/admin/settings#observability.clienttiming.minsamples)) | Samples a period needs before a percentile is shown. | 1 | Administrators (global) |
| **Folder load times** ([`observability.clientTiming.folderLoads`](/admin/settings#observability.clienttiming.folderloads)) | Measure folder loads. Also needs the setting above. | On | Administrators (global, domain, user); each person can switch it off unless locked |
| **Message load times** ([`observability.clientTiming.messageLoads`](/admin/settings#observability.clienttiming.messageloads)) | Measure message loads. Also needs the first setting. | On | Same as folder loads |
| **Browser error reports** ([`observability.client.enabled`](/admin/settings#observability.client.enabled)) | Collect browser error reports. | On | Administrators (global) |
| **Error report sample rate** ([`observability.client.sampleRate`](/admin/settings#observability.client.samplerate)) | Fraction of error reports sent, 0 to 1. | 1 | Administrators (global) |
| **Error reports per page load** ([`observability.client.maxPerSession`](/admin/settings#observability.client.maxpersession)) | Reports one page load may send, 1 to 1000. | 20 | Administrators (global) |
| **Add account hash to error reports** ([`observability.client.includeAccountHash`](/admin/settings#observability.client.includeaccounthash)) | Add a keyed one-way hash of the account to signed-in reports. | Off | Administrators (global) |
| **Report console errors** ([`observability.client.captureConsole`](/admin/settings#observability.client.captureconsole)) | Also report `console.error` calls, at a throttled rate. | Off | Administrators (global) |
| **Report CSP violations** ([`observability.client.captureCsp`](/admin/settings#observability.client.capturecsp)) | Report Content-Security-Policy violations the browser sees. | On | Administrators (global) |
| **Resolve stack traces with source maps** ([`observability.client.sourceMaps`](/admin/settings#observability.client.sourcemaps)) | Resolve stack frames with the build's hidden source maps. | On | Administrators (global) |
| **Forward error reports** ([`observability.client.forward`](/admin/settings#observability.client.forward)) | Also post scrubbed reports to `MW_ERROR_FORWARD_URL`. | Off | Administrators (global) |
| **Error occurrences kept (days)** ([`retention.clientErrors.eventDays`](/admin/settings#retention.clienterrors.eventdays)) | Days an individual occurrence is kept, 1 to 3650. | 30 | Administrators (global) |
| **Error groups kept (days)** ([`retention.clientErrors.groupDays`](/admin/settings#retention.clienterrors.groupdays)) | Days a group is kept after it was last seen, 1 to 3650. | 180 | Administrators (global) |
| **Send error reports** ([`privacy.errorReports`](/admin/settings#privacy.errorreports)) | Whether a browser sends error reports. | On | Administrators (global, domain, user); each person can switch it off unless locked |
| **Default request history range** ([`metrics.defaultRangeSeconds`](/admin/settings#metrics.defaultrangeseconds)) | Range of **Administrator default**. | 3600 | Administrators (global) |
| **Maximum request history range** ([`metrics.maxRangeSeconds`](/admin/settings#metrics.maxrangeseconds)) | Longest range a request statistics query may cover. | 2592000 | Administrators (global) |
| **Metrics history** ([`metrics.rollupDays`](/admin/settings#metrics.rollupdays)) | Days of hourly request and browser statistics kept, 1 to 1095. | 1095 | Administrators (global) |
| **Request path detail retention** ([`retention.requestMetrics.detailDays`](/admin/settings#retention.requestmetrics.detaildays)) | Days normalised not-found paths are kept. | 30 | Administrators (global) |

Changes to the `observability.clientTiming` settings are picked up within about ten seconds. History is written every 30 seconds, so a restart keeps it, and the same tables are used with SQLite and with PostgreSQL.

### Settings that have no effect

[`observability.otlpMetrics`](/admin/settings#observability.otlpmetrics) and [`metrics.drilldownTopN`](/admin/settings#metrics.drilldowntopn) can be saved but nothing in the server reads them. Operational telemetry is configured on the **Configure operational telemetry** tab and by `MW_OTLP_ENDPOINT`.

## Troubleshooting

- **Every figure says "Not available" right after the server started.** The history is written every 15 to 30 seconds. Reload after a minute.
- **A percentile tile says "Only 3 samples; at least 5 are needed."** The period has fewer samples than [`observability.clientTiming.minSamples`](/admin/settings#observability.clienttiming.minsamples). Choose a longer period or lower the setting.
- **No client load times at all.** Check that collection is on, that the sample rate is above 0, and that browsers do not have Do Not Track or Global Privacy Control on. Browsers pick up a change at their next page load.
- **`/metrics` answers 401.** The token is missing or wrong, or `MW_METRICS_TOKEN` is not set on the server. `deploy.sh status` shows whether the applied `app.env` has one.
- **`/healthz` answers 404 from a monitor outside the network.** The address is not internal. Use `MW_HEALTH_TOKEN`, or change [`security.health.mode`](/admin/settings#security.health.mode).
- **`CHECK FAILED (container): the app container of <project> is not running`** (or `is starting, not healthy`). Look at `bash deploy.sh logs`.
- **A client error group shows raw frames.** No source map matched. The web build must have been made with `MW_SOURCEMAP=hidden`, and the server must serve the same build.
- **The Client errors tab says "Browser error reporting is switched off (observability.client.enabled). Existing reports are kept until they expire."** Switch the setting on to receive new reports.

## Related

- [Request latency and 404 paths](/admin/latency-metrics)
- [Statistics that survive a restart](/admin/metrics-persistence)
- [Database growth and maintenance](/admin/database-maintenance)
- [Logs](/admin/logs)
- [Diagnostics](/admin/diagnostics)
- [Observability and privacy](/security/observability)
