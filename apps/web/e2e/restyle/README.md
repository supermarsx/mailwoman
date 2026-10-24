# Restyle harness (t32)

Measures the web UI before and after the t32 restyle on the same host, server image
and seeded data, and checks that nothing got bigger, that contrast holds and that
state is not shown by colour alone. It is a local tool: no CI job runs it.

| file | what it does |
|---|---|
| `stack.sh` | starts and removes an engine-mode stack (Greenmail + `mailwoman-engine`) that serves a given `dist` directory |
| `seed.mjs` | delivers the fixed mail set (called by `stack.sh`) |
| `playwright.restyle.config.ts` | Playwright config: projects `desktop-dark`, `desktop-light`, `phone-dark`, `phone-light` |
| `probes.ts` | the screens, how to reach each one, and the elements measured on it |
| `measure.restyle.ts` | geometry of every probe → `measure/<label>/…json`, merged into `measure-<label>.json` |
| `compare.mjs` | compares two measure runs and fails on growth |
| `contrast.restyle.ts` | axe colour contrast, control boundaries, focus indicators, "not colour alone" |
| `checks.ts` | the in-page checks behind `contrast.restyle.ts` |
| `selftest.restyle.ts` | runs `checks.ts` on a fixed page: a good page gives no finding, each seeded defect gives its finding (desktop-dark only, no stack needed) |
| `shots.restyle.ts` | full-page screenshots → `screens/<label>/<viewport>/<theme>/<screen>.png` and `INDEX.md` |
| `teardown.ts` | merges the per-screen JSON and writes the screenshot index |

Outputs go to `T32_OUT` (default `.orchestration/reports/t32/`, which git ignores).

## Run

From `apps/web`, Git Bash. Use your own port block (t32.md §5.3) and project id.

```sh
# 1. Build the tree to measure into a scratch directory (never apps/web/dist).
./node_modules/.bin/vite build --outDir F:/mw-t32-<id>-dist --emptyOutDir

# 2. Start a stack on that dist. Prints the env file to pass as T32_STACK_ENV.
T32_STACK_DIR=<your scratch dir> bash e2e/restyle/stack.sh up <id> <portBase> F:/mw-t32-<id>-dist

# 3. Measure, check, capture. Always --workers=1 (one shared mail account).
export T32_STACK_ENV=<the env file> T32_LABEL=after
./node_modules/.bin/playwright test -c e2e/restyle/playwright.restyle.config.ts --workers=1 measure
./node_modules/.bin/playwright test -c e2e/restyle/playwright.restyle.config.ts --workers=1 contrast
./node_modules/.bin/playwright test -c e2e/restyle/playwright.restyle.config.ts --workers=1 shots

# 4. Compare with the baseline (path recorded in state.md as T32-E2-BASELINE).
node e2e/restyle/compare.mjs ../../.orchestration/reports/t32/measure-before.json \
  ../../.orchestration/reports/t32/measure-after.json

# 5. Remove the stack: containers, networks and volumes.
T32_STACK_DIR=<your scratch dir> bash e2e/restyle/stack.sh down <id>
```

Subsets: `--project=desktop-dark`, `--grep @mail` (tags: `@login`, `@shell`, `@mail`,
`@compose`, `@settings`, `@pim`, `@admin`), or `T32_SCREENS=mailbox,reader`. Compare a
subset with `--partial`, which limits the comparison to the screens present in the
after run; without it a screen missing after is a failure.

`stack.sh swap <id> <portBase> <dist>` replaces the stack with one serving another dist,
with fresh volumes and the same seed. `stack.sh down <id>` removes everything the
project created. `MW_IMAGE` overrides the server image (default `mailwoman:t31-w0`);
`MW_BASE_PATH=/mail` serves under a sub-path (the base URL in the env file follows).

## Same conditions on both sides

- Same server image (`mailwoman:t31-w0`), Greenmail 2.1.4, `--no-build`.
- Same seed: 30 messages with fixed senders, subjects and `Date` headers, one with a PDF
  attachment, one with a long subject; messages 16–25 are marked read over IMAP so the
  first screen of the list mixes read and unread rows. The reader opens message 25,
  which is already read, so no screen changes the list.
- Each test first deletes the account's stored appearance and seeds
  `mw.theme.prefs = {mode: 'fixed', theme, chosen: true}` (the T32-BASE parser ignores
  `chosen`), so the theme is the project's and nothing carries over between tests.
- Fixed browser clock 2027-03-10 10:00 UTC (`page.clock.setFixedTime`), timezone UTC,
  locale en-US, reduced motion, no service worker. Message dates in the list come from
  the server (delivery time) and differ between stacks; widths of text are therefore not
  compared, and heights are.
- PIM items (one event, contact, task and note with fixed names) and one admin-provisioned
  user (`t32user@example.org`) are created through the UI by the first test that needs
  them; later tests find them.
- Desktop is Desktop Chrome at 1280×800; phone is Pixel 7 (412×839).

## What compare.mjs fails on

For each probe instance present in both runs (same viewport, theme, screen):

| rule | threshold |
|---|---|
| height grows | > 1 px (every kind) |
| width grows | > 1 px (kind `control`) |
| font-size grows | at all |
| line-height (px; `normal` resolved by measuring) grows | > 1 px |
| any padding side grows | > 1 px |
| phone, interactive probe: width or height changes | > 1 px either way |
| desktop, interactive probe: width or height | below `min(before, 24 px)` |
| fully visible list rows on a screen | fewer than before |
| probe instance visible before | absent or invisible after ("vanished") |

Two listed exemptions, each instance printed when it applies:

| exemption | what may grow | conditions |
|---|---|---|
| `FLEX_FILL` | a listed probe sized by its row or column (a `flex: 1` field, the reader frame), along the listed axis | font-size and padding/border not grown; its band (or the probe enclosing it) not grown; the neighbours freed the space |
| `CHECK_TARGET` (t32-h12) | a listed native checkbox, up to the 24×24 box every checkbox shares (`styles/checks.css.ts`, WCAG 2.2 §2.5.8) | after, width and height each ≤ 24 px; font-size, padding and border not grown; a checkbox past 24 px is still a violation |

It also prints the instances that shrank by more than 1 px (allowed), for the report.

## What contrast.restyle.ts fails on

- axe `color-contrast` (WCAG 2 AA) violations; the message-body iframe is excluded.
- Control boundaries (inputs, selects, textareas, buttons bordered on all four sides): the
  border or the control's own fill must reach 3:1 against the nearest opaque ancestor
  background.
- Focus: the first 12 Tab stops each need an outline or a box-shadow ring of 3:1
  against the background next to it. Chrome's own `outline: auto` ring is two-toned
  (#101010 with a white edge) and is scored by the better tone.
- Not colour alone: unread rows have weight ≥ 600 or a marker; a selected list row or
  option has an inset bar, outline, 3 px start border or marker pseudo-element; links in
  running text are underlined; a selected tab has a bar.

Visually hidden controls (1 px or less, such as a file input inside a styled label) are not
boundary-checked; the visible wrapper is. Elements inside an `inert` subtree (the page behind a dialog, the list behind the phone
reader) are skipped by the boundary, focus and signal checks; axe scans the whole page.

`T32_CONTRAST_REPORT_ONLY=1` records findings without failing; the T32-BASE baseline was
taken that way, because the old styles predate these rules.

## Showing a check fail

`T32_SEED_CSS` adds a stylesheet to each screen after it opens (served from a same-origin
URL, because the CSP refuses inline styles). Use it to see a check fail on a deliberate
defect before trusting it, for example:

```sh
T32_LABEL=seed T32_SCREENS=mailbox T32_SEED_CSS='.mail-search__submit{padding-top:10px !important}' \
  ./node_modules/.bin/playwright test -c e2e/restyle/playwright.restyle.config.ts --workers=1 --project=desktop-dark measure
node e2e/restyle/compare.mjs ../../.orchestration/reports/t32/measure-before.json \
  ../../.orchestration/reports/t32/measure/seed --partial   # exits 1: padding-top and height
```

## Limits

- Proven on one Windows host where the T32-BASE font is the Segoe UI fallback; Linux
  metrics are not measured.
- The admin panel does not apply the theme packs at T32-BASE (no `data-theme`), so its
  before screens look the same in both themes. Admin probes are located by role and,
  for the section nav, by position, because the admin track rebuilds that screen and
  renames sections.
- Colour packs other than `dark` and `light` are not run here.
- The OAuth consent screen is not reached (it needs an OAuth flow).
- The screens open root paths (`/`, `/admin`). With `MW_BASE_PATH=/mail` the stack and
  its env file follow the prefix, but the screens do not: a sub-path run is a separate,
  manual check.
