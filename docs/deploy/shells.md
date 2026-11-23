# Building the desktop and Android apps for your server

The desktop app (`apps/desktop`) and the Android app (`apps/mobile`) are not served
by the Mailwoman server they talk to, so each needs that server's address. Users
never type it in. You set it when you build the app, or, on the desktop, with a
provisioning file. This page covers both, plus branding, desktop updates and how
the apps identify themselves to the server. Platform build details are on
[Desktop app](./desktop.md) and [Android app](./mobile-android.md); what users see
is on [Desktop and mobile apps](../user/apps-desktop-mobile.md).

## How the app finds its server

At start the app looks for an address in this order and uses the first it finds:

1. a **provisioning file** that an administrator installed on the computer (desktop
   only, see [Desktop provisioning file](#desktop-provisioning-file));
2. the address **compiled into the build** from `MW_SHELL_SERVER_URL`;
3. none. The app then shows **This app has not been set up for a server. Contact
   your administrator.** and offers no way to continue.

<!-- SCREENSHOT deploy/shell-not-configured: desktop app window, built without a server address and no provisioning file, full window showing "This app has not been set up for a server. Contact your administrator." -->

The address is read once, at start. The sign-in screen has no server field and
Settings has no server list. If the server does not answer at start, the app shows
the error with **Retry** and, below it, **If this keeps happening, contact your
administrator.**

<!-- SCREENSHOT deploy/shell-server-unreachable: desktop app window, configured server not answering at start, crop of the error card with the Retry button and "If this keeps happening, contact your administrator." -->

So each deployment needs its own build, or, for a desktop fleet, one build plus a
provisioning file.

## Building with a server address

```sh
# Linux and macOS
MW_SHELL_SERVER_URL=https://mail.example.org bash scripts/build-shells.sh
```

```powershell
# Windows
$env:MW_SHELL_SERVER_URL = 'https://mail.example.org'; pwsh scripts/build-shells.ps1
```

`MW_SHELL_SERVER_URL` must have the form `https://host[:port][/path]`: the address
users open Mailwoman at in a browser, including the sub-path if the server runs
under one (`MW_BASE_PATH`). It must not contain a user name, password, query or
fragment. `http://` is accepted only for a loopback host (`localhost`, `127.x.x.x`,
`[::1]`), for testing against a local server.

`build-shells` stops before building anything if the variable is missing or has
another form. The shell crates check the value again when they compile, so a build
started some other way (`cargo build`, `tauri build`) with a bad value fails too:

```
error[E0080]: evaluation panicked: MW_SHELL_SERVER_URL must be https://host[:port][/path] (http only to localhost, 127.x or [::1])
```

A build made with plain `cargo build` or `tauri build` and no variable at all has no
address. It shows the not-set-up notice unless a provisioning file names a server.

### Building without a server address

For a desktop fleet that you configure only through the provisioning file, or for a
build that only checks sizes, say so explicitly:

```sh
MW_SHELL_NO_SERVER_URL=1 bash scripts/build-shells.sh
```

### Android

`build-shells` does not build the Android app. The variable works the same way,
because the Rust part of the app is compiled by the Android build:

```sh
MW_SHELL_SERVER_URL=https://mail.example.org pnpm -C apps/mobile exec tauri android build --apk
```

The full Android procedure is on [Android app](./mobile-android.md). Android has no
provisioning file and no managed configuration (MDM app restrictions): the compiled
address is the only source.

## Desktop provisioning file

An administrator can point installed desktop apps at a server, or move them to
another one, without a new build, by installing this file:

| OS | Path |
|---|---|
| Windows | `%ProgramData%\Mailwoman\server.json` (`C:\ProgramData\Mailwoman\server.json` when `ProgramData` is not set) |
| macOS | `/Library/Application Support/Mailwoman/server.json` |
| Linux | `/etc/mailwoman/server.json` |

The file contains exactly one field:

```json
{ "serverUrl": "https://mail.example.org" }
```

The same address rules apply as for `MW_SHELL_SERVER_URL`. The app reads the file
when it starts, so a change applies at the next start.

The app uses the file only when an administrator owns both the file and its folder:

- **Windows**: the owner must be `BUILTIN\Administrators` or `SYSTEM`. You get this
  when an elevated administrator, Group Policy, Intune or another tool running as
  SYSTEM creates the file. A standard user can create folders and files under
  `%ProgramData%`, so the app does not trust a file or folder a user created there.
- **macOS and Linux**: the owner must be `root`, and neither the file nor the folder
  may be writable by group or others (for example `chmod 644` for the file and
  `chmod 755` for the folder).

::: warning A broken file disables the compiled address
If the file exists but cannot be used (not owned by an administrator, not JSON, an
unknown field, an address that breaks the rules, or larger than 16 KiB), the app
does **not** fall back to the compiled address. It shows the not-set-up notice, so a
broken override is visible at once instead of silently sending users to the server
the file was meant to replace. The reason is written to the app's standard error
output as `[mailwoman] no server address: …`.
:::

Windows, in an elevated PowerShell:

```powershell
New-Item -ItemType Directory -Force "$env:ProgramData\Mailwoman" | Out-Null
Set-Content -Encoding utf8 "$env:ProgramData\Mailwoman\server.json" '{ "serverUrl": "https://mail.example.org" }'
```

If `%ProgramData%\Mailwoman` already exists and a standard user created it, delete
it first or take ownership (`takeown /F "%ProgramData%\Mailwoman" /A /R`).

Linux:

```sh
sudo install -d -m 755 /etc/mailwoman
echo '{ "serverUrl": "https://mail.example.org" }' | sudo tee /etc/mailwoman/server.json >/dev/null
sudo chmod 644 /etc/mailwoman/server.json
```

## Branding the build

| Variable | What it does | Default |
|---|---|---|
| `MW_SHELL_PRODUCT_NAME` | The app name: window title, Windows error box caption, installer and bundle names. At most 64 characters, no control characters, `/` or `\`. | `Mailwoman` |
| `MW_SHELL_IDENTIFIER` | The desktop bundle identifier, in reverse DNS form such as `org.example.mail`. It also decides where the app keeps its local data, so changing it for an existing installation starts with an empty profile. | `com.mailwoman.desktop` |

`build-shells` passes both to `tauri build` as a config overlay (`--config`), so
`tauri.conf.json` stays unchanged in the repository.

For Android, the variable sets the window title only. The app label, which also
titles the unread-count notification, comes from `productName`, so pass the overlay
yourself:

```sh
pnpm -C apps/mobile exec tauri android build --apk --config '{"productName":"Example Mail"}'
```

The Android identifier stays `com.mailwoman.mobile`, because the Kotlin plugins are
in that package.

## Desktop updates

A desktop build has an updater only when it is given an update feed and the public
key the updates are signed with. The committed `tauri.conf.json` names neither, so
by default there is no updater.

| Variable | What it does | Default |
|---|---|---|
| `MW_SHELL_UPDATER_URL` | The update feed. `https://` only, no user name or password. Normally a template such as `https://updates.example.org/mail/{{target}}/{{arch}}/{{current_version}}`; a fixed URL (one JSON file for every platform) is accepted with a note. | unset |
| `MW_SHELL_UPDATER_PUBKEY` | The minisign public key, exactly as `tauri signer generate` prints it (the base64 text of the whole `.pub` file, not the bare `RW…` line). | unset |

With both set, `build-shells` adds `plugins.updater` (the key, the feed, Windows
install mode `passive`) and `bundle.createUpdaterArtifacts: true` to the config
overlay, and the app registers the updater plugin when it starts. With either one
missing there is no updater block, the plugin is not registered, and the build says
so, for example `updater off: MW_SHELL_UPDATER_PUBKEY is not set (both are needed);
this build will not update itself`. An invalid value (not `https://`, a user name in
the URL, a key that is not a minisign public key) stops `build-shells` before it
builds anything. The check is `scripts/shells/config-overlay.mjs`; its tests run
with `node --test scripts/shells/*.test.mjs`.

::: warning The app does not check for updates on its own
The updater plugin is registered and the app window is allowed to call it
(`updater:default` in `apps/desktop/src-tauri/capabilities/default.json`), but no
screen and no start-up step calls it. A build with a feed does not download updates
by itself.
:::

The Android app has no updater; it is updated through the store it was installed
from.

### Signing keys, one pair per deployment

```sh
pnpm -C apps/desktop exec tauri signer generate -w ~/.mailwoman/updater.key
```

The command prints the public key: use it as `MW_SHELL_UPDATER_PUBKEY`. Keep the
private key file and its password out of the repository. The installer build reads
them from `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` and
signs each update bundle (a `.sig` file next to it). An app accepts only updates
signed by the key it was built with, so if you lose the private key you must ship a
new build with a new key through some other channel.

`build-shells` builds with `--no-bundle` (no installers, no update bundles). To make
the installers and update bundles of a deployment, pass the same overlay to
`tauri build`:

```sh
node scripts/shells/config-overlay.mjs > overlay.json
pnpm -C apps/desktop exec tauri build --config overlay.json
```

### Feed layout

The app replaces `{{target}}` (`windows`, `linux`, `darwin`), `{{arch}}`
(`x86_64`, `aarch64`, `i686`, `armv7`) and `{{current_version}}` in the feed URL.
The feed answers `204 No Content` when there is nothing newer, or `200` with:

```json
{ "version": "<version>", "notes": "…", "pub_date": "2026-11-01T12:00:00Z",
  "url": "https://updates.example.org/mail/files/Mailwoman_<version>_x64-setup.exe",
  "signature": "<contents of the .sig file for that bundle>" }
```

A fixed URL instead serves one JSON file with a `platforms` object keyed
`<target>-<arch>` (such as `windows-x86_64`), each entry with its `url` and
`signature`. A canary channel is a second feed URL used by a second build.

### Turning updates off

Leave both variables unset, which is the default. A build that already has a feed
keeps it: to stop such builds from updating, answer `204 No Content` at the feed.

## Client version header

The apps name themselves to the server in the `X-Mailwoman-Client` header:

| Client | Value | Sent |
|---|---|---|
| Desktop app | `desktop/<version>; updater=on` or `updater=off` | on sign-in and on every request to the server |
| Android app | `mobile/<version>; updater=off` | on sign-in and on every request to the server |
| Browser | `browser/<version>` | on sign-in (`/api/login`, `/api/login/2fa…`, `/api/login/passkey/finish`) |

`<version>` is the app's own version number. A value with other characters, or
longer than 32 characters, is sent as `unknown`. The server stores the header with
the session, and the admin console shows it on the **Sessions** page in the
**Device and client** column.

## What the server needs

The server must accept the apps' requests. The Tauri origins
(`http://tauri.localhost`, `https://tauri.localhost`, `tauri://localhost`) are
accepted by default; `MW_NATIVE_ORIGINS` (comma-separated) adds more, for example
for a shell with another scheme. Browser deployments are not affected. The apps sign
in with a bearer token instead of a cookie; see
[Desktop app](./desktop.md#signing-in).

## Related

- [Desktop app](./desktop.md)
- [Android app](./mobile-android.md)
- [Desktop and mobile apps (user guide)](../user/apps-desktop-mobile.md)
- [Packaging and store channels](./packaging.md)
- [Push relay](./push.md)
