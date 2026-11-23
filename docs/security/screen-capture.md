# Screen-capture protection

An administrator can ask the Mailwoman apps to keep their window out of screenshots
and screen recordings. Whether that works depends on the platform: some operating
systems provide a way to exclude a window from capture, and web browsers do not.

## Turning it on

<!-- screenshot: id=security/admin-capture-policy; screen=Admin console > Security policy; state=default policy; element=the "Screen capture protection" field with Off / Protect -->

In the admin console, open **Security policy** and set **Screen capture protection**
to **Protect** (the default is **Off**). Each server process reads the policy again
within 30 seconds, and signed-in clients receive it as `capturePolicy` from
`GET /api/policy`.

When a client loads the policy and finds **Protect**, it asks its platform to
protect the whole app window. Details:

- Protection starts when the policy is loaded after sign-in. The sign-in screen and
  anything shown before the policy arrives are not protected.
- The client only turns protection on. Changing the policy back to **Off** takes
  effect in the desktop and Android apps after they restart.

## What each platform does

| Platform | Mechanism | Result |
|---|---|---|
| Windows desktop app | Tauri `set_content_protected(true)`, which sets `WDA_EXCLUDEFROMCAPTURE` on the window | The OS excludes the window from screenshots and recordings; it appears black in a capture. |
| macOS desktop app | Tauri `set_content_protected(true)`, which sets the window's sharing type to none | The OS excludes the window from screenshot and screen-recording APIs. |
| Android app | `FLAG_SECURE` on the activity window | The OS blocks screenshots and screen recording of the app and hides it in the recent-apps view. |
| Linux desktop app | None. The app makes no call and reports the feature as unsupported. | No protection. |
| Web browser | None. Browsers have no API to prevent or detect capture. | No protection. |
| iOS | No iOS app is built. The source tree contains a detection-only Swift template, not wired into any build. | Not available. |

Sources: `apps/desktop/src-tauri/src/capture.rs`,
`apps/mobile/src-tauri/android-src/FlagSecurePlugin.kt`,
`apps/mobile/src-tauri/src/commands/capture.rs`,
`apps/web/src/platform/browser.ts`.

## Limits

Where the OS does exclude the window, it still cannot stop:

- a photo of the screen taken with another device;
- capture by software running with enough privilege to bypass the OS mechanism, or
  on a modified OS;
- someone copying text out of a message, printing it, or downloading attachments;
- remote-desktop or screen-sharing tools that do not honour the exclusion.

Treat it as a way to prevent casual or accidental capture on managed devices, not as
a data-loss control.

## The web watermark is not rendered

The server has a watermark setting (`MW_WATERMARK`, `MW_WATERMARK_OPACITY`) and an
endpoint, `GET /api/security/watermark`, that returns the viewer's identity, a
timestamp and a note that a watermark is a deterrent only. The web app does not
call this endpoint and draws no watermark. Setting `MW_WATERMARK` has no visible
effect, and a browser session has no screen-capture control of any kind.

## Checking it on Windows

The Windows path is covered by unit tests and an end-to-end test that checks the
command reports `{ supported: true }`. Whether the OS actually blacks out the
window is checked by hand:

1. Set **Screen capture protection** to **Protect**.
2. Sign in to the Windows desktop app and open a message.
3. Take a screenshot with Print Screen or the Snipping Tool, or start a screen
   recording.
4. Expected: the Mailwoman window is black in the capture while it stays visible on
   screen.

## Related

- [Security policy (admin)](/admin/security-policy)
- [Desktop and mobile apps](/user/apps-desktop-mobile)
- [Security model overview](./README.md)
