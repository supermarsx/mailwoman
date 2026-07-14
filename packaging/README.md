# Packaging recipes

Build **recipes and manifests** for every distribution channel Mailwoman targets
(SPEC §16). This directory contains *no secrets, no signing keys, and no submitted
artifacts* — everything that needs a human account, certificate, or store submission
is marked with a `HUMAN` placeholder and explained in
[`../docs/deploy/packaging.md`](../docs/deploy/packaging.md).

| Channel | Recipe | Human-gated input |
|---|---|---|
| Desktop auto-update (Tauri) | [`updater/`](updater/) | minisign signing keypair, hosted update feed |
| Flatpak (Flathub) | [`flatpak/`](flatpak/) | Flathub PR review + hosting |
| F-Droid | [`fdroid/`](fdroid/) | fdroiddata merge request, reproducible-build review |
| winget (Windows) | [`winget/`](winget/) | signed installer URL + SHA-256, winget-pkgs PR |
| macOS (notarized) | [`macos/`](macos/) | Apple Developer ID cert, notarization Apple ID |
| Linux deb/rpm/AppImage | [`linux/`](linux/) | (built by CI; signing/repo hosting is ops) |

The Tauri bundle config that drives deb/rpm/AppImage/msi/dmg lives in
`apps/desktop/src-tauri/tauri.conf.json` (`bundle` + `plugins.updater`); the mobile
shell config is `apps/mobile/src-tauri/tauri.conf.json`.

> **App identifier:** the shells ship the reverse-DNS namespace `com.mailwoman.*`
> (`com.mailwoman.desktop`, `com.mailwoman.mobile`). Flatpak/AppStream reuse
> `com.mailwoman.Mailwoman` for cross-channel consistency; the F-Droid metadata file
> name is fixed to the Android applicationId `com.mailwoman.mobile`.

None of this **submits** anything. The CI workflow
`.github/workflows/packaging.yml` asserts the artifacts **build** and meet the §16
size budgets (thin shell < 10 MB, self-contained desktop < 40 MB); it never signs or
uploads.
