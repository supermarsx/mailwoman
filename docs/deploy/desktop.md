# Desktop app

The desktop app (`apps/desktop`) is a Tauri 2 app for Windows, macOS and Linux. It
embeds the same web client the server serves (`apps/web/dist`), checks that client's
files at every start, and talks to one Mailwoman server over the same API a browser
uses. It contains no mail protocol code of its own. This page is for whoever builds
and distributes the app; the server address, branding and update feed are set as
described in [Building the desktop and Android apps](./shells.md), and what users
see is in [Desktop and mobile apps](../user/apps-desktop-mobile.md). There is no
iOS app.

## Requirements

- The Rust toolchain pinned in `rust-toolchain.toml` (1.98.1), Node.js 22 and pnpm.
  CI uses pnpm 11.12.0.
- **Windows**: the Microsoft Edge WebView2 Runtime on every computer that runs the
  app. Without it the app stops at start with a message box (see
  [Troubleshooting](#troubleshooting)).
- **Linux** build hosts: the WebKitGTK development packages. On Ubuntu, as in CI:

  ```sh
  sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
    libayatana-appindicator3-dev librsvg2-dev libssl-dev patchelf
  ```

- **macOS**: the app requires macOS 10.15 or later (`minimumSystemVersion` in
  `tauri.conf.json`).

## Build the app

1. Set the server address, or say there is none (see
   [Building with a server address](./shells.md#building-with-a-server-address)).
2. Run the build script from the repository root:

   ```sh
   # Linux and macOS
   MW_SHELL_SERVER_URL=https://mail.example.org bash scripts/build-shells.sh
   ```

   ```powershell
   # Windows
   $env:MW_SHELL_SERVER_URL = 'https://mail.example.org'; pwsh scripts/build-shells.ps1
   ```

`build-shells` runs five steps, in this order:

1. builds the web client (`pnpm -C apps/web build`);
2. writes the hash manifest of that build (`node scripts/emit-bundle-hash.mjs`), which
   the app compiles in;
3. builds the release server and copies it into the app's resources as
   `mw-server` (`scripts/bundle-server.sh` or `.ps1`) for
   [self-contained mode](#self-contained-mode); `MW_SELF_CONTAINED=0` skips this;
4. builds the app (`tauri build --no-bundle`, plus the config overlay for branding
   and updates);
5. checks the binary sizes (`node scripts/check-bundle-size.mjs`).

The order matters: the app embeds the files from step 1 and compares them at start
with the manifest from step 2. If you build by hand, keep the same order:

```sh
pnpm -C apps/web build
node scripts/emit-bundle-hash.mjs
pnpm -C apps/desktop install
pnpm -C apps/desktop exec tauri build
```

The binary is `target/release/mailwoman-desktop` (`.exe` on Windows).

### Installers

`build-shells` makes no installers. To build them, run `tauri build` without
`--no-bundle` and pass the overlay the script would use:

```sh
node scripts/shells/config-overlay.mjs > overlay.json
pnpm -C apps/desktop exec tauri build --config overlay.json
```

`tauri.conf.json` builds every installer format Tauri offers for the platform
(`"targets": "all"`); the Windows NSIS installer installs per machine. The Windows
certificate thumbprint and the macOS signing identity and team ID in
`tauri.conf.json` are placeholders (`HUMAN_PROVIDE_…`). Replace them before you build
signed installers; [Packaging](./packaging.md) covers code signing and the store
channels.

### Size check

`scripts/check-bundle-size.mjs` fails the build when:

| Measured | Limit |
|---|---|
| The desktop binary alone | 27 MB |
| The desktop binary plus the bundled `mw-server` (when present in `apps/desktop/src-tauri/resources`) | 204 MB |

The measurements behind these limits are in
[Size budget revision](../perf/size-budget-revision.md). CI runs the same build on
Windows, Linux and macOS in the `desktop-shell` job (macOS is advisory), with
`MW_SHELL_NO_SERVER_URL=1`.

## What happens at start

1. On Windows the app checks for the WebView2 Runtime and stops if it is missing.
2. It hashes every embedded web client file (SHA-256) and compares the result with
   the manifest compiled into the binary. A missing or changed file stops the app
   before any window opens or any server is contacted. A good build logs
   `[mailwoman] UI-bundle integrity OK: N/N files match`.
3. It reads the server address: the provisioning file, then the compiled address
   (see [How the app finds its server](./shells.md#how-the-app-finds-its-server)).
   The log says which one it used, for example
   `[mailwoman] server address from the build`.
4. It opens the main window (1200 x 800, at least 720 x 560) with the web client.

Messages go to standard error with the prefix `[mailwoman]`. A release build on
Windows has no console, so the fatal errors in steps 1 and 2 also appear in a
message box.

## Signing in

The sign-in screen is the web client's: **Email or username**, **Password** and
**Sign in**, with no server field. The desktop app asks the server for a bearer
session instead of a cookie:

1. It sends `POST /api/login` with `"clientType": "native"`.
2. The server answers with the session token in the JSON body and sets no cookie.
3. The app stores the token in the operating system's credential store through the
   `keyring` crate (Windows Credential Manager, macOS Keychain, Linux Secret
   Service), under the service name `mailwoman.session`.
4. Every later request carries `Authorization: Bearer <token>`.

Bearer requests carry no cookie, so the cookie-based CSRF check does not apply to
them. The server accepts the Tauri origins by default; `MW_NATIVE_ORIGINS` adds
others (see [What the server needs](./shells.md#what-the-server-needs)).

## Operating system features

These desktop features are used by the web client:

- **Notifications** for new mail. Clicking one brings the window to the front and
  opens the message.
- **Unread count**: the dock badge on macOS, the launcher count on Linux, and a small
  overlay on the taskbar icon on Windows (the number up to 9, then `9+`).
- **Credential store** for the session token, as above.
- **Screen-capture protection**, turned on when the administrator's client policy
  requires it. What this does on each OS is in
  [Screen capture](../security/screen-capture.md).
- **Background sending of queued changes.** When the app has offline changes
  waiting, it registers a job for the current user (Task Scheduler on Windows, a
  `systemd --user` timer on Linux, a LaunchAgent on macOS) that starts the app with
  `--background-replay` every five minutes. The background run opens a hidden window,
  sends what is queued and exits after at most 140 seconds. The job contains only
  the program path and that switch; the token stays in the credential store. The
  user must be logged in to the computer, and the OS may refuse or delay the job.
  The job is removed once nothing is queued.

The app also contains commands for registering as the `mailto:` handler, Windows
Hello app lock, dragging attachments out, and self-contained mode. No screen in the
web client uses them, so users cannot reach these features.

## Self-contained mode

A build made with `build-shells` (unless `MW_SELF_CONTAINED=0`) carries the release
server as `resources/mw-server`. The app can start it as a separate process on the
same computer:

- it runs `mw-server serve` with `MW_BIND=127.0.0.1:<free port>`,
  `MW_MODE=engine`, `MW_COOKIE_SECURE=false`, `MW_RETAINED_ASSETS_DIR=off`, a random
  32-byte `MW_SERVER_KEY` generated for that run, and the database at
  `<app data folder>/self-contained/mailwoman.db`;
- it waits up to 20 seconds for `GET /healthz` to answer `200`;
- it restarts the server up to five times if it exits, and stops it when the app
  quits.

No screen in the web client starts this mode, so a user cannot switch to it. A build
with the bundled server behaves like a build without it.

## Updates

The desktop app updates itself only if the build has an update feed and a signing
key; see [Desktop updates](./shells.md#desktop-updates). No screen and no start-up
step checks the feed.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Message box: "Mailwoman needs the Microsoft Edge WebView2 Runtime, which was not found …" | WebView2 is not installed (Windows). | Install the WebView2 Runtime from Microsoft and start the app again. |
| The app exits at start; standard error shows `UI-bundle integrity check failed: UI-bundle asset hash mismatch for …` or `… missing from shell` | The embedded web client does not match the compiled manifest: the build steps ran out of order, or the binary was changed. | Rebuild in order: web client, `emit-bundle-hash.mjs`, then `tauri build`. |
| **This app has not been set up for a server. Contact your administrator.** | No compiled address and no provisioning file, or a provisioning file that cannot be used. | Read the `[mailwoman] no server address: …` line on standard error, then fix the file or rebuild with `MW_SHELL_SERVER_URL`. |
| The error card with **Retry** and **If this keeps happening, contact your administrator.** | The configured server did not answer at start. | Check that the address in the build or provisioning file is reachable from that computer. |

## Related

- [Building the desktop and Android apps](./shells.md)
- [Android app](./mobile-android.md)
- [Packaging and store channels](./packaging.md)
- [Desktop and mobile apps (user guide)](../user/apps-desktop-mobile.md)
- [Screen capture](../security/screen-capture.md)
