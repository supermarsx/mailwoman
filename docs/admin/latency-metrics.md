# Request latency precision

Live and persisted request statistics measure time until the response is ready. They do not measure the complete response transfer or browser rendering time.

New samples retain integer milliseconds from 0 through 1,000 ms. Longer durations use bounded histogram bins: 10 ms through 10 seconds, 100 ms through 60 seconds, and 1 second through 10 minutes. Above that range the observed maximum supplies the overflow estimate. Percentiles are histogram estimates; submillisecond precision is not recorded.

The live and historical APIs include `legacyLatencyEstimate`, `p50ResolutionMs`, and `p95ResolutionMs` for each point. A resolution of 1 means integer-millisecond samples; a larger value identifies the wider bin. A null resolution identifies legacy or overflow estimates. For example, new samples at 37 ms and 83 ms remain 37 ms and 83 ms, rather than being rounded to the old 50 ms and 100 ms bounds.

Persisted version 2 documents retain the original coarse histogram and add a sparse fine histogram. Older documents remain readable. A point combining old and new samples uses the compatible coarse histogram and is explicitly marked as a legacy estimate. Old measurement precision cannot be reconstructed. The administrator history page explains this limitation when displaying older or mixed data.

Each fine histogram has at most 2,942 bins. History reads limit the number of rows to 20,000, each JSON document to 128 KiB, and combined JSON bytes to 16 MiB. SQLite and PostgreSQL check these byte budgets before returning JSON documents to the application. Malformed histograms, inconsistent counts and exceeded budgets fail the history request rather than silently producing misleading statistics.
