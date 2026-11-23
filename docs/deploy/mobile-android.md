# Android app

The Android app (`apps/mobile`) is a Tauri 2 app that embeds the same web client the
server serves (`apps/web/dist`) and talks to one Mailwoman server, compiled into the
build. It contains no mail protocol code of its own. This page is for whoever builds
and distributes the APK; the server address and branding are set as described in
[Building the desktop and Android apps](./shells.md), and what users see is in
[Desktop and mobile apps](../user/apps-desktop-mobile.md). There is no iOS app, and
the server does not send Apple push notifications (APNs).

## Requirements

These are the versions the `android-apk` CI job uses:

- a JDK 17 (CI: Temurin 17);
- the Android SDK with `platform-tools`, and NDK r27 (the exact package is
  `ndk_version` in the `android-apk` job of `.github/workflows/ci.yml`), with
  `NDK_HOME` set to the NDK folder, `$ANDROID_HOME/ndk/<ndk version>`;
- the Rust toolchain from `rust-toolchain.toml` (1.98.1) with the four Android
  targets:

  ```sh
  rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
  ```

- Node.js 22, pnpm (CI: 11.12.0) and Python 3.

The app runs on Android 7.0 and later (`minSdkVersion` 24 in
`apps/mobile/src-tauri/tauri.conf.json`).

## Build the APK

Run these from the repository root.

1. Build the web client and its hash manifest:

   ```sh
   pnpm -C apps/web install --frozen-lockfile
   pnpm -C apps/web build
   node scripts/emit-bundle-hash.mjs
   ```

2. Generate the Android project in `apps/mobile/src-tauri/gen/android/` (not tracked
   in git):

   ```sh
   pnpm -C apps/mobile install
   pnpm -C apps/mobile exec tauri android init --ci
   ```

3. Merge Mailwoman's Android sources into the generated project:

   ```sh
   python3 apps/mobile/src-tauri/android-src/merge.py
   ```

4. Build the APK with your server's address:

   ```sh
   MW_SHELL_SERVER_URL=https://mail.example.org pnpm -C apps/mobile exec tauri android build --apk
   ```

   To change the app name, add `--config '{"productName":"Example Mail"}'` (see
   [Branding the build](./shells.md#branding-the-build)).

The APK is written under `apps/mobile/src-tauri/gen/android/app/build/outputs/`. It
is not signed with a release key; sign it for your distribution channel as described
in [Packaging](./packaging.md).

`merge.py` can run again safely: each step is skipped when it has already been
applied. It does four things:

1. copies `MailwomanMobilePlugin.kt`, `UnifiedPushReceiver.kt` and
   `FlagSecurePlugin.kt` into the app's `com.mailwoman.mobile` package;
2. adds the Gradle dependencies `org.unifiedpush.android:connector:2.5.0` and
   `androidx.security:security-crypto:1.1.0-alpha06`;
3. adds to `AndroidManifest.xml` the share and file-open intent filters
   (`message/rfc822`, `text/calendar`, `text/vcard`, `application/vnd.ms-outlook`),
   the `mailto:` and `mailwoman:` link filter, the UnifiedPush receiver, and the
   `POST_NOTIFICATIONS` and `USE_BIOMETRIC` permissions;
4. writes `proguard-mailwoman.pro` with the R8 rules the release build needs for
   those dependencies.

The steps are also described by hand in `apps/mobile/src-tauri/android-src/README.md`.

## What the app does on Android

- **Server address**: compiled in from `MW_SHELL_SERVER_URL`. There is no
  provisioning file and no managed configuration on Android. Without an address the
  app shows **This app has not been set up for a server. Contact your
  administrator.**
- **Sign-in**: the same bearer-token sign-in as the desktop app (see
  [Desktop app](./desktop.md#signing-in)). The token is kept in
  `EncryptedSharedPreferences`, encrypted with AES-256-GCM under a key held in the
  Android Keystore.
- **Push**: UnifiedPush, so the app needs no Google Play Services. Registering needs
  a UnifiedPush distributor app on the phone; if several are installed and none was
  chosen before, the app uses the first one Android lists. Server setup is in
  [Push relay](./push.md).
- **Notifications and unread count**: new-mail notifications, and the unread count as
  a notification reading `<count> unread`.
- **Screen-capture protection**: `FLAG_SECURE` on the app window when the
  administrator's client policy requires it (see
  [Screen capture](../security/screen-capture.md)).

The manifest also declares share targets, file handlers for `.eml`, `.ics`, `.vcf`
and `.msg`, and `mailto:` links, and the app contains a biometric app-lock command.
No screen in the web client uses them, so sharing to the app, opening those files
with it or tapping a `mailto:` link starts the app without passing the content on.

## F-Droid

`packaging/fdroid/metadata/com.mailwoman.mobile.yml` is a draft recipe for the
F-Droid repository; its notes are in `packaging/fdroid/README.md`. The app has no
proprietary dependencies: push goes through UnifiedPush instead of Firebase Cloud
Messaging. The recipe has not been submitted to F-Droid or built on F-Droid's build
server.

::: warning The recipe builds an app without a server address
The recipe's build steps do not set `MW_SHELL_SERVER_URL`, so an APK built from it
shows the not-set-up notice for every user. Add the variable for your server before
you use the recipe.
:::

## Updates

The Android app has no updater of its own. Users get new versions from the store or
repository they installed it from.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `tauri android init` fails with `Android NDK invalid. Make sure the NDK_HOME environment variable has correct value.` | `NDK_HOME` is empty or points at the wrong folder. | Set `NDK_HOME` to the installed NDK folder, such as `$ANDROID_HOME/ndk/<ndk version>`. |
| `merge.py` prints `[android-merge] ERROR: … build.gradle.kts not found — did 'tauri android init' run?` | The generated project does not exist yet. | Run step 2 first. |
| The build fails with `MW_SHELL_SERVER_URL must be https://host[:port][/path] …` | The address has another form. | Use `https://`, or `http://` only to a loopback host. |

## Related

- [Building the desktop and Android apps](./shells.md)
- [Desktop app](./desktop.md)
- [Push relay](./push.md)
- [Packaging and store channels](./packaging.md)
- [Desktop and mobile apps (user guide)](../user/apps-desktop-mobile.md)
