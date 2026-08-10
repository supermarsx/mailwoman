# Screen-capture protection: an honest statement

Mailwoman's rule for this feature has not changed since V4: **say exactly what
each platform can and cannot do, and never pretend otherwise.** What changed in
**V5** is that the thin desktop and mobile shells now carry **real, OS-enforced
capture exclusion** where the operating system provides it — and honestly fall
back to the visible watermark where it does not.

**A web browser still cannot prevent, block, or detect screenshots or screen
recordings.** There is no web API for it. Any product that claims its *web app*
stops screen capture is wrong or lying. Mailwoman does not make that claim.

> ⚠️ **Correction (26.19) — the browser watermark does not render.** This
> document said that in the browser you get the watermark deterrent. You do not.
> The server side is real: `watermark.rs` and its route exist and the admin
> setting is honoured server-side. **The web client has no consumer for it** —
> no overlay component, no CSS, no fetch. So a web deployment today has *no*
> screen-capture control of any kind, not a weak one. Two source comments
> elsewhere also refer to "the V4 watermark" as though it shipped.
>
> This is the file that exists to be the honest edition, which makes it the
> worst place in the tree to have been wrong. Where the matrix below says
> "Watermark-only", read **"nothing, until the overlay is built"**. The
> `{ supported: false }` return value and the OS-enforced rows are unaffected
> and remain accurate.

## The honest matrix

| Platform | Mechanism | What it does | Result |
|---|---|---|---|
| **Windows** (desktop shell) | `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` via Tauri `set_content_protected` | OS excludes the window from screenshots + screen recording; a capture of the window is **black** | **Prevents** — `{ supported: true }` |
| **macOS** (desktop shell) | `NSWindow.sharingType = .none` via Tauri `set_content_protected` | OS excludes the window from `ScreenCaptureKit`/screenshot capture | **Prevents** — `{ supported: true }` |
| **Android** (mobile shell) | `FLAG_SECURE` on the activity window (custom Kotlin plugin) | OS excludes the app from screenshots, screen recording, and the recents thumbnail | **Prevents** — `{ supported: true }` |
| **iOS** (mobile shell) | Capture **detection** (`UIScreen.isCaptured`, screenshot notification) | Cannot prevent capture; can **detect** recording and react (hide content, blur snapshot, notify) | **Detect-only** — `{ supported: false }` + watermark |
| **Linux** (desktop shell) | none reliable (X11/Wayland/WebKitGTK expose no exclusion) | nothing | **Watermark-only** — `{ supported: false }` |
| **Browser** (any web deployment) | none (no web API exists) | nothing | **Watermark-only** — `{ supported: false }` |

`{ supported }` is the value the frozen `Platform.setCaptureProtection(enabled)`
capability returns (§2.1). When it is `false`, the SPA keeps the V4 watermark. This
is deliberate: **no security theatre.** A platform either genuinely excludes the
window from capture, or it says it cannot and shows the deterrent instead.

## Where the code lives

- **Desktop** (`apps/desktop/src-tauri/src/capture.rs`) — the `set_capture_protection`
  Tauri command. On Windows/macOS it calls
  `WebviewWindow::set_content_protected(enabled)` and returns `{ supported: true }`;
  on Linux it makes **no** OS call and returns `{ supported: false }`.
- **Android** (`apps/mobile/src-tauri/android-src/FlagSecurePlugin.kt`) — a custom
  Kotlin Tauri plugin toggling `FLAG_SECURE`, resolving `{ supported: true }`.
- **iOS** (`apps/mobile/src-tauri/ios-src/ScreenCaptureDetection.swift`) — a
  best-effort **detection** skeleton (there is no prevention API). Documented gap:
  iOS needs macOS + Xcode + an Apple account, unavailable on the Windows dev/CI
  machine — the code is tracked, the local build is the gap.
- **Browser / any unsupported platform** — the capability layer's browser
  implementation returns `{ supported: false }`, and the SPA renders the watermark.

## The watermark (the fallback that stays)

Where genuine capture protection is impossible (Linux, iOS, browser), Mailwoman
falls back to V4's **visible watermark overlay** — and only that. It is a deterrent,
not a control.

When enabled (`MW_WATERMARK=true`), the SPA tiles a low-opacity overlay across
sensitive views stamping the **viewer's identity and a server timestamp**, to:

- discourage casual sharing, and
- make a leaked screenshot **attributable** to whoever took it.

It is pure DOM/CSS under the existing `script-src 'self'` policy, pulls no external
resource (CSP-safe), and its opacity is tunable via `MW_WATERMARK_OPACITY` (0.0–1.0,
default 0.08). It does **not** prevent, block, or detect a screenshot, a screen
recording, a photo of the monitor, or the print-screen key — it only makes a capture
attributable.

The canonical honesty note the server returns with every watermark config response
(mirrored verbatim from `mw-server`'s `watermark::HONEST_NOTE`):

> This watermark is a visual deterrent only. A web browser cannot prevent, block,
> or detect screenshots or screen recordings, so this overlay cannot stop this
> content from being captured. It stamps the viewer's identity and the time across
> the view to discourage casual sharing and to make a leaked screenshot
> attributable — it is not a security control. Genuine screen-capture protection
> requires the native desktop application, planned for a later release.

The note names "the native desktop application" as the path to genuine protection —
**that path is what V5 ships** (the Windows/macOS/Android exclusion above). The
watermark remains the honest fallback for the platforms that cannot enforce it.

## Manual verification (Windows)

The desktop content-protection path is unit-tested (`capture.rs` asserts the call
path and the honest `{ supported }` matrix), but OS enforcement is verified by hand,
since a headless runner cannot prove a real screenshot came out black:

1. Launch the desktop shell on Windows and open a sensitive view.
2. Call `setCaptureProtection(true)` (the SPA's screen-capture setting, or `invoke('set_capture_protection', { enabled: true })` from the devtools console).
3. Take a screenshot (PrtSc) or start a screen recording, and/or use the Snipping
   Tool.
4. **Expected:** the Mailwoman window appears **black / blank** in the capture while
   remaining visible on screen. Toggling `setCaptureProtection(false)` restores
   normal capture.

This manual check is recorded in the e4 log and is the documented fallback for the
capture assertion in the native E2E (plan §6 R7).
