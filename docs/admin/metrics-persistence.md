# Statistics that survive a restart

The performance statistics in the admin console (median and 95th percentile
cards, the Speed panels, rate-limit and request-protection counts) used to live
in memory for several sources, so a restart emptied them. This page lists what
was kept where before, and what is stored now.

## What was in memory and what was persisted (audit)

| Statistic | Where it is recorded | Before | Now |
|---|---|---|---|
| Browser page / app load (`timeToUsable`, `ttfb`, `dcl`, `load`, `fcp`, `lcp`) | `client_timing.rs`, beacon `POST /api/client-timing` | Hourly histograms in memory for the current and previous hour, written to `client_timing_bucket` every 30 s. The admin view read the database, so history already survived a restart; up to 30 s were lost on a hard stop. Hour resolution only. | Unchanged, and the same samples also go to `metrics_bucket` at one-minute resolution with a device-class dimension (see below). |
| Folder load, message load (`folderLoadCold/Cached`, `messageLoadCold/Cached`, `folderOpen`) | same | same as above | same as above |
| Sanitize time, search time | not measured by the browser beacon | none | none (the beacon has no such metric; the sanitize rate limiter is counted under rate limits) |
| Image proxy latency: median, p95, p99, miss latency, phases, outcomes | `remote_image_latency.rs`, process-wide recorder | One-minute slices in a 24 h ring in memory only; empty after a restart; `resetsOnRestart: true` | Every request also lands in `metrics_bucket`; at start-up the ring is filled from the stored minutes, so the panel is not empty. |
| Image proxy worker history (served, busy, bytes) | `remote_image_history.rs` | In memory only | Unchanged (counters of the live process; the panel says so). |
| Request metrics by route class (requests, 4xx, 5xx, latency) | `http_metrics.rs` (60 minute ring and Prometheus counters), `request_history.rs` | The ring is in memory; `request_history` writes minute and hour buckets to `request_metrics_bucket` every 15 s | Unchanged: already persisted. |
| Request-protection decision counts (allowed, throttled, blocked) | `request_protection/history.rs` | 24 h ring in memory only; decision samples go to `request_protection_event` (kept 30 days) | Counts also go to `metrics_bucket` (`requestProtection.*`). |
| Rate-limit counters per limiter class: allowed, queued, rejected | `rate_leeway.rs` | Process counters and a recent list in memory; empty after a restart | The change in the counters is written to `metrics_bucket` at each flush (`rateLimit.<class>.<outcome>`). The recent-refusals list stays in memory. |
| Job runtimes | `jobs.rs` | `job_run` rows (start, finish, outcome), kept `retention.jobRuns.days` | Unchanged: already persisted. |

## The store

Table `metrics_bucket` (migration `0098`, SQLite and Postgres). One row per
instance, metric, bucket and dimension set. The primary key is
`(instance_id, metric, bucket_start, resolution_seconds, dim_domain, dim_folder_class, dim_client_kind, dim_device_class, dim_backend)`, every part `NOT NULL`.

| Column | Meaning |
|---|---|
| `instance_id` | Random id of the server process that wrote the row; a restart is a new instance. |
| `metric` | A fixed name from the code, for example `imageProxy.outcome.miss`, `clientTiming.folderLoadCold`, `rateLimit.jmap.rejected`, `requestProtection.blocked`. |
| `bucket_start`, `resolution_seconds` | Start of the bucket (unix seconds) and its length: 60, 3600 or 86400. |
| `dim_*` | Bounded dimensions; empty string when not applicable. Filled today: `dim_client_kind` (`web`) and `dim_device_class` (`desktop`, `mobile`, `unknown`) for browser timings. `dim_domain`, `dim_folder_class` and `dim_backend` are reserved for later and always empty now. |
| `sample_count`, `sum_value`, `min_value`, `max_value` | Exact count, sum, minimum and maximum. For a counter metric the count is the sum of increments. |
| `sketch` | The distribution: counts per log-linear bucket (values 0 to 7 each have a bucket; every power of two above is split into 8; up to 2^26), stored as varint pairs of non-empty buckets. Latencies are stored in the metric's own unit: microseconds for the image proxy, milliseconds for browser timings. |
| `flushed_at` | When the row was last written; the console shows the newest as "last saved". |

A percentile is never stored. Sketches merge by adding bucket counts, so a
range, or several instances, give the median and 95th percentile of the union
exactly as if every sample had been recorded in one sketch. A reported value is
within 1/16 (6.25 percent) of the true one above 7 units, and exact below.

No user id, session, address, path, message or folder name is stored. At most 32
distinct dimension sets are kept per metric and minute; further ones are folded
into `other`. With `metrics.dimensions.enabled` off every row has empty
dimensions.

## Flush, restart and shutdown

* Samples are added to an in-memory sketch of their minute. Every
  `metrics.persist.flushSeconds` (default 60) the minutes that changed are
  written as absolute snapshots under this process's instance id. A snapshot only
  replaces a row with fewer samples, so a retried write never counts twice. A
  failed write keeps the minutes and retries at the next flush.
* On a graceful stop (SIGTERM or Ctrl+C) the server flushes once more, in the same
  shutdown hook as the other final saves (`flush_trust_tiers`), before exiting. A
  hard kill loses at most one flush interval.
* After a restart the new process has a new instance id. Reads merge every
  instance's rows, so history is continuous and medians and p95 appear at once.
  The image proxy panel's live ring is filled from the stored minutes at start-up.
* Several instances on one database each write their own rows and need no
  locking; a read merges them.
* The admin read first flushes the local process, so the newest samples are in
  the answer.

## Retention and roll-up

The retention sweeper (`retention_sweep` job, daily) rolls the table up:

| Resolution | Kept | Setting |
|---|---|---|
| 1 minute | 48 hours | `retention.metrics.minuteBucketsHours` (at least 1) |
| 1 hour | 60 days | `retention.metrics.hourBucketsDays` (at least 1) |
| 1 day | 1095 days | `retention.metrics.dayBucketsDays` (0 keeps without limit; the console warns) |

Minute rows older than the first limit are merged into hour rows (sketches
added, per instance and dimension set), hour rows older than the second into day
rows; day rows past the third are deleted. The roll-up is idempotent. The data
map lists `metrics_bucket` as exempt: aggregate metrics with no person
identifiers.

## Settings

* `metrics.persist.enabled` (default on): turn the store off; the console shows
  "Statistics are not being saved" and nothing is written.
* `metrics.persist.flushSeconds` (default 60, 5 to 3600).
* `metrics.dimensions.enabled` (default on).
* `retention.metrics.minuteBucketsHours`, `retention.metrics.hourBucketsDays`,
  `retention.metrics.dayBucketsDays`.

## Console

The Speed panel (Privacy, Image proxy), Client load times and Request protection
show a "Saved statistics" card with a range selector (last hour, 24 hours, 7
days, 30 days, 1 year), a "Saved" badge with the time of the last save and the
row count, "History since" the first stored bucket, line charts of the median and
95th percentile (counts for counters) and a "Reset statistics" action. Reset asks
for confirmation, deletes the stored buckets of that source (and, for browser
load times, the older hourly table) and writes an audit entry `metrics-reset`.

API: `GET /admin/metrics/persisted?source=imageProxy|clientTiming|rateLimit|requestProtection&range=1h|24h|7d|30d|1y`
(`observability.read`) and `POST /admin/metrics/persisted/reset` with
`{"source": ...}` (`settings.write`).
