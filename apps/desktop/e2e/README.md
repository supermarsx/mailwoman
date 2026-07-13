# Desktop shell live E2E (tauri-driver + WebdriverIO)

Live end-to-end tests for the Mailwoman thin desktop shell (Tauri v2, plan §3 e9 /
§7 DoD). They drive the **real built desktop app** through
[`tauri-driver`](https://v2.tauri.app/develop/tests/webdriver/) — the official
Tauri WebDriver bridge in front of the platform's native WebView driver — and
assert every capability the SPA reaches the OS through actually round-trips.

`specs/desktop.e2e.mjs` (one WebDriver session, ordered):

| Step | Asserts (LIVE) |
|------|----------------|
| boot | Launch succeeds → the §7.4 UI-bundle integrity gate passed (a hash mismatch aborts launch); the shared SPA mounts into `#root`; `window.__MW_CONFIG__` = `{platform.kind:"desktop", native:true, capabilities:true}` |
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
