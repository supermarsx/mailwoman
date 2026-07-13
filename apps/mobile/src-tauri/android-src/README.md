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
- `MailwomanMobilePlugin.kt` — **e2** the mobile-capability Tauri plugin:
  UnifiedPush registration, share targets/file handlers, badge counts. Called from
  the Rust `commands` plugin (`src/commands/`) via `run_mobile_plugin(...)`.
- `UnifiedPushReceiver.kt` — **e2** the UnifiedPush `MessagingReceiver` +
  `PushBridge` (endpoint cache + live-plugin forwarding). Opaque wakes only (§2.3).
- `FlagSecurePlugin.kt` — **e4** the custom screen-capture-protection plugin (§7.6).
  Sets `WindowManager.LayoutParams.FLAG_SECURE` on the activity window.
- `manifest-intents.xml` — `<intent-filter>` snippets (share targets + `.eml/.ics/
  .vcf/.msg` file handlers), the UnifiedPush `<receiver>`, and the required
  `<uses-permission>` lines to merge into the generated `AndroidManifest.xml`.

## CI handoff — exact steps after `tauri android init` (e7/e8)

Steps 1–3 below are AUTOMATED by [`merge.py`](./merge.py) (idempotent), which the
`android-apk` CI job runs after `tauri android init`. The manual description is kept
for reference / local runs.

`tauri android init` (needs JDK — the local gap) generates `gen/android/`. Then:

1. **Copy the Kotlin** under `gen/android/app/src/main/java/com/mailwoman/mobile/`:
   `MailwomanMobilePlugin.kt`, `UnifiedPushReceiver.kt`, `FlagSecurePlugin.kt`.
2. **Merge the manifest** (`gen/android/app/src/main/AndroidManifest.xml`):
   - the `<intent-filter>` blocks go inside `<activity>`;
   - the `<receiver>` block goes as a direct child of `<application>`;
   - uncomment the `<uses-permission>` lines under `<manifest>`.
3. **Add the Gradle dependencies** in `gen/android/app/build.gradle.kts`:
   - `implementation("org.unifiedpush.android:connector:2.4.0")` — UnifiedPush
     (Apache-2.0). The Kotlin targets the connector **2.x** API (`UnifiedPush.registerApp/
     getDistributor/getDistributors/saveDistributor/unregisterApp`, `MessagingReceiver`
     with `String` endpoints). If CI pins connector 3.x, adjust the receiver overrides
     (`PushEndpoint`/`PushMessage`) accordingly.
   - `implementation("androidx.security:security-crypto:1.1.0-alpha06")` —
     `EncryptedSharedPreferences` / `MasterKey`, backing the `keychain*` secure-store
     commands (Apache-2.0). Both clear the §3 license floor; record them in the
     JS/native license gate.
4. **Register the Rust native-bridge plugin** (`src/lib.rs` `run()`):
   `.plugin(commands::init())` — a SETUP-ONLY plugin that registers the
   `MobileBridge` (MailwomanMobilePlugin) and `CaptureBridge` (FlagSecurePlugin)
   handles. The capability COMMANDS are registered at the **app level** in the same
   `run()` via `.invoke_handler(tauri::generate_handler![commands::config::mw_server_list,
   …])` with their frozen bare `mw_*` names (t7-e8 reconciliation).
5. **ACL:** app-level commands are NOT ACL-gated in Tauri v2 (only plugin + core
   commands are), so the frozen `mw_*` commands need **no** `permissions/` schema or
   `mailwoman-mobile:default` grant — the earlier e7-flagged plugin-ACL gap is
   resolved by moving them to the app level. `capabilities/default.json` still only
   grants the third-party plugin defaults (`core/os/notification/deep-link`), and
   `capabilities/mobile.json` grants `biometric:default`. `run_mobile_plugin`
   (Rust → Kotlin) is internal and not subject to the JS ACL.
6. **Register the Kotlin plugins** in the generated plugin registry (Tauri v2
   `generatedPlugins`): `MailwomanMobilePlugin` (this file) AND `FlagSecurePlugin`
   (e4) — the Rust `commands::init` setup registers BOTH by class name. Biometric
   app-lock uses `tauri-plugin-biometric` directly (already `#[cfg(mobile)]`-gated in
   `lib.rs`), adapted to the frozen `mw_biometric_*` names in `commands/biometric.rs`.

## Emitted plugin events (for e6's `tauri.ts` to listen on)
- `mailwoman-mobile:shareTarget` — a share/file-open payload
  `{ title?, text?, url?, files:[{name,mime,bytesB64|contentUri}] }`
  (`bytesB64` → decode to `Uint8Array` for the frozen `ShareTargetPayload`).
- `mailwoman-mobile:newEndpoint` — a `PushSubscriptionInfo` (UnifiedPush endpoint
  grant/rotation) → POST to `/api/push/subscribe`.
- `mailwoman-mobile:pushWake` — an opaque wake → JMAP `/changes` refetch (§2.3).
