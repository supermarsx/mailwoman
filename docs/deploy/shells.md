# Desktop and mobile apps: building and provisioning per deployment

The desktop app (`apps/desktop`) and the Android app (`apps/mobile`) are not served
by the Mailwoman server they talk to, so each needs to know that server's address.
Users never enter it. The address is set by whoever builds or installs the app:

1. a **provisioning file** on the desktop, installed by an administrator, or
2. the address **compiled into the build** (`MW_SHELL_SERVER_URL`), or
3. neither: the app shows "This app has not been set up for a server. Contact your
   administrator." and offers no way to continue.

The provisioning file wins over the compiled address. Both are read once, at start.
There is no server field on the sign-in screen and no server list in Settings. If
the server does not answer at start, the app shows the error with **Retry** and
"If this keeps happening, contact your administrator."

The cost of this model is one build per deployment (or one build plus a provisioning
file per desktop fleet).

## Building with a server address

```sh
# Linux/macOS
MW_SHELL_SERVER_URL=https://mail.example.org bash scripts/build-shells.sh
# Windows
$env:MW_SHELL_SERVER_URL = 'https://mail.example.org'; pwsh scripts/build-shells.ps1
```

`MW_SHELL_SERVER_URL` must be `https://host[:port][/path]`: the address users open
Mailwoman at in a browser, including a sub-path if the server runs under one
(`MW_BASE_PATH`). No user name, password, query or fragment. `http://` is accepted
only for a loopback host (`localhost`, `127.x.x.x`, `[::1]`), for testing against a
local server.

`build-shells` stops before building anything if the variable is missing or is not
such an address. The shell crates check it again when they compile, so a build run
some other way (`cargo build`, `tauri build`) with a bad value fails too:

```
error[E0080]: evaluation panicked: MW_SHELL_SERVER_URL must be https://host[:port][/path] (http only to localhost, 127.x or [::1])
```

A build made with plain `cargo build` or `tauri build` and no variable at all has no
address; it shows the not-set-up notice unless a provisioning file names a server.

### Building without a server address

For a desktop fleet that is configured only through the provisioning file, or for a
CI build that only checks sizes, say so explicitly:

```sh
MW_SHELL_NO_SERVER_URL=1 bash scripts/build-shells.sh
```

### Android

The APK build is not run by `build-shells` (it needs the Android SDK, NDK and a
JDK). The variable applies the same way, because the Rust part of the app is
compiled by the same command:

```sh
MW_SHELL_SERVER_URL=https://mail.example.org pnpm -C apps/mobile exec tauri android build --apk
```

Android has no provisioning file. Managed configuration through an MDM (Android app
restrictions) is not implemented yet; it needs a Kotlin plugin. iOS is not covered.

## Desktop provisioning file

An administrator can point already-installed desktop apps at a server, or move them
to another one, without a new build, by installing this file:

| OS | Path |
|---|---|
| Windows | `%ProgramData%\Mailwoman\server.json` (normally `C:\ProgramData\Mailwoman\server.json`) |
| macOS | `/Library/Application Support/Mailwoman/server.json` |
| Linux | `/etc/mailwoman/server.json` |

Content, nothing else:

```json
{ "serverUrl": "https://mail.example.org" }
```

The same address rules apply as for `MW_SHELL_SERVER_URL`. The file is read when the
app starts; a change applies at the next start.

The app uses the file only when an administrator owns both the file and its folder:

- **Windows**: the owner must be `BUILTIN\Administrators` or `SYSTEM`. This is what
  you get when an elevated administrator, Group Policy, Intune or another tool that
  runs as SYSTEM creates the file. On Windows a standard user can create folders and
  files under `%ProgramData%`, so the app does not trust a file or folder a user
  created there.
- **macOS and Linux**: the owner must be `root`, and neither the file nor the folder
  may be writable by group or others (for example `chmod 644` for the file and
  `chmod 755` for the folder).

If the file exists but cannot be used (not owned by an administrator, not JSON, an
unknown field, an address that breaks the rules, or larger than 16 KiB), the app
does **not** fall back to the compiled address: it shows the not-set-up notice. A
broken override then shows up at once instead of silently sending users to the
server the file was meant to replace. The reason is written to the app's standard
error output (`[mailwoman] no server address: …`).

Example, Windows (elevated PowerShell):

```powershell
New-Item -ItemType Directory -Force "$env:ProgramData\Mailwoman" | Out-Null
Set-Content -Encoding utf8 "$env:ProgramData\Mailwoman\server.json" '{ "serverUrl": "https://mail.example.org" }'
```

If `%ProgramData%\Mailwoman` already existed and was created by a standard user,
delete it first, or take ownership (`takeown /F "%ProgramData%\Mailwoman" /A /R`).

Example, Linux:

```sh
sudo install -d -m 755 /etc/mailwoman
echo '{ "serverUrl": "https://mail.example.org" }' | sudo tee /etc/mailwoman/server.json >/dev/null
sudo chmod 644 /etc/mailwoman/server.json
```

## Branding the build

| Variable | Effect | Default |
|---|---|---|
| `MW_SHELL_PRODUCT_NAME` | The app name: window title, error box caption, installer and bundle names, and on Android the app label and the unread-count notification title. At most 64 characters, no control characters, `/` or `\`. | `Mailwoman` |
| `MW_SHELL_IDENTIFIER` | The desktop bundle identifier (reverse DNS, such as `org.example.mail`). It also decides where the app keeps its local data, so changing it for an existing installation starts with an empty profile. | `com.mailwoman.desktop` |

`build-shells` passes them to `tauri build` as a config overlay (`--config`), so
`tauri.conf.json` stays unchanged in the repository. For Android pass the overlay
yourself, for example
`tauri android build --apk --config '{"productName":"Example Mail"}'`; the Android
identifier stays `com.mailwoman.mobile`, because the Kotlin plugins live in that
package.

## Desktop updates

A desktop build has an updater only when it is given an update feed and the public
key the updates are signed with. Without both it has none: the committed
`tauri.conf.json` names no feed and no key.

| Variable | Effect |
|---|---|
| `MW_SHELL_UPDATER_URL` | The update feed. `https://` only, no user name or password. Normally a template such as `https://updates.example.org/mail/{{target}}/{{arch}}/{{current_version}}`; a fixed URL (one JSON file for every platform) is accepted with a note. |
| `MW_SHELL_UPDATER_PUBKEY` | The minisign public key, exactly as `tauri signer generate` prints it (the base64 text of the whole `.pub` file, not the bare `RW…` line). |

With both set, `build-shells` adds `plugins.updater` (the key, the feed, Windows
install mode `passive`) and `bundle.createUpdaterArtifacts: true` to the config
overlay, and the app registers the updater plugin when it starts. With either one
missing there is no updater block, the plugin is not registered, and the build says
so (`updater off: MW_SHELL_UPDATER_PUBKEY is not set …`). An invalid value (not
`https://`, a user name in the URL, a key that is not a minisign public key) stops
`build-shells` before it builds anything. The check is
`scripts/shells/config-overlay.mjs`; its tests are
`node --test scripts/shells/*.test.mjs`.

### Signing keys, one pair per deployment

```sh
pnpm -C apps/desktop exec tauri signer generate -w ~/.mailwoman/updater.key
```

The command prints the public key: that is `MW_SHELL_UPDATER_PUBKEY`. The private
key file and its password stay out of the repository; the installer build reads
them from `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` and
signs each update bundle (`.sig` next to it). An app accepts only updates signed by
the key it was built with, so losing the private key means shipping a new build
with a new key by some other channel.

`build-shells` builds with `--no-bundle` (no installers, no update bundles). For the
installers and update bundles of a deployment, pass the same overlay to
`tauri build`:

```sh
node scripts/shells/config-overlay.mjs > overlay.json
pnpm -C apps/desktop exec tauri build --config overlay.json
```

### Feed layout

The app replaces `{{target}}` (`windows`, `linux`, `darwin`), `{{arch}}`
(`x86_64`, `aarch64`, `i686`, `armv7`) and `{{current_version}}` in the feed URL. The
feed answers `204 No Content` when there is nothing newer, or `200` with

```json
{ "version": "26.21.0", "notes": "…", "pub_date": "2026-11-01T12:00:00Z",
  "url": "https://updates.example.org/mail/files/Mailwoman_26.21.0_x64-setup.exe",
  "signature": "<contents of the .sig file for that bundle>" }
```

A fixed URL instead serves one JSON file with a `platforms` object keyed
`<target>-<arch>` (such as `windows-x86_64`), each entry with its `url` and
`signature`. A canary channel is a second feed URL used by a second build.

### Turning updates off

Leave both variables unset; that is the default. A build that already has a feed
keeps it: to stop such builds from updating, answer `204 No Content` at the feed.

### What is not wired yet

Nothing in the app asks the feed for an update yet. The plugin is registered and the
app window may call it (`updater:default` in `capabilities/default.json`), but no
screen and no start-up step does. Mobile apps have no updater; they are updated
through their store.

## Client version header

The apps name themselves to the server in `X-Mailwoman-Client`:

| Client | Value | Sent |
|---|---|---|
| Desktop app | `desktop/<version>; updater=on` or `updater=off` | on sign-in and every request to the server |
| Android and iOS app | `mobile/<version>; updater=off` | on sign-in and every request to the server |
| Browser | `browser/<version>` | on sign-in (`/api/login`, `/api/login/2fa`) |

`<version>` is the app's own release number (`26.20.0`); anything else is sent as
`unknown`. The server allows the header in its answer to the apps' cross-origin
preflight. Recording it with the session and listing the versions in use in the
console come with the session overview; this build does not store it yet.

## The server side

The server must accept the apps' sign-in: the fixed Tauri origins
(`http://tauri.localhost`, `https://tauri.localhost`, `tauri://localhost`) are
accepted by default, and `MW_NATIVE_ORIGINS` adds more (see `push.md`). The apps
sign in with a bearer token, not a cookie (see `desktop.md`, "Native auth").

## Not covered

- The self-contained local server mode of the desktop app (`startLocalServer`) is
  not wired to any screen (HALF-WIRED, decision D11).
- Android managed configuration (MDM app restrictions) and iOS.
