# Desktop shell live E2E (tauri-driver + WebdriverIO)

Live end-to-end tests for the Mailwoman thin desktop shell (Tauri v2, plan §3 e9 /
§7 DoD). They drive the **real built desktop app** through
[`tauri-driver`](https://v2.tauri.app/develop/tests/webdriver/) — the official
Tauri WebDriver bridge in front of the platform's native WebView driver — and
assert every capability the SPA reaches the OS through actually round-trips.

`specs/desktop.e2e.mjs` (one WebDriver session, ordered):

| Step | Asserts (LIVE) |
|------|----------------|
| boot | Launch succeeds → the §7.4 UI-bundle integrity gate passed (a hash mismatch aborts launch); on a fresh profile the SPA shows the sign-in form with the server-address field (`[data-testid=login-server-address]`, labelled "Mailwoman server address" in English) — the boot-error card (`.async-state--error`) or an empty `#root` fails the run; `window.__MW_CONFIG__` = `{platform.kind:"desktop", native:true, capabilities:true}` |
| keychain | `mw_keychain_set` → `mw_keychain_get` → `mw_keychain_delete` → get(null) against the **real OS keychain** (DPAPI on Windows) |
| capture | `mw_set_capture_protection({enabled:true})` → `{supported:true}` on Windows (WDA_EXCLUDEFROMCAPTURE), then disable |
| notification | `mw_notify(...)` fires a native toast without error |
| badge | `mw_set_badge_count` accepts a count and clears |
| self-contained | `mw_self_contained_status` is `"off"`; `mw_start_local_server` **spawns the bundled `mw-server` sibling on loopback** (blocking until `/healthz` 200) → status `"ready"` → stop → `"off"` |

## Prerequisites

1. **A PRODUCTION desktop build.** This is load-bearing and non-obvious:
   - `cargo build [--release] -p mailwoman-desktop` produces a **dev-mode** binary
     that loads `build.devUrl` (`http://localhost:5173`) and does **not** serve the
     embedded SPA — the E2E will see a blank page.
   - You **must** build via the Tauri CLI so the app embeds + serves `frontendDist`
     from the hash-verified bundle (origin `http://tauri.localhost/`):
     ```sh
     cargo tauri build --no-bundle          # from apps/desktop
     # or the sequenced: scripts/build-shells.{sh,ps1}
     ```
   Override the binary path with `MW_DESKTOP_BIN`.

2. **`tauri-driver`** on PATH or `~/.cargo/bin` (`cargo install tauri-driver`).
   Override with `TAURI_DRIVER`.

3. **A native WebDriver matching the installed WebView:**
   - **Windows:** `msedgedriver.exe` whose version matches the **WebView2 runtime**
     (e.g. WebView2 `146.0.3856.84` → MSEdgeDriver `146.0.3856.84`, from
     `https://msedgedriver.microsoft.com/<version>/edgedriver_win64.zip`). Point
     `MSEDGEDRIVER` at it (passed to `tauri-driver --native-driver`), or put it on
     PATH.
   - **Linux:** `WebKitWebDriver` on PATH (tauri-driver finds it); the CI Linux job
     runs headless under `xvfb-run`.

4. **The bundled `mw-server` resource** (for the self-contained step): copy the
   release `mw-server` (the `mailwoman` binary) to `resources/mw-server[.exe]` next
   to the desktop binary — `scripts/bundle-server.*` does this as part of
   `build-shells`. If absent, the self-contained step **skips** with a clear note
   (the Rust integration test `cargo test -p mailwoman-desktop selfcontained`
   proves the spawn + `/healthz` + a JMAP round-trip live regardless).

## Run

```sh
pnpm install                       # in apps/desktop/e2e
# Windows example:
MW_DESKTOP_BIN=../../../target/release/mailwoman-desktop.exe \
MSEDGEDRIVER=/path/to/msedgedriver.exe \
  pnpm test
```

## Verified LIVE on this machine (Windows 11, WebView2 146.0.3856.84)

All six steps pass against `target/release/mailwoman-desktop.exe` (production
`cargo tauri build --no-bundle`) with tauri-driver `2.0.5` + MSEdgeDriver
`146.0.3856.84`. The self-contained step spawned the bundled `mw-server` live on a
loopback ephemeral port and shut it down cleanly.

Re-run on 2026-10-12 (same machine, not elevated, tauri-driver `2.0.5`, shell built with
`tauri build --no-bundle` and no bundled server): steps 1-5 pass and step 6 skips, on
the Evergreen runtime `146.0.3856.84` and on a Fixed Version runtime `153.0.4234.48`,
each with the MSEdgeDriver of the same version. The `desktop-e2e` CI job has not passed
yet; the next section is what is known about that.

## When the session does not open (Windows)

How a session starts: `tauri-driver` forwards the request to `msedgedriver` with
`browserName: "webview2"` and the shell as `binary`. `msedgedriver` starts the shell
with no arguments and hands it everything else in two environment variables:
`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` (among others `--remote-debugging-port=0`) and
`WEBVIEW2_USER_DATA_FOLDER` (a temporary `scoped_dir…` folder, or
`webviewOptions.userDataFolder` when one is given). It then waits up to 60 s for
`<user data folder>\EBWebView\DevToolsActivePort`, which the WebView2 browser process
writes once its debug port is open.

What the driver's error says about the shell:

| `session not created: …` | Meaning |
|---|---|
| `DevToolsActivePort file doesn't exist`, after 60 s | The shell process stayed alive, and no WebView2 browser process wrote a debug port into the folder the driver watches. |
| `Chrome instance exited…` or `Microsoft Edge failed to start: was killed`, at once | The shell process exited. |

Two ways to reach the first row:

- **The shell cannot create its WebView.** Tauri then shows a dialog titled "Error"
  ("Could not find the WebView2 Runtime…") and the process stays alive behind it. A
  check that only asks whether the process is still running cannot tell this from a
  working shell. To reproduce, run with `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` set to a
  folder that does not exist.
- **The shell runs elevated.** From WebView2 Runtime 150 an elevated (high-integrity)
  host ignores the `WEBVIEW2_*` environment variables and `HKCU` policy; only `HKLM`
  policy and arguments passed in code are honoured (Microsoft's statement in
  `MicrosoftEdge/WebView2Feedback#5640`). The shell then opens its WebView in its own
  user data folder with no debug port. GitHub-hosted Windows runners run every step as
  an elevated administrator. This case was not reproduced on a developer machine: it
  needs an elevated session.

  For an elevated run, supply the debug port through machine policy and tell the driver
  which folder the shell uses:

  ```pwsh
  $key = 'HKLM:\SOFTWARE\Policies\Microsoft\Edge\WebView2\AdditionalBrowserArguments'
  New-Item -Path $key -Force | Out-Null
  New-ItemProperty -Path $key -Name 'mailwoman-desktop.exe' -PropertyType String `
    -Value '--remote-debugging-port=0' -Force | Out-Null
  $env:MW_E2E_WEBVIEW_USER_DATA_FOLDER = "$env:LOCALAPPDATA\com.mailwoman.desktop"
  ```

  Verified without elevation: `MW_E2E_WEBVIEW_USER_DATA_FOLDER` reaches `msedgedriver`
  as `webviewOptions.userDataFolder`, the driver reads the port from that folder, and a
  stale `DevToolsActivePort` left there does not break the next session. Not verified:
  the policy half, and the two together on an elevated machine. The policy makes every
  launch of `mailwoman-desktop.exe` on that machine open a debug port, so it belongs on
  throw-away CI machines only.

What a failed run prints: when `MSEDGEDRIVER` is set, `wdio.conf.mjs` starts the driver
through a wrapper script with `--verbose` and writes `logs/msedgedriver.log`. 25 s after
start it runs `diagnostics/session-snapshot.ps1`, which records whether the run is
elevated, the shell's windows and any dialog text, and the shell's WebView2 browser
process with its user data folder and whether `--remote-debugging-port` reached it. Both
are printed under `---- session diagnostics ----` when the run fails, and not when it
passes.

To run against another WebView2 Runtime without installing it, extract a Fixed Version
runtime, point `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` at it, and put the `msedgedriver` of
the same version in `MSEDGEDRIVER`.

## When the SPA does not mount (Windows)

The session opens, the window is titled `Mailwoman`, and the `before` hook fails with
`SPA did not mount into #root` after 30 s. CI run 39895491554 failed this way: the driver
log shows the page committed `http://tauri.localhost/` as `text/plain`, with a service
worker for `http://tauri.localhost/sw.js` already registered.

Cause (reproduced on WebView2 154.0.4258.62, not elevated): the SPA registers `sw.js`
on startup, also inside the shell. On every later launch in the same WebView2 user data
folder that worker answers the first navigation, and its `fetch()` of
`http://tauri.localhost/` (and of `sw.js`) goes to the network instead of the shell's
asset handler. Whatever answers there becomes the document, with the shell's IPC
available to it. With nothing answering, the fetch fails and the worker serves the
`index.html` it stored on the first launch, which is why a second local launch usually
mounts. CI launches the shell once directly (the diagnostic step) in
`%LOCALAPPDATA%\com.mailwoman.desktop` and then points the driver at that same folder,
so the WebDriver launch is a second launch. What answered on the runner is not known.

To reproduce: run the suite twice with the same `MW_E2E_WEBVIEW_USER_DATA_FOLDER`, after
deleting `EBWebView\Default\Service Worker\CacheStorage` from it between the runs; the
second run fails with the WebView2 error page (`ERR_FAILED`). Launching the shell
directly with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--proxy-server=http://127.0.0.1:<port>
--proxy-bypass-list=<-loopback>"` shows the worker's requests arriving at that proxy on
the second launch and none on the first.

The boot step fails on the first screen it sees other than the sign-in form: an empty
`#root` (`SPA did not mount into #root`), or the boot-error card, whose text is in the
message (`SPA did not reach the sign-in form …: {"kind":"error", …}`). The card ends the
wait at once; anything else (still loading, another screen) fails after 30 s. "Fresh
profile" also means no selected server in `%APPDATA%\com.mailwoman.desktop\servers.json`:
with one that does not answer, the SPA shows the card and the run fails.

What a failed run prints: the document the WebView holds (URL, MIME type, start of the
HTML, the navigation's `responseStatus`, `deliveryType` and `workerStart`, the service
worker in control), then the same after unregistering every service worker and
reloading. `workerStart` above 0 means a worker answered the navigation;
`deliveryType: "cache-storage"` means it answered from its cache. The test fails either
way.

`MW_E2E_DRIVER_PORT` moves `tauri-driver` off port 4444 (the native driver takes the next
port), for running beside another session.

## CI / driver notes for e8

- The `desktop-e2e` job must install `tauri-driver` and a WebView2-matched
  `msedgedriver` (Windows) or `WebKitWebDriver` (Linux, under `xvfb-run`), build
  the shell with **`tauri build`** (not plain `cargo build`), stage the bundled
  `mw-server`, then run `pnpm -C apps/desktop/e2e test`.
- **Documented desktop capability limit (not a driver gap):** OS-side notification
  **action-button** delivery back to the SPA is plugin/OS-limited on desktop Tauri
  (mobile-centric); the `mw://notification-action` choke-point event bridge is
  wired and unit-tested, but the button-click round-trip is not headless-driveable.
  The `mw_notify` toast dispatch itself is the live acceptance here.
