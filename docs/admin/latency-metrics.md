---
title: Request latency and 404 paths
---
# Request latency and 404 paths

The request figures on the **Overview** tab of [Observability](/admin/observability) (and on the **Dashboard**) count the requests this server handled and measure how long each one took. This page explains what those figures mean, how precise the response times are, and how the **Most frequent not-found paths** table is built.

## What is measured

- A request is counted when the server has produced its response. **Response time** is the time until the response is ready. It does not include the transfer of a long download, and it does not include the time the browser needs to display the result.
- Requests are classed by route kind (the console, API routes, static files and so on). No path, address, account, query text or token becomes a label.
- The live figures belong to one server process. With several servers behind one address, each has its own.
- The recent minutes come from memory and the older ones from saved history, written every 15 seconds. A period shows **History covers N of M requested minutes**, and a gap is never drawn as zero.
- Request history is kept as minute figures and as hourly figures. Hourly figures are kept for [`metrics.rollupDays`](/admin/settings#metrics.rollupdays) days, so the minute figures can expire without losing the hourly history. See [Statistics that survive a restart](/admin/metrics-persistence).

## How precise the response times are

A percentile is read from a histogram of response times, not from individual requests.

| Response time | Resolution of the histogram |
|---|---|
| Under 1 ms | Whole microseconds. A measured duration shorter than one microsecond is shown as 0.001 ms. |
| 1 ms to 1 second | 1 ms |
| 1 second to 10 seconds | 10 ms |
| 10 seconds to 60 seconds | 100 ms |
| 60 seconds to 10 minutes | 1 second |
| Over 10 minutes | One overflow bin; the largest value seen stands in for it. |

A 37 ms request and an 83 ms request stay 37 ms and 83 ms. A percentile is the upper edge of the bin that holds the requested rank. Values under 1 ms are shown with up to three decimals, including in chart tooltips and the table form of a chart. Samples recorded with an older, coarser histogram keep that resolution, and the note under the period says that older snapshots keep their original precision, which cannot be recovered.

Saved history is bounded. A history query reads at most 20,000 rows, 128 KiB for each stored histogram and 16 MiB in all. A histogram that is malformed, inconsistent or over a limit fails the query instead of producing a misleading figure.

## Not-found (404) answers

Request history keeps 404 answers as a series of their own, in the minute and the hourly figures, with the same retention and coverage rules as the request counts. A period that was recorded before 404 answers were tracked is **Not available** rather than zero, and the page says **404s were not tracked before …**.

![The Most frequent not-found paths table: each row has the method, the redacted path and a count, with the app's own clients listed apart from other traffic.](/screenshots/admin-ops/opsa-observability-not-found.webp){.mw-crop}

### Read the table

1. Open **Observability → Overview** and scroll to **Tables**.
2. **Most frequent not-found paths** lists up to 50 paths for each hour of the period, rounded out to whole hours. You need the `observability.read` permission; the **Dashboard** permission alone shows only the counts.
3. Each row has **Method**, **Path** and **Count**.
4. The paths are in two lists. **From the app's own clients** are paths that the web app, its service worker or the desktop and mobile apps request: files of the web build, the worker script, fonts, themes, icons, and API, JMAP, DAV, console and MCP routes. A miss here usually means a client runs an older build or calls a route that does not exist. **Other traffic** is everything else, such as scanners looking for PHP files or dotfiles and links from elsewhere. A path that looks like a probe is listed under **Other traffic** even when it is under an app route: a segment ending in a script, archive, configuration or dump extension (`.php`, `.env`, `.sql`, `.zip` and similar), a dotfile segment other than `.well-known`, or a common scanner target. The list is decided when the table is read, so it also applies to hours recorded earlier.
5. When an hour received more different paths than it keeps, the page says **More distinct paths arrived than are kept, so some counts are a range rather than an exact figure.** A count is then shown as a range, from a lower bound to an upper bound.

Paths are redacted before they are stored. The query string and fragment are dropped. Email addresses become `{email}` and numbers `{n}`. UUIDs, hexadecimal strings, base64, JWT-like and percent-encoded values become `{uuid}`, `{hex}` or `{token}`. Identifiers with a word prefix become `{id}`, segments longer than 40 characters `{long}`, and the values of `key=value` segments `{value}`. The signed token after `/api/rc/<kind>/` becomes `{rc-token}`. A short file extension is kept (`/{hex}.zip`). Anything outside printable ASCII becomes `_`. A path keeps at most 16 segments and 160 bytes. Methods other than the ones this server answers are recorded as `OTHER`. No address, header, account or session is part of the table.

Sampled paths are removed after [`retention.requestMetrics.detailDays`](/admin/settings#retention.requestmetrics.detaildays) days (30 by default) while the counts and latency figures stay. With 0, only counters are kept once an hour has closed.

### 404 answers that are not counted

Some 404 answers are part of how the web app works, so they are not counted as not-found (they still count as requests):

- `GET` or `HEAD` of `/favicon.ico`, `/favicon.svg`, `/favicon.png` and `/apple-touch-icon*.png` at the root, which browsers request on their own.
- `GET /api/account/mdn-decision/{id}`: no receipt decision is recorded for the message.
- `GET /api/track/open`: no tracking pixel is bound to the message.
- `GET /api/zeroaccess/pair/envelope/{id}`: a new device polls until the envelope arrives.
- `GET` of the console's `/search` route: the command palette's optional server search, which this server does not mount.

A JMAP `unknownMethod` is a method error inside an HTTP 200 answer, so it is never a 404. Every other 404, from any client, is counted.

Request protection does not use status classes, so 404 counts are not passed to it, and there is no alert on a 404 spike.

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Metrics history** ([`metrics.rollupDays`](/admin/settings#metrics.rollupdays)) | Days of hourly request statistics kept, 1 to 1095. | 1095 | Administrators (global) |
| **Minute statistics kept (hours)** ([`retention.metrics.minuteBucketsHours`](/admin/settings#retention.metrics.minutebucketshours)) | Hours of minute figures kept for request history (1 to 8760). | 48 | Administrators (global) |
| **Request path detail retention** ([`retention.requestMetrics.detailDays`](/admin/settings#retention.requestmetrics.detaildays)) | Days normalised not-found paths are kept; 0 keeps counters only. | 30 | Administrators (global) |
| **Default request history range** ([`metrics.defaultRangeSeconds`](/admin/settings#metrics.defaultrangeseconds)) | The range of **Administrator default** in the period control. | 3600 | Administrators (global) |
| **Maximum request history range** ([`metrics.maxRangeSeconds`](/admin/settings#metrics.maxrangeseconds)) | The longest range a request history query may cover. | 2592000 | Administrators (global) |

## Troubleshooting

- **The table says "No not-found answers in this period."** Nothing was counted. Choose a longer period.
- **The table says "404s were not tracked during this period."** The period is older than the first recorded 404 data.
- **"Not-found paths could not be loaded. Refresh to try again."** The query failed; refresh. The request counts above are unaffected.
- **A real client path is in "From the app's own clients."** Look at the redacted path. Usually a browser still runs an older build of the web app, or an integration calls a route that does not exist.

## Related

- [Observability](/admin/observability)
- [Statistics that survive a restart](/admin/metrics-persistence)
- [Request protection](/admin/request-protection)
