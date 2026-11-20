# Web delivery and translation controls

Admin Settings exposes seven global controls. These are public performance hints;
they confer no authentication, feature, rendering, sanitizer or offline-data authority.

| Key | Default | Bound |
| --- | --- | --- |
| `web.i18n.retainedLocales` | 3 | 2–8 parsed locale bundles |
| `web.i18n.concurrentLoads` | 4 | 1–8 active catalog jobs |
| `web.prefetch.enabled` | true | boolean |
| `web.prefetch.idleDelayMs` | 2500 | 0–30000 milliseconds |
| `web.offline.precacheMode` | all | all / core |
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
an admin save does not reinstall a live worker or flush browser caches. `all`
preserves the existing full prewarm asset set. `core` prewarms the same fixed build
list except `.ftl` catalogs and `/pdfjs/` resources; unopened translations/PDF
features may need a connection on first use. Previously demanded hashed resources
continue to be integrity-checked and cached normally. SaveData chooses `core` when
enabled and reported by the worker's browser. If public configuration is unavailable,
the worker retains the historical all/four-download defaults (and respects SaveData
when available). Neither policy supplies URLs or relaxes digest checks, logout
cleanup, navigation freshness or the exclusion of private mailbox data.

Lazy Compose/Settings splitting and response compression are separate follow-ups;
there are no controls pretending those changes have been implemented. Build-cache
transform times are not browser startup measurements. Bundle sizes, real cold/warm
navigation, language switching, offline installation/update and CSP/sanitizer
behavior must be verified before claiming a delivery speed improvement.
