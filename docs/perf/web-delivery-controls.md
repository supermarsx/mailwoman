# Web delivery and translation controls

Admin Settings exposes seven global controls. These are public performance hints;
they confer no authentication, feature, rendering, sanitizer or offline-data authority.

| Key | Default | Bound |
| --- | --- | --- |
| `web.i18n.retainedLocales` | 3 | 2–8 parsed locale bundles |
| `web.i18n.concurrentLoads` | 4 | 1–8 active catalog jobs |
| `web.prefetch.enabled` | true | boolean |
| `web.prefetch.idleDelayMs` | 2500 | 0–30000 milliseconds |
| `web.offline.precacheMode` | app | app / all (core = app) |
| `web.offline.precacheConcurrency` | 4 | 1–4 install downloads |
| `web.offline.respectSaveData` | true | boolean |

The existing Fluent registry already reuses parsed translations in RAM. The
bounded registry retains the active locale and English fallback, plus least
recently used inactive locales up to the configured count. This is a locale-count
bound, not a hard parser/JavaScript heap limit. Development raw-import module
caches are outside this registry. Eviction removes matching completed-load state;
the finite touched-module set survives so switching languages reloads the right
catalogs. Per-locale generations discard obsolete in-flight results. Active-chain
jobs take priority over inactive jobs. Each built catalog key has at most one
pending job, so queue cardinality is bounded by the build manifest. A failed load
does not become a successful cache entry and can be requested again.

Fresh public configuration is fetched with a five-second abort signal on App
startup without delaying rendering or demanded catalog loads. A visible document
refreshes it after returning to the foreground at least 30 seconds later; changing
a web setting in the current admin page invalidates any pre-commit pending result
and requests at most one fresh follow-up. Newer commits supersede older callers.
No stale SWR
configuration is used by this bootstrap. If it is unavailable, speculative page
work is skipped; ordinary navigation and default-bounded translations continue.
Reducing concurrency permits current jobs to finish before starting more jobs.

The prefetch switch covers speculative mailbox restoration and idle modules and
catalogs. Demanded lazy imports remain available. Idle delay begins after an
authenticated mail session and fresh performance configuration, not a measured
first-list paint. Existing SaveData/slow-network suppression of page prefetch stays.

Worker prewarm controls are read freshly at the next service-worker installation;
an admin save does not reinstall a live worker or flush browser caches. `app` (the
default, t52w) installs the build's app tier (apps/web/src/sw/offlineTiers.ts): the
entry with vendor-core and core, the mailbox and its panes, the UI-plugin tier,
compose with the editor, sign-in, their stylesheets, the boot script, the network,
calendar, sanitizer and crypto workers, the pbkdf2 helper and the sanitizer's wasm.
Measured on 2026-11-24 at the t52w-e5 build: 29 files, 5.3 MB raw, 1.56 MB brotli.
`all` installs every script, stylesheet, wasm module, app translation pack and
PDF.js file: 459 files, 33.0 MB raw at the same build. The former value `core` is
read as `app`. SaveData chooses `app` when `web.offline.respectSaveData` is on and
the worker's browser reports it. If public configuration is unavailable, the worker
installs `app` with four downloads. Not installed by `app`, and stored on first use
(cache-first, digest-checked): Settings, PIM, admin, viewers, dialogs, brand marks,
fonts, translation packs, PDF.js and the mw_crypto wasm. After the mailbox is up and
`web.prefetch.idleDelayMs` has passed, the page downloads the active locale's app
pack and the Settings and PIM files, one file at a time (src/boot/idleFill.ts), when
`web.prefetch.enabled` is on and the browser does not ask to save data; the admin
pack is never fetched ahead. Neither policy supplies URLs or relaxes digest checks,
logout cleanup, navigation freshness or the exclusion of private mailbox data.

Lazy Compose/Settings splitting and response compression are separate follow-ups;
there are no controls pretending those changes have been implemented. Build-cache
transform times are not browser startup measurements. Bundle sizes, real cold/warm
navigation, language switching, offline installation/update and CSP/sanitizer
behavior must be verified before claiming a delivery speed improvement.
