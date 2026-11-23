---
title: Desktop and mobile apps
---

# Desktop and mobile apps

Mailwoman has a desktop app for Windows, macOS and Linux and an app for Android. Each is
the same web client you use in a browser, in its own window, connected to your
organisation's Mailwoman server. Everything on the other pages of this guide applies in
the apps. This page covers what is different. There is no iPhone or iPad app.

## Get the app

Your administrator builds the app for your server and tells you where to get it. The
server's address is part of the app, so you never type it. If you start an app that was
not built for a server, it shows "This app has not been set up for a server. Contact your
administrator." and offers no way to continue.

On Windows, the desktop app needs the Microsoft Edge WebView2 Runtime. If it is missing
the app says so when it starts; install the runtime and start the app again.

## Sign in

The sign-in screen is the one in the browser: **Email or username**, **Password** and
**Sign in**, with no server field. Two-factor authentication works the same way.

The app signs in with its own session instead of a browser cookie, and keeps that
session in your device's secure storage: Windows Credential Manager, the macOS Keychain
or the Linux Secret Service on a computer, and Android's encrypted storage on a phone.

If the app cannot reach the server when it starts, it shows an error with **Retry** and
the line "If this keeps happening, contact your administrator."

## What the apps add

| Feature | Desktop | Android |
|---|---|---|
| New-mail notifications | Yes. Clicking one brings the window to the front and opens the message. | Yes |
| Unread count | A badge on the dock icon (macOS), the launcher (Linux) or the taskbar icon (Windows, up to 9, then 9+) | A notification that reads "N unread" |
| Push while the app is closed | No | Yes, through UnifiedPush; see below |
| Sending queued changes in the background | Yes; see below | No; keep the app open |
| Screen-capture protection | When your administrator requires it | When your administrator requires it |

**Notifications.** The notification settings are in **Settings**, then **Notifications**; see
[Settings: interface](/user/settings/interface).

**Push on Android.** The app receives push messages through UnifiedPush, so it does not
need Google Play Services. You need a UnifiedPush distributor app on the phone. If you
have several installed and have not chosen one, the app uses the first one Android
lists.

**Background sending on desktop.** While offline changes are waiting to be sent, the app
asks your operating system to start it every five minutes (Task Scheduler on Windows, a
user timer on Linux, a LaunchAgent on macOS). The background run opens a hidden window,
sends what is queued, and exits within about two minutes. It works only while you are
logged in to the computer, and the operating system may delay or refuse it. On Android,
and for anything else, keep the app open. See
[Sync and offline access](/user/offline-sync).

**Screen-capture protection.** If your administrator requires it, the app asks the
operating system to keep its window out of screenshots and screen recordings.

## What does not work in the apps

These are not available in the apps, even though the app may look as though they should
be:

- Choosing Mailwoman as the program that opens `mailto:` links. Clicking such a link
  elsewhere does not start a message in Mailwoman.
- Sharing a file or message from another Android app to Mailwoman, and opening `.eml`,
  `.ics`, `.vcf` or `.msg` files with it.
- Locking the app with Windows Hello or a fingerprint.
- Dragging attachments out of a message onto the desktop.

## Updates

Desktop apps update only when your administrator has set up an update feed for them. The
Android app has no updater of its own; you get new versions from the store or repository
you installed it from. Ask your administrator which applies.

## Troubleshooting

- **"This app has not been set up for a server. Contact your administrator."** The app
  was built without a server address. Ask your administrator for a build for your server.
- **The app closes right after it starts, or reports a mismatch in its files.** The
  installation is damaged. Reinstall the app.
- **Notifications do not arrive on Android.** Check that a UnifiedPush distributor app
  is installed and that notifications are allowed for Mailwoman.

## Related

- [Sync and offline access](/user/offline-sync)
- [Settings: interface](/user/settings/interface)
- [Desktop app (for administrators)](/deploy/desktop)
- [Android app (for administrators)](/deploy/mobile-android)
