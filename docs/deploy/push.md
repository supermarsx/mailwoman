# Push relay (self-hostable) — WebPush / UnifiedPush / APNs

Mailwoman's push is a **self-hostable relay** that ships inside `mw-server` (§28.7):
there is **no Mailwoman-operated infrastructure in the path**. It wakes clients so
they re-fetch; **no message content ever transits push**.

## The privacy model (read this first)

A push message carries a single opaque marker whose only meaning is *"your account
changed — wake and re-fetch `/changes`."* The subject, sender, and body of a message
**never** leave the server over push. The wake triggers the same foreground JMAP
`/changes` refetch the WebSocket/SSE realtime path already does; the client then
renders a notification from data it fetched over the authenticated JMAP surface. The
VAPID private key is **sealed at rest** (the same `mw-store` seal used for upstream
credentials).

## Transports

| Platform | Transport | Notes |
|---|---|---|
| Web + desktop | **Web Push (VAPID)** | RFC 8188 encrypted wake to the browser push service. The desktop shell subscribes in its WebView via the browser fallback; while the app is open the WS/SSE realtime path already drains the same change broadcast. |
| Android | **UnifiedPush** | Self-hostable, **no Google dependency**. The app registers with an on-device distributor (ntfy, NextPush, …) and hands the endpoint to the server. |
| iOS | **APNs** | Opaque wake only, content never transits push. **Mocked/recorded in CI** — live APNs needs an Apple account (a documented gap, not a V5 gate). |

The dispatcher is a second consumer of the engine `StateChange` broadcast (the same
source the realtime `push.rs` WS/SSE path drains); on a change for an account with
active subscriptions it sends the opaque wake, respecting quiet hours.

## Endpoints

- `GET /api/push/vapid` → `{ publicKey }` — the VAPID **public** key (generated and
  persisted on first boot; the private key never leaves the server).
- `POST /api/push/subscribe` — body is the frozen `PushSubscriptionInfo`
  (`transport`, `endpoint`, `keys` for WebPush, `appId` for UnifiedPush/APNs). Stored
  in `push_subscriptions` (migration `0006_v5.sql`). Returns `{ id, vapidPublicKey }`.
- `POST /api/push/unsubscribe` — `{ id | endpoint }`.

These are additive; a browser that never subscribes sees no change.

## Server configuration

| Env / setting | Default | Meaning |
|---|---|---|
| `MW_NATIVE_ORIGINS` | *(empty → off)* | Comma-separated shell origins to allow via CORS (e.g. `tauri://localhost,https://tauri.localhost`). Empty emits no CORS headers, so browser deployments are byte-identical. Required for a **remote** server to accept the desktop/mobile shell. |
| `MW_VAPID_CONTACT` | *(none)* | The `mailto:`/`https:` contact the server includes in the VAPID JWT `sub` (push services want an operator contact). Set it to an address you monitor. |
| `push.quiet_hours` *(settings value)* | *(none → never quiet)* | A `"start-end"` UTC-hour window, e.g. `"22-7"`. Inside it the dispatcher suppresses wakes (§17.3). Absent/unparseable → never quiet. |

VAPID keys are generated automatically on first boot — no key management is required
to turn Web Push on.

## Self-hosting UnifiedPush (Android, no Google)

UnifiedPush needs an on-device **distributor**; the user installs one (e.g.
[ntfy](https://ntfy.sh) pointed at *your* ntfy server, or NextPush against a
Nextcloud). The Mailwoman app auto-selects the installed distributor, registers, and
POSTs the granted endpoint to `/api/push/subscribe`. The server later POSTs opaque
wakes to that endpoint. Nothing here touches Google Play Services — this is the
committed, self-hostable Android path, alongside Web Push (VAPID) on web/desktop.

## What is NOT here (documented gaps)

- **APNs live delivery** — mocked/recorded in CI; needs an Apple account. UnifiedPush
  + Web Push are the committed live paths.
- **A Mailwoman-operated push service** — by design there is none; the relay is
  yours. See [`mobile-android.md`](./mobile-android.md) for the app-side build/gaps.
