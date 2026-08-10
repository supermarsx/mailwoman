# Mutation testing

## What 26.19 ships, and what it does not

SPEC §25 says:

> Coverage floor 80% on protocol/crypto crates; mutation testing on `mw-crypto`,
> `mw-sanitize`, `mw-export`.

Before 26.19 the repository had no mutation tooling at all — a repo-wide search
for `cargo-mutants` returned nothing, in any config file or any workflow.

**26.19 ships the mutation harness: a scope, a nightly schedule, and a published
report. It does not ship a mutation score, a floor, or a gate, and it makes no
claim that this code is mutation-clean.** The first real run establishes the
baseline. Based on the sampling described below, that baseline is expected to
show a substantial number of surviving mutants.

This is the sibling of [`coverage.md`](coverage.md), and the two answer different
questions:

| | question | 26.19 state |
|---|---|---|
| coverage | did a test *execute* this line? | measured; web gates on a ratchet, Rust reports |
| mutation | did a test *notice* when this line changed? | measured nightly, reported, gates nothing |

Coverage is the cheaper and blunter instrument: a line executed by a test with no
assertion is 100% covered and 0% pinned. Mutation testing is what tells the two
apart, which is why SPEC §25 asks for it specifically on the crates where a
silent behaviour change is most expensive.

## How it runs

`.github/workflows/mutants.yml` — **nightly at 04:23 UTC, and on manual
dispatch. It is not attached to pushes or pull requests.**

The scope of record is [`.cargo/mutants.toml`](../../.cargo/mutants.toml): the
three crates SPEC §25 names, `src/` only, minus the two
`#[cfg(target_arch = "wasm32")]` browser boundaries, which do not compile into a
native build. That file is heavily commented — including for the options
deliberately *not* set — and is the place to change scope.

`cargo-mutants` is a **CI/dev binary**, installed by `taiki-e/install-action` in
CI and `cargo install` locally. It is never a workspace dependency: `Cargo.lock`
is untouched and the shipped binaries gain nothing. The workflow pins version
`27.1.0`, because the config was verified against that version — see
[Things that are not obvious](#things-that-are-not-obvious).

### Why nightly and non-blocking

The same reasoning that makes the coverage gate a ratchet rather than a cliff. A
full pass is measured at roughly five machine-hours (below), which is not a thing
to put in front of a pull request. And a gate that is red on day one, for an
unknown number of pre-existing survivors unrelated to the change under review, is
a gate people learn to route around within a week. A nightly report that names
specific unpinned functions is something a person can act on.

What *does* fail the workflow is a broken harness, and the distinction is
deliberate:

| exit | meaning | workflow |
|---:|---|---|
| 0 | every viable mutant was caught | pass |
| 2 | some mutants survived | **reported, does not fail** |
| 3 | some mutants timed out | **reported, does not fail** |
| 4 | the *unmutated* baseline suite failed | **fails, loudly** |
| 1, 5, 6, 70 | usage / diff / internal error | **fails** |

"This code is not pinned by a test" is information. "The tool could not run" is a
bug to fix before any of the information means anything.

### Sharding

Eight matrix jobs: six for `mw-crypto`, one each for `mw-export` and
`mw-sanitize`. Sharding is not a speed optimisation here — it is what keeps
`mw-crypto` inside GitHub's six-hour job ceiling. `--shard k/n` uses zero-based
`k`, and every shard of a crate must use the same `n` or the split is
meaningless.

`--in-place` is used in CI to avoid copying the tree onto a throwaway runner. It
also forbids `--jobs`, so a shard is serial internally; parallelism comes from
the matrix.

## Cost, measured

Measured on the 26.19 development host (Windows, warm dependency build,
cargo-mutants 27.1.0, one job, `--test-threads=1` as configured). Mutant counts
are from `cargo mutants --list` and are exact; the per-mutant figures come from
small real shards.

| crate | mutants | baseline (build + test) | per mutant | projected full pass |
|---|---:|---:|---:|---:|
| `mw-sanitize` | 125 | 0 s + 1 s | ~3.6 s | **~8 min** |
| `mw-export` | 354 | 27 s + 1 s | ~4 s | **~25 min** |
| `mw-crypto` | 338 | 47 s + 37 s | ~46 s | **~4.5 h** |
| **total** | **817** | | | **~5 h** |

`mw-crypto` is 96% of the bill and none of it is the mutation tool: its suite
takes 37 s per run because it does real Argon2id derivation and real PGP/RSA key
material, and mutation testing pays that once per mutant. Timeouts cost more
still — a mutant that removes a loop bound burns the full 120 s floor rather
than failing fast, and one appeared in the first five `mw-crypto` mutants
sampled.

CI runners are not this host, so treat these as an order of magnitude rather than
a promise. The `timeout-minutes` in the workflow are deliberately generous for
the same reason `coverage.yml`'s are; tune them down once a few real runs have
happened.

### The `--test-threads=1` cost, specifically

`.cargo/mutants.toml` runs tests single-threaded, matching the repo's canonical
gate (`.cargo/config.toml`'s `gate` alias, `VERSIONING.md`). On `mw-crypto` that
is the difference between:

```
cargo test -p mw-crypto                       11 s
cargo test -p mw-crypto -- --test-threads=1   54 s
```

Roughly 5×, and since the test phase dominates, roughly 5× on the whole
`mw-crypto` pass — about four hours of the five. That is a real price and it is
worth stating rather than burying.

It is kept anyway, for two reasons. Single-threaded is the behaviour the release
gate certifies, and mutation testing that scores against a *different* test
configuration answers a question nobody asked. And a flake rate that is tolerable
in a suite run once becomes a stream of unattributable "caught" results when the
suite runs 817 times — a mutant scored caught by a flake is a false negative that
looks exactly like good news.

Two things make it relaxable, and both should be understood before anyone does:

- None of these three crates links `sqlx`, so the shared `_sqlx_migrations`
  cross-binary collision that motivated the canonical flag cannot occur at this
  scope. `t19-e2` (`b13b138`) additionally made test databases isolated per test,
  which removes the underlying hazard rather than working around it — that work
  is what makes a mutation run *attributable* at all, since a suite re-run 817
  times over will find any shared-state collision that exists.
- `test_workspace` is left at its default, so only the mutated package's own
  tests run. No cross-crate test binary is in play.

If the scope is ever widened to a store-touching crate, the flag stops being
belt-and-braces and becomes load-bearing again. Relaxing it is a one-line edit to
`additional_cargo_test_args`, and it should come with a re-measured baseline,
not just a faster number.

## What the sampling found

Small shards were run by hand while writing this, as a config sanity check. This
is **not a baseline** — it is 16 mutants out of 817, and because the default
`slice` sharding takes a contiguous block, they are the first few in source
order rather than a random sample. Do not quote the score.

| crate | sampled | caught | survived | timed out |
|---|---:|---:|---:|---:|
| `mw-sanitize` | 5 | 5 | 0 | 0 |
| `mw-export` | 6 | 0 | 6 | 0 |
| `mw-crypto` | 5 | 0 | 4 | 1 |

The survivors are concrete and are the kind of thing this workflow exists to
surface:

- `crates/mw-export/src/lib.rs` — `export_many`, `export_stream` and
  `trim_trailing_newline` can each be replaced with a constant or have an
  operator flipped without any test failing. The single-message `export_one`
  path is well covered; the *multi*-message and streaming wrappers around it are
  not pinned.
- `crates/mw-crypto/src/rng.rs` — the `Rc10` `rand_core` adapter can return
  `Ok(0)` from `try_next_u32`/`try_next_u64` unnoticed. The timeout was
  `try_fill_bytes` replaced with `Ok(())`: filling no bytes at all does not fail
  a test, it hangs one, which is its own kind of finding.

Nothing here is a shipped-behaviour bug — the mutations are hypothetical. They
say the tests would not have caught the bug if someone had written it.

## What to do with a surviving mutant

A surviving mutant is a finding, not a failure. The report names the file, the
line, the function, and the exact replacement, and the run artifact carries the
diff. The procedure:

1. **Read the diff.** `mutants.out/diff/` in the artifact has one file per
   mutant. Reproduce a single file locally with
   `cargo mutants -f crates/mw-export/src/lib.rs`.

2. **Ask whether a user could observe the mutated behaviour.** That is the whole
   question.

3. **If yes — write the test.** It must fail with the mutation applied and pass
   without it; if it passes both ways it has pinned nothing. This is the normal
   outcome and the point of the exercise. State in the test what behaviour it
   pins, the same rule `coverage.md` asks for.

4. **If no — decide which "no" it is.**
   - *The code is unreachable or unused.* Delete it. A mutant nobody can observe
     in code nobody calls is a dead-code report.
   - *The mutation is genuinely equivalent* (a performance hint, a redundant
     clamp, a defensive branch that cannot be entered). Add an `exclude_re`
     entry to `.cargo/mutants.toml` **with the reason written next to it**. An
     unexplained exclusion is indistinguishable from hiding a finding, and the
     file is the record of what was decided on purpose.

5. **Timeouts count as survivors.** A mutant that hangs usually mutated a loop's
   exit condition, and the test suite could not tell "wrong answer" from "no
   answer". Treat it as case 3.

6. **Unviable mutants need no action.** They did not compile — the type system
   already rejected that change. They are neither good news nor bad.

Two things to avoid:

- **Do not pin the literal.** Asserting that a function returns exactly the
  constant the mutation replaced kills the mutant and tests nothing. Pin the
  behaviour that makes the constant correct.
- **Do not treat the mutation score as a target.** It has the same Goodhart
  problem as coverage, with a worse failure mode: score is trivially raised by
  excluding awkward mutants. There is deliberately no floor file and no ratchet
  for mutation in 26.19, and adding one should wait until the baseline is known
  and the equivalent-mutant exclusions have been reviewed once.

## Reading the results

Every nightly run publishes three things:

- **A job summary** on the `report` job: per-shard counts, the overall mutation
  score, and every surviving mutant grouped by file inside a collapsible block,
  each with a copy-pasteable command to reproduce that file.
- **`mutation-report`** — the same content as one markdown file, so a finding can
  be pasted into an issue or a follow-up tag's backlog without opening eight
  artifacts. Retained 90 days.
- **`mutants-<crate>-<shard>`** — the raw `mutants.out` per shard, including
  `diff/`, `log/`, `missed.txt`, `timeout.txt`, `unviable.txt` and
  `outcomes.json`. Retained 30 days.

cargo-mutants also emits GitHub annotations, so surviving mutants appear against
the source lines in the run view.

If a shard dies, the report names it explicitly and marks the totals as a floor
rather than quietly reporting a smaller, better-looking number.

## Running it locally

```sh
cargo install cargo-mutants --version 27.1.0   # dev binary, never a workspace dep

cargo mutants -p mw-sanitize                   # ~8 min, the cheap one
cargo mutants -f crates/mw-export/src/lib.rs   # one file
cargo mutants -p mw-crypto --shard 0/6         # one CI shard, ~45 min
cargo mutants --list                           # what is in scope, no build
```

Output lands in `target/mutants.out/`. That location is set in the config on
purpose: `/target` is already gitignored, so a run never leaves untracked
directories in the working tree.

**Do not pass `--in-place` on a tree with uncommitted work.** CI uses it because
its checkout is disposable. Locally it edits your actual source files during the
run, and an interrupted run can leave a mutation behind. On Windows it also
rewrites every file it touches with LF line endings, so the files show as
modified afterwards even when the content was correctly restored. Without the
flag, cargo-mutants copies the tree to a temp directory and your files are never
touched — at the cost of a cold dependency build in that copy.

## Things that are not obvious

Four behaviours that cost time to discover and are worth not rediscovering:

1. **`additional_cargo_test_args` needs a literal leading `"--"`.** The tokens
   are appended to the `cargo test` command line verbatim, so
   `["--test-threads=1"]` produces `cargo test … --test-threads=1` and the
   baseline dies with `unexpected argument '--test-threads' found` before a
   single mutant runs. The config uses `["--", "--test-threads=1"]`.

2. **`gitignore` is not on by default.** Without it, a copy-mode run tries to
   reproduce `apps/desktop/e2e/node_modules` — a pnpm store that is mostly
   symlinks — and on Windows fails outright with `A required privilege is not
   held by the client (os error 1314)`. The config sets `gitignore = true`.

3. **`--in-place` and `--jobs` are mutually exclusive.** cargo-mutants rejects
   the combination. In-job parallelism and in-place testing cannot both be had;
   the workflow chooses in-place and shards instead.

4. **The baseline appears in `outcomes.json` as the bare string `"Baseline"`,**
   not as an object like the mutant scenarios. Anything parsing that file has to
   expect both shapes — the report script in `mutants.yml` does.

All four were verified against cargo-mutants 27.1.0, which is why the workflow
pins that version rather than tracking latest. Bumping it is fine and probably
overdue whenever you read this; do it deliberately, and re-run one shard by hand
afterwards to confirm the config still behaves.

## Not covered

- **The wasm boundaries.** `mw-crypto/src/wasm.rs` and `mw-sanitize/src/wasm.rs`
  are excluded: they do not exist in a native build, so every mutant there would
  report "unviable". Mutating them needs a `wasm32` test runner, which this repo
  does not have. `mw-sanitize`'s wasm seam matters — it is where decrypted E2EE
  HTML is sanitized client-side — so this is a real gap, not a tidy-up.
- **"Function always returns `Err`" mutants.** cargo-mutants generates these only
  when given a constructible error value, and the config key is global while our
  error types are per-crate. The more dangerous direction — a verifier that
  always succeeds — *is* generated by default. `.cargo/mutants.toml` records the
  per-crate command to close the gap for one crate at a time.
- **Everything outside the three SPEC §25 crates.** Widening the scope is a
  deliberate decision with a wall-clock price; read the cost table first.
