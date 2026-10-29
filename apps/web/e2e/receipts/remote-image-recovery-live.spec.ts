// "Could not update remote image permissions. Please try again." (t82), live on
// the receipts stack (scripts/e2e-receipts-stack.mjs): a real server, Greenmail
// and the built app. No route mocks except where a leg says it injects a
// transient failure (Playwright `page.route` in a real browser).
//
// Legs, for Chromium and Firefox at 1440 and 390 px:
//   1. primary mailbox: Show images grants and the banner reads "Images shown";
//   2. a linked (child) mailbox selected: Show images and "Always show images
//      from <sender>" grant for the CHILD and the banner reads "Images shown".
//      Before t82 the app refused the grant routes in a linked context before
//      sending anything (`linkedContextUnsupported`), so the user saw the error;
//   3. two injected 503s then success: the grant recovers, no error toast;
//   4. every attempt fails: exactly one error toast with a Retry action, and
//      Retry (after the fault is lifted) grants.
import { test, expect, type Page } from '../csrf.ts';
import { connect } from 'node:net';
import { mkdirSync } from 'node:fs';

const SHOTS = process.env['MW_T82_SHOTS'] ?? 'test-results/t82';
const PHASE = process.env['MW_T82_PHASE'] ?? 'after';
const VIEWPORTS = [{ width: 1440, height: 900 }, { width: 390, height: 844 }];

async function smtp(to: string, from: string, message: string): Promise<void> {
  const port = Number(process.env['MW_E2E_SMTP_PORT'] ?? 42225);
  const steps = ['EHLO t82', `MAIL FROM:<${from}>`, `RCPT TO:<${to}>`, 'DATA', `${message}\r\n.`, 'QUIT'];
  await new Promise<void>((resolve, reject) => {
    const socket = connect(port, '127.0.0.1'); let i = 0; let buffer = '';
    socket.on('data', (chunk) => { buffer += chunk.toString(); if (!/\r\n$/.test(buffer)) return; const code = Number(buffer.slice(0, 3)); buffer = ''; if (code >= 400) { reject(new Error(`SMTP ${code}`)); socket.destroy(); return; } if (i < steps.length) socket.write(`${steps[i++]}\r\n`); else socket.end(); });
    socket.on('close', () => resolve()); socket.on('error', reject);
  });
}
async function seed(to: string, subject: string, from: string): Promise<void> {
  const html = `<p>Newsletter ${subject}</p><img alt="remote" width="24" height="24" src="http://1.2.3.4/img.png?${subject}"><p>${'Body text. '.repeat(20)}</p>`;
  await smtp(to, from, `From: Sender <${from}>\r\nTo: ${to}\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <${Date.now()}.${Math.random()}@example.test>\r\nSubject: ${subject}\r\nMIME-Version: 1.0\r\nContent-Type: text/html; charset=utf-8\r\n\r\n${html}`);
}

async function signIn(page: Page, user: string, pass: string): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill(user);
  await page.getByLabel('Password', { exact: true }).fill(pass);
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByLabel('Email or username', { exact: true })).toHaveCount(0, { timeout: 30000 });
  await expect(page.locator('[data-account-indicator]').first()).toBeAttached({ timeout: 30000 });
}
async function openInbox(page: Page): Promise<void> {
  if ((page.viewportSize()?.width ?? 1440) < 761) await page.getByRole('button', { name: 'Open folders and apps' }).click();
  await page.getByRole('navigation', { name: 'Mailboxes' }).getByRole('button', { name: 'Inbox' }).click();
}
async function open(page: Page, subject: string): Promise<void> {
  const row = page.locator('.list__row').filter({ hasText: subject });
  await expect(async () => {
    if (!(await row.isVisible())) await openInbox(page);
    await expect(row).toBeVisible({ timeout: 3000 });
  }).toPass({ timeout: 60000 });
  await row.click();
  await expect(page.getByTestId('remote-content-bar').filter({ visible: true })).toBeVisible({ timeout: 20000 });
}
const bar = (page: Page) => page.getByTestId('remote-content-bar').filter({ visible: true });
const errorToasts = (page: Page) => page.locator('[data-testid="toast-region"] [role="alert"]');
async function shot(page: Page, project: string, name: string): Promise<void> {
  mkdirSync(SHOTS, { recursive: true });
  await page.screenshot({ path: `${SHOTS}/${PHASE}-${project}-${page.viewportSize()?.width}-${name}.png` });
}
/** Counts error toasts as they appear, so a toast that flashes and goes is seen. */
async function watchToasts(page: Page): Promise<void> {
  await page.evaluate(() => {
    const w = window as unknown as { __t82: string[] };
    w.__t82 = [];
    new MutationObserver(() => {
      for (const el of document.querySelectorAll<HTMLElement>('[data-testid="toast-region"] [role="alert"]')) {
        if (!el.dataset['t82']) { el.dataset['t82'] = '1'; w.__t82.push(el.textContent ?? ''); }
      }
    }).observe(document.body, { subtree: true, childList: true });
  });
}
const seenToasts = (page: Page): Promise<string[]> => page.evaluate(() => (window as unknown as { __t82: string[] }).__t82);

async function unlinkAll(page: Page): Promise<void> {
  const me = await (await page.request.get('/api/me')).json() as { storageAuthority: string };
  const headers = { 'x-mailwoman-storage-authority': me.storageAuthority };
  const stale = (await (await page.request.get('/api/accounts', { headers })).json()).accounts as { id: string; revision: string }[];
  if (!stale.length) return;
  expect((await page.request.post('/api/security/step-up', { data: { password: 'parent-pass' } })).ok()).toBe(true);
  for (const a of stale) expect((await page.request.delete(`/api/accounts/${a.id}`, { headers: { ...headers, 'if-match': JSON.stringify(a.revision) } })).ok()).toBe(true);
}
async function linkChild(page: Page): Promise<string> {
  const me = await (await page.request.get('/api/me')).json() as { storageAuthority: string };
  const headers = { 'x-mailwoman-storage-authority': me.storageAuthority };
  expect((await page.request.post('/api/security/step-up', { data: { password: 'parent-pass' } })).ok()).toBe(true);
  const res = await page.request.post('/api/accounts/link', { headers, data: { username: 'child@child.test', password: 'child-pass', displayName: 'Child' } });
  expect(res.ok(), await res.text()).toBe(true);
  const listing = await (await page.request.get('/api/accounts', { headers })).json() as { accounts: { id: string; address?: string }[] };
  return listing.accounts.find((a) => a.address === 'child@child.test')!.id;
}
async function chooseLink(page: Page, link: string): Promise<void> {
  const trigger = page.locator('button[data-account-indicator="picker"]');
  const drawer = page.getByRole('button', { name: 'Open folders and apps' });
  await expect(trigger.filter({ visible: true }).or(drawer)).toBeVisible({ timeout: 30000 });
  if (!(await trigger.isVisible())) await page.getByRole('button', { name: 'Open folders and apps' }).click();
  await trigger.click();
  await page.getByRole('menu', { name: 'Mailbox account', exact: true }).locator(`[data-mailbox-item][data-link="${link}"]`).click();
  await expect(trigger).toHaveAttribute('aria-expanded', 'false');
  if ((page.viewportSize()?.width ?? 1440) < 761) await page.keyboard.press('Escape');
}

for (const viewport of VIEWPORTS) {
  test.describe(`t82 at ${viewport.width}`, () => {
    test.use({ viewport });

    test('primary mailbox: Show images grants', async ({ page }, info) => {
      const subject = `t82-primary-${info.project.name}-${viewport.width}-${Date.now()}`;
      await seed('testuser@example.org', subject, `news-${Date.now()}@news.test`);
      await signIn(page, 'testuser', 'testpass');
      await open(page, subject);
      await watchToasts(page);
      await bar(page).getByRole('button', { name: 'Show images' }).click();
      await expect(bar(page)).toHaveAttribute('data-state', 'shown');
      expect(await seenToasts(page)).toEqual([]);
      await shot(page, info.project.name, 'primary-shown');
    });

    test('linked mailbox: Show images and Always show from the sender grant for the child', async ({ page }, info) => {
      await signIn(page, 'parent@example.org', 'parent-pass');
      await unlinkAll(page);
      const link = await linkChild(page);
      await page.reload();
      const subject = `t82-linked-${info.project.name}-${viewport.width}-${Date.now()}`;
      const sender = `news-${Date.now()}@linked-news.test`;
      await seed('child@child.test', `${subject}-a`, sender);
      await seed('child@child.test', `${subject}-b`, sender);
      await expect(page.locator('[data-account-indicator]').first()).toBeAttached({ timeout: 30000 });
      await chooseLink(page, link);
      await open(page, `${subject}-a`);
      await watchToasts(page);
      await bar(page).getByRole('button', { name: 'Show images' }).click();
      await shot(page, info.project.name, 'linked-after-show');
      await expect(bar(page)).toHaveAttribute('data-state', 'shown');
      expect(await seenToasts(page)).toEqual([]);
      // The grant belongs to the child, not to the parent.
      const child = await (await page.request.get('/api/remote-images/grants', { headers: { 'x-mailwoman-link': link } })).json() as { list: { scopeKind: string }[] };
      expect(child.list.some((g) => g.scopeKind === 'single')).toBe(true);
      const parent = await (await page.request.get('/api/remote-images/grants')).json() as { list: { scopeKind: string }[] };
      expect(parent.list.some((g) => g.scopeKind === 'single')).toBe(false);
      // Always show from the sender.
      await open(page, `${subject}-b`);
      await bar(page).getByRole('button', { name: 'More image options' }).click();
      // The "More image options" tooltip can sit over the item at 390 px; choose it
      // from the keyboard as a menu allows.
      const item = page.getByRole('menuitem', { name: /Always show images from/ }).filter({ hasText: sender });
      await item.focus();
      await page.keyboard.press('Enter');
      await expect(bar(page)).toHaveAttribute('data-state', 'shown');
      expect(await seenToasts(page)).toEqual([]);
      await shot(page, info.project.name, 'linked-sender-shown');
      await unlinkAll(page);
    });

    test('transient failures recover with no error toast', async ({ page }, info) => {
      test.skip(PHASE === 'before', 'recovery is t82');
      const subject = `t82-transient-${info.project.name}-${viewport.width}-${Date.now()}`;
      await seed('testuser@example.org', subject, `news-${Date.now()}@news.test`);
      await signIn(page, 'testuser', 'testpass');
      await open(page, subject);
      await watchToasts(page);
      let failures = 2; let attempts = 0;
      await page.route('**/api/remote-images/grant', async (route) => {
        attempts += 1;
        if (failures > 0) { failures -= 1; await route.fulfill({ status: 503, body: 'unavailable' }); return; }
        await route.continue();
      });
      await bar(page).getByRole('button', { name: 'Show images' }).click();
      await expect(bar(page)).toHaveAttribute('data-state', 'shown', { timeout: 20000 });
      expect(attempts).toBe(3);
      expect(await seenToasts(page)).toEqual([]);
      await shot(page, info.project.name, 'transient-recovered');
      await page.unroute('**/api/remote-images/grant');
    });

    test('offline: the grant waits for the connection and is sent once back online', async ({ page, context }, info) => {
      test.skip(PHASE === 'before', 'recovery is t82');
      const subject = `t82-offline-${info.project.name}-${viewport.width}-${Date.now()}`;
      await seed('testuser@example.org', subject, `news-${Date.now()}@news.test`);
      await signIn(page, 'testuser', 'testpass');
      await open(page, subject);
      await watchToasts(page);
      let sent = 0;
      page.on('request', (r) => { if (r.url().endsWith('/api/remote-images/grant')) sent += 1; });
      await context.setOffline(true);
      await bar(page).getByRole('button', { name: 'Show images' }).click();
      await expect(bar(page).getByRole('button', { name: 'Show images' })).toContainText('Waiting for a connection', { timeout: 5000 });
      await shot(page, info.project.name, 'offline-waiting');
      expect(sent).toBe(0);
      await context.setOffline(false);
      await expect(bar(page)).toHaveAttribute('data-state', 'shown', { timeout: 20000 });
      expect(sent).toBe(1);
      // The app's own "Connection lost" notice may appear while offline; the
      // grant itself raises no error.
      expect((await seenToasts(page)).filter((m) => /remote image/i.test(m))).toEqual([]);
    });

    test('a final failure shows one toast whose Retry works', async ({ page }, info) => {
      test.skip(PHASE === 'before', 'recovery is t82');
      const subject = `t82-final-${info.project.name}-${viewport.width}-${Date.now()}`;
      await seed('testuser@example.org', subject, `news-${Date.now()}@news.test`);
      await signIn(page, 'testuser', 'testpass');
      await open(page, subject);
      await watchToasts(page);
      let broken = true;
      await page.route('**/api/remote-images/grant', async (route) => {
        if (broken) { await route.abort('connectionrefused'); return; }
        await route.continue();
      });
      // While the browser reports itself online, a refused connection is retried
      // a few times and then reported once.
      await bar(page).getByRole('button', { name: 'Show images' }).click();
      await expect(errorToasts(page)).toHaveCount(1, { timeout: 30000 });
      await shot(page, info.project.name, 'final-failure-toast');
      const toastEl = errorToasts(page).first();
      await expect(toastEl).toContainText('remote image permissions');
      expect((await seenToasts(page)).length).toBe(1);
      broken = false;
      await toastEl.getByRole('button', { name: 'Retry' }).click();
      await expect(bar(page)).toHaveAttribute('data-state', 'shown', { timeout: 20000 });
      await expect(errorToasts(page)).toHaveCount(0);
      expect((await seenToasts(page)).length).toBe(1);
      await shot(page, info.project.name, 'final-retry-shown');
      await page.unroute('**/api/remote-images/grant');
    });
  });
}
