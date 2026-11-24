# Public static asset RAM cache

The cache retains only public Web/PDF/retained-build representations. Mail,
authenticated attachment bytes, API responses and final HTML containing dynamic
analytics/OIDC content are excluded. Existing edge/security middleware still
runs. The cache adds no SQL to ordinary asset requests; this does not mean the
whole request is SQL-free.

Defaults: enabled, 128 MiB owned resident payloads, 8 MiB per owned file, 4096
entries, one-second revalidation of deployed unhashed files, and the "app"
preload. The 8 MiB file limit covers the current `mw_crypto` WebAssembly module
(6.6 MB raw), so it is retained and served with its Brotli/gzip sidecar instead
of as identity bytes read from disk on every request.
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

There are two blocking loader slots and sixteen dispatcher slots for unique
misses. Same-key misses share one accepted job. A unique miss that finds all
sixteen slots taken waits for one; one five-second deadline covers that wait and
the load. Only when it passes does the request get 503 with Retry-After and a
fixed asset-error header, never a fallback HTML success. Status reports
`admissionWaiting` and the `admissionWaits` counter. Accepted blocking work continues after an HTTP waiter disappears;
its permits remain held until actual worker completion. Ordinary filesystem
not-found/read failure retains the previous missing-asset behavior. Worker panic
or dispatcher failure is reported as busy, with a failed-worker counter.

Clear or a changed configuration increments an epoch before releasing entries.
Older accepted responses can finish, but cannot repopulate the new epoch. Active
responses can retain old bytes until completion. Preload may refill after clear.
Only one preload task exists. Modes: `off`; `shell` loads index.html; `core`
additionally attempts sw.js, manifest.webmanifest and at most sixteen
scripts/styles named by the shell (discovery scans at most 256 KiB of shell);
`app` (default) loads index.html, sw.js, manifest.webmanifest, then every
content-hashed `.js`, `.mjs`, `.css` and `.wasm` file under `assets/` (entry
chunks, lazy chunks, worker scripts, both WebAssembly modules), then the `en`
app translation pack (`assets/catalog.en.app-<hash>.json`). The server has no
default-locale setting, so only `en` (the build's source locale) is preloaded;
the other locales' packs, the admin packs, fonts, images, `font-licenses/`,
pdf.js and any unhashed file other than the three named fill on first use.

The `app` list comes from the build itself: `MW_WEB_DIR` is walked on a blocking
thread (symbolic links and Windows junctions/reparse points are not followed or
listed, names the request path guard refuses are skipped, depth ≤ 8, ≤ 50,000
entries); an embedded build uses its embedded file list. Each preload load waits
until no foreground miss is pending or waiting and a loader slot is free, then
uses one slot, so a foreground miss always has the other. The pass stops early
(state `limited`) when resident owned bytes reach three quarters of the budget
or entries reach three quarters of `maxEntries`, so preloading never evicts.
Preload starts without delaying startup. `preloadProgress` in status reports the
generation, mode, state (`idle`, `running`, `done`, `limited`, `superseded`),
planned, ready, absent and skipped counts, and the number of translation packs planned. A
clear, configuration change or shell change starts a new pass.

Deployed (`MW_WEB_DIR`) files with a content-hashed name (`is_content_hashed`,
the same rule that selects the `immutable` Cache-Control header) are kept without
expiry, like embedded bytes: a new build has new names. Other deployed files,
including the shell and service worker, expire after the configured 1–60 seconds.
On expiry, the loader compares the file's length and modification time with the
values taken before the previous read; if both are unchanged the entry is kept
without re-reading, hashing or re-verifying sidecars (`revalidatedUnchanged`).
A rewrite that keeps both length and modification time is not detected until a
clear. When a load of the deployed `index.html` that started in the current
generation finds different content from the previous load, the generation is
incremented as for a clear (`shellChanges`): the previous build's entries are
dropped and the preload runs again. A load that started before a clear or
another generation change cannot trigger or undo this. Until the shell is next
requested, hashed files of the previous build stay resident; a name the new build
no longer has is then a miss and falls through to the retained-build store.
Retained-build bytes still expire after the configured lifetime. This does not
monitor files continuously. Embedded build
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
