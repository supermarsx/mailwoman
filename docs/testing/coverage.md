# Coverage measurement and the ratchet

## What 26.19 ships, and what it does not

SPEC §25 says:

> Coverage floor 80% on protocol/crypto crates; mutation testing on `mw-crypto`,
> `mw-sanitize`, `mw-export`.

Before 26.19 the repository had **no coverage tooling of any kind** — no
`cargo-llvm-cov`, no `tarpaulin`, no `grcov`, no `vitest --coverage`, no
`cargo-mutants`, and nothing in any workflow. Every release note up to and
including 26.18 reported raw *test counts* ("1482 Rust tests, 0 failed"). A test
count is not a coverage figure and was never what §25 asked for.

**26.19 ships the measurement and the ratchet that stops it sliding backwards.
It does not ship the 80% floor.** The 80% figure is carried in
`.github/coverage-floors.toml` as a `target` per crate: reported on every run,
enforced by nothing. Saying otherwise would be exactly the kind of over-claim
that previous audits of this repo caught.

Concretely, as shipped:

| half | state |
|---|---|
| web | measured, floors recorded, **gating** |
| Rust | measured and reported every run; floors recorded from the first `master` run — see [Populating the Rust baseline](#populating-the-rust-baseline) |

The distance between the measured baseline and the target is visible in every
run's job summary. Closing it is ongoing work, not a shipped state.

## The ratchet rule

One rule, and it is the whole design:

> A floor may only move **up**. CI fails when a unit drops **below** its
> recorded floor. CI never fails for being below `target`.

Rationale: a hard 80% on day one turns the pipeline red on an unknown number of
crates for reasons unrelated to the change under review, and a pipeline that is
red by default is a pipeline everyone learns to ignore. A ratchet fails only
when *this change* made things worse, which is a signal people act on.

Mechanically:

- `.github/coverage-floors.toml` holds one `floor` per unit, measured, checked in.
- `.github/workflows/coverage.yml` measures, then compares.
- A small `tolerance_pp` (see `[meta]`) absorbs sub-point jitter so that a
  reordered test does not redden CI.
- **Raising a floor** is a normal part of any change that adds tests: re-run,
  read the new number off the job summary, write it into the file. Do this
  deliberately — the file is the record of what has been earned.
- **Lowering a floor** is allowed but must be a visible, reviewed edit with a
  written reason in the same commit. There is no automatic decay.
- A crate that is recorded in the file but **absent from the report** fails the
  gate. Silently losing a measurement is treated the same as losing coverage.

## Running it locally

Rust — one instrumented pass over the whole workspace:

```sh
cargo llvm-cov --workspace \
  --exclude mailwoman-desktop --exclude mailwoman-mobile \
  --no-report --no-fail-fast -- --test-threads=1

cargo llvm-cov report --workspace \
  --exclude mailwoman-desktop --exclude mailwoman-mobile \
  --ignore-filename-regex '[\\/](tests|benches|examples|fuzz)[\\/]|[\\/]build\.rs$'
```

The ignore regex accepts both path separators so the same command gives the same
denominator on Windows and on Linux.

`cargo-llvm-cov` is a **CI/dev binary**, installed with
`cargo install cargo-llvm-cov` locally and `taiki-e/install-action` in CI. It is
never a workspace dependency: `Cargo.lock` is untouched and the shipped binaries
gain nothing. It needs the `llvm-tools-preview` rustup component. It builds into
`target/llvm-cov-target/`, so it does not evict the normal `target/` cache.

Web:

```sh
pnpm -C apps/web test:coverage
```

`@vitest/coverage-v8` is a devDependency only and never enters the shipped
bundle. Reports land in `apps/web/test-results/coverage/` — that path is chosen
because `test-results/` is already gitignored repo-wide, so no `.gitignore`
change was needed and the reports never show up as untracked files.

Coverage thresholds are deliberately **not** configured in
`apps/web/vite.config.ts`. `.github/coverage-floors.toml` is the single ratchet
for Rust and web alike; the workflow reads `coverage-summary.json` and compares.
Two places to edit a floor would guarantee they disagree.

## Why `--test-threads=1`

A whole-workspace instrumented pass is exactly the shape that trips this repo's
known `_sqlx_migrations` cross-binary flake, and `--test-threads=1` is already
the canonical release gate (see `VERSIONING.md` / the 26.18 notes). Running
single-threaded here is slower but correct, and it means the coverage workflow
does not depend on the separate test-isolation work landing first.

It may be relaxed to default parallelism only after a clean 3/3 default-parallel
workspace run, and the single-threaded form stays the release gate regardless.

## What is measured, and the caveats

Measured: `lines` coverage, aggregated per crate/plugin from the per-file
`llvm-cov` export.

Excluded from the report via `--ignore-filename-regex`:

- `tests/`, `benches/`, `examples/`, `fuzz/` directories — test harness sources
  are ~100% covered by definition and would inflate every crate that owns a
  `tests/` directory.
- `build.rs`.
- `mailwoman-desktop` / `mailwoman-mobile`, which are excluded from the build
  (they need platform WebView libraries; `ci.yml` tests them separately).

**Two caveats worth stating plainly rather than burying:**

1. **Inline `#[cfg(test)] mod tests` blocks still count.** `llvm-cov` reports
   per file, and an inline test module lives in the same file as the code it
   tests. There is no stable way to exclude it (`#[coverage(off)]` is unstable).
   This repo has 156 files with inline test modules, so the reported figures are
   **inflated by an unmeasured amount** relative to a pure production-code
   figure. The ratchet is unaffected — it compares like with like — but the
   absolute numbers should not be read as "N% of production code is tested".

2. **Coverage is an instrument, not a goal.** A line executed by a test with no
   assertion counts exactly the same as a line whose behaviour is pinned. When
   adding tests to move a number, state what behaviour the test pins.

## The floors file

`.github/coverage-floors.toml`:

```toml
[meta]
tolerance_pp       = 0.25    # absorbs sub-point jitter
baseline_margin_pp = 0.0     # only non-zero if floors are transplanted hosts

[rust]
gate = false                 # false => whole half reported, not enforced

[rust.crate."mw-crypto"]
floor  = 63.2                # measured; may only move up
target = 80.0                # SPEC §25; reported, never enforced
class  = "crypto"

[rust.crate."mw-sandbox"]
floor = 41.0
gate  = false                # per-crate opt-out (cfg(target_os) bodies)

[web]
gate = true

[web.metric.lines]
floor = 83.13                # measured 83.13% (18928/22767)
```

Crates measured but **not** listed are reported under "measured but not gated"
in the job summary. Adding a crate to the file is what puts it under the
ratchet.

### Populating the Rust baseline

The web floors are recorded and gating from day one. **The Rust floors are
deliberately empty in the shipped file**, and that is the honest state rather
than an oversight: a coverage floor is only meaningful when it was measured on
the platform CI measures on.

A repo-wide search for `#[cfg(target_os|unix|windows)]` finds exactly three
crates — `mw-sandbox` and `mw-render` (seccomp, Landlock, namespaces, the media
jail) and `mw-server`. Their platform-conditional bodies do not exist in a
non-Linux build at all, so those crates have a genuinely different denominator
per host. Floors transplanted from a developer's machine would redden CI for
reasons unrelated to any change under review — which is precisely the failure
mode the ratchet exists to avoid.

So until the floors are recorded, the `ratchet` job reports every measured Rust
crate under "measured but not gated" and enforces nothing on the Rust side.
**The first `coverage.yml` run on `master` is the baseline-generating run.**

Populating it is mechanical, and should be the first follow-up after this
workflow goes green:

1. Read the per-crate table from a green `coverage.yml` run on `master` (it is
   written to the job summary, and `rust-coverage.json` is uploaded as an
   artifact).
2. Add a `[rust.crate."<name>"]` block per crate with `floor = <measured>`.
3. Add `target = 80.0` and `class = "protocol" | "crypto"` to the SPEC §25 set
   (the membership list is recorded in `coverage-floors.toml` itself, so the
   reading of "protocol/crypto crates" is reviewable rather than implicit).
4. Flip `[rust] gate = true` in the same commit.

If a crate needs to be exempted — the three `cfg(target_os)` crates are the
likely candidates if the Linux figure turns out to be volatile — give it a
per-crate `gate = false` rather than dropping its floor. Reported-but-not-
enforced is a state the file models explicitly.

`meta.baseline_margin_pp` exists for the case where floors *must* be
transplanted from another host: recorded floors become `measured − margin`. It
is `0.0` while floors come straight from an `ubuntu-latest` run, and should stay
that way.

This mirrors the land-non-blocking-then-promote pattern the 3-OS CI matrix uses
in the same tag, and for the same reason: a gate nobody trusts is worse than a
gate that arrives one step later.

## Mutation testing

Coverage says a line ran. Mutation testing says the line's behaviour was
actually pinned. SPEC §25 asks for it on `mw-crypto`, `mw-sanitize`, and
`mw-export`; it is set up separately in `docs/testing/mutation.md` and runs
nightly and non-blocking.

## Workflow layout

`.github/workflows/coverage.yml` has three jobs:

| job | what it does |
|---|---|
| `rust` | instrumented workspace pass, uploads `rust-coverage.json` + `.lcov` |
| `web` | `vitest run --coverage`, uploads `coverage-summary.json` + `lcov.info` |
| `ratchet` | downloads both, compares against the floors, writes the job summary |

The ratchet is a separate job on purpose: a *measurement* failure (the test suite
broke) and a *ratchet* failure (coverage regressed) should never be confused for
one another when reading a red run.

`ci.yml` is not modified — it has a single owner, and coverage lands as its own
workflow file.
