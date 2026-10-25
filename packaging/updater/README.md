# Auto-update: per-deployment feed and signing key (Tauri updater)

The desktop shell can use `tauri-plugin-updater`, but only when its build is given an
update feed and a signing key. The committed
`apps/desktop/src-tauri/tauri.conf.json` has no `plugins.updater` block and sets
`bundle.createUpdaterArtifacts: false`; the shell registers the plugin only when a
block with a key and a feed is present (`updater_configured`,
`apps/desktop/src-tauri/src/lib.rs`).

The full recipe is in [`docs/deploy/shells.md`](../../docs/deploy/shells.md),
"Desktop updates". In short:

- `MW_SHELL_UPDATER_URL` (https feed, normally
  `https://host/path/{{target}}/{{arch}}/{{current_version}}`) and
  `MW_SHELL_UPDATER_PUBKEY` (the minisign public key as `tauri signer generate`
  prints it) together turn the updater on; `scripts/shells/config-overlay.mjs`
  writes the overlay (`plugins.updater` with Windows install mode `passive`, and
  `bundle.createUpdaterArtifacts: true`) that `scripts/build-shells.*` and
  `tauri build --config` apply.
- Either one missing: no updater (SPEC §16 "self-hosters can pin/disable").
- The private key and its password stay outside the repository, in
  `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` for the installer
  build. Nothing here is signed or published by CI.
- A canary channel is a second feed URL used by a second build; promote a release by
  serving it at the stable feed.

Nothing in the app asks the feed for an update yet (no screen or start-up step calls
the plugin).

Mobile (`apps/mobile`) has **no self-updater**: Android and iOS updates are delivered
by the stores (Play, App Store, F-Droid). See `docs/deploy/packaging.md`.

Optional, so the downloaded update itself is trusted by the OS: Windows Authenticode
and macOS Developer ID certificates; see `../macos/notarize.md` and the
`bundle.windows.certificateThumbprint` placeholder in `tauri.conf.json`.
