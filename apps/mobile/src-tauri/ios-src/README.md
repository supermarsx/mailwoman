# iOS native source (templates — best-effort, documented gap)

The real iOS project lives in `apps/mobile/src-tauri/gen/apple/`, which is
**generated** by `tauri ios init` (needs **macOS + Xcode + a paid Apple account**)
and is git-ignored. This directory holds the **tracked source templates** to merge
into that generated project on a Mac — so the custom Swift survives a `gen/` regen.

**Toolchain note:** iOS cannot be built on this Windows dev/CI machine (plan §1.9,
§6 R1/R5). iOS is a **best-effort, documented gap**, not a V5 gate — same discipline
as V4's iOS/aws-lc-rs deferrals. The code here is real and reviewable; the local
build/run is the tracked gap.

Contents:
- `ScreenCaptureDetection.swift` — screen-capture **detection** (§7.6). iOS has **no
  API to prevent** screenshots/recording of webview content, so this is a best-effort
  detection + react path (hide content while recording, blur the app-switcher
  snapshot, notify on screenshot). The capability layer reports iOS as
  `{ supported: false }` for `setCaptureProtection` (the SPA keeps the V4 watermark),
  and this detection signal is additive.

After `tauri ios init`, copy `ScreenCaptureDetection.swift` into the generated Xcode
project (`gen/apple/`), wire `onCaptureStateChanged` to a secure overlay over the
webview + a Tauri event, and blur the window in `sceneWillResignActive`.

See `docs/security/screen-capture.md` for the honest OS-by-OS matrix.
