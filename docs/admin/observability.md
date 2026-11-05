# Observability

## Client load times

Admin > Observability > Statistics shows how long the web app takes to load in
signed-in browsers: the median (50th percentile) and the 95th percentile of
"time to usable", first contentful paint, largest contentful paint and page
load (app load); of folder loads and of message loads, each split into
"from cache" and "from the network"; a chart of each over the selected period,
and a breakdown by device class. Every card shows its sample count. It follows the same period control as the rest of the page and
needs the `observability.read` capability.

### What is collected

Once per page load, after the signed-in user's message list has shown its first
rows, the browser sends one small request, `POST /api/client-timing`, with:

- a device class, `desktop` or `mobile` (from the browser's mobile hint or a
  narrow viewport), or none when the browser cannot say;
- up to sixteen timings, each a name and a whole number of milliseconds:
  `timeToUsable` (navigation start until the message list first shows rows),
  `ttfb` (first byte), `dcl` (DOM content loaded), `load`, `fcp`, `lcp`, and
  the first release's `folderOpen`, which is no longer sent but is still shown
  for periods that contain it.

Folder and message loads are sent in the same shape, batched into one request
every few seconds (and when the page is hidden), from the moment the user
asked until the result was painted:

- `folderLoadCold` and `folderLoadCached`: the user picks a folder (or the
  mailbox) until its message list is on screen. "Cached" means rows were shown
  at once from the browser's read cache; "cold" waited for the network.
- `messageLoadCold` and `messageLoadCached`: the user opens a message until its
  headers and sanitised body are on screen, in full. "Cached" covers a message
  that was already prefetched or read; "cold" waited for the network. A
  refresh of the message already open is not a new open.

The browser also records each as a `performance.measure` (`mw:folder`,
`mw:message`) so the same figures can be seen in DevTools. Time to load images
is not measured. Only the metric name and whole milliseconds are sent: no
folder or message id or name, subject, address or folder kind.

The server adds each value to a counter and discards the request. It keeps, per
hour, per metric and per device class: a sample count, a sum, and a 48-bin
histogram. That is all.

### What is not collected

No URL or path, account, address, session, IP address, user agent string,
browser name, clock timestamp or any mail content is sent or stored. The
server records only its own receive time, rounded to the hour. There is no
per-session or per-account row, so a figure cannot be traced to a person. The
signed-in session is used only in memory, to rate limit a single browser
(at most 6 page-load beacons at once, then one every 30 seconds; at most 12
folder/message beacons at once, then one every 5 seconds); it is never stored.
An unknown metric name, an unknown field or a negative or fractional value is
refused with `400`; values above 120 s are clamped to 120 s.
Nothing is collected on the sign-in screen or any other anonymous page, and
the endpoint answers `401` without a session.

The browser sends nothing when Do Not Track or Global Privacy Control is on
(the same rule as the optional analytics tag), when its session is outside the
sample, when the server says collection is off, or when its performance APIs
are missing. Failures are silent and never block the page.

### Accuracy

Each metric is a log-scale histogram: one bin under 10 ms, then 47 bins from
10 ms to 120 000 ms, each about 22% wider than the one before. A percentile is
located by rank across the merged histograms of the period and interpolated
inside its bin, so it is within one bin of the true value: at most about 22%
off, usually under 10%. Counts and means are exact. Values above 120 s are
counted as 120 s. Figures are sampled (see `sampleRatePercent`) and come only
from browsers that stayed on the page long enough to report, so very slow
loads that were abandoned are under-represented. Percentiles are hidden for a
period or chart interval with fewer samples than the minimum (default 20).

### Settings

| Key | Default | Effect |
| --- | --- | --- |
| `observability.clientTiming.enabled` | on | Turn collection on or off. Hot reload; browsers pick the change up at their next page load, and the server ignores beacons meanwhile. |
| `observability.clientTiming.sampleRatePercent` | 100 | Share of sessions that report (1 to 100). |
| `observability.clientTiming.minSamples` | 20 | Samples needed before a percentile is shown. |
| `observability.clientTiming.folderLoads` | on | Folder load times. Global, domain and user scope: the administrator sets the default or locks it, and each person can switch it off for themselves in Settings > Privacy unless it is locked. |
| `observability.clientTiming.messageLoads` | on | Message load times. Same scopes and behaviour as `folderLoads`. |

Folder and message loads also need `observability.clientTiming.enabled`. When
either is off for a person, that browser sends nothing for it (the switch is
read live, no reload needed), and the server answers `204` and discards any
folder or message figure it still receives from that person (a stale tab, for
example). Admin > Observability notes when the instance default is off.

History is kept for `metrics.rollupDays` (default 30 days, hourly), the same
as request history. It is isolated per application instance and written
every 30 seconds, so a restart keeps it. Both SQLite and PostgreSQL store it
in `client_timing_bucket`.

### Turning it off

Set **Browser load times** to off in Admin > Settings > Observability (key
`observability.clientTiming.enabled`). The server stops recording at once and
tells browsers on their next policy read. Existing aggregates stay until they
age out of `metrics.rollupDays`.
