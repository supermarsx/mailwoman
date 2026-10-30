// t91, live on the receipts stack (scripts/e2e-receipts-stack.mjs): a real
// server built from HEAD, Greenmail and the built app. The only interference is
// the injected fault, made with Playwright `page.route` in a real browser.
//
// The fault is the worst one for a create: the request REACHES the server (the
// route forwards it with `route.fetch()`, so the record is made) and the answer
// is then replaced by a 503, as a proxy that loses the answer would. A client
// that retried without an idempotency key would create a duplicate; one that
// gave up would show an error for a change that happened.
//
// Legs, in Chromium and Firefox:
//   1. a note create whose first two answers are lost recovers: one note on
//      the server, three sends with one Idempotency-Key, no error toast;
//   2. linking a mailbox whose first two answers are lost recovers: one link,
//      three sends with one key, no error toast;
//   3. an unlink whose first answer is lost recovers (the retry's 404 means it
//      is gone): no link left, no error toast.
import { test, expect, type Page, type Route } from '../csrf.ts';
import { mkdirSync } from 'node:fs';

const SHOTS = process.env['MW_T91_SHOTS'] ?? 'test-results/t91';

async function signIn(page: Page, user: string, pass: string): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill(user);
  await page.getByLabel('Password', { exact: true }).fill(pass);
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByLabel('Email or username', { exact: true })).toHaveCount(0, { timeout: 30000 });
  await expect(page.locator('[data-account-indicator]').first()).toBeAttached({ timeout: 30000 });
}
async function shot(page: Page, project: string, name: string): Promise<void> {
  mkdirSync(SHOTS, { recursive: true });
  await page.screenshot({ path: `${SHOTS}/${project}-${page.viewportSize()?.width}-${name}.png` });
}
/** Records every error toast as it appears, so one that flashes and goes is seen. */
async function watchToasts(page: Page): Promise<void> {
  await page.evaluate(() => {
    const w = window as unknown as { __t91: string[] };
    w.__t91 = [];
    new MutationObserver(() => {
      for (const el of document.querySelectorAll<HTMLElement>('[data-testid="toast-region"] [role="alert"]')) {
        if (!el.dataset['t91']) { el.dataset['t91'] = '1'; w.__t91.push(el.textContent ?? ''); }
      }
    }).observe(document.body, { subtree: true, childList: true });
  });
}
const seenToasts = (page: Page): Promise<string[]> => page.evaluate(() => (window as unknown as { __t91: string[] }).__t91);

/** Forward the request (the server acts on it), then answer 503 instead of its reply. */
async function loseAnswer(route: Route): Promise<void> {
  await route.fetch();
  await route.fulfill({ status: 503, contentType: 'text/plain', body: 'unavailable' });
}

async function owner(page: Page): Promise<Record<string, string>> {
  const me = await (await page.request.get('/api/me')).json() as { storageAuthority: string };
  return { 'x-mailwoman-storage-authority': me.storageAuthority };
}
async function links(page: Page): Promise<{ id: string; revision: string; address?: string }[]> {
  return (await (await page.request.get('/api/accounts', { headers: await owner(page) })).json()).accounts;
}
async function unlinkAll(page: Page): Promise<void> {
  const stale = await links(page);
  if (!stale.length) return;
  expect((await page.request.post('/api/security/step-up', { data: { password: 'parent-pass' } })).ok()).toBe(true);
  const headers = await owner(page);
  for (const a of stale) expect((await page.request.delete(`/api/accounts/${a.id}`, { headers: { ...headers, 'if-match': JSON.stringify(a.revision) } })).ok()).toBe(true);
}
async function openSettings(page: Page, section: string): Promise<void> {
  const settings = page.locator('.sidebar, #shell-nav').getByRole('button', { name: 'Settings', exact: true }).first();
  if (!(await settings.isVisible())) await page.getByRole('button', { name: 'Open folders and apps', exact: true }).click();
  await settings.click();
  await page.getByRole('navigation', { name: 'Settings sections' }).getByRole('button', { name: section, exact: true }).click();
}

async function noteTitles(page: Page): Promise<string[]> {
  const me = await (await page.request.get('/api/me')).json() as { accountId: string; storageAuthority: string };
  const res = await page.request.post('/jmap/api', {
    headers: { 'x-mailwoman-storage-authority': me.storageAuthority },
    data: { using: ['urn:ietf:params:jmap:core', 'urn:mailwoman:notes'], methodCalls: [['Note/get', { accountId: me.accountId, ids: null }, 'g']] },
  });
  const body = await res.json() as { methodResponses: [string, { list?: { title: string }[] }, string][] };
  return (body.methodResponses[0]?.[1].list ?? []).map((n) => n.title);
}

test.use({ viewport: { width: 1440, height: 900 } });

test('a note create whose answers are lost recovers: one note, one key, no toast', async ({ page }, info) => {
  await signIn(page, 'testuser', 'testpass');
  const before = (await noteTitles(page)).length;
  await page.goto('/#/notes');
  const create = page.getByRole('button', { name: /New note/ }).first();
  await expect(create).toBeVisible({ timeout: 30000 });
  await watchToasts(page);
  const keys: (string | null)[] = [];
  let lose = 2;
  await page.route('**/jmap/api', async (route) => {
    const body = route.request().postData() ?? '';
    if (!body.includes('"Note/set"') || !body.includes('"create"')) { await route.continue(); return; }
    keys.push(await route.request().headerValue('idempotency-key'));
    if (lose > 0) { lose -= 1; await loseAnswer(route); return; }
    await route.continue();
  });
  await create.click();
  await expect.poll(() => keys.length, { timeout: 20000 }).toBe(3);
  await expect.poll(async () => (await noteTitles(page)).length, { timeout: 20000 }).toBe(before + 1);
  // Nothing else arrives later.
  await page.waitForTimeout(1500);
  expect((await noteTitles(page)).length).toBe(before + 1);
  expect(keys[0]).toBeTruthy();
  expect(new Set(keys).size).toBe(1);
  expect(await seenToasts(page)).toEqual([]);
  await shot(page, info.project.name, 'note-create-recovered');
  await page.unroute('**/jmap/api');
});

test('linking and unlinking a mailbox whose answers are lost recover: one link, then none, no toast', async ({ page }, info) => {
  await signIn(page, 'parent@example.org', 'parent-pass');
  await unlinkAll(page);
  // A recent confirmation, so the link is not interrupted by the step-up dialog.
  expect((await page.request.post('/api/security/step-up', { data: { password: 'parent-pass' } })).ok()).toBe(true);
  await page.reload();
  await expect(page.locator('[data-account-indicator]').first()).toBeAttached({ timeout: 30000 });
  await openSettings(page, 'Linked mailboxes');
  await watchToasts(page);

  const linkKeys: (string | null)[] = [];
  let lose = 2;
  await page.route('**/api/accounts/link', async (route) => {
    linkKeys.push(await route.request().headerValue('idempotency-key'));
    if (lose > 0) { lose -= 1; await loseAnswer(route); return; }
    await route.continue();
  });
  await page.getByRole('button', { name: 'Link a mailbox', exact: true }).click();
  const editor = page.getByRole('region', { name: 'Link a mailbox', exact: true });
  await editor.getByLabel('Username or email address', { exact: true }).fill('child@child.test');
  await editor.getByLabel('Password', { exact: true }).fill('child-pass');
  await editor.getByLabel('Display name (optional)', { exact: true }).fill(`Child ${info.project.name}`);
  await editor.getByRole('button', { name: 'Verify and link', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Linked mailboxes', exact: true })).toBeVisible({ timeout: 30000 });
  expect(linkKeys).toHaveLength(3);
  expect(linkKeys[0]).toBeTruthy();
  expect(new Set(linkKeys).size).toBe(1);
  const linked = await links(page);
  expect(linked.filter((a) => a.address === 'child@child.test')).toHaveLength(1);
  expect(linked).toHaveLength(1);
  expect(await seenToasts(page)).toEqual([]);
  await shot(page, info.project.name, 'link-recovered');
  await page.unroute('**/api/accounts/link');

  // Unlink: the first answer is lost after the server removed the link.
  let deletes = 0; let loseDelete = 1;
  await page.route('**/api/accounts/*', async (route) => {
    if (route.request().method() !== 'DELETE') { await route.continue(); return; }
    deletes += 1;
    if (loseDelete > 0) { loseDelete -= 1; await loseAnswer(route); return; }
    await route.continue();
  });
  await page.getByRole('button', { name: /^Unlink .*child@child\.test/ }).click();
  const dialog = page.getByRole('alertdialog', { name: /^Unlink \W?child@child\.test\W?\?$/ });
  const confirm = dialog.getByRole('button', { name: 'Confirm unlink', exact: true });
  await confirm.click();
  const proof = page.getByRole('dialog', { name: 'Confirm it is you' });
  if (await proof.isVisible({ timeout: 2000 }).catch(() => false)) {
    await proof.getByLabel('Password', { exact: true }).fill('parent-pass');
    await proof.getByRole('button', { name: 'Confirm and continue', exact: true }).click();
  }
  await expect.poll(() => deletes, { timeout: 20000 }).toBe(2);
  await expect.poll(async () => (await links(page)).length, { timeout: 20000 }).toBe(0);
  await expect(page.getByRole('button', { name: /^Unlink .*child@child\.test/ })).toHaveCount(0, { timeout: 20000 });
  expect(await seenToasts(page)).toEqual([]);
  await shot(page, info.project.name, 'unlink-recovered');
  await page.unroute('**/api/accounts/*');
});

// t102/t91: the user's production 404 on POST /api/accounts/{id}/renew is the
// server's own `linkedAccountNotFound`, for a link that was unlinked in another
// tab or on another device. Here the link is really removed on the server after
// the page loaded; the one injection is the page's first listing, rewritten to
// show the (stale) link as expired so its Renew button is offered. The renew
// itself goes to the real server, which answers 404 linkedAccountNotFound.
test('renewing a link removed elsewhere closes the dialog, reloads the list and says so', async ({ page }, info) => {
  await signIn(page, 'parent@example.org', 'parent-pass');
  await unlinkAll(page);
  expect((await page.request.post('/api/security/step-up', { data: { password: 'parent-pass' } })).ok()).toBe(true);
  const res = await page.request.post('/api/accounts/link', { headers: await owner(page), data: { username: 'child@child.test', password: 'child-pass', displayName: 'Child' } });
  expect(res.ok(), await res.text()).toBe(true);
  let stale = true;
  await page.route('**/api/accounts', async (route) => {
    if (route.request().method() !== 'GET' || !stale) { await route.continue(); return; }
    const answer = await route.fetch();
    const body = await answer.json() as { accounts: { status: string }[] };
    for (const a of body.accounts) a.status = 'reauthRequired';
    await route.fulfill({ response: answer, json: body });
  });
  let renews = 0;
  page.on('request', (r) => { if (/\/api\/accounts\/[^/]+\/renew$/.test(new URL(r.url()).pathname)) renews += 1; });
  await page.reload();
  await expect(page.locator('[data-account-indicator]').first()).toBeAttached({ timeout: 30000 });
  await openSettings(page, 'Linked mailboxes');
  const renew = page.locator('[data-linked-renew]').first();
  await expect(renew).toBeVisible({ timeout: 20000 });
  // Another device unlinks it now.
  await unlinkAll(page);
  stale = false;
  await renew.click();
  const dialog = page.getByRole('dialog', { name: /^Renew access to / });
  await dialog.getByLabel('Password', { exact: true }).fill('child-pass');
  await dialog.getByRole('button', { name: 'Renew access', exact: true }).click();
  await expect(dialog).toHaveCount(0, { timeout: 20000 });
  await expect(page.locator('[data-testid="toast-region"]').getByText('This mailbox is no longer linked.')).toBeVisible({ timeout: 10000 });
  await expect(page.locator('[data-linked-renew]')).toHaveCount(0, { timeout: 20000 });
  expect(renews).toBe(1);
  await shot(page, info.project.name, 'renew-gone');
  await page.unroute('**/api/accounts');
});
