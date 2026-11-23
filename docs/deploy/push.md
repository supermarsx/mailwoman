# Push notifications

The server sends push "wakes" so a closed browser tab or the Android app learns
that an account changed. The relay runs inside the Mailwoman server; no
Mailwoman-operated service is involved. This page covers what a wake contains,
which transports work, the server settings, and the realtime connection the open
app uses.

## What a push message contains

Every wake carries the same fixed 7-byte marker, `mw-wake`, and nothing else. It
means only "this account changed". No subject, sender or message text is sent over
push. The app then fetches the changes over its signed-in connection and builds
any notification from that.

The VAPID key pair is generated when the server first starts. Its private key is
sealed with the server key in the store.

## Transports

| Platform | Transport | Status |
|---|---|---|
| Browser and desktop app | Web Push with VAPID | Implemented. The wake is encrypted to the subscriber's key (RFC 8291) and signed with a VAPID token (RFC 8292). |
| Android app | UnifiedPush | Implemented. The app registers with a distributor installed on the phone (for example ntfy) and sends the endpoint to the server. No Google services are used. |
| iOS | APNs | Not implemented. The server accepts `apns` subscriptions, but it only writes a log line instead of contacting Apple. No wake reaches the device. |

Endpoints for Web Push and UnifiedPush must be `https://` URLs without user
name, password or fragment, and not a private or loopback IP address. Each
account can hold at most 32 subscriptions.

## When a wake is sent

The server sends a wake for an account change only when all of these hold:

- the deployment-wide quiet hours (`push.quietHours`) are not active;
- the account has not restricted or withdrawn consent for push processing;
- `notifications.pushAllowed` is true (an administrator setting, default true);
- the user's notifications are on (`notifications.enabled`, default false) and
  the user's own quiet hours are not active;
- the user has no notification rules, native notifications are on,
  **Only notify matching rules** is off, the scope is
  all received folders and no folders are muted. A wake carries no content, so the
  server cannot apply these filters and sends nothing instead.

A subscription whose endpoint answers `404` or `410`, or whose expiry time has
passed, is deleted. Each send times out after 10 seconds.

Users register a browser in **Settings** > **Notifications** under
**Background notifications on this browser** with
**Register background notifications**. The button is disabled while
notifications are off or the administrator does not allow push.

<!-- SCREENSHOT deploy/push-browser-registration: Settings > Notifications, "Background notifications on this browser" card with the Register button, crop of the card -->

## Server configuration

| Variable or setting | What it does | Default |
|---|---|---|
| `MW_VAPID_CONTACT` | Contact URI (`mailto:` or `https:`) sent as the `sub` claim of the VAPID token. Push services use it to reach the operator. Set it to an address you read. | `mailto:push@mailwoman.invalid` |
| `MW_NATIVE_ORIGINS` | Extra origins, comma-separated, that the server treats as native app origins (CORS headers and sign-in without a cookie). The desktop and mobile apps' own origins `http://tauri.localhost`, `https://tauri.localhost` and `tauri://localhost` are always accepted, so you need this only for a shell that uses another origin. | empty |
| `push.quietHours` (setting) | Deployment-wide quiet window in UTC hours, as `{start, end}` with hours 0-23. The window includes `start` and excludes `end`, and wraps past midnight when `start` is later than `end`. No wakes are sent inside it. | null (never quiet) |
| `notifications.pushAllowed` (setting) | Administrator ceiling for push registration and delivery, per deployment or domain. | true |

If `push.quietHours` has never been configured, the server reads the
`push.quiet_hours` value instead (a `"start-end"` string such as `"22-7"`).
Setting `push.quietHours`, even to null, stops that fallback. See
[Notifications](../admin/notifications.md) for the user-level settings.

## UnifiedPush on Android

The user installs a UnifiedPush distributor, such as ntfy pointed at a server you
run, or NextPush on a Nextcloud. The Mailwoman app registers with it and posts
the endpoint it receives to `/api/push/subscribe`. The server then sends each
wake as a plain `POST` with the marker as the body. See
[Android app](mobile-android.md) for building the app.

## Push API

All three routes need a signed-in session (cookie or bearer token).

- `GET /api/push/vapid` returns `{ "publicKey": ... }`, the VAPID public key.
- `POST /api/push/subscribe` takes `transport` (`webpush`, `unifiedpush` or
  `apns`), `endpoint`, `keys` (`p256dh` and `auth`, required for `webpush`),
  and optional `appId` and `expiresAt`. It returns `{ "id": ..., "vapidPublicKey": ... }`.
  It answers `403` with `notificationsDisabled` when the user's notifications
  are off or push is not allowed.
- `POST /api/push/unsubscribe` takes `{ "id": ... }` or `{ "endpoint": ... }` and
  answers `204`.

## Realtime connection while the app is open

While the app is open it does not rely on push. It keeps a connection to
`/jmap/ws` (WebSocket) or `/jmap/eventsource` (Server-Sent Events). Both routes
exist in engine and proxy mode and need a signed-in session.

- The browser tries WebSocket first, then EventSource, with at most two failed
  attempts per transport. Opening a connection has a 10-second deadline.
  Opening and closing a connection straight away does not count as success; the
  failure count resets only after a valid state change or heartbeat.
- If both fail, the app switches to degraded mode and checks for changes every
  30 seconds. It retries the full sequence up to three more times, after 30, 60
  and 120 seconds, then stops until you retry, sign in again, or a push wake
  triggers a reconnect.
- A response saying the session has ended stops the retries at once.

A failed connection does not mean the mail server lacks realtime support. Check
the HTTP status, the session, and whether your reverse proxy forwards WebSocket
upgrades and does not buffer Server-Sent Events. See
[Reverse proxy](reverse-proxy.md).

### Linked mailboxes

Linked mailboxes (engine mode) get their updates over the same connection as
the sign-in mailbox; there is no second connection. This works whenever linking
is allowed for both mailboxes (`accounts.linkingEnabled`, `accounts.useLinked`).

- **Source of changes:** each linked mailbox is a normal engine account, watched
  with IMAP IDLE on the Inbox. Servers without IDLE, and POP3 accounts, use the
  periodic resync instead. A dropped watch reconnects after 1 second, doubling up
  to 60 seconds. One upstream connection serves every user who linked that
  mailbox.
- **Messages:** each change arrives as a `MailboxChange` frame
  (`apps/web/src/contracts/push.ts`) naming the link id and an opaque storage
  authority, never the raw account id, plus the Inbox unread count. While a
  connection carries links, changes to the sign-in mailbox are also sent in this
  form with `"link": null`.
- **Limits and checks:** a connection carries at most `accounts.maxLinked` links.
  The server works out the set of links when the connection opens, when a link is
  created or removed, and at every re-check (`MW_REALTIME_RECHECK_SECS`, default
  30 seconds). Each change is checked again before it is sent, so unlinking,
  revoking sessions, signing out, disabling either account or a changed authority
  stops delivery from the next change.

## Related

- [Desktop app](desktop.md)
- [Android app](mobile-android.md)
- [Reverse proxy](reverse-proxy.md)
- [Notifications (admin)](../admin/notifications.md)
- [WebSocket](websocket.md)
