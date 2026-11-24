# Public static compression

This source candidate negotiates gzip and Brotli only for public static JS/MJS,
CSS, Fluent catalogs, JSON, SVG and WebAssembly. It does not compress mail/API
responses, streams, private console HTML or dynamic documents. Fonts, raster
images and archives are left alone. No generic response-compression middleware
is added and no ordinary mail SQL path changes.

The Vite post-build hook runs after raw SRI and service-worker digests are final.
It emits deterministic gzip level 6 and Brotli quality 5/window 22 sidecars when at
least 5% and 128 bytes smaller. Eligible source limits are 16 MiB/file, 256 MiB total,
8192 files and 20000 traversed entries. Source reads are exact-size bounded;
changed sidecars are replaced atomically, while identical bytes keep their existing mtimes. Symlinks are refused. Sidecars are not added to
SW precache/digest lists and do not create extra browser requests. This increases
artifact files/disk bytes; it is separate from reducing JS/CSS chunk count.

The server verifies a sidecar by bounded streaming decompression and exact
comparison against the already-adapted raw representation, including complete
encoded consumption. Names, mtimes or unsigned metadata alone are not proof.
A stale/corrupt/missing sidecar falls back to bounded generation on a cache miss,
including prefix-adapted CSS. Both codec paths run only inside the existing two
blocking workers and sixteen dispatcher slots for unique loads. Files above the
per-file limit (default 8 MiB) are served identity and not retained. Hits share cached Bytes and
validators without compressing, decoding or hashing again. Codec scratch,
in-flight bodies and embedded immutable bytes are not a hard process-RSS budget.
A cooperative HTTP wait timeout cannot preempt an OS read or a running codec.

Identity and encoded variants have separate strong validators over their actual
served bytes. Eligible responses vary by Accept-Encoding, including 304; HEAD
uses the selected representation length with no body. Quality exclusions,
wildcards and identity are respected; an unavailable representation with
identity forbidden returns 406. Private HTML/source-map/CSP guards and PDF
library CORS/revalidation remain. `.ftl` is `text/plain; charset=utf-8`.

Three global controls apply through the existing public-cache policy refresh:
`web.assets.cache.compression.enabled`, `.brotli`, `.minBytes` (256–65536,
default 1024). Encoding preparation also requires the RAM cache to be enabled;
with it disabled, identity is served (or406 if identity is excluded), avoiding
per-request codec work without retention. Policy/clear generations invalidate all variants together. Owned
encoded payloads are charged to the same resident budget as raw owned payloads;
embedded sidecars can remain borrowed. Status separately reports encoded payload
bytes/owned bytes and verified-sidecar/generated-encoding load counts, not bytes
saved on the network. All 14 admin locale catalogs contain the new keys.

## Actual HAR evidence and limits

The private 224-entry HAR audit retained only public asset paths and numeric
metadata in `.orchestration/reports/perf-investigation/web-delivery-har-v1/summary.json`
(SHA256 `4dd43ce8e6030397b1039d309f5727185d6bd17ddaaa740b2d420a04a8a1315a`).
All 190 static responses lacked Content-Encoding; public HTML already used gzip
behind OpenResty. All 190 asset requests explicitly sent no-cache/Pragma:no-cache,
so the capture does not establish failure of normal immutable browser caching.
Firefox's bodySize includes headers here; HTML content.size 1522 differs from its
8934 decoded bytes, so these fields are not treated as direct wire-byte counters.

Captured JS+CSS raw 4,647,624 bytes encoded offline to 1,394,602 with gzip 6 or
1,283,523 with Brotli 5. Five ambiguous FTL captures were excluded from compression
estimates. These are possible payload sizes, not observed savings or measured
latency gains. No current-source web build/browser/HTTP gate has executed for
this candidate. The old local dist predates current lazy-pane source and lacks a
matching bundle graph. Current size/CSP/i18n gates remain mandatory; compression
does not reduce decoded JS parse work or waive the 250 KiB entry budget.

## Authored verification

New Rust codec/HTTP tests cover strict negotiation, malformed/zero quality,
complete sidecar proof/truncation/trailing data, regenerated corrupt sidecars,
output budgets, encoding-specific validators/304/406 and text catalogs. Existing
real-router test now proves gzip GET/HEAD/304 and identical decoded public bytes.
A cache test proves prefix-adapted CSS generation, hit reuse and disable/clear;
a PDF test checks compressed HEAD length and existing CORS/no-cache headers.

Four new web build-hook tests cover deterministic decoded equivalence, skipped
HTML/maps/binary/tiny inputs, real raw SRI/SW-digest ordering and source bounds.
The admin card gains a status/control regression. Tests are source-authored only
until the root-owned compiler/runtime lane runs them. A real browser check of
SRI plus the service-worker cached decoded response remains necessary before
claiming end-to-end delivery qualification.

Negotiation follows the acceptance rules in [RFC9110 section12.5.3](https://www.rfc-editor.org/rfc/rfc9110.html#section-12.5.3): explicit zero-quality exclusions are retained, and identity remains the fallback unless excluded. Explicit identity weights compete with requested encodings; an omitted identity does not override the client’s positive coding preferences.
