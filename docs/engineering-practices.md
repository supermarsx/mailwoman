# Engineering practices

Rules this project arrived at by getting them wrong first. Each one is here
because it cost something real, and each is stated with the incident that
produced it — a rule whose reason has been forgotten is a rule someone deletes
during a refactor.

They are not style preferences. Most of them exist because a check reported
success while the thing it was supposed to check was untrue.

---

## 1. A check that cannot see the thing returns a clean-looking answer

**This is the single shape behind nearly every false pass found on this
project.** It keeps arriving in new disguises, so it is written down as a
first-class rule rather than as a list of past bugs.

Instances, all real:

* A dependency sweep grepped for `Client::builder()`. Most production clients
  were `reqwest::Client::new()` — the same thing, the same ambient-proxy
  exposure, and **invisible to that grep**. It reported 10 sites where there
  were 21.
* A Windows process-liveness filter matched on the command line containing a
  path. `cargo` had been invoked with *relative* arguments, so its command line
  never contained one. The filter returned `0` and was read as "the build
  died"; `0` actually meant compilation had finished — the opposite.
* A test worktree that shares `target/` with the main tree overwrites test
  binaries and yields a **silent false pass**, with tests vanishing from the
  count rather than failing.
* A coverage ignore-regex was not anchored to a package root, so it also
  excluded a production `build.rs` — half a crate could not trip the ratchet.

**The rule: any check must be shown capable of failing, or capable of matching,
before a clean result from it is believed.** Write the negative control first.

Corollaries:

* **Artifact counts are not a progress metric.** Cargo replaces intermediates,
  so the count goes down as well as up. Live `rustc` processes are the signal.
* **Prefer an instrument that is load-independent.** A statement count, a node
  identity, a commit count and a call count all mean the same thing on a busy
  machine as on an idle one. Elapsed time does not.
* **Assert a constant, not a bound.** `stmts(get(50)) == stmts(get(5))` catches
  an N+1 that `stmts < 150` passes at 100.

### The same shape at the level of a measurement

A check can also see something real, accurately, and still be looking at the
wrong thing — and nothing in the output will say so.

A flaky spec was measured at 333 ms against a 1 028 ms budget, giving a
plausible 3.1× margin and an obvious "widen the budget" fix, comment and all.
That measurement came from a **single-file cold run, which is not the
configuration in which the test fails**; warmed, the same work takes under
50 ms. The fix would have passed CI and left the real cause — a contended
dynamic import inside a 1 s assertion ceiling — in place, ready to fail again on
a busier day.

**Measure in the configuration where the thing actually fails**, and be
suspicious of a number that arrived without a control.

---

## 2. Verify the file, not the commit you were handed

`git status` cannot distinguish "committed and clean" from "never started" —
after a pathspec commit the tree looks identical either way. Use
`git log --oneline -- <path>`.

The same applies to reading code: checking the commit someone named you tells
you what was true at that commit, not what is true now. A hazard note was once
committed warning about a bug that had already been fixed **four commits
earlier in the same tree**, because only the named commit was checked.

**A prominent warning that is wrong is worse than no warning**: the next reader
either wastes time confirming it, or learns to skim past that kind of comment.

---

## 3. A struct that holds a secret unsealed writes its own `Debug`

A `#[derive(Debug)]` over a type holding a plaintext credential is invisible at
the call site, silent at review, and leaks into **every `tracing` event, panic
payload and error body that ever formats the value**.

Three types needed this in a single release (`ProxyAuth`, `EgressProxyRow`,
`BridgeOauthTokenRow`), and the only reason any of them was caught is that
someone happened to be looking.

Write the `Debug` by hand and render the secret as `<redacted>`. Request types
matter as much as stored ones — a request struct is what an extractor rejection
or a `?` in a handler is most likely to render.

**When you test a redaction, assert first that the value is really there.**
"Absent" and "redacted" are indistinguishable in rendered output, and only one
of them is the property you want: a type that had quietly stopped storing the
secret would sail through every redaction assertion.

### `audit_log` is append-only *by design*

There is no update or delete method, deliberately. A secret written into
`detail_json` can therefore **never** be redacted — not "hard to redact":
impossible, by construction, in the table specifically built to be permanent.
Audit events carry no secret material. Say so at the emission site.

---

## 4. Shared-tree rules

Several lanes may work in one checkout at once. Anything that writes files it
was not asked to write is banned:

* Commit with an **explicit pathspec** (`git commit -- <paths>`). Never `-a`,
  never `git add .`.
* Never `git stash`, `git commit --amend`, or `git reset` — each destroys or
  rewrites work that may not be yours.
* **`cargo fmt --all` is banned.** It rewrites every file in the workspace,
  including peers' uncommitted work. Use `cargo fmt -p <your-crate>`, or
  `rustfmt` on your own files. Note that `-p` still formats the whole crate, so
  it is only safe when you are the sole writer in that crate.
* **`cargo fmt --all --check` is fine** — it only reports — but it is *also*
  whole-tree, so a peer's WIP can make your `--check` fail for reasons that are
  not yours. **Read the reported paths before assuming the diff is yours.**

`git add .` and `git stash` were banned long before `cargo fmt --all` was, which
is the point: the same hazard walked in through a different door and went
unnoticed for three releases.

---

## 5. Flake policy: no blanket `retry`

There is deliberately **no `retry`** configured in the web suite, and adding one
would be a mistake worth arguing about first. A retry converts every
nondeterminism into a silent pass — the same defect as a test that passes for
the wrong reason, installed as policy, applied to the whole suite, and hiding
the next one too. A flake nobody can see is a flake nobody fixes.

Instead: **diagnose the mechanism, then match the fix to it.** Two specs flaked
in one release and looked identical from the outside — green in isolation, red
under load — with genuinely different causes:

| spec | mechanism | fix |
|---|---|---|
| a 2000-row list lifecycle test | the **test timeout**: 2 382 ms of real work against the 5 000 ms default | a per-test timeout, row count untouched |
| a composer test | the **find timeout**, a different ceiling: `findBy*` gives up after 1 000 ms regardless of `testTimeout`, and the await was a real dynamic import | import the chunk in `beforeAll`, budgets unchanged |

The second is the cautionary one: **raising `testTimeout` would have done
nothing for it.** The obvious knob was the wrong knob.

Rules that follow:

* Measure the real cost against **the budget that actually applies**, in the
  configuration where it actually fails.
* **Prefer removing the cost from the budget over widening the budget.**
* Never widen a budget globally.
* **Never shrink the work being measured.** The 2000 rows are what make that
  test's bound meaningful; a smaller list would pass for a leaking
  implementation too.

---

## 6. Migrations: a retired number stays retired

Migration **`0024` is permanently retired and must never be filled**, in either
dialect, by any lane. `0025` was created before it; `0024` was never created at
all.

This matters because it does **not** fail loudly. `sqlx` applies any
resolved-but-unapplied migration **regardless of version order** —
`validate_applied_migrations` only raises on an *applied* migration missing from
disk, or a checksum change to an applied one. So a file added at `0024` would
run **after** `0025` on every existing database and **before** it on every fresh
one. Fresh installs and upgrades would diverge permanently, and nothing would
error on either path.

A permanent hole is safe. A re-used number is a silent ordering fork.

A tombstone test asserts `0024` is absent from both migration directories, and
that both dialects contain the identical set of version numbers. It deliberately
does **not** assert contiguity — that would fail today for the very reason the
test exists, and a test weakened on the day it is written teaches everyone to
weaken tests.

---

## 7. Writing an assertion that means something

Collected from the reviews that caught each of these:

* **Success is an allowlist, not the absence of a known failure.** A `!failure`
  check means every variant added later arrives pre-approved as healthy.
* **Pair every "it does the right thing" with a control showing the harness
  could have observed the wrong thing.** "No offline transition was announced"
  also holds for a client that never announces one; "the query ended" also holds
  for one that ends on every page.
* **A count asserted before a property** stops a run that measured nothing from
  passing as a run that found nothing wrong — assert `sites >= 30` before
  asserting no offences, and `created > 0` before asserting creates equal
  revokes.
* **Assert what is absent, not just what is present.** `'password' in body ===
  false` fails for `''`, for `null` and for a mask alike, where a truthiness
  check passes for at least one of them.
* **Write characterization tests before the refactor, not after.** Written
  after, they characterise the new behaviour — which is the whole failure mode.
* **Run the assertion against the unfixed code first and record that it fails.**
  An assertion that passes before the fix is not evidence of the fix.
