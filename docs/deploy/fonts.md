# Self-hosted fonts (`mailwoman fonts pull`)

Mailwoman self-hosts its web fonts so the app runs under a strict
`font-src 'self'` CSP with no third-party font CDN. The SPA ships with bundled
defaults (Inter / Newsreader / JetBrains Mono); `mailwoman fonts pull` refreshes
or extends them from Google Fonts at build/deploy time, keeping Google's
per-unicode-range subsets and rewriting the stylesheet's `url()`s to be
origin-relative.

## Usage

```sh
mailwoman fonts pull "Inter:wght@400;500;600;700" \
  --out fonts \
  --url-prefix /fonts \
  --css-name fonts.css
```

- Positional args are Google Fonts family specifiers (`Family:axis@values`).
- `--out` (default `fonts`) — output directory for the `.woff2` subsets +
  stylesheet.
- `--url-prefix` (default `/fonts`) — the origin path the rewritten `url()`s
  point at; must match where you serve the directory.
- `--css-name` (default `fonts.css`) — the generated stylesheet filename.
- `--text "…"` — optional: restrict to the glyphs actually needed (further
  shrinks the subsets).

The command fetches the Google `css2` sheet with a browser UA so Google returns
`woff2`, downloads each per-unicode-range subset, and writes them next to a
stylesheet whose `@font-face` `src` URLs are `url-prefix`-relative. No glyph
re-subsetting happens locally, so there is no native font-manipulation
dependency.

## At deploy

1. Run `mailwoman fonts pull …` during your build/deploy step.
2. Serve the resulting directory at `--url-prefix` (e.g. `/fonts`) from the same
   origin as the app — behind the reverse proxy or from `mw-server` — so
   `font-src 'self'` is satisfied.
3. Reference the generated `fonts.css` from the app's font layer.

## CI note

The network fetch is behind a `FontSource` trait; CI exercises the whole
pull/rewrite pipeline over a recorded fixture (no live Google Fonts call). A
`DirSource` also enables fully offline/air-gapped pulls from a local mirror.
