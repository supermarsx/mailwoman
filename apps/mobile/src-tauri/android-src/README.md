# Android native source (templates)

The real Android project lives in `apps/mobile/src-tauri/gen/android/`, which is
**generated** by `tauri android init` (needs Android SDK + NDK + a JDK) and is
git-ignored. This directory holds the **tracked source templates** e2/e4 merge into
that generated project — so the custom Kotlin + manifest survive a `gen/` regen.

**Toolchain note (e0 probe):** this Windows machine has the Android SDK
(`C:\android-sdk`) + NDK (25.1 / 27.0) but **no JDK on PATH**, so `tauri android
init` / `build` cannot run locally — Android is the **CI gate** (plan §1.11/§9,
the commented `android-apk` job in `.github/workflows/ci.yml`).

Contents:
- `FlagSecurePlugin.kt` — the custom screen-capture-protection plugin (§7.6).
  Sets `WindowManager.LayoutParams.FLAG_SECURE` on the activity window. **e4** fills
  the command wiring; the capability layer calls it via `setCaptureProtection`.
- `manifest-intents.xml` — `<intent-filter>` snippets (share targets + `.eml/.ics/
  .vcf/.msg` file handlers) to merge into the generated `AndroidManifest.xml`. **e2**
  wires the share/file-handler commands.

After `tauri android init`, copy `FlagSecurePlugin.kt` under
`gen/android/app/src/main/java/com/mailwoman/mobile/` and merge the manifest
snippets into `gen/android/app/src/main/AndroidManifest.xml`.
