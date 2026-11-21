# Native fixture alignment and prepared-mail experiment

The representative profile still contains TEN authenticated HTTP shapes. Only
its standalone `Email/get` (two explicit IDs, exact production DEFAULT36) carries
`x-mailwoman-read-shape: native-metadata-v1`. This is a declaration, not authority.
The paged query+headers HTTP keeps its four sequential query/get/query/get methods
and result references. Reader body, attachment download, identity, mailbox/query,
refresh, flag/restore and move/restore remain unchanged and unmarked. The ordinary
minimal-five-property baseline stays unmarked. Engine header batching can still
apply inside an ineligible multi-method HTTP; that does not make the whole request
eligible for the native guard or ready-response path.

Node reads the actual production canonical JSON; it does not import browser
runtime modules. The test compares the Node predicate with the actual pure client
TypeScript implementation (types stripped only in the test), and tests all ten
constructed shapes. Reports include actual constructed marked shape labels, not
an assertion of observed guard-input reuse. Actual guard consumption is unknown
unless separately instrumented/verified.

Prepared mail remains OFF by default. For a deliberate experiment set BOTH:

    MW_NATIVE_LOAD_PREPARED_MAIL=on
    MW_NATIVE_LOAD_POLICY=loaded

with the existing representative workload. Optional strict JSON:

    MW_NATIVE_LOAD_PREPARED_MAIL_CONFIG={"enabled":true,"budgetMiB":256,"maxEntries":10000,"maxAccounts":10000,"idleSeconds":30}

The fixture uses Engine option decoding/validation and the existing real locked
global admin-policy Store API; it never directly enables an Engine cache. Unknown
keys, invalid bounds, disabled-on configurations, over4096-byte JSON, or JSON with
modeoff fail. Defaults for on are the actual64MiB/256entries/128accounts/30seconds;
these must cover the SELECTED principal count at least by entry/account caps.
Ten-thousand-user experiments therefore require explicit larger caps. This is
not a promise that the byte budget fits all rows or the process RSS stays bounded
by that cache budget; evictions/charged bytes/working-set effects remain measured.

The handoff reports requested options and the actual registry-resolved policy.
Token-protected diagnostics add actual Engine prepared-mail scalar status outside
the measured request loop. Both before/after observations must match the requested
applied config for an on-profile. Null/legacy/missing observations cannot qualify.
For every phase with actual marked Email/get attempts, a positive raw ready-result
delta must be observed; leading identity-only controls report unexercised and
cannot qualify the prepared-mail experiment. ready-hit/build/seed/error
counters and resident/held-byte gauges remain visible. Counter deltas include drain
and are not accepted-in-window operation counts. Normal gauge decreases are valid.
Defaults/off and legacy fixtures do not acquire new performance qualifications.

Before handoff, the on-only fixture primes the FIRST real principal through
actual guarded DEFAULT36 HTTP (canonical marker/cookie/CSRF/source/Origin), at
most40 attempts within a cooperative10second deadline, until actual Engine
configuration matches. No direct setter is called; no global/env auth bypass
is added. The scalar startup receipt records one primed principal, attempts and
elapsed time, with zero measured requests. Ordinary campaign warmup stays
unchanged and can expire/rebuild ready entries normally. On with no observed/applied
config or zero raw results fails explicitly, rather than silently benchmarking
an ordinary path. All existing security, storage, telemetry, positive mutation and
index-publication gates remain in force. The experiment changes no method mix,
source semantics, guard, durability, SQLCipher, or actual RPS/byte definition.

The immutable perf Docker source includes both real native-read and websocket
contracts before mirroring, and the runtime harness overlay includes the Node
helper and the same actual JSON paths. No generated/private contract fork exists.
No benchmark, Rust compilation or throughput claim is made by these source changes.
