# Mobile shell (Tauri v2) — Android build, iOS & store gaps

The V5 mobile shell (`apps/mobile`) is the same thin-client model as the desktop
shell: it ships the hash-verified shared SPA and adds mobile OS integration only.
**Android is the buildable gate; iOS is best-effort/documented.**

## What ships on Android

Reached through the SPA's feature-detected capability layer, backed by the frozen
`mw_*` commands (registered app-level in `apps/mobile/src-tauri/src/lib.rs`) and two
custom Kotlin plugins:

- **Multi-server config** (`mw_server_*`) and **secure store** (`mw_keychain_*`,
  backed by Android `EncryptedSharedPreferences` — AES-256 keyed by the hardware
  Android Keystore, the mobile counterpart of desktop's `keyring`),
- **UnifiedPush** registration (`mw_push_*`) — the self-hostable, no-Google push path
  (see [`push.md`](./push.md)),
- **Share targets & file handlers** for `.eml`/`.ics`/`.vcf`/`.msg`, **badge**,
  **native notifications** (`mw_notify`), **biometric app-lock** (`mw_biometric_*`),
- **Screen-capture protection** via `FLAG_SECURE` (`mw_set_capture_protection`) — a
  real OS control on Android; see [`../security/screen-capture.md`](../security/screen-capture.md),
- **`mailto:` / deep links** — declared statically in the manifest.

The command names are the **frozen bare `mw_*` names** the SPA invokes (identical to
the desktop shell), so the one SPA drives both shells with no forked UI.

## Building the APK

The dev machine used for V5 has the Android SDK + NDK but **no JDK on PATH**, so
`tauri android init/build` runs only in CI (the `android-apk` job). The APK build is
the mobile-shell verification gate: it cross-compiles the Rust cdylib for the Android
targets (exercising the `#[cfg(mobile)]` capability bindings and the `mw_*`
registration) and compiles the custom Kotlin plugins.

The CI job:

1. Provisions a JDK (`temurin` 17), the Android SDK, and NDK (`ndk;27.0.12077973`)
   plus the four Android Rust targets.
2. Builds the shared SPA and emits the UI-bundle hash.
3. `tauri android init --ci` generates `gen/android/` (git-ignored).
4. `python3 apps/mobile/src-tauri/android-src/merge.py` merges the tracked Kotlin plugins
   (`MailwomanMobilePlugin.kt`, `UnifiedPushReceiver.kt`, `FlagSecurePlugin.kt`), the
   share/file-handler/deep-link `<intent-filter>`s + UnifiedPush `<receiver>` +
   runtime `<uses-permission>`s into the manifest, and the Gradle deps
   (`org.unifiedpush.android:connector`, `androidx.security:security-crypto`).
5. `tauri android build --apk` and uploads the APK artifact.

Locally, the merge steps are documented in
`apps/mobile/src-tauri/android-src/README.md`.

> **Honest status:** the `android-apk` job is **`continue-on-error`**. The full
> Android toolchain provisioning + the `gen/` project merge cannot be verified on the
> Windows dev machine (no JDK), so a provisioning or merge failure is a **tracked
> gap**, not a pipeline-red — the same discipline V4 used for its wasm/iOS deferrals.
> The Rust host build, the `mw_*` command-name reconciliation (grep-verified against
> the frozen `tauri.ts`), and the mobile unit tests are green locally; the APK build
> is the CI-only leg.

## ACL note

The `mw_*` capability commands are registered at the **app level** (not as plugin
commands), and Tauri v2 does not ACL-gate an app's own commands. So the frozen
commands need **no** `permissions/` schema or `mailwoman-mobile:default` grant — this
resolves the earlier deferred mobile-plugin-ACL item. `capabilities/default.json`
still grants only the third-party plugin defaults (`core`/`os`/`notification`/
`deep-link`), and `capabilities/mobile.json` grants `biometric:default`.

## F-Droid friendliness

The Android build uses no proprietary dependencies: push is **UnifiedPush** (no
Google Play Services / FCM), and the crypto is RustCrypto/pure-Rust. This keeps the
build F-Droid-compatible. **Store submission itself is out of V5** (see below).

## Documented gaps (ops / sponsorship follow-ups, §28.7 — not V5 gates)

- **iOS shell build/run** — needs macOS + Xcode + a paid Apple account, unavailable
  on the Windows dev/CI machine. The iOS code is a tracked best-effort skeleton
  (`apps/mobile/src-tauri/ios-src/`): matching command names, an APNs registration
  stub, and screen-capture **detection** (there is no iOS prevention API). The local
  build is the gap.
- **APNs live delivery** — needs an Apple account; **mocked/recorded in CI**. The
  wake is opaque (no content) as on every transport.
- **App-store submission + signing accounts** — Play Console / Apple Developer
  account logistics are org/legal, not code. V5 plans the **build** + F-Droid-friendly
  config + the Tauri updater signing plumbing only; submission is a sponsorship/ops
  follow-up.
- **Auto-update feed hosting** — the updater config is wired but inactive; a hosted
  update feed + signing key is an ops follow-up (shared with the desktop shell).
