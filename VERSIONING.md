# Versioning

Mailwoman uses a **rolling release** scheme in **`YY.N`** format.

- **`YY`** — two-digit calendar year of the release (e.g. `26` for 2026).
- **`N`** — the release number within that year, starting at **1** and
  **resetting to 1 each new calendar year**.

Examples, in order: `26.1`, `26.2`, `26.3`, … then `27.1`, `27.2`, …

There is no separate major/minor/patch. Each tagged release is a self-contained
rolling snapshot; the sequence is strictly increasing within a year, and the
year boundary resets `N`.

## Git tags

Releases are tagged **bare** (no `v` prefix): `26.1`, `26.2`, …
Tags are annotated and signed where possible.

## Package manifests (semver-shaped ecosystems)

Cargo and npm require semver (`X.Y.Z`). We map `YY.N` → **`YY.N.0`**:

- `Cargo.toml` `[workspace.package] version` → `26.1.0`
- `apps/web/package.json` `version` → `26.1.0`

The third component (`.0`) is reserved for the rare out-of-band hotfix to an
already-tagged release (`26.1.1`); normal forward progress increments `N`
(`26.2`), not the patch field.

## Release checklist

1. Bump `version` in `Cargo.toml` and `apps/web/package.json` to `YY.N.0`.
2. Update this file's example if the year rolled over.
3. Commit `chore(release): YY.N`.
4. Tag: `git tag -a YY.N -m "Mailwoman YY.N"` then `git push origin YY.N`.

## History

- **`26.1`** — first rolling release. V0 walking skeleton (SPEC §27): wired
  webmail path (SolidJS client → mw-server JMAP proxy + sanitize worker →
  JMAP upstream), Docker/CI, E2E. Supersedes the pre-adoption `v0.0.0`
  placeholder tag, which was removed.
- **`26.3`** — V2: modern mail layer + theming. Engine-side Tantivy search
  (operators + saved searches), offline (Service Worker + encrypted OPFS +
  replay queue), WebSocket/SSE realtime push, multi-window (BroadcastChannel),
  the modern mail UX (tags/pins/snooze/sweep/undo-send/outbox/send-later/
  follow-up/focused+unified inbox/virtualized list), Sieve rules, identities,
  EML/mbox/TXT/Markdown export, the vanilla-extract design-token theming system
  (light/dark/HC/AMOLED + Grove woody themes) with self-hosted font puller and
  an optional ribbon preset, and sandboxed embedded attachment viewers
  (image/PDF/video) + a global Attachments module. Server gains a rustls-acme
  TLS listener, per-message CSP + CSRF/session hardening, and a blob-download
  route. New crates: mw-search, mw-sieve, mw-export. Verified: 283 Rust + 214
  web tests; live-stack Playwright E2E across all V2 features (offline, push,
  multi-window, viewers, search operators, theming, export). Six real
  end-to-end gaps caught and fixed at the E2E gate before release.
- **`26.2`** — V1: real mail backends. IMAP4rev2 + POP3 + SMTP submission +
  MIME parse/build behind a frozen `AccountBackend` seam, driven by
  `mw-engine` which presents the same JMAP surface the web UI already speaks
  (engine mode vs V0 proxy mode, config-switched). Sync ladder
  (QRESYNC/CONDSTORE/UID-window + POP3 UIDL), engine-side JWZ threading,
  autoconfig ladder, encrypted message cache. New crates: mw-imap, mw-pop3,
  mw-smtp, mw-mime, mw-engine, mw-autoconfig. Greenmail/Dovecot CI
  conformance + a Playwright E2E driving a real IMAP account through the
  unmodified web UI.
