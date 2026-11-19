# mw-remote-content

Shared remote-image services extracted from the web server:

- RAM/disk cache, eviction, freshness and persistence primitives.
- Adaptive budgets, concurrency gates, per-account/message admission and negative cache.
- Image resize and format checks; upstream bytes still pass through the media jail in the host.
- Reader/cache policy models and value-to-policy conversion.
- Bounded operational history with typed host-supplied gauges.

This crate does not authenticate requests, grant message access, choose an egress
route or bypass SSRF checks. HTTP handlers, signed references, authorisation,
origin fetching and stale revalidation remain in `mw-server`.

`history::ensure_sampler` receives a function returning a `Sample`; the server
adapter assembles its external gauges using `HostCounters`. Cache lookup timing
uses `mw-observability` rather than depending on the web server. The server's
legacy module paths re-export these concrete types during extraction.
