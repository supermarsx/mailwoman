# Mailwoman documentation

Start here. The [browsable manual](./index.md) includes a topic sidebar, local full-text search and a dark-first theme.

## Working on the manual site

The site is built with [VitePress](https://vitepress.dev) from the Markdown in this
folder. From `docs/`, with Node.js and pnpm 11:

```sh
pnpm install     # once
pnpm dev         # live preview at http://localhost:5173
pnpm build       # static site in .vitepress/dist; fails on a broken internal link
pnpm preview     # serve the built site
```

Set `DOCS_BASE` when hosting under a subdirectory (for example `/mailwoman/` on
GitHub Pages); the `Documentation` workflow (`.github/workflows/docs.yml`) builds
with it on every change under `docs/`.

- **Sidebar**: groups and page order are set in `.vitepress/sidebar.mjs`, labelled by
  each page's `# heading`. A page missing from that list still appears: in "More"
  under Administering for `admin/` and `deploy/`, in Engineering notes for loose
  top-level pages.
- **Theme**: `.vitepress/theme/style.css` maps the web app's colour tokens
  (`apps/web/src/theme/tokens.ts`) onto VitePress's variables. The fonts (Jost,
  JetBrains Mono) are self-hosted in `.vitepress/theme/fonts/`; the site makes no
  request to another origin.
- **Static files**: `public/` (icon, favicon, screenshots) is served from the site root.
- Raw HTML in pages is rendered as text, so placeholders such as `<accountId>`
  show literally. Use Markdown, and `{.class}` attributes where styling is needed.

## Writing docs

- **Page template**: a screen page has the title as the UI labels it, one-paragraph purpose, orientation shot, numbered
  callout legend, steps, settings and limits).
- **Images** use markdown-it attributes (built into VitePress); raw HTML is off.
  Put the class in braces directly after the image or list, no space before `{`:

  ```md
  ![The mailbox with the folder list, message list and reader](/screenshots/core/mailbox.webp){.mw-shot}

  ![The compose toolbar](/screenshots/core/compose-toolbar.webp){.mw-crop}

  1. The folder list.
  2. The message list.
  3. The reader.
  {.mw-callouts}
  ```

  `.mw-shot` is a full screenshot (column width, never scaled past its natural size,
  3px border, offset shadow). `.mw-crop` is a cropped element, centred at its natural
  width up to the column, 2px border. Both open at full size in an overlay on click
  or Enter (Esc closes), so 2x captures stay readable. `.mw-callouts` turns an ordered
  list into the legend for the numbered markers drawn on a shot: yellow `#f5c400`
  disc, black numeral. Number the list in the same order as the markers.
- **Moved pages**: leave a stub at the old path. It is skipped in the sidebar and
  the search index, and sends the browser to the new page (script redirect plus a
  `<meta refresh>`); the body is the fallback for anyone who is not redirected.

  ```md
  ---
  title: Moved
  redirect: /user/offline-sync
  ---
  # This page has moved
  See [Sync and offline access](/user/offline-sync).
  ```

  `redirect` is a site-absolute path without `.md`.
- **Console-linked anchors**: the admin console's "Learn more" links point at
  `admin/<area>.md#<id>`, and the page must contain a literal `<a id="<id>"></a>`
  (the site renders it as an anchor). Keep the id when you rewrite a section.
  `apps/web/src/screens/Admin/shell/docsAnchors.test.ts` checks every such link; run
  it after editing an admin page:
  `pnpm -C apps/web exec vitest run src/screens/Admin/shell/docsAnchors.test.ts`.
- **Jargon check** (run from `docs/`; the `**/` globs also work from the repo root with `docs` as the path). Reader-facing pages must be clean; engineering notes, generated references and redirect stubs are excluded explicitly:

  ```sh
  rg -n -P -f screenshots/jargon.txt . -g '*.md' -g '!**/node_modules/**' -g '!**/perf/**' -g '!**/testing/**' -g '!**/engineering/**' -g '!**/changelog/**' -g '!**/deploy/upgrade-notes/**' -g '!**/security/audit-prep/**' -g '!**/admin/configuration.md' -g '!**/admin/erasure-coverage.md' -g '!**/protocols/imap-extensions.md' -g '!**/session-completion.md' -g '!**/ROADMAP-1.0.md' -g '!**/i18n.md' -g '!**/engineering-practices.md' -g '!**/deploy/scaling.md' -g '!**/deploy/load-testing.md' -g '!**/deploy/websocket-load-testing.md' -g '!**/screenshots/**'
  ```

  Remaining hits are listed in `screenshots/jargon-allow.txt`.
- **Building without colliding**: `docs/.vitepress/dist` and the cache are shared, so
  build into scratch: `node node_modules/vitepress/dist/node/cli.js build . --outDir
  <scratch>/out --cacheDir <scratch>/cache` from `docs/`. Serve the output with any
  static file server (`vitepress preview` ignores `--outDir`).

## The manual by reader

- **Start here**: [what Mailwoman is](./getting-started/what-is-mailwoman.md),
  [a quick install](./getting-started/install-quick.md) and
  [your first sign-in](./getting-started/first-sign-in.md).
- **Using Mailwoman**: [the mailbox](./user/index.md), one page per task, and
  [personal settings](./user/settings/index.md), one page per category.
- **Administering**: [the admin console](./admin/index.md);
  [install and upgrade](./deploy/README.md) (with reverse-proxy recipes and upgrade
  notes); accounts and domains; security policy; privacy and compliance; mail
  features; appearance; operations; [troubleshooting](./admin/troubleshooting.md).
  The sidebar groups the `admin/` and `deploy/` pages by task.
- **Security model**: [`security/`](./security/README.md), one page per surface.
  [`security/egress.md`](./security/egress.md) states what the outbound-fetch
  policy guarantees and what it does not.
- **Integrations and protocols**: [mail provider bridges](./bridges/index.md),
  [Nextcloud](./integrations/nextcloud.md), [plugins](./plugins/index.md),
  [message export](./export/index.md).
- **Reference**: [command line](./reference/cli.md),
  [environment variables](./reference/environment.md),
  [configuration](./admin/configuration.md), [every setting](./admin/settings.md),
  [glossary](./reference/glossary.md), [IMAP extensions](./protocols/imap-extensions.md).
- **Changelog**: [release notes](./changelog/26.8.md).

## For contributors

- **[`engineering-practices.md`](./engineering-practices.md)** — the rules this
  project arrived at by getting them wrong first: why a check that cannot see the
  thing returns a clean-looking answer, the shared-tree git and `cargo fmt` rules,
  why a struct holding a secret writes its own `Debug`, the flake policy, the
  retired migration number, and how to write an assertion that means something.
  **Read this before your first change.**
- [`testing/coverage.md`](./testing/coverage.md),
  [`testing/mutation.md`](./testing/mutation.md) — the coverage ratchet and
  mutation harness.
- [`engineering/`](./engineering/a11y-manual-checklist.md): history and UI-internals notes (accessibility checklist, load baseline, build history),
  [`i18n.md`](./i18n.md).
- [`perf/`](./perf/index.md) — build-profile and size-budget measurements, with their
  methods stated.

## Project-level

- [`../SPEC.md`](https://github.com/supermarsx/mailwoman/blob/master/SPEC.md) — the product and architecture specification,
  including what is **cut** or **specced-not-built**, which it says out loud.
- [`../VERSIONING.md`](https://github.com/supermarsx/mailwoman/blob/master/VERSIONING.md) — the rolling `YY.N` scheme and the
  **release history**, where each entry records what a release was believed to
  deliver at the time, with later corrections appended rather than rewritten.
- [`../SECURITY.md`](https://github.com/supermarsx/mailwoman/blob/master/SECURITY.md) — reporting a vulnerability.
- [Engineering history notes](./ROADMAP-1.0.md).
