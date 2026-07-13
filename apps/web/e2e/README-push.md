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

Verified LIVE on this machine: both tests green against the engine stack; the
opaque `mw-wake` was delivered to the mock over `host.docker.internal`.

## Scope / documented gaps (for e8)

- **`pushManager.subscribe` (the real browser Web Push registration leg) is NOT
  exercised here.** It requires the headless browser to be registered with a live
  push service (FCM / Mozilla autopush), which the compose stack does not provide;
  in Playwright's Chromium it throws. This spec instead registers the subscription
  through the same server endpoint the SPA's browser fallback (`platform/browser.ts`
  `pushSubscribe`) calls, which is the genuinely-live, deterministic path and the
  one that carries the privacy-critical dispatcher assertion. A full
  push-service-backed subscribe is a CI/service follow-up.
- **Flakiness:** the wake + client-refetch both ride the engine watch-loop's
  ingestion latency (a tracked V2 robustness follow-up — Greenmail's IMAP watch can
  break under accumulated session load). The spec uses `retries: 2` for exactly this
  reason, mirroring `realtime-push.spec.ts`.
- The **server-side dispatcher** (opaque WebPush/UnifiedPush, no content, VAPID
  sealed at rest) is additionally proven at the Rust level in
  `crates/mw-server/tests/push_v5.rs` (e5).

## CI wiring (e8)

A dedicated `push` Playwright project (in `playwright.config.ts`) targets the
engine baseURL. Wire an `e2e-push` job that boots the engine stack (greenmail +
mailwoman-engine), ensures `host.docker.internal` reachability (see above), and runs
`npx playwright test --project=push`. Alternatively fold `--project=push` into the
existing `e2e-engine` job once host-networking is provisioned there.
