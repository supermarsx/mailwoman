// t102 — a tab left open across a deploy keeps working, live.
//
// The spec runs its own server (the HEAD binary in MW_T102_BIN) so it can deploy:
// it starts the server on web build A, signs in to the console, then stops it,
// replaces the web directory with build B (A's files gone, as after a new image)
// and starts it again on the same database and key. In the tab still running A:
//
//   1. retained assets on: opening a section not yet loaded imports A's chunk,
//      which the server still serves -> the section opens, no 404, no reload;
//      the old worker registration updates cleanly (registration.update()).
//   2. retained assets off: the chunk 404s -> the app shows "A new version is
//      available, reloading…" and reloads ONCE into build B, then opens the section.
//   3. a worker registered at an address the server does not serve (/mail/sw.js)
//      receives the cleanup worker and removes its own registration.
//
// The not-found table is read after the scripted session and written to
// MW_T102_EVIDENCE (with MW_T102_LABEL), so the HEAD baseline and the fixed
// build can be compared. MW_T102_BASELINE=1 records without the fixed-build
// assertions.
//
// Env: MW_T102_BIN, MW_T102_DIST_A, MW_T102_DIST_B, MW_T102_PORT (default 47102).
import { test, expect, type Page } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import { cpSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { randomBytes } from 'node:crypto';

const BIN = process.env['MW_T102_BIN'] ?? '';
const DIST_A = process.env['MW_T102_DIST_A'] ?? '';
const DIST_B = process.env['MW_T102_DIST_B'] ?? '';
const PORT = Number(process.env['MW_T102_PORT'] ?? '47102');
const BASELINE = process.env['MW_T102_BASELINE'] === '1';
const EVIDENCE = process.env['MW_T102_EVIDENCE'] ?? '';
const LABEL = process.env['MW_T102_LABEL'] ?? 'run';
const ORIGIN = `http://localhost:${PORT}`;
const KEY = randomBytes(32).toString('hex');

test.skip(BIN === '' || DIST_A === '' || DIST_B === '', 'set MW_T102_BIN, MW_T102_DIST_A and MW_T102_DIST_B');
test.describe.configure({ mode: 'serial' });
test.use({ baseURL: ORIGIN, serviceWorkers: 'allow' });
test.setTimeout(300_000);

const data = mkdtempSync(join(tmpdir(), 'mw-t102-live-'));
const web = join(data, 'web');
let server: ChildProcess | null = null;

async function startServer(dist: string, extra: Record<string, string> = {}): Promise<void> {
  rmSync(web, { recursive: true, force: true });
  cpSync(dist, web, { recursive: true });
  mkdirSync(join(data, 'uploads'), { recursive: true });
  server = spawn(BIN, ['serve'], {
    env: {
      ...process.env,
      MW_MODE: 'proxy',
      MW_BIND: `127.0.0.1:${PORT}`,
      MW_DB_PATH: join(data, 'mw.db'),
      MW_UPLOAD_DIR: join(data, 'uploads'),
      MW_SERVER_KEY: KEY,
      MW_WEB_DIR: web,
      MW_COOKIE_SECURE: 'false',
      MW_ADMIN_USER: 'root',
      MW_ADMIN_PASSWORD: 'synthetic-t102-password',
      MW_ADMIN_PATH: '/admin',
      MW_PUBLIC_URL: ORIGIN,
      RUST_LOG: 'warn',
      ...extra,
    },
    stdio: 'ignore',
  });
  for (let i = 0; i < 240; i++) {
    try { if ((await fetch(`${ORIGIN}/healthz`)).ok) return; } catch { /* not yet */ }
    await new Promise((r) => setTimeout(r, 250));
  }
  throw new Error('server did not start');
}

async function stopServer(): Promise<void> {
  const s = server;
  server = null;
  if (s === null || s.exitCode !== null) return;
  await new Promise<void>((resolve) => { s.once('exit', () => resolve()); s.kill(); });
}

test.afterAll(async () => {
  await stopServer();
  rmSync(data, { recursive: true, force: true });
});

async function signIn(page: Page): Promise<void> {
  await page.goto('/admin');
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('synthetic-t102-password');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.locator('#admin-main')).toBeVisible({ timeout: 30_000 });
}

const entry = (page: Page): Promise<string | null> => page.evaluate(() => {
  const src = document.querySelector('script[type="module"][src*="assets/"]')?.getAttribute('src') ?? '';
  return src.slice(src.lastIndexOf('/') + 1) || null;
});

function watch(page: Page): { misses: string[]; loads: number } {
  const seen = { misses: [] as string[], loads: 0 };
  page.on('response', (r) => { if (r.status() === 404) seen.misses.push(`${r.request().method()} ${new URL(r.url()).pathname}`); });
  page.on('load', () => { seen.loads += 1; });
  return seen;
}

const misses404: Record<string, string[]> = {};

test('a tab on build A opens a new section after build B is deployed (retained assets)', async ({ page }, info) => {
  await startServer(DIST_A);
  await signIn(page);
  const before = await entry(page);
  // Let the console's worker registration settle (it is best-effort).
  // (`ready` never settles where no worker is registered, so it is bounded.)
  await page.evaluate(() => Promise.race([navigator.serviceWorker?.ready.then(() => 'ready'), new Promise((r) => setTimeout(() => r('none'), 5000))])).catch(() => undefined);
  await stopServer();
  await startServer(DIST_B);
  const seen = watch(page);
  await page.evaluate(() => { location.hash = '#/analytics'; });
  if (!BASELINE) await expect(page.locator('#admin-main h1, #admin-main h2').first()).toBeVisible({ timeout: 30_000 });
  await page.waitForTimeout(BASELINE ? 6000 : 1500);
  misses404['retained'] = seen.misses;
  await page.screenshot({ path: info.outputPath('retained.png') });
  const update = await page.evaluate(async () => {
    const reg = await navigator.serviceWorker?.getRegistration();
    if (!reg) return 'no-registration';
    try { await reg.update(); return 'updated'; } catch (e) { return `failed: ${String(e)}`; }
  });
  misses404['retained-update'] = [update];
  if (BASELINE) return;
  expect(seen.misses, 'no 404 for the old build').toEqual([]);
  expect(seen.loads, 'no reload needed').toBe(0);
  expect(await entry(page)).toBe(before);
  expect(['updated', 'no-registration']).toContain(update);
});

test('without retained assets the tab reloads once into build B', async ({ page }, info) => {
  await stopServer();
  await startServer(DIST_A, { MW_RETAINED_ASSETS_DIR: 'off' });
  await signIn(page);
  const before = await entry(page);
  await stopServer();
  await startServer(DIST_B, { MW_RETAINED_ASSETS_DIR: 'off' });
  const seen = watch(page);
  await page.evaluate(() => { location.hash = '#/nextcloud'; });
  if (!BASELINE) {
    await expect(page.getByText('A new version is available, reloading…')).toBeVisible({ timeout: 20_000 });
    await page.screenshot({ path: info.outputPath('notice.png') });
  }
  await page.waitForTimeout(8000);
  misses404['no-retention'] = seen.misses;
  await page.screenshot({ path: info.outputPath('after-reload.png') });
  if (BASELINE) return;
  expect(seen.loads, 'exactly one reload').toBe(1);
  expect(await entry(page)).not.toBe(before);
  await expect(page.locator('#admin-main')).toBeVisible();
});

test('a worker registered at an address this server does not serve removes itself', async ({ browser }) => {
  // The app's Trusted Types policy refuses to register any worker but its own,
  // so the stale registration (as an earlier build or deploy prefix left it) is
  // made with the page CSP bypassed.
  const context = await browser.newContext({ baseURL: ORIGIN, bypassCSP: true, serviceWorkers: 'allow' });
  const page = await context.newPage();
  await page.goto('/admin');
  const result = await page.evaluate(async () => {
    try {
      const reg = await navigator.serviceWorker.register('/mail/sw.js', { scope: '/mail/' });
      for (let i = 0; i < 40; i++) {
        const regs = await navigator.serviceWorker.getRegistrations();
        if (!regs.some((r) => r.scope.endsWith('/mail/'))) return 'removed';
        await new Promise((r) => setTimeout(r, 250));
      }
      return `still registered (${reg.scope})`;
    } catch (e) {
      return `register failed: ${String(e)}`;
    }
  });
  misses404['legacy-worker'] = [result];
  await context.close();
  if (BASELINE) return;
  expect(result).toBe('removed');
});

test('the registered worker of build A updates cleanly to build B', async ({ browser }) => {
  await stopServer();
  await startServer(DIST_A);
  // Registered as the mail app registers it; CSP bypassed only so the page need
  // not sign in to reach the code path (the app registers /sw.js the same way).
  const context = await browser.newContext({ baseURL: ORIGIN, bypassCSP: true, serviceWorkers: 'allow' });
  const page = await context.newPage();
  const seen = watch(page);
  await page.goto('/');
  const registered = await page.evaluate(async () => {
    const reg = await navigator.serviceWorker.register('/sw.js', { scope: '/' });
    await navigator.serviceWorker.ready;
    return reg.active?.scriptURL ?? null;
  });
  expect(registered).toMatch(/\/sw\.js$/);
  await stopServer();
  await startServer(DIST_B);
  const update = await page.evaluate(async () => {
    const reg = await navigator.serviceWorker.getRegistration('/');
    if (!reg) return 'no-registration';
    try {
      await reg.update();
      const next = reg.installing ?? reg.waiting;
      if (next) await new Promise((r) => { next.addEventListener('statechange', () => { if (next.state === 'activated' || next.state === 'redundant') r(null); }); setTimeout(r, 10_000); });
      return `updated:${reg.active?.state ?? 'none'}`;
    } catch (e) { return `failed: ${String(e)}`; }
  });
  misses404['worker-update'] = [update, ...seen.misses];
  await context.close();
  if (BASELINE) return;
  expect(update).toBe('updated:activated');
  expect(seen.misses).toEqual([]);
});

test('the not-found table after the session', async ({ page }) => {
  await signIn(page);
  // The request history writer flushes every few seconds.
  await page.waitForTimeout(17_000);
  const res = await page.request.get('/admin/metrics/not-found');
  expect(res.status()).toBe(200);
  const body = await res.json() as { total: number | null; paths: { method: string; path: string; count: number; source?: string }[]; listedBySource?: unknown };
  if (EVIDENCE !== '') {
    mkdirSync(EVIDENCE, { recursive: true });
    writeFileSync(join(EVIDENCE, `not-found-${LABEL}-${test.info().project.name}.json`), JSON.stringify({ label: LABEL, project: test.info().project.name, browserMisses: misses404, table: body }, null, 2));
  }
  if (BASELINE) return;
  const assetMisses = body.paths.filter((p) => p.path.startsWith('/assets/') && p.source === 'client');
  // Only the retention-off leg may 404 an old chunk; the retained leg may not.
  expect(misses404['retained']).toEqual([]);
  expect(body.paths.every((p) => p.source === 'client' || p.source === 'external')).toBe(true);
  expect(assetMisses.length).toBeLessThanOrEqual(3);
});
