# Request latency precision

Live and persisted request statistics measure time until the response is ready. They do not measure the complete response transfer or browser rendering time.

New samples below 1 ms retain whole microseconds (0.001 ms resolution). Positive measured durations shorter than one microsecond are reported at the 0.001 ms measurement floor. Samples from 1 through 1,000 ms retain integer milliseconds. Longer durations use bounded histogram bins: 10 ms through 10 seconds, 100 ms through 60 seconds, and 1 second through 10 minutes. Above that range the observed maximum supplies the overflow estimate. Percentiles are histogram estimates, not individual request traces. The shared display renders sub-millisecond values with up to three decimal places, including graph tooltips and accessible tables.

The live and historical APIs include `legacyLatencyEstimate`, `p50ResolutionMs`, and `p95ResolutionMs` for each point. A resolution of 0.001 means measured microsecond samples; 1 means integer-millisecond samples; a larger value identifies the wider bin. A null resolution identifies legacy or overflow estimates. For example, new samples at 37 ms and 83 ms remain 37 ms and 83 ms, rather than being rounded to the old 50 ms and 100 ms bounds.

Persisted version 3 documents retain the original coarse and version 2 fine histograms and add at most 1,000 sub-millisecond bins. Version 1 and 2 documents remain readable. Combining version 2 and 3 samples conservatively retains integer-millisecond resolution; combining original coarse data with newer samples uses the coarse histogram and is explicitly marked as a legacy estimate. The API continues to express latency as JSON numbers in milliseconds, now allowing fractional values. The history `latencySampleUnit` describes current capture, while each point's resolution describes its actual data. Older binaries that only understand version 2 cannot read newly written version 3 history; preserve a pre-upgrade database backup before rolling back. Old measurement precision cannot be reconstructed. The administrator history page explains this limitation when displaying older or mixed data.

Each fine histogram has at most 2,942 bins. History reads limit the number of rows to 20,000, each JSON document to 128 KiB, and combined JSON bytes to 16 MiB. SQLite and PostgreSQL check these byte budgets before returning JSON documents to the application. Malformed histograms, inconsistent counts and exceeded budgets fail the history request rather than silently producing misleading statistics.

## Not-found (404) answers

Request history keeps 404 answers as a series of their own, in the minute tier (48 hours) and the hourly tier (`metrics.rollupDays`), with the same retention and coverage rules as request counts. The history API returns `notFound` and `notFoundCoverage` for each point and `coverage.notFound` (`observedSeconds`, `trackedSince`, `notTracked`) for the range. Snapshots written before 404 tracking existed have no 404 series: their periods are reported as `notTracked` and shown as "not available", never as zero. A point that mixes tracked and untracked replicas is `partial`.

`GET /admin/metrics/not-found?from=&to=` lists the most frequent not-found `method + path` pairs over the whole hours that overlap the window. It needs unscoped Observability read; Dashboard read alone sees only the counts. Each hourly bucket keeps at most 50 pairs using the Space-Saving algorithm, so a client sending endless distinct paths cannot grow memory or storage. When an hour received more distinct paths than it keeps, a pair's `count` is an upper bound and `minCount` a lower bound, and the console shows a range.

Paths are redacted before they are kept. The query string and fragment are dropped. Email addresses (including `%40`) become `{email}`, numbers `{n}`, UUIDs `{uuid}`, hex strings `{hex}`, base64, base64url, JWT-like and percent-encoded values `{token}`, word-prefixed ids `{id}`, segments longer than 40 characters `{long}`, `key=value` values `{value}`, and the signed token after `/api/rc/<kind>/` `{rc-token}`. A short file extension is kept (`/{hex}.zip`). Anything outside printable ASCII becomes `_`. A path keeps at most 16 segments and 160 bytes. Methods outside the set this server answers are recorded as `OTHER`. No IP address, header, account or session is part of the table; per-address data stays with the request protection features.

These 404s are the app's own designed misses and are not counted (they still count as requests):

- `GET`/`HEAD` `/favicon.ico`, `/favicon.svg`, `/favicon.png` and `/apple-touch-icon*.png` at the root, which browsers request on their own.
- `GET /api/account/mdn-decision/{id}`: no receipt decision is recorded for the message.
- `GET /api/track/open`: no tracking pixel is bound to the message.
- `GET /api/zeroaccess/pair/envelope/{id}`: the new device polls until the envelope arrives.
- `GET {admin}/search`: the command palette's optional server search, which this server does not mount.

JMAP `unknownMethod` is a method error inside an HTTP 200 answer, so it is never a 404 and needs no exclusion. Every other 404, from any client, is counted.

Request protection does not read status classes ("never infer abuse from a generic HTTP status code"), so 404 counts are not passed to it. There is no admin alert mechanism for request history, so no 404 spike alert is offered.
