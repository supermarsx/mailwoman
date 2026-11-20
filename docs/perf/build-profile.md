# Build profile + leanness (26.19)

**Status:** measured (t19-e1). Records what `.cargo/config.toml` and the root
`Cargo.toml` `[profile.*]` block are set to, the measurements that justify each
setting, and the options that were measured and **rejected** — including the
thin-LTO question `Cargo.toml` had deferred to a human since 26.9 (DQ-8).

Every number here is wall-clock on the dev host. Nothing is estimated. Where a
measurement is contaminated or does not transfer to the shipped platform, it says so.

## 2026-11-20: native release fixture critical path

Cargo's timing artifact from the GNU release fixture build
`f8kztuvay0167r02m0q3d99gj` records the following units. The Docker compiler step
took about 1,016 seconds with eight jobs and 16 release codegen units. Windows
metadata checks overlapped part of this build, so these are diagnostic wall-time
observations, not an isolated build-speed comparison or deployment guarantee.

| Unit | Total | Frontend | Codegen |
| --- | ---: | ---: | ---: |
| `mw-server` library | 791.5 s | 573.4 s | 218.1 s |
| `mw-engine` | 76.3 s | 18.6 s | 57.7 s |
| `mw-store` | 37.7 s | 18.1 s | 19.7 s |
| `mailwoman` binary | 52.2 s | not separated | not separated |
| Native fixture executable | 40.7 s | not separated | not separated |

Units overlap and their durations must not be summed. The library's frontend
accounts for 72% of its recorded time; linking alone does not explain this run.
Cargo's frontend category does not isolate borrow checking, macro expansion,
type checking or individual compiler queries. A more specific attribution needs
compiler profiling rather than guesses about borrowing or SIMD. Runtime
throughput also requires separate measurements.

The compile succeeded and seven WebSocket helper tests passed, but the subsequent
readiness unit failed. Its zero-duration timeout could cancel an acquired
connection in the single-connection in-memory test database. The actual Store
probe later reproduced the resulting missing-table error; this was a test-gate
failure, not a successful complete image build. The timing HTML and logs are
retained under `.orchestration/reports/perf-investigation/`.

The subsequent `m5s6anyuqejzmve6rg8attidm` build, through source revision
`72d9f20e`, completed the native release fixture image. Its compiler step took
995.8 seconds; readiness (two tests), transport (one test), and WebSocket helper
(seven tests) gates passed. Windows Cargo checks and load measurements were held
until this build finished. Different source and cache state prevent treating the
two build durations as a controlled speedup measurement.

The optional `fixture-profile` target then took 36.9 seconds, including 28.38
seconds reported by Cargo, to retain symbols in a separate executable. Its hash
gate confirmed that `/out/request-load-fixture` remained byte-identical. The
normal executable stays the default; `/out/request-load-fixture-profile` is for
diagnostic profiling. This target does not change production release settings.
Image and executable hashes, timing HTML, and gate logs are retained under
`.orchestration/reports/perf-investigation/projection-release-build/`.

## 2026-11-18: web compilation and Docker inputs

The current web build was profiled on Windows with Node 24.14.1 and pnpm
11.12.0. The following runs used the same source tree and `pnpm build:profile`,
without another compiler running. They are individual wall-clock samples, not
statistical averages or Linux deployment estimates.

| Run | Total | Module transformation | Rendering/writing | JSX cache |
|---|---:|---:|---:|---:|
| Previous pipeline (`MW_BUILD_CACHE=0 MW_CATALOG_ASSETS=0`) | 60.785 s | 48.518 s | 9.621 s | disabled |
| New pipeline, first build | 53.698 s | 46.966 s | 4.385 s | 576 misses |
| New pipeline, unchanged repeat | 37.019 s | 29.522 s | 5.054 s | 576 hits, zero misses |

The repeat build is **39.1% faster** than the previous pipeline. The first build
was **11.7% faster** in these samples. All **2,666 output files** of the first and
repeat optimized builds had identical SHA-256 hashes, including the service-worker
integrity manifest. These are samples of the final source including the new admin
screens, measured with other compilers stopped; they are not Linux deployment timings.

CPU sampling identified Babel JSX compilation and Rollup tree traversal as major
costs, with garbage collection and filesystem work also visible. Transforming the
module graph remains the largest phase after this pass.

### Lazy translation assets

Production previously emitted one executable JavaScript module per Fluent
catalog. Catalogs now use Vite's hashed, same-origin text assets and retain lazy
loading by locale and namespace. The main bundle went from **1,979 to 255 JavaScript
chunks**, with 1,726 text assets in the optimized build. English common strings
remain in the initial bundle; the full asset URL map loads with the first
noncritical catalog instead of inflating the entry. An earlier same-source comparison before the additional admin screens measured
entry gzip size decreasing from 289.6 to 275.7 KiB. The pre-existing 250 KiB size limit and literal-style
CSP checks still fail on both pipelines; this pass does not relax those gates. Development and unit tests keep their raw imports.

Text catalogs are never inlined as data URLs. They remain in the generated
integrity manifest and offline precache. Browser tests cover lazy loading below a
deployment URL prefix and loading a previously unopened locale while offline in
both Chromium and Firefox.

### Persistent production JSX transform cache

Only the pure `vite-plugin-solid` production JSX/TSX transform is cached. Vite
resolution, CSS extraction, tree shaking, minification, source maps, SBOM
generation and integrity hashing still run. Development/HMR and SSR bypass the
cache. Its key includes module identity and source, lockfile and package bytes,
Vite configuration dependencies, Node version, mode, public environment, defines,
output target and source-map mode. Cached results carry a checksum; corrupt,
missing or unwritable caches fall back to normal compilation.

The cache keeps one slot per module under
`apps/web/node_modules/.vite/mailwoman-solid`, replacing changed revisions.
Docker preserves this directory in a BuildKit cache mount. Set `MW_BUILD_CACHE=0`
to bypass it for diagnosis. `MW_CATALOG_ASSETS=0` restores the previous catalog
pipeline for an explicit comparison; it is not needed for normal builds.

### Repeatable profiling

Run from `apps/web`:

```sh
pnpm build:profile
```

The command prints phase timings, module/chunk counts, and cache hit/miss counts,
and writes `node_modules/.cache/mailwoman-build/profile.json`. It uses the real
production pipeline and does not add profiling code to shipped assets. Keep other
compilers stopped while measuring; distinguish a first cache population, a warm
rebuild, and a completely cached Docker layer. To gather a Node CPU profile:

```sh
node --cpu-prof scripts/profile-build.mjs
```

### Docker context and layer reuse

A real Docker COPY probe confirmed that the old root-only ignore rules admitted
nested browser-test output. On this workstation, `apps/web/.orchestration` alone
contained **24,366 files / 695.7 MiB**. Recursive rules now exclude fixture builds,
reports, coverage and local Vite caches. A second COPY probe confirmed the nested
marker could no longer enter the context. This is excluded input, not deleted
user data or a claimed reduction in final runtime-image size.

The web stage now copies explicit production inputs instead of the entire web
project. Browser test files are also excluded, so editing browser tests and
reports no longer invalidates a production web layer. Dependency installation
retains its manifest-only cache boundary; JSX transforms have a separate
persistent BuildKit cache.

The pinned Node 22 Alpine web stage also built successfully with hidden source
maps: Vite reported **73 s**. That run included concurrent validation and is a
compatibility result, not a comparison against the Windows timings. Exporting
that diagnostic builder image took another **52.8 s**, including unpacking; the
builder image includes development dependencies and is not the shipped runtime.

The embedded empty SPA placeholder now has a fixed timestamp. A fresh `touch`
was marking an unchanged `include_bytes!` dependency newer than Cargo's last
artifact whenever a compilation layer reran. A focused Cargo fixture reproduced
the unnecessary rebuild and verified that the fixed timestamp leaves it fresh.
This also avoids embedding a variable modification time for that empty file.

The image workflow now separately restores/saves the Cargo target/registry,
pnpm store and JSX transform cache mounts. Docker's `gha` layer exporter alone
[does not export cache mounts](https://docs.docker.com/build/ci/github-actions/cache/#cache-mounts).
The mount-transfer action and its utility image are pinned. `actionlint` and a
check of all four mappings against the Dockerfile passed; hosted cache transfer
and CI speedup have not been measured in this session. Mount restoration is an
optional acceleration: its failure does not block a normal compilation.

The default musl server build now also records Cargo's per-crate HTML timeline.
Export the most recent build invocation's report without exporting the source tree:

```sh
docker buildx build --target build-timings \
  --output type=local,dest=.orchestration/build-timings .
```

Open `.orchestration/build-timings/cargo-timing.html`. A reused compiler cache
produces a warm/no-op report; it must not be presented as a cold compilation.
The report remains outside the runtime image. The final/default Docker target is
still `runtime-static`.

The Rust release profile is unchanged. Its earlier LTO measurements below still
apply; enabling LTO would spend more build time. Rust source inputs remain
separate from the SPA, and the `rust-source` Docker stage was verified after the
ignore changes. A source-stage check alone does not prove a full runtime build.

## Measurement host and method

| | |
|---|---|
| Host | Windows 11 Pro, `x86_64-pc-windows-msvc`, 64 logical CPUs |
| Toolchain | rustc 1.95.0 / cargo 1.95.0, `rust-toolchain.toml` channel `stable` |
| Tree | `master` @ `14c7674` (tag `26.18`), `target/` **absent** at entry |
| Cold build | `rm -rf target && cargo build --workspace` |
| No-op rebuild | `cargo build --workspace` immediately after, nothing changed |
| Incremental | `touch crates/mw-server/src/lib.rs && cargo build -p mw-server` |
| Test gate sample | `cargo test -p mw-crypto -p mw-oauth -p mw-mfa -p mw-passwd -- --test-threads=1` (117 tests) |
| Release | `cargo build --release -p mw-server --bin mailwoman` (what `scripts/perf/binary-size.sh` builds) |

The baseline was recorded **before the first edit**, with no other executor running:
cargo takes an exclusive lock on the target directory, so a concurrent lane would
have serialized against these builds and invalidated every number.

## Result

| Metric | 26.18 | 26.19 | Δ |
|---|---|---|---|
| Cold workspace dev build | 802 s | **373 s** | **−53%** |
| No-op workspace rebuild | 54 s | **19 s** | **−65%** |
| Incremental `mw-server` rebuild | 465 s | **246 s** | **−47%** |
| Test-gate sample, 117 tests, `--test-threads=1` | 248 s | **50 s** | **−80%** |
| Test-gate sample, build | 66 s | 53 s | −20% |
| `Cargo.lock` packages | 1109 | **1107** | −2 |
| Crate names with >1 version | 119 | **117** | −2 |
| Surplus versions | 154 | **152** | −2 |
| `mailwoman` release binary (MSVC) | 94 815 744 B | 94 804 992 B | −0.01% |

The release binary is deliberately ~unchanged: nothing in this pass alters
`[profile.release]` (see DQ-8 below). The 10 KB delta is the dependency dedupe.

## What changed

### `.cargo/config.toml` (new)

Aliases for the canonical gates (`cargo gate`, `cargo gate-core`, `cargo lint`,
`cargo dupes`), `[net] retry = 3`, and `[future-incompat-report]`.

It contains **no `rustflags` and no linker override**, which is a deliberate choice
rather than an omission. `ci.yml` gains windows and macOS legs in this same tag;
mold / lld / sold / sccache would each have to be present on all three runners and
none is guaranteed. Setting `[build] rustflags` also changes the fingerprint of
every crate in the graph for anyone whose environment differs from CI's — which is
precisely the cache thrash this lane exists to remove. `-C target-cpu=native` is
excluded for a different reason: the release binary is a shipped artifact and must
run on hosts other than the one that built it.

Profiles are **not** set in `.cargo/config.toml`. Config-file profiles silently
override the manifest, which makes the effective settings hard to find; they live
in the root `Cargo.toml` where they are reviewable in a diff.

### `[profile.dev]` — debug info

```toml
[profile.dev]           debug = "line-tables-only"
[profile.dev.package."*"] debug = false
[profile.dev.build-override] debug = false
```

This is where the build-time win comes from, and it is larger than expected. The
workspace compiles ~850 units on this host, and `mw-server` alone links 60+ test
binaries, each of which pays the full debug-info write. Dropping the type/variable
payload while keeping line tables cut the cold build roughly in half and the no-op
fingerprint pass by two thirds.

Backtraces still resolve to `file:line`, which is what a failing test actually
prints. What is lost is variable inspection in a debugger — for dependencies,
entirely (nothing steps into them), and for workspace crates, the local-variable
view. A developer who needs that can override for one build with
`RUSTFLAGS="-Cdebuginfo=2"` or a scratch profile.

### `[profile.dev.package.<crypto>] opt-level = 3`

The canonical gate runs `--test-threads=1`, so test wall-clock is serial and was
dominated by crypto primitives executing unoptimized. Optimizing those
*dependencies* — never workspace crates, so the edit/rebuild loop stays at
opt-level 0 — took the 117-test sample from **248 s to 50 s**, and it cost nothing:
the sample's build step went *down* (66 s → 53 s), because these are small crates
and the debug-info saving more than paid for the optimization.

Scope is a curated list (hashing, AEAD, bignum, elliptic-curve, PQC, compressors),
not `"*"`. Optimizing every dependency would have moved the cold-build number in the
wrong direction for a workspace this size, and the measurement above shows the win
is concentrated in the primitives.

## DQ-8 — thin LTO: **rejected**

`Cargo.toml:344-355` had deferred this to a human since 26.9. It is now measured.
All three variants were built back-to-back on the final settings, same host, same
session, so they are mutually comparable.

| Variant | `mailwoman` build | `mailwoman` size | `mailwoman-desktop` build | `mailwoman-desktop` size |
|---|---|---|---|---|
| **A — shipped** (no LTO, default codegen-units) | **386 s** | **94 804 992 B** (90.41 MiB) | **157 s** | **15 861 760 B** (15.13 MiB) |
| B — `lto="thin"` + `codegen-units=1` | 1163 s (**+201%**) | 79 227 392 B (**−16.4%**) | 533 s (**+239%**) | 14 729 728 B (−7.1%) |
| C — `lto="thin"` only | 627 s (**+62%**) | 96 760 320 B (**+2.1%**) | not built | not built |

**Decision: reject. `[profile.release]` keeps `overflow-checks = true` + `strip = true`
and gains nothing.**

Reasoning, against DQ-8's own stated bar ("adopt only if the size win is real and
the clean-build regression is under ~25%"):

1. **Thin LTO on its own is a straight loss.** Variant C is the one the planner did
   not ask for and it is the one that settles the question: `lto="thin"` alone made
   the build 62% slower *and* the binary 2% **larger**. There is no version of "adopt
   thin LTO" that is defensible on this workspace.
2. **The size win in B is `codegen-units = 1`, not LTO.** Attributing it to LTO would
   have been the wrong lesson to write down. A future lane that genuinely needs the
   ~16% should reach for codegen-units, not LTO. Its cost was not separately priced —
   only the combined B variant was measured — so treat "codegen-units=1 alone" as
   unmeasured rather than cheap.
3. **The build-time regression is 201%, not 25%.** It misses DQ-8's bar by an order
   of magnitude. Release builds are not rare here — `packaging.yml` builds desktop
   bundles on three OSes and the Docker image build is a release build.
4. **The size win buys nothing that is currently constrained.**
   `scripts/perf/binary-size.sh` budgets 91 MB against a Linux artifact measuring
   ~79 MB; that gate is green with headroom, and this would spend ~13 extra minutes
   of CI per leg to widen headroom nobody is short of.
5. **It does not rescue the budget that *is* tight.**
   `scripts/check-bundle-size.mjs` budgets the thin desktop shell at 10 MB. Under B
   it measures 14.05 MiB here against 15.13 MiB without — still over. Adopting on
   those grounds would not have worked.
6. `panic = "abort"` was not measured and must not be set: it breaks the test
   harness and any `catch_unwind` (DQ-8, restated).

DQ-8 also asked for Δincremental-build. It is not tabulated because it is not a
meaningful axis for this decision: `[profile.release]` does not affect the dev/test
incremental loop at all, and nobody iterates on release builds.

### Caveats on these numbers

- **Platform.** Measured on `x86_64-pc-windows-msvc`. The gated, shipped artifact is
  Linux (`scripts/perf/binary-size.sh` says so explicitly, and notes MSVC builds run
  roughly 2× larger). The *direction* and rough proportion of LTO's effects transfer;
  the absolute byte counts do not.
- **The 811 s baseline release build is not a valid "before".** It was measured on
  the pre-edit tree, but `[profile.release]` is byte-identical between that run and
  variant A (386 s) — this pass changed nothing that could affect a release build.
  The gap is environmental (cold OS page cache and antivirus scanning of a freshly
  written multi-gigabyte debug tree), not a profile effect, and it is recorded here
  so nobody later quotes it as a 2× release-build improvement. It is not one.
- **The desktop-shell numbers are raw `cargo build --release -p mailwoman-desktop`,
  not the gate's own path.** `scripts/check-bundle-size.mjs` runs inside
  `scripts/build-shells.*` after `tauri build --no-bundle`. The 15.13 MiB measured
  here exceeding the script's 10 MB budget is therefore **not** a claim that the gate
  is red — it is a different build path on a different OS. It is flagged for whoever
  owns that gate; it is pre-existing and outside this lane's locks.

## Alternating `cargo build` and `cargo test` costs 18 crates

Worth knowing before someone blames the profiles for it. On a fully warm tree:

| Sequence | Recompiled | Time |
|---|---|---|
| `cargo build --workspace` twice in a row | 0 | 34 s |
| `cargo test --workspace --no-run` twice in a row | 0 | 5 s |
| alternating the two | **18 workspace crates, every time** | 160–190 s |

The 18 are `mw-crypto`, `mw-sanitize`, `mw-render`, `mw-engine`, `mw-plugin`,
`mw-imap`, `mw-pop3`, `mw-server`, the two Tauri shells and the eight plugins.
`cargo test` activates dev-dependencies, which enable additional features on shared
dependencies, so the two commands resolve different feature sets and each
invalidates the other's artifacts. This is cargo's feature unification, not a
profile setting — it predates this pass and no `[profile.*]` change affects it.

Practical consequence: pick one command and stay on it. `cargo test --workspace
--no-run` warms everything `cargo build --workspace` would have, plus the test
binaries, so there is rarely a reason to run both.

## Dependency dedupe

`Cargo.lock` went 1109 → 1107 packages and 119 → 117 duplicate crate names by
pinning `tokio-tungstenite` to the version `axum`'s `ws` feature already resolves
(0.29), so the tree compiles one `tokio-tungstenite`/`tungstenite` pair instead of
two. This is safe in a way worth stating: production `/jmap/ws` is served by axum's
own copy, already 0.29; the only in-tree use of the workspace dependency is the
WebSocket **test client** in `crates/mw-server/tests/{pim,push_hardening,security}.rs`.
No shipped network-facing code changed version.

### Three live RUSTSEC advisories closed in passing

`cargo deny check` was **failing `advisories`** on the 26.18 lock — pre-existing, and
unrelated to anything above (verified: the affected crates are untouched by the
dedupe). Because this lane is the tag's sole `Cargo.lock` owner and every other lane
is forbidden from editing it, leaving these would have meant they stayed open for the
whole tag. All three had semver-compatible fixes inside the existing requirements:

| Advisory | Crate | 26.18 | 26.19 |
|---|---|---|---|
| [RUSTSEC-2026-0213](https://rustsec.org/advisories/RUSTSEC-2026-0213) — XSS via SVG `animate`/`set` tags | `ammonia` | 4.1.3 | **4.1.4** |
| [RUSTSEC-2026-0222](https://rustsec.org/advisories/RUSTSEC-2026-0222) — stores mix up type indices between engines | `wasmtime` | 46.0.1 | **46.0.2** |
| [RUSTSEC-2026-0223](https://rustsec.org/advisories/RUSTSEC-2026-0223) — preemption/traps during bulk ops break VM state | `wasmtime` | 46.0.1 | **46.0.2** |

`cargo update -p ammonia -p wasmtime` (plus the cranelift/pulley crates that move
with wasmtime) left the package count, duplicate count and crate-name set **exactly
unchanged** — 1107 / 117 / zero new names. `cargo deny check` is now
`advisories ok, bans ok, licenses ok, sources ok`.

The `ammonia` one is the consequential one: it is the HTML sanitizer behind
`mw-sanitize`, i.e. the code path that renders untrusted message bodies.

### Measured and rejected

- **A blanket `cargo update`.** Semver-compatible bumps across the whole lock removed
  one duplicate name but took the package count 1109 → **1119** and pulled in 13 new
  crate names (`jiff`* ×5, `defmt`* ×3, `zopfli`, `zcheapstr`, …). Net-negative on
  both leanness metrics; reverted.
- **Unifying `reqwest` 0.12 → 0.13** (tauri and `tokio-rustls-acme` already pull
  0.13, so the tree builds reqwest twice). 0.13 renamed the TLS feature
  (`rustls-tls` → `rustls`) and moved `multipart`/`charset` behind features that nine
  in-tree crates would each have to re-declare. That is a functional dependency
  migration across nine crate manifests, not a build change, and those manifests are
  not this lane's locks. Deferred with the reason recorded, not silently dropped.

### The remaining 117 duplicates are structural

They are not neglect, and a future lane should not expect to grind them down from the
root manifest. Traced to their parents, the top offenders come from third-party trees
this repo does not control:

| Duplicate | Versions | Driven by |
|---|---|---|
| `windows-sys` / `windows-targets` / `windows_*_msvc` … | 6 / 4 / 4 | the GTK, tauri and misc Windows trees, independently pinned |
| `hashbrown` | 5 | `indexmap` 1.x, `ordered-multimap`, `sqlx`, and the `wasmtime`/`cranelift` tree |
| `toml_edit` / `toml` / `toml_datetime` / `winnow` | 4 / 3 / 3 / 3 | `proc-macro-crate` 1/2/3 via `gtk3-macros`, `glib-macros`, `zbus_macros` |
| `getrandom` / `rand` / `rand_core` | 3 / 3 / 3 | the RustCrypto 0.2→0.3→0.4 and rand 0.8→0.9→0.10 transitions, mid-flight upstream |
| `sha2` / `digest` / `crypto-common` | 2 each | RustCrypto `digest` 0.10 vs 0.11; in-tree crates are already on the newer side |
| `zip` | 3 | `docx-rs` (0.6), `tauri-plugin-updater` (4.6), `mail-auth` (8.6) |

One duplicate *is* in reach but not from here: the workspace pins `rand = "0.8"` while
the rest of the tree has moved to 0.9/0.10. Bumping it is an API migration across
`mw-crypto`, `mw-mfa`, `mw-oauth`, `mw-server`, `mw-sso` and `mw-store` — worth a
future lane that owns those crates, and out of scope for a build-profile pass.

## Findings handed on

- **`crates/mw-server/Cargo.toml:104` declares `tokio-tungstenite` as a production
  dependency, but no file under `crates/mw-server/src/` uses it** — every call site
  is in `tests/`. It also appears correctly in `[dev-dependencies]` at line 142.
  Moving it out of `[dependencies]` looks like a free removal from the shipped graph.
  Not done here: that manifest is not this lane's lock.
- `scripts/check-bundle-size.mjs`'s 10 MB thin-shell budget deserves the same
  measured-revision treatment `docs/perf/size-budget-revision.md` gave the server
  binary in 26.9, or a check that the Linux artifact it actually gates is under it.
  **Done in 26.20** (t24-e11): the gate's own Linux artifact measured 18.20 MB and
  the budget is now 21 MB (self-contained 126 MB) — same file, new section. The
  numbers quoted elsewhere in this document are the 26.x measurements as taken then
  and are left as written.


The production snapshot supplied during this investigation copied a 104,854,867,968-byte
SQLite file and took **1,025 seconds** just for the data copy. That is a separate
source of deployment delay from compilation. See [database maintenance](../admin/database-maintenance.md)
for measurement, repair and the explicit `--skip-backup` option. No production
repair or data deletion was performed from this workspace.

A sampled cold musl compiler was still solving nested future `Send` auto-traits
in the large server crate after dependencies finished, using several GiB of RAM.
The full release experiment was stopped without a completed runtime image; it
is not evidence of a successful image build or of a measured server speedup.
Boxed-future experiments were reverted because their release improvement was not
established. The host library/binary check passed; Rust release settings remain unchanged.

## Server crate extraction

See [server crate boundaries](server-crates.md) for the six extracted libraries,
their regression gates and controlled before/after measurement procedure.
No release build-time improvement is claimed from the extraction without that
comparison.
