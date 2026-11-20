# Public static asset RAM cache

The cache retains only public Web/PDF/retained-build representations. Mail,
authenticated attachment bytes, API responses and final HTML containing dynamic
analytics/OIDC content are excluded. Existing edge/security middleware still
runs. The cache adds no SQL to ordinary asset requests; this does not mean the
whole request is SQL-free.

Defaults: enabled, 64 MiB owned resident payloads, 4 MiB per owned file, 1024
entries, deployed revalidation after one second, and best-effort shell preload.
The global `web.assets.cache.*` controls appear in Settings and Maintenance →
Cache. Read/clear require the existing unscoped maintenance capabilities.
Settings writes apply immediately on this server; independent writes are polled
every 30 seconds. Refresh completion is fenced by request order, so an older
periodic snapshot/error cannot overwrite a newer administrative refresh. Refresh failures retain the last applied configuration and
mark it stale in status.

Prepared `Bytes` and a content ETag are shared across responses. Already embedded
borrowed bytes are referenced directly, not copied into a second raw cache;
adapted HTML/CSS bytes are owned and charged. The entry limit also bounds metadata
for embedded references. The budget excludes response lifetimes, loader buffers,
metadata/allocator overhead, and already embedded executable data. It is not an
RSS bound. Owned files over the per-file limit still load, but bypass retention.

Public/console/raw, Web/PDF/retained source, base prefix and cache generation are
separate keys. All path/source-map exclusions and outer response/security headers
remain active. Console markers never enter public shell entries. PDF source
selection still requires the pinned embedded manifest and preserves worker/CSP
coupling, CORS, HEAD and no-cache headers.

There are two blocking loader slots and sixteen queued unique misses. Same-key
misses share one accepted job. Admission exhaustion or a five-second observer
wait returns 503 with Retry-After and a fixed asset-error header, never a fallback
HTML success. Accepted blocking work continues after an HTTP waiter disappears;
its permits remain held until actual worker completion. Ordinary filesystem
not-found/read failure retains the previous missing-asset behavior. Worker panic
or dispatcher failure is reported as busy, with a failed-worker counter.

Clear or a changed configuration increments an epoch before releasing entries.
Older accepted responses can finish, but cannot repopulate the new epoch. Active
responses can retain old bytes until completion. Preload may refill after clear.
Only one preload task exists: shell loads index.html; core additionally attempts
sw.js, manifest.webmanifest and at most sixteen scripts/styles named by the shell.
Shell discovery scans at most 256 KiB; oversized shells skip optional discovery
but remain served unchanged. It starts without delaying startup, does not queue ahead of pending foreground
work, and requires an immediately free loader slot. It is best effort, not a
promise that every core resource will fit or be present.

Deployed and retained bytes, including unhashed shell/service-worker content,
can remain stale for the configured 1–60 second cache lifetime. Their next access
reloads after expiry; this does not monitor files continuously. Embedded build
bytes are immutable for this cache instance, except that a PDF embedded fallback
with a deployed source directory expires so source eligibility can be rechecked. Deployed PDF manifest validation is
also bounded to a one-second stamp-validation lifetime and 32 source directories.

Status/clear return fixed numeric/boolean/configuration fields with no filesystem
paths, keys, cookies or cached contents. Counters are per process and reset at
restart. `loads` counts completed worker jobs, including misses and failures;
`misses` counts accepted unique loads, `coalesced` counts additional waiters,
and `loadFailures` counts failed/panicked worker jobs rather than ordinary missing
files. No throughput or memory improvement is claimed until the focused gates
and a separate controlled measurement execute.
