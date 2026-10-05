import { test, expect, type APIRequestContext } from '@playwright/test';
import { V7, adminLogin, expectMounted } from './v7-helpers.ts';

/**
 * Engine-plugin registry live E2E, against the real server (`--project=v7`).
 *
 * What a loaded plugin may do — the grant it runs with, a guest network call being
 * refused, a tampered component failing its digest — is proven in the Rust harness
 * `crates/mw-server/tests/t28_plugin_register.rs`, in engine mode with the real
 * `spam-rspamd.wasm`. The `v7` stack is a PROXY-mode server: it has no engine, so it
 * never loads a plugin. This spec proves the browser-facing contract on that server:
 * the routes are mounted and admin-gated, a plugin can be registered, approved,
 * granted and enabled over HTTP, the answers carry the server's shapes, and an
 * enabled plugin that is not running is reported as not loaded, never as running.
 */

interface PluginView {
  id: string;
  firstParty: boolean;
  trust: string;
  approved: boolean;
  enabled: boolean;
  capabilities: string[];
  granted: string[];
  netAllowlist: string[];
  limits: { memoryMb: number; deadlineMs: number; fuel: number | null };
  loaded: boolean;
  loadedCapabilities: string[];
  restartRequired: boolean;
  notLoadedReason: string | null;
}

const PLUGIN = 'spam-spamassassin';

async function listed(request: APIRequestContext): Promise<PluginView[]> {
  const list = await request.get('/admin/plugins');
  expectMounted(list, 'GET /admin/plugins');
  expect(list.status()).toBe(200);
  return ((await list.json()) as { plugins: PluginView[] }).plugins;
}

/** POST a change that must succeed and return the plugin the server answers with. */
async function change(request: APIRequestContext, path: string, data?: unknown): Promise<PluginView> {
  const resp = await request.post(path, data === undefined ? {} : { data });
  expectMounted(resp, `POST ${path}`);
  expect(resp.ok(), `POST ${path} → ${resp.status()} ${await resp.text()}`).toBe(true);
  return ((await resp.json()) as { plugin: PluginView }).plugin;
}

test.describe('Plugin registry (V7) — admin surface on the real server', () => {
  test('register, approve, grant, enable: the answers say what is stored and what runs', async ({ request }) => {
    await adminLogin(request);
    // Start from a registry without this plugin (a previous run may have left it).
    await request.post(`/admin/plugins/${PLUGIN}/uninstall`);
    expect((await listed(request)).some((p) => p.id === PLUGIN)).toBe(false);

    const registered = await request.post('/admin/plugins', { data: { id: PLUGIN, netAllowlist: ['spamd.internal'] } });
    expect(registered.status()).toBe(201);
    let p = ((await registered.json()) as { plugin: PluginView }).plugin;
    expect(p).toMatchObject({
      id: PLUGIN,
      firstParty: true,
      trust: 'first-party-digest',
      approved: false,
      enabled: false,
      capabilities: ['spam-action', 'net', 'store-kv-scoped'],
      granted: [],
      netAllowlist: ['spamd.internal'],
      limits: { memoryMb: 32, deadlineMs: 10000, fuel: null },
      loaded: false,
      loadedCapabilities: [],
      restartRequired: false,
      notLoadedReason: 'proxy-mode',
    });

    // Registering it twice, or with a manifest of the caller's own, is refused.
    const again = await request.post('/admin/plugins', { data: { id: PLUGIN } });
    expect(again.status()).toBe(409);
    expect((await again.json()).code).toBe('already-registered');
    const widened = await request.post('/admin/plugins', {
      data: { id: 'spam-rspamd', capabilities: ['account-backend'] },
    });
    expect(widened.status(), 'a first-party manifest cannot be supplied').toBe(400);

    // Enable before approval is refused; a grant outside the manifest is refused.
    const early = await request.post(`/admin/plugins/${PLUGIN}/enable`);
    expect(early.status()).toBe(400);
    expect((await early.json()).code).toBe('not-approved');
    p = await change(request, `/admin/plugins/${PLUGIN}/approve`);
    expect(p.approved).toBe(true);
    for (const capabilities of [['account-backend'], ['everything']]) {
      const refused = await request.post(`/admin/plugins/${PLUGIN}/grant`, { data: { capabilities } });
      expect(refused.status(), `grant ${capabilities.join()} is refused`).toBe(400);
    }
    p = await change(request, `/admin/plugins/${PLUGIN}/grant`, { capabilities: ['spam-action'] });
    expect(p.granted).toEqual(['spam-action']);

    // Enabled is stored; this server has no engine, so it is not loaded and says so.
    p = await change(request, `/admin/plugins/${PLUGIN}/enable`);
    expect(p.enabled).toBe(true);
    expect(p.loaded).toBe(false);
    expect(p.notLoadedReason).toBe('proxy-mode');
    const tested = await request.post(`/admin/plugins/${PLUGIN}/test`);
    expect(tested.status(), 'a plugin that is not loaded cannot be tested').toBe(409);

    // allow-unsigned takes `{allow}` and does not apply to a first-party component.
    const bodiless = await request.post(`/admin/plugins/${PLUGIN}/allow-unsigned`);
    expect(bodiless.status()).toBe(400);
    const firstParty = await request.post(`/admin/plugins/${PLUGIN}/allow-unsigned`, { data: { allow: true } });
    expect(firstParty.status()).toBe(400);
    expect((await firstParty.json()).code).toBe('first-party');

    // Every lifecycle route answers an unknown id with its own JSON 404.
    for (const action of ['approve', 'enable', 'disable', 'test']) {
      const resp = await request.post(`/admin/plugins/does-not-exist/${action}`);
      expectMounted(resp, `POST /admin/plugins/{id}/${action}`);
      expect(resp.status()).toBe(404);
    }

    const removed = await request.post(`/admin/plugins/${PLUGIN}/uninstall`);
    expect(removed.status()).toBe(200);
    expect((await removed.json()).unregistered).toBe(true);
    expect((await listed(request)).some((x) => x.id === PLUGIN)).toBe(false);
  });

  test('the registry is admin-gated (unauthenticated ⇒ 401)', async ({ request }) => {
    for (const resp of [
      await request.get('/admin/plugins'),
      await request.post('/admin/plugins', { data: { id: PLUGIN } }),
      await request.post(`/admin/plugins/${PLUGIN}/grant`, { data: { capabilities: ['net'] } }),
      await request.post(`/admin/plugins/${PLUGIN}/allow-unsigned`, { data: { allow: true } }),
    ]) {
      expect(resp.status()).toBe(401);
    }
  });

  test('the admin Plugins screen registers a plugin and shows it as not loaded', async ({ page }) => {
    await adminLogin(page.request);
    await page.request.post('/admin/plugins/spam-rspamd/uninstall');

    await page.goto('/admin');
    await page.getByRole('button', { name: 'Plugins', exact: true }).click();
    const form = page.getByRole('form', { name: 'Register a plugin' });
    await form.getByLabel('Component', { exact: true }).selectOption('spam-rspamd');
    await form.getByRole('button', { name: 'Register' }).click();

    const card = page.locator('[data-plugin-id="spam-rspamd"]');
    await expect(card.getByTestId('trust-chip')).toHaveText('First-party, digest built in');
    await expect(card.getByTestId('status-chip')).toHaveText('Not loaded');
    await card.getByRole('button', { name: 'Approve' }).click();
    await card.getByLabel('Grant spam-action to Rspamd spam classifier').check();
    await card.getByRole('button', { name: 'Save grants' }).click();
    await card.getByRole('button', { name: 'Enable' }).click();
    // Enabled on a server that cannot run it: the screen says why, and never "Loaded".
    await expect(card.getByTestId('status-line')).toHaveText(
      'Not loaded: this server runs in proxy mode and does not process mail itself.',
    );
    await expect(card.getByTestId('status-chip')).toHaveText('Not loaded');
    await expect(card.getByRole('button', { name: 'Disable' })).toBeVisible();

    // The UI plugins screen is reachable from the nav and reads the real route.
    await page.getByRole('button', { name: 'UI plugins' }).click();
    await expect(page.getByRole('region', { name: 'UI plugins' })).toBeVisible();
    await expect(page.getByRole('form', { name: 'Register a UI plugin' })).toBeVisible();

    await page.request.post('/admin/plugins/spam-rspamd/uninstall');
  });
});

test.beforeEach(async ({ request }, testInfo) => {
  const probe = await request.get('/admin/plugins').catch(() => null);
  test.skip(
    probe === null,
    `[e16 SKIP] ${testInfo.title}: no V7 mw-server reachable (start the e2e-v7 stack; admin=${V7.adminUser}).`,
  );
});
