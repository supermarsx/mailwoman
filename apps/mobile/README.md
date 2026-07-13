# apps/mobile — Mailwoman thin mobile shell (Tauri v2)

A **thin** mobile shell, same model as the desktop shell (SPEC §16 / plan t7 §0):
embeds the hash-verified `apps/web/dist` SPA + mobile OS integration only. **No
protocol logic, no forked UI.**

- **Android is the buildable gate** (APK, F-Droid-friendly, CI).
- **iOS is best-effort / documented** (needs macOS + an Apple account — not
  available on this Windows machine; plan §1.9/§9).

## Toolchain status (e0 probe)
This machine has the Android SDK (`C:\android-sdk`) + NDK (25.1 / 27.0) but **no JDK
on PATH**, so `tauri android init` / `build` cannot run locally. Android is the
**CI gate** (the commented `android-apk` job in `.github/workflows/ci.yml`).

## Layout (e0 scaffold)
- `src-tauri/tauri.conf.json`, `src-tauri/src/lib.rs` — the mobile launching stub
  (mobile entry point + `__MW_CONFIG__` handshake with `platform.kind`
  = `android` | `ios`). Mobile-only plugins (biometric) are gated `#[cfg(mobile)]`.
- `src-tauri/android-src/` — tracked source templates (FLAG_SECURE Kotlin plugin +
  manifest share/file-handler intents) merged into the generated `gen/android/`.

## Who fills what (plan §3)
- **e2** mobile capabilities (UnifiedPush subscribe, share targets, file handlers,
  badge, biometric, multi-server) + iOS skeleton.
- **e4** the Android `FLAG_SECURE` Kotlin plugin.
- **e7** mount/wire + the APK build.
