# Browser Web Push live E2E (`push.spec.ts`, `--project=push`)

Proves the committed V5 browser push path LIVE against the engine stack (:8090),
plan §2.3 / §3 e9:

1. `GET /api/push/vapid` serves a real P-256 VAPID public key (65-byte uncompressed
   SEC1 point).
2. A subscription round-trips through `POST /api/push/subscribe` → `{id,
   vapidPublicKey}`, stored against the logged-in account.
3. A real `StateChange` (a new SMTP delivery the engine ingests) drives e5's push
   **dispatcher** — a second consumer of the same broadcast the WS/SSE path drains —
   to deliver an **opaque wake** to the subscription endpoint. The endpoint points
   at a test-controlled mock receiver; the delivered body is the fixed `mw-wake`
   marker carrying **no message content** (the privacy invariant, §2.3 / risk #8).
4. The client refetches on the same change (the row renders without a manual
   refresh — the realtime path the wake mirrors).

## Running

```sh
# engine stack up on :8090 (docker compose -f docker-compose.dev.yml up greenmail mailwoman-engine)
npx playwright test --project=push        # in apps/web
```

Env:
- `PLAYWRIGHT_ENGINE_BASE_URL` — the engine server (default `http://localhost:8090`).
- `MW_E2E_WAKE_HOST` — the host the **containerized** server uses to reach the
  test's mock receiver. Default `host.docker.internal` (works on Docker Desktop
  Win/Mac). On Linux CI, run the server container with
  `--add-host=host.docker.internal:host-gateway`, or set this to the host-gateway
  IP / a service reachable on the compose network.

Previous local runs verified both dispatcher tests against the engine stack,
including opaque `mw-wake` delivery over `host.docker.internal`. This is separate
from the incoming-mail browser regression suite below.

## Scope / documented gaps (for e8)

- **`pushManager.subscribe` (the real browser Web Push registration leg) is NOT
  exercised here.** It requires the headless browser to be registered with a live
  push service (FCM / Mozilla autopush), which the compose stack does not provide;
  in Playwright's Chromium it throws. This spec instead registers the subscription
  through the same server endpoint the SPA's browser fallback (`platform/browser.ts`
  `pushSubscribe`) calls, which is the genuinely-live, deterministic path and the
  one that carries the privacy-critical dispatcher assertion. A full
  push-service-backed subscribe is a CI/service follow-up.
- The older dispatcher spec uses retries. The incoming-mail regression suite
  below explicitly uses zero retries so a missed notification fails the run.
- The **server-side dispatcher** (opaque WebPush/UnifiedPush, no content, VAPID
  sealed at rest) is additionally proven at the Rust level in
  `crates/mw-server/tests/push_v5.rs` (e5).

## CI wiring (e8)

A dedicated `push` Playwright project (in `playwright.config.ts`) targets the
engine baseURL. Wire an `e2e-push` job that boots the engine stack (greenmail +
mailwoman-engine), ensures `host.docker.internal` reachability (see above), and runs
`npx playwright test --project=push`. Alternatively fold `--project=push` into the
existing `e2e-engine` job once host-networking is provisioned there.

## Incoming-mail delivery and recovery

`realtime-push.spec.ts` injects real SMTP messages into an IMAP fixture and checks
the rendered Inbox and incoming toast without calling the Refresh endpoint. Its
six cases cover WebSocket, EventSource, blocked-stream polling, shared windows
with a native browser notification, inactive linked mailboxes, and mail filed
directly into a previously unknown folder. The linked alert must open the reader
in the correct account. The server-filed case uses IMAP APPEND.

The expected fast path is IMAP IDLE → engine reconciliation → JMAP StateChange →
client delta/list update and account-policy-filtered notification. Recovery is
automatic: no-IDLE IMAP connections poll every 30 seconds; IDLE renews with a
catch-up every 60 seconds; an independent engine sweep checks folders every 60
seconds after the preceding sweep completes. These are scheduling intervals,
not hard delivery deadlines: upstream latency and backfill work still apply.
When browser streaming is unavailable, client catch-up runs every 30 seconds.

Watchers start before historical backfill. Mailbox cursor writes are serialized,
partial fetches cannot advance checkpoints, and raw fetches are bounded to 64
messages per batch. Client list/delta failures and transient notification-policy
failures retry with bounded backoff. Expired delta tokens recover from a bounded
recent-message query. Inactive verified linked mailboxes have separate cursors
and policy reads; they never inherit the active mailbox's notification policy.

From `apps/web`, against a built, running engine fixture:

```sh
PLAYWRIGHT_ENGINE_BASE_URL=http://127.0.0.1:8090 \
MW_E2E_ENGINE_USERNAME=realtime@audit.test \
MW_E2E_ENGINE_PASSWORD=testpass MW_E2E_ENGINE_SELF=realtime@audit.test \
MW_E2E_SMTP_PORT=3025 MW_E2E_IMAP_PORT=3143 \
MW_E2E_LINKED_USERNAME=linked@audit.test MW_E2E_LINKED_PASSWORD=linkedpass \
pnpm exec playwright test e2e/realtime-push.spec.ts \
  --project=engine --project=realtime-firefox --workers=1 --retries=0
```

Use dedicated fixture accounts, with linking enabled by the fixture administrator.
The tests explicitly enable each account's notification policy. The shared-window
test controls document focus because headless browsers otherwise report every
page as focused; the native Notification constructor still runs with permission.
This proves browser API acceptance, not visual OS delivery. External FCM/Autopush
registration and production-provider behavior require separate live verification.
Closed-browser delivery requires that account's registered push subscription.
