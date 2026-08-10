# Build profile + leanness (26.19)

**Status:** measured (t19-e1). Records what `.cargo/config.toml` and the root
`Cargo.toml` `[profile.*]` block are set to, the measurements that justify each
setting, and the options that were measured and **rejected** — including the
thin-LTO question `Cargo.toml` had deferred to a human since 26.9 (DQ-8).

Every number here is wall-clock on the dev host. Nothing is estimated. Where a
measurement is contaminated or does not transfer to the shipped platform, it says so.

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
