---
title: Statistics that survive a restart
---
# Statistics that survive a restart

Some performance statistics in the console are saved to the database, so a restart does not empty them. They appear as a **Saved statistics** card with a range selector. This page lists what is saved, where the cards are, how long the data is kept and how to reset it.

![The top of the Saved statistics card on the Observability Overview, with markers 1 and 2 on the range buttons and the Saved badge with the row count.](/screenshots/admin-ops/opsa-observability-saved-stats.webp){.mw-crop}

1. The range buttons: **Last hour**, **Last 24 hours**, **Last 7 days**, **Last 30 days** and **Last year**.
2. The **Saved** badge, with the time of the last save, the number of stored rows and the start of the history.
{.mw-callouts}

Below the charts of each card is the **Reset statistics** button, described under [Reset the saved statistics](#reset-the-saved-statistics).

## What is saved and where it is shown

| Source | What it records | Where the card is |
|---|---|---|
| Browser load times | The same samples as the Client load times figures, at one-minute resolution and with a device class. | **Observability → Overview** |
| Image proxy | Latency of image proxy requests: median and 95th percentile, fetched images and cache hits. | **Privacy → Image proxy → Speed** |
| Rate limits | The change in the allowed, queued and rejected counters of each rate limiter. | **Security centre → Aggregate and adaptive protection** |
| Request protection | Decisions of request protection: allowed, throttled and blocked. | **Security centre → Aggregate and adaptive protection** |

Other statistics keep their own storage. Request counts and response times of the Dashboard and Overview are written every 15 seconds ([Request latency and 404 paths](/admin/latency-metrics)). Job runs are kept as job history.

Each card has line charts of the median and the 95th percentile (or counts, for counters). Percentiles are never stored. They are read from stored counts, which can be added together, so a range, or several servers that share one database, give the same median and 95th percentile as if every sample had been recorded in one place. A reported value is within 6.25% of the true value above 7 units, and exact below that.

## How saving works

- Samples are added to an in-memory sketch for their minute. Every `metrics.persist.flushSeconds` seconds the minutes that changed are written to the database under the id of the server process. A retried write never counts twice, and a failed write keeps the minutes and retries at the next flush.
- On a graceful stop (SIGTERM or Ctrl+C) the server saves once more before it exits. A hard kill loses at most one flush interval.
- After a restart the new process finds the earlier rows and shows them at once, so medians and percentiles are not empty. The image proxy panel is refilled from the stored minutes.
- Servers that share one database each write their own rows without locking, and a read merges them. A read from the console saves the local server's newest samples first.
- No user, session, address, path, message or folder name is stored. At most 32 different dimension sets are kept per metric and minute; further ones are folded into `other`. The dimensions in use are the client kind (`web`) and the device class (`desktop`, `mobile`, `unknown`) of browser load times. With **Split statistics by client kind and device class** off, every row has empty dimensions.

## Retention

A retention sweep (the `retention_sweep` job, daily) rolls the saved rows up:

| Resolution | Kept | Setting |
|---|---|---|
| 1 minute | 48 hours | **Minute statistics kept (hours)**, at least 1 |
| 1 hour | 60 days | **Hourly statistics kept (days)**, at least 1 |
| 1 day | 1095 days | **Daily statistics kept (days)**; 0 keeps them without limit |

Minute rows older than the first limit are merged into hourly rows and hourly rows older than the second into daily rows. Daily rows older than the third limit are deleted. The roll-up can be repeated safely. The saved statistics are listed in the data map as aggregate metrics with no person identifiers.

## Reset the saved statistics

1. Open the card and choose **Reset statistics**. You need the `settings.write` permission.
2. Read the confirmation (**Reset these statistics?**). It deletes the stored history of that statistic for every server instance, including the older hourly table for browser load times. This cannot be undone.
3. Choose **Reset statistics** to confirm. The reset is recorded in the audit log as `metrics-reset`.

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Keep statistics across restarts** ([`metrics.persist.enabled`](/admin/settings#metrics.persist.enabled)) | Save statistics. When it is off the card says **Statistics are not being saved. Turn on "Keep statistics across restarts" in Settings to keep them after a restart.** and nothing is written. | On | Administrators (global) |
| **Statistics save interval (seconds)** ([`metrics.persist.flushSeconds`](/admin/settings#metrics.persist.flushseconds)) | Seconds between saves, 5 to 3600. | 60 | Administrators (global) |
| **Split statistics by client kind and device class** ([`metrics.dimensions.enabled`](/admin/settings#metrics.dimensions.enabled)) | Keep the client kind and device class as dimensions. | On | Administrators (global) |
| **Minute statistics kept (hours)** ([`retention.metrics.minuteBucketsHours`](/admin/settings#retention.metrics.minutebucketshours)) | Hours of one-minute rows, 1 to 8760. | 48 | Administrators (global) |
| **Hourly statistics kept (days)** ([`retention.metrics.hourBucketsDays`](/admin/settings#retention.metrics.hourbucketsdays)) | Days of one-hour rows, 1 to 3650. | 60 | Administrators (global) |
| **Daily statistics kept (days)** ([`retention.metrics.dayBucketsDays`](/admin/settings#retention.metrics.daybucketsdays)) | Days of one-day rows, 0 to 3650 (0 keeps them without limit). | 1095 | Administrators (global) |

The request history minute figures follow **Minute statistics kept (hours)** too; its hourly figures keep their own limit, **Metrics history** (`metrics.rollupDays`).

## Troubleshooting

- **The card says "Saved statistics are temporarily unavailable."** The read failed or took longer than eight seconds. Reload; if it persists, check the database with **Diagnostics**.
- **The card says "Nothing saved yet."** The server has not written a save since the last reset or since the first start. The first save follows within the flush interval.
- **The history starts after the day the server was installed.** **History since** is the first stored row. Rows older than the retention limits are rolled up or deleted.
- **A chart says "Nothing was recorded in this range."** No samples fell in the range. Browser load times need signed-in browsers that report ([Observability](/admin/observability#client-load-times)).

## Related

- [Observability](/admin/observability)
- [Request latency and 404 paths](/admin/latency-metrics)
- [Request protection](/admin/request-protection)
- [Retention](/admin/retention)
