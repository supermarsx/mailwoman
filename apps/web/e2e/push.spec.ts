import { test, expect, type Page } from '@playwright/test';
import http from 'node:http';
import { engineLogin, injectViaSmtp, messageRow, sidebarInbox } from './helpers.ts';

/**
 * V5 browser Web Push, end-to-end against the engine stack (plan §2.3 / §3 e9).
 *
 * Proves the committed browser push path LIVE against the real server:
 *   1. `GET /api/push/vapid` serves a real P-256 VAPID public key (e5).
 *   2. A subscription round-trips through `POST /api/push/subscribe` (e5), stored
 *      against the logged-in account.
 *   3. A real `StateChange` (a new SMTP delivery the engine ingests) drives e5's
 *      push DISPATCHER — a second consumer of the same broadcast the WS/SSE path
 *      drains — to deliver an OPAQUE wake to the subscription endpoint. We point
 *      that endpoint at a test-controlled mock receiver and assert the delivered
 *      body is the fixed `mw-wake` marker carrying NO message content (§2.3 / the
 *      privacy invariant, risk #8).
 *   4. The client refetches on the same change (the message row renders without a
 *      manual refresh — the realtime path the wake mirrors).
 *
 * The mock receiver runs on the host; the server (in a container) reaches it via
 * `host.docker.internal` (override with MW_E2E_WAKE_HOST). Real browser Web Push
 * (`pushManager.subscribe`) needs a live push service the headless browser is
 * registered with, which the compose stack does not provide — so the subscription
 * is registered through the server endpoint the SPA's browser fallback uses (the
 * `webpush`/`unifiedpush` transport + endpoint), which is the genuinely-live,
 * deterministic path. The full pushManager.subscribe browser leg is documented in
 * apps/web/e2e/README-push.md as needing a push service (CI/service gap).
 */

// Serial + retries: the engine watch-loop's ingestion + broadcast + dispatch can
// take tens of seconds and can transiently break under accumulated session load
// (same rationale as realtime-push.spec.ts); a fresh retry recovers.
test.describe.configure({ mode: 'serial', retries: 2 });

/** Host the containerized server uses to reach the test's mock receiver. */
const WAKE_HOST = process.env['MW_E2E_WAKE_HOST'] ?? 'host.docker.internal';

interface WakeHit {
  method: string;
  body: string;
  headers: http.IncomingHttpHeaders;
}

/** A throwaway HTTP receiver capturing the dispatcher's opaque wake POST. */
class MockPushReceiver {
  private server: http.Server;
  private hits: WakeHit[] = [];
  private waiters: ((hit: WakeHit) => void)[] = [];

  private constructor(server: http.Server) {
    this.server = server;
  }

  static async start(): Promise<MockPushReceiver> {
    let receiver!: MockPushReceiver;
    const server = http.createServer((req, res) => {
      const chunks: Buffer[] = [];
      req.on('data', (c) => chunks.push(c as Buffer));
      req.on('end', () => {
        const hit: WakeHit = {
          method: req.method ?? '',
          body: Buffer.concat(chunks).toString('utf8'),
          headers: req.headers,
        };
        receiver.hits.push(hit);
        const w = receiver.waiters.shift();
        if (w) w(hit);
        res.statusCode = 201; // acknowledge the wake (2xx keeps the subscription)
        res.end('ok');
      });
    });
    await new Promise<void>((resolve) => server.listen(0, '0.0.0.0', resolve));
    receiver = new MockPushReceiver(server);
    return receiver;
  }

  get port(): number {
    return (this.server.address() as { port: number }).port;
  }

  /** Resolve with the next wake, or reject after `timeout`. */
  waitForWake(timeout: number): Promise<WakeHit> {
    const existing = this.hits.shift();
    if (existing) return Promise.resolve(existing);
    return new Promise<WakeHit>((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error('no wake delivered within timeout')), timeout);
      this.waiters.push((hit) => {
        clearTimeout(timer);
        resolve(hit);
      });
    });
  }

  async stop(): Promise<void> {
    await new Promise<void>((resolve) => this.server.close(() => resolve()));
  }
}

/** In-page fetch (same-origin, cookie-authed) with the double-submit CSRF header. */
async function apiPost(page: Page, path: string, body: unknown): Promise<{ status: number; json: unknown }> {
  return page.evaluate(
    async ({ path, body }) => {
      const csrf = document.cookie
        .split('; ')
        .find((c) => c.startsWith('mw_csrf='))
        ?.slice('mw_csrf='.length);
      const res = await fetch(path, {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'content-type': 'application/json', ...(csrf ? { 'x-csrf-token': csrf } : {}) },
        body: JSON.stringify(body),
      });
      let json: unknown = null;
      try {
        json = await res.json();
      } catch {
        json = null;
      }
      return { status: res.status, json };
    },
    { path, body },
  );
}

test.describe('V5 Web Push (engine mode)', () => {
  test('serves a real VAPID public key', async ({ page }) => {
    await engineLogin(page);
    const vapid = await page.evaluate(async () => {
      const res = await fetch('/api/push/vapid', { credentials: 'same-origin' });
      return { status: res.status, json: (await res.json()) as { publicKey?: string } };
    });
    expect(vapid.status).toBe(200);
    const publicKey = vapid.json.publicKey ?? '';
    expect(publicKey.length).toBeGreaterThan(80); // base64url of a 65-byte point
    // Decode base64url → uncompressed SEC1 P-256 point: 0x04 || X(32) || Y(32).
    const raw = Buffer.from(publicKey.replace(/-/g, '+').replace(/_/g, '/'), 'base64');
    expect(raw.length).toBe(65);
    expect(raw[0]).toBe(0x04);
  });

  test('subscribe round-trips + a StateChange delivers an opaque wake, client refetches', async ({
    page,
  }) => {
    // Generous ceiling: the wake wait (≤120s) and the client-refetch wait (≤90s)
    // both ride the engine watch-loop's ingestion latency, so allow both to elapse.
    test.setTimeout(240_000);
    await engineLogin(page);
    await expect(sidebarInbox(page)).toBeVisible();

    const mock = await MockPushReceiver.start();
    try {
      const endpoint = `http://${WAKE_HOST}:${mock.port}/wake`;

      // Subscribe (UnifiedPush transport → the dispatcher does a plain POST of the
      // opaque marker, the cleanest content-free assertion). Stored against the
      // logged-in engine account so the dispatcher fans this change out to it.
      const sub = await apiPost(page, '/api/push/subscribe', {
        transport: 'unifiedpush',
        endpoint,
        appId: 'e2e-web-push',
      });
      expect(sub.status).toBe(200);
      const subBody = sub.json as { id?: string; vapidPublicKey?: string };
      expect(subBody.id).toBeTruthy();
      expect(subBody.vapidPublicKey).toBeTruthy();

      // A brand-new delivery straight over SMTP → the engine ingests it → a
      // StateChange → (a) the push dispatcher wakes the endpoint, (b) the client
      // refetches over the realtime path.
      const subject = `WebPush ${Date.now()}`;
      await injectViaSmtp({
        from: 'Push Bot <pushbot@example.org>',
        subject,
        text: `web push wake ${subject}`,
      });

      // (a) The opaque wake reaches the mock endpoint: body is the fixed marker,
      // carrying NO message content (no subject/body ever transits push, §2.3).
      const wake = await mock.waitForWake(120_000);
      expect(wake.method).toBe('POST');
      expect(wake.body).toBe('mw-wake');
      expect(wake.body).not.toContain(subject);

      // (b) The client refetched the same change (row renders without a manual
      // refresh — the realtime path the wake mirrors for backgrounded clients).
      await expect(messageRow(page, subject)).toBeVisible({ timeout: 90_000 });

      // Clean up the subscription.
      const unsub = await apiPost(page, '/api/push/unsubscribe', { id: subBody.id });
      expect(unsub.status).toBe(204);
    } finally {
      await mock.stop();
    }
  });
});
