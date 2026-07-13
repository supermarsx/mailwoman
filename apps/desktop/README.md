# apps/desktop — Mailwoman thin desktop shell (Tauri v2)

A **thin** Windows/macOS/Linux shell (SPEC §16 / plan t7 §0): it embeds the
hash-verified `apps/web/dist` SPA (the byte-identical bundle `mw-server` serves) and
adds OS integration only. **No protocol logic, no forked UI** — it loads the same
SPA a browser would and talks to the user's Mailwoman server over the identical JMAP
surface, selecting the native capability layer (`apps/web/src/platform/tauri.ts`) via
feature-detect.

## Layout (e0 scaffold)
- `src-tauri/tauri.conf.json` — `frontendDist` → the shared `../../web/dist`.
- `src-tauri/src/lib.rs` — the launching stub: loads the SPA + injects the frozen
  `__MW_CONFIG__` handshake (§2.1/§2.5) + registers the first-party plugins.
- `src-tauri/capabilities/` — least-privilege permission set.
- `src-tauri/icons/` — generated app icons (`cargo tauri icon app-icon.png`).

## Build
```
pnpm -C ../web build          # build the shared SPA first
node ../../scripts/emit-bundle-hash.mjs
pnpm install
pnpm exec tauri build         # or `cargo build -p mailwoman-desktop`
```
Prereqs (verified by the e0 probe on this machine): Rust (MSVC), WebView2 (Win11),
Tauri CLI 2.x.

## Who fills what (plan §3)
- **e1** desktop capabilities (notifications+actions, OS keychain via `keyring`,
  deep-link/mailto, badge, biometric, drag-out, multi-server).
- **e3** self-contained mode (spawn the bundled `mw-server` as a sibling process).
- **e4** screen-capture protection (`set_content_protection`).
- **e7** mount/wire (bundle-hash gate + Tauri commands ↔ `tauri.ts`) + builds.
- **e9** live tauri-driver E2E.
