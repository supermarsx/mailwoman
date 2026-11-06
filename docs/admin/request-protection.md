# Request limits and leeway

Every limit that can answer a request with `429 Too Many Requests` (or a limiter's `503`), what it is keyed by, its default, who is exempt, what the response carries, and the setting that changes it. Defaults below are the ones in the code at this release; `settings.md` has each key's full text.

Two rules hold for all of them:

* **Every refusal paces the client.** A `429`, and a `503` from a limiter, carries `Retry-After` (whole seconds). Refusals from the concurrency gates and the per-address or per-session budgets also carry `RateLimit-Limit`, `RateLimit-Remaining: 0` and `RateLimit-Reset` (the draft `ratelimit-headers` fields). A refusal that does not name itself still gets `Retry-After` (1 s) from a response layer at the edge of the router.
* **A `429` is never evidence of abuse.** Adaptive request protection records only typed anomalies (malformed or oversized request, rejected credential, probe path, bot user-agent); a status code is not a signal, so a client that was refused never raises the penalty against itself.

## Where signed-in clients get leeway

| Mechanism | What it does | Setting |
| --- | --- | --- |
| Account, not address | A request with a valid mailbox session is charged to its account (or, in proxy mode, to the verified upstream account), never to the address it came from, so colleagues behind one NAT, or a phone on carrier-grade NAT, do not share one budget. Sign-in, step-up, password and recovery routes always stay on the address budget. | `security.protection.accountRequests` |
| Known source | A signed-in mailbox used from a source that has signed in successfully before gets `ratelimit.trustedMultiplier` percent of its account allowance (default 150). Anonymous, administrator and user-agent budgets are not scaled by it. | `ratelimit.trustedMultiplier`, `security.trust.knownTtlDays` |
| Queue before refusing | JMAP concurrency, image decoders, preview workers, delivery-status parsing and the package refresh wait up to `ratelimit.queueMs` for a slot, in arrival order, before they refuse. | `ratelimit.queueMs` |
| Per account, with a process ceiling | JMAP API requests are limited per account (8 at once) and per process (256), not by one process-wide gate of four shared by every user. | `ratelimit.jmap.perAccountConcurrent`, `ratelimit.jmap.globalConcurrent` |
| Larger bursts | Client timing reports are keyed by session with a burst of 60; the anonymous cookie-consent routes allow an address a burst of 60. | `ratelimit.session.*`, `ratelimit.anon.*` |
| Streams apart | `/jmap/eventsource` streams have their own pool (the connection limit), so open tabs cannot use up the capacity for the requests they make. | `MW_HTTP_MAX_CONNECTIONS` |
| Client pacing | The web app waits out a `Retry-After`, retries safe requests (see the last section), holds prefetch back while the server asks for time, and reports its counts. | none |

## Every limiter

"Anonymous" means no valid session. Per replica means in-process state that resets on restart; the request-protection budgets are in the settings store and count across replicas.

| Limiter | Scope key | Default rate, burst, window | Exempt | Response | Setting keys |
| --- | --- | --- | --- | --- | --- |
| Aggregate: anonymous address (`client`) | Source address (IPv6 as given by the trusted proxy resolver), sign-in routes always | Off by default. Enforced: 600 per 60 s fixed window; unknown source 100 %, known source 200 % | `security.protection.exemptClients` (CIDRs); signed-in requests to non-sign-in routes (they use the account budget); verified image and inline-part downloads of an admitted batch; `/healthz`, assets, logout, `public-config` | `429` JSON `requestProtectionLimited`, `retryAfterSecs`, `Retry-After`, `Cache-Control: private, no-store` | `security.protection.rateMode`, `clientRequests`, `rateWindowSecs`, `exemptClients`; `security.trust.unknownPercent`, `knownPercent` |
| Aggregate: account | Mailbox account id across its sessions and replicas | Off by default. Enforced: 1800 per 60 s, 150 % from a known source | `exemptAccounts`; display subresources | as above | `accountRequests`, `exemptAccounts`, `ratelimit.trustedMultiplier` |
| Aggregate: administrator | Console administrator | Off by default. Enforced: 1200 per 60 s | `exemptAdmins` | as above | `adminRequests`, `exemptAdmins` |
| Aggregate: user-agent classes | Source address and claimed class (`missing`, `uncommon`, automation, search, AI) | Enforced when `userAgentMode` is on: 600 weak, 300 crawler per 60 s; anonymous only unless configured | `userAgentExceptions`; sessions unless `userAgentScope` says otherwise; a class set to `allow` in `security.trust.ua*` | as above; a class set to `block` answers `403` to anonymous requests only | `security.protection.userAgent*`, `security.trust.ua*` |
| Adaptive subjects | Client, account, administrator | Throttle 3 s on first threshold crossing, block 300 s on a later one, decay 24 h | Same exemptions; never triggered by a `429` | `429` with `retryAfterSecs` | `security.protection.adaptive*`, `short*`, `long*`, `throttleSecs`, `blockSecs`, `decaySecs`, `signals` |
| Recognized bot user-agent (first request) | Source address | Blocks declared bot classes on first request, when enabled | A request with a valid session is limited, not blocked; Playwright's default `HeadlessChrome` user-agent is flagged on purpose | `403` `automatedClientBlocked`, or the budget's `429` | `security.protection.userAgentBlockBots`, `userAgentCategories` |
| Baseline warming up | Subject | The normal-traffic baseline is learned per minute bucket; no ratio decision before `minimumSamples` | All real browsers: it never refuses by itself, only an absolute anomaly count can | none while warming | `shortMinimumSamples`, `longMinimumSamples`, `normalCapPerBucket` |
| Admin probe guard | Source address | Per `security.adminProbe.*` policy (default blocks a source after probing for the console) | Operators with a console session; assets | Configured status (`429` default) | `security.adminProbe.*` |
| Protocol guard | Request | Framing, Host, target, header and body rules | `/healthz` | `400`/`405`/`413`/`414`/`417`/`431`, never `429` | `security.protocol.*` |
| JMAP API concurrency | Account id; process | 8 per account, 256 per process, wait 3 s | Proxy mode (upstream limits apply) | `429` problem `limit: maxConcurrentRequests`, `Retry-After`, `RateLimit-*` | `ratelimit.jmap.perAccountConcurrent`, `globalConcurrent`, `queueMs` |
| JMAP blob upload concurrency | Account id; process | 4 per account (half the request value, never below 4), 128 per process, wait 3 s | Proxy mode | `429` problem `limit: maxConcurrentUpload` | same |
| Image proxy fetch rate | Account id (per replica) | Auto-sized: burst 64 to 2048, refill 300 to 6000 per minute, burst never below the message's image count; waits for a token inside the queue deadline | Cache hits; negative-cache answers | `429` plain text, `Retry-After: 5`, `x-mw-remote-retryable: 1` | `privacy.imageRate*`, `privacy.ratePerMin` (1200), `privacy.rateBurst` (1024) |
| Image proxy concurrency | Message, account, process | 30 per message, 128 per account, 128 per process, queue of 2048, wait up to `privacy.fetchQueueTimeoutMs` | Cached images | `503` `image proxy busy`, `Retry-After: 5`, `Cache-Control: no-store`, `x-mw-remote-retryable: 1` | `privacy.maxConcurrent*`, `privacy.maxQueuedRequests`, `privacy.fetchQueueTimeoutMs` |
| Image proxy ungranted request | Account id | Same bucket, no waiting | n/a | `429` plain text | `privacy.rateBurst`, `ratePerMin` |
| Inline part and image download | Account id | Fixed bucket 1024 burst, 1200 per minute, no waiting | n/a | `429` plain text | `privacy.rateBurst`, `ratePerMin` |
| `/api/sanitize` | Process | Parallelism of the CPU count (2 to 8); callers wait, no refusal | n/a | none | none |
| Contact photo decoder | Process | 2 decoders, wait 3 s | n/a | `429`, `Retry-After`, `RateLimit-Limit: 2` | `ratelimit.queueMs` |
| Attachment preview workers | Process | 2 workers, wait 3 s | n/a | `429` JSON, `Retry-After` | `ratelimit.queueMs` |
| Delivery-status parse | Process | 2 workers, wait 3 s | n/a | `429` | `ratelimit.queueMs` |
| Package refresh | Process (one slot) and package cooldown | 1 refresh at a time, wait 3 s; per package `refresh_secs` | n/a | `429` `packageBusy`, `packageCooldown` | `ratelimit.queueMs` |
| Client timing reports | Session id | Burst 60, 30 per minute | Collection off answers `204` | `429`, `Retry-After` (seconds to one token), `RateLimit-Limit` | `ratelimit.session.burst`, `perMinute` |
| Cookie-consent routes (anonymous) | Source address | Burst 60, 30 per minute | n/a | `429`, `Retry-After`, `RateLimit-Limit` | `ratelimit.anon.burst`, `perMinute` |
| Analytics events | Process queue | Bounded queue | n/a | `429` when full | none |
| Sign-in source lock | Source address and surface | Locks after repeated rejected credentials | none (stays strict) | `429` `signinLocked` | see [signin-locks.md](signin-locks.md) |
| Sign-in cooldown | Source network | 3 s after an attempt that does not end in a session; scaled by trust tier | none | `429` `signinCooldown`, `retryAfterSecs` | `signin.retryCooldownSecs` |
| Sign-in challenge | Source | Burst 2, one per 3 s | none | `429` | `signin.antiBot*` |
| Step-up lockout | Administrator or account | Locks after repeated wrong confirmations | none | `429` `stepUpLocked`, `retryAfter` | see [security-center.md](security-center.md) |
| Linked-mailbox renewal | Owner and link | Per attempt window | none | `429` `linkedRenewLimit`, `Retry-After` | none |
| Second factor | Account | Attempt limit | none | `429` | see [security-center.md](security-center.md) |
| Discovery (`/api/discover`) | Source address; process | Burst 20, 30 per minute per source; process burst 100, 300 per minute; scaled by trust tier | none | `429`, `Retry-After` | none |
| Open-tracking pixel | Source address | Burst 10, one per 6 s | none | `429` | none |
| OIDC token and revoke | Source address; process | Burst 30, 1 per second per source; 32 in flight | none | `429` `temporarily_unavailable` | none |
| Client access key | Source address; hashing workers | Sign-in guard plus a hashing pool | none | `429`, `Retry-After: 1` | see [client-access.md](client-access.md) |
| Data requests | Account | `dsr.rateLimit.perDay`; wrong-code attempts | none | `429` `rateLimited`, `tooManyTries` | `dsr.rateLimit.perDay` |
| Diagnostics budget | Admin session id | Burst 10, 10 per minute | Admins only | `429` `rate_limited`, `retryAfterSecs` | `rateLimit.diagnosticsBurst`, `diagnosticsPerMinute` |
| SSH probes | Administrator | 6 per minute | Admins only | `429` `probeRateLimited` | none |
| Archive import | Process | 2 at a time | Admins only | `429`, `Retry-After: 5` | none |
| Assistant | Account | `assistant.rateLimitPerMin` | 0 = none | `429` | `assistant.rateLimitPerMin` |
| Server request capacity | Process | 256 requests in flight (streams excluded), 1024 connections | `/jmap/eventsource` streams (own pool) | `503`, `Retry-After: 1` | `MW_HTTP_MAX_REQUESTS`, `MW_HTTP_MAX_CONNECTIONS` |
| Static assets | none | No limit; assets are outside request protection | all | none | none |
| WebSocket and SSE streams | Connection | No per-stream rate; bounded by the connection limit | all | none | `MW_HTTP_MAX_CONNECTIONS` |

## What a legitimate session triggers, and what was refusing it

The web app on load runs a handful of JMAP calls, `/api/me`, `/api/policy` and the settings reads, then a folder list, then for the selected folder prefetches the next messages and, when idle, the other folders. Opening a newsletter loads its images through the image proxy (the reader signs a batch, then each image is one fetch), contact photos through `/api/account/contact-photo`, the cookie banner calls `/api/cookies/*`, and one timing report is sent per page load plus one per few seconds of interactions. A second tab doubles all of it; an office behind one address multiplies it by the staff.

Before this change, under default settings, the refusals a normal session met were:

1. **JMAP concurrency.** One process-wide gate of four requests, taken with `try_acquire`, so the fifth simultaneous request from anyone answered `429 maxConcurrentRequests`. A reader loading, a prefetch, a send and a second tab reached it; so did two users at once. This was the main cause.
2. **Decoder pools of two** (contact photos, attachment preview, delivery status) refused instead of waiting; a message list full of avatars exceeded it.
3. **Timing reports**: six in a row, then one per 30 s per session, so a person opening folders and messages got `429` on every report after the sixth.
4. **Cookie-consent routes**: 20 per address, 12 per minute after that, shared by everyone behind one address.
5. **Streams counted as requests**: `/jmap/eventsource` held one of the 256 request permits per open tab, so a large deployment's open tabs could use them all and every other request got `503`.
6. **Package refresh**: one slot for the whole server, refused instead of waiting.

The aggregate request-protection budgets are off by default and, when enforced, charge a signed-in session to its account (1800 per minute), so they did not cause the above; the image proxy already queues and sizes its budget from the message. The `t106_rate_leeway` integration tests reproduce item 1 with `ratelimit.jmap.perAccountConcurrent=4`, `globalConcurrent=4`, `queueMs=0` and show a sixty-request burst passing under the defaults.

## Seeing refusals

`GET {console}/bans/protection` returns `limiters`: per class the counts allowed, queued (had to wait for a slot) and rejected since the server started, the last 100 refusals (time, status, class, reason, key type, seconds to wait; never an address, an account or a path), and what the web clients reported about themselves through the timing report (`rateLimited`, `retries`, `gaveUp`). Counters are per process and reset on restart.

## What the web app does on a 429

`apps/web/src/api/rateAware.ts`, applied to every request through `apiFetch`:

* A `429`, or a `503` that names a `Retry-After`, on a request that is safe to send twice (GET, a JMAP batch of only `get`/`query`/`changes`/`queryChanges` calls, `/api/sanitize`) is sent again after the `Retry-After` plus up to 250 ms of jitter, or an exponential delay with full jitter (500 ms, 1 s, 2 s), at most three times and at most ten seconds of waiting in all. A wait that does not fit the budget is not started.
* Writes are never repeated by this layer. A JMAP `limit` refusal ran none of its calls, and `client.ts` repeats it.
* Background work (prefetch of neighbouring messages and folders) shares six slots and does not start while a `Retry-After` window is open; user-initiated requests are sent at once.
* Identical GETs in flight with no abort signal are sent once.
* Nothing is shown for a request that succeeds after waiting. When a safe request has used its retries, one toast (`common-rate-limited`) says the server is busy, with a countdown, and goes away on the next success.
* The counts go to the server in the next timing report.

## Trade-offs

* A queue holds a connection and a task for up to `ratelimit.queueMs` per waiting request; the process ceiling and the connection limit still bound the total. Set `ratelimit.queueMs` to 0 for the old immediate refusal.
* The per-account gates mean one account can use 8 of the process's 256 slots. A single compromised account is limited to that, and to its own request budget when aggregate protection is enforced.
* The known-source multiplier trusts an address that has signed in successfully (inside `security.trust.knownTtlDays`); an attacker on that address with a stolen session gets 150 % of one mailbox's allowance, not more.
* The anonymous consent budget is larger (60 in a row, 30 per minute per address). It can only record or list cookie-consent entries and cannot sign in or reach a mailbox.
* The knobs apply within about fifteen seconds of a change, per replica.
