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

## First-party plugin components (`.wasm`) — shipping contract (26.9, t9-e5)

The server no longer embeds the five first-party bridge/plugin components
(`bridge-graph`, `bridge-ews`, `bridge-gmail`, `languagetool`, `nextcloud`) in its
binary. They ship as **external data files** and the server **digest-verifies** each
against a compiled-in SHA-256 pin before it loads
(`crates/mw-server/src/v7_mount.rs` → `FIRST_PARTY_DIGESTS` / `resolve_component`);
a missing or tampered component **fails closed** (logged, never silently loaded).

**Canonical shipped layout:** `plugins/dist/<id>.wasm` (in-repo source of truth;
regenerated digests via `plugins/gen-digests.sh`). Every channel installs those five
files into a data dir and points the server's resolver at it. The resolver looks, in
order, at `$MW_PLUGIN_DIR`, then `<dir-of-the-mailwoman-binary>/plugins`, then
`/usr/lib/mailwoman/plugins`.

| Channel | Where the `.wasm` go | How the server finds them |
|---|---|---|
| **Docker runtime image** | `COPY plugins/dist → /usr/lib/mailwoman/plugins` (see `Dockerfile`) | `ENV MW_PLUGIN_DIR=/usr/lib/mailwoman/plugins` |
| **deb / rpm (self-contained server)** | package data dir `/usr/lib/mailwoman/plugins` | OS default path — no env needed |
| **Flatpak (self-contained)** | `/app/lib/mailwoman/plugins` (see `flatpak/…yml`) | `MW_PLUGIN_DIR=/app/lib/mailwoman/plugins` in the shell's spawn env |
| **AppImage / Tauri self-contained** | Tauri `resources/plugins/*.wasm`, beside the bundled `mw-server` sidecar | the resolver's `<exe-dir>/plugins` fallback (the sidecar's dir), or `MW_PLUGIN_DIR` set by the shell |

> **Tauri wiring (apps/, not this dir):** self-contained shells add
> `"resources/plugins/*.wasm"` to `bundle.resources` in
> `apps/desktop/src-tauri/tauri.conf.json` (next to the existing
> `"resources/mw-server*"`) and `scripts/bundle-server.sh` copies `plugins/dist/*`
> into `apps/desktop/src-tauri/resources/plugins/`. Because the wasm land beside the
> bundled `mw-server` binary, the resolver's `<exe-dir>/plugins` fallback finds them
> with no env set; a shell MAY still export `MW_PLUGIN_DIR` explicitly. The
> **thin** shell (no bundled server) ships no components — it talks to a remote
> server that ships its own.

Rebuild-a-component workflow and rationale: `../docs/perf/size-budget-revision.md`.
