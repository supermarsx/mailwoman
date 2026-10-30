// t67 cleanup sweep, live on the receipts stack (scripts/e2e-receipts-stack.mjs),
// Chromium and Firefox at 1440 px:
//   1. the loading-images toast does not stay on "Loading images…" when the
//      images fail or time out: it turns into "N images couldn't load · Retry",
//      and Retry signs the images again with retry:true;
//   2. a toast does not cover the floating bulk-selection bar in a folder tab;
//   3. Settings editors ask in the in-app dialog, never window.confirm, and a
//      bulk tab close asks once;
//   5. an account preference changed just before a reload is not lost.
// Images go through the t56 egress-proxy fixture (1.2.3.4 tunnelled): /hang*
// never answers, /missing* is a 404. Screenshots: MW_T67_SHOTS.
import { test, expect, type Page, type APIRequestContext } from '../csrf.ts';
import { createServer as createHttpServer } from 'node:http';
import { deflateSync } from 'node:zlib';
import { connect, type Socket } from 'node:net';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { postJmap, useTabbedWorkspace } from '../helpers.ts';

const SHOTS = process.env['MW_T67_SHOTS'] ?? 'test-results/t67';
const adminPath = process.env['MW_E2E_ADMIN_PATH'] ?? '/admin';
const HANGING = 4;

function png(size = 8): Buffer {
  const crc = (b: Buffer): number => { let c = 0xffffffff; for (const v of b) { c ^= v; for (let i = 0; i < 8; i++) c = c & 1 ? (c >>> 1) ^ 0xedb88320 : c >>> 1; } return (c ^ 0xffffffff) >>> 0; };
  const chunk = (name: string, data: Buffer): Buffer => { const bytes = Buffer.concat([Buffer.from(name), data]), len = Buffer.alloc(4), sum = Buffer.alloc(4); len.writeUInt32BE(data.length); sum.writeUInt32BE(crc(bytes)); return Buffer.concat([len, bytes, sum]); };
  const head = Buffer.alloc(13); head.writeUInt32BE(size); head.writeUInt32BE(size, 4); head[8] = 8; head[9] = 6;
  const data = Buffer.alloc(size * (1 + size * 4)); for (let y = 0; y < size; y++) for (let x = 0; x < size; x++) { const p = y * (1 + size * 4) + 1 + x * 4; data[p] = 40; data[p + 1] = 120; data[p + 2] = 200; data[p + 3] = 255; }
  return Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk('IHDR', head), chunk('IDAT', deflateSync(data)), chunk('IEND', Buffer.alloc(0))]);
}

/** Counts tunnelled requests per path prefix so the negative cache can be checked. */
async function proxyFixture(): Promise<{ port: number; hits: Map<string, number>; close: () => Promise<void> }> {
  const sockets = new Set<Socket>(); const hits = new Map<string, number>();
  const origin = createHttpServer((request, response) => {
    const path = new URL(request.url ?? '/', 'http://1.2.3.4').pathname;
    hits.set(path, (hits.get(path) ?? 0) + 1);
    if (/^\/hang/.test(path)) return; // never answer: the server's clock must end it
    if (/^\/ok/.test(path)) { response.writeHead(200, { 'content-type': 'image/png', 'cache-control': 'no-store' }); response.end(png(48)); return; }
    response.writeHead(404); response.end();
  });
  const proxy = createHttpServer((request, response) => origin.emit('request', request, response));
  proxy.on('connect', (request, socket: Socket, head) => {
    sockets.add(socket); socket.on('close', () => sockets.delete(socket)); socket.on('error', () => undefined);
    if (request.url !== '1.2.3.4:80') { socket.destroy(); return; }
    socket.write('HTTP/1.1 200 Connection Established\r\n\r\n'); if (head.length) socket.unshift(head); origin.emit('connection', socket);
  });
  proxy.on('connection', (socket) => { sockets.add(socket); socket.on('close', () => sockets.delete(socket)); });
  await new Promise<void>((done) => proxy.listen(0, '127.0.0.1', done));
  return { port: (proxy.address() as { port: number }).port, hits, close: async () => { for (const s of sockets) s.destroy(); await new Promise<void>((done) => proxy.close(() => done())); origin.close(); } };
}

async function smtp(from: string, message: string): Promise<void> {
  const port = Number(process.env['MW_E2E_SMTP_PORT'] ?? 42225);
  const steps = ['EHLO t56', `MAIL FROM:<${from}>`, 'RCPT TO:<testuser@example.org>', 'DATA', `${message}\r\n.`, 'QUIT'];
  await new Promise<void>((resolve, reject) => {
    const socket = connect(port, '127.0.0.1'); let i = 0; let buffer = '';
    socket.on('data', (chunk) => { buffer += chunk.toString(); if (!/\r\n$/.test(buffer)) return; const code = Number(buffer.slice(0, 3)); buffer = ''; if (code >= 400) { reject(new Error(`SMTP ${code}`)); socket.destroy(); return; } if (i < steps.length) socket.write(`${steps[i++]}\r\n`); else socket.end(); });
    socket.on('close', () => resolve()); socket.on('error', reject);
  });
}
async function seed(subject: string, from: string, html: string): Promise<void> {
  await smtp(from, `From: Sender <${from}>\r\nTo: testuser@example.org\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <${Date.now()}.${Math.random()}@example.test>\r\nSubject: ${subject}\r\nMIME-Version: 1.0\r\nContent-Type: text/html; charset=utf-8\r\n\r\n${html}`);
}
const hangingHtml = (tag: string): string =>
  `<p>Hanging newsletter ${tag}</p>` +
  Array.from({ length: HANGING }, (_, n) => `<p><img alt="Hanging banner ${n}" width="300" height="120" src="http://1.2.3.4/hang${n}.png?${tag}"></p>`).join('') +
  `<p><img alt="Fast logo" width="48" height="48" src="http://1.2.3.4/ok.png?${tag}"></p><p>${'Body text. '.repeat(30)}</p>`;

async function adminLogin(api: APIRequestContext): Promise<void> {
  expect((await api.post(`${adminPath}/login`, { data: { username: 'root', password: 'hunter2' } })).status()).toBe(200);
  expect((await api.post(`${adminPath}/step-up`, { data: { password: 'hunter2' } })).status()).toBe(200);
}
async function setGlobal(api: APIRequestContext, key: string, value: unknown): Promise<() => Promise<void>> {
  const before = await api.get(`${adminPath}/settings/${key}`); expect(before.status(), `${key} exists`).toBe(200);
  const saved = await before.json() as { globalValue: unknown; globalLocked: boolean };
  const put = await api.put(`${adminPath}/settings/${key}`, { data: { globalValue: value, locked: false }, headers: { 'if-match': before.headers()['etag']! } }); expect(put.ok(), await put.text()).toBe(true);
  return async () => { const cur = await api.get(`${adminPath}/settings/${key}`); await api.put(`${adminPath}/settings/${key}`, { data: { globalValue: saved.globalValue ?? null, locked: saved.globalLocked }, headers: { 'if-match': cur.headers()['etag']! } }); };
}
async function userSetting(page: Page, key: string, value: unknown): Promise<void> {
  const me = await (await page.request.get('/api/me')).json() as { storageAuthority: string };
  const r = await page.request.put(`/api/settings/${key}`, { data: { value }, headers: { 'x-mailwoman-storage-authority': me.storageAuthority } }); expect(r.ok(), await r.text()).toBe(true);
}
async function signIn(page: Page): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill('testuser');
  await page.getByLabel('Password', { exact: true }).fill('testpass');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 30000 });
}
async function open(page: Page, subject: string): Promise<number> {
  const row = page.locator('.list__row').filter({ hasText: subject });
  await expect(async () => {
    if (!(await row.isVisible())) await page.getByRole('navigation', { name: 'Mailboxes' }).getByRole('button', { name: 'Inbox' }).click();
    await expect(row).toBeVisible({ timeout: 3000 });
  }).toPass({ timeout: 60000 });
  const started = Date.now();
  await row.click();
  await expect(page.locator('iframe.reader__frame').filter({ visible: true }).first()).toBeVisible({ timeout: 20000 });
  return started;
}
async function withProxy(admin: APIRequestContext, tag: string, run: () => Promise<void>): Promise<void> {
  const fixture = await proxyFixture();
  const id = `t67-${tag}-${Date.now()}`; let previous: string | undefined;
  try {
    await adminLogin(admin);
    previous = ((await (await admin.get(`${adminPath}/egress/proxies`)).json()) as { proxies: { id: string; active: boolean }[] }).proxies.find((p) => p.active)?.id;
    expect((await admin.post(`${adminPath}/egress/proxies`, { data: { id, scheme: 'http', host: '127.0.0.1', port: fixture.port, username: '', allowPlaintext: true, trafficClasses: ['remote_content'], scope: 'global' } })).ok()).toBe(true);
    expect((await admin.post(`${adminPath}/egress/proxies/${id}/activate`)).ok()).toBe(true);
    await run();
  } finally {
    await admin.post(`${adminPath}/egress/proxies/${id}/deactivate`).catch(() => undefined);
    if (previous) await admin.post(`${adminPath}/egress/proxies/${previous}/activate`).catch(() => undefined);
    await admin.delete(`${adminPath}/egress/proxies/${id}`).catch(() => undefined);
    await fixture.close();
  }
}

test.describe.configure({ mode: 'default' });
test.beforeEach(async ({ page }, info) => {
  test.skip(info.project.name === 'phone', 'desktop engines at 1440 px');
  await page.setViewportSize({ width: 1440, height: 900 });
  mkdirSync(SHOTS, { recursive: true });
});

test('1. the loading-images toast ends in a failure count with Retry, not "Loading images…"', async ({ page, playwright, baseURL }, info) => {
  test.setTimeout(4 * 60_000);
  const admin = await playwright.request.newContext({ baseURL: baseURL ?? '' });
  const seen: { at: number; text: string }[] = [];
  await withProxy(admin, info.project.name, async () => {
    const restore = await setGlobal(admin, 'privacy.fetchTotalTimeoutMs', 3000);
    try {
      const run = `${info.project.name}-${Date.now()}`; const subject = `T67 toast ${run}`;
      await seed(subject, `toast-${run}@example.test`, hangingHtml(run) + `<p><img alt="Missing" width="40" height="40" src="http://1.2.3.4/missing.png?${run}"></p>`);
      await page.waitForTimeout(3100);
      await signIn(page);
      await userSetting(page, 'privacy.remoteContent', 'proxy');
      await page.reload(); await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 30000 });
      const toast = page.getByTestId('remote-images-toast');
      const t0 = Date.now();
      const net: string[] = [];
      page.on('request', (r) => { const u = new URL(r.url()); if (/\/api\/rc\/(i|sign|stats)/.test(u.pathname)) net.push(`${Date.now() - t0} > ${u.pathname.slice(0, 14)}`); });
      page.on('response', (r) => { const u = new URL(r.url()); if (/\/api\/rc\/(i|sign|stats)/.test(u.pathname)) void r.text().then((b) => net.push(`${Date.now() - t0} < ${r.status()} ${u.pathname.slice(0, 14)} ${u.pathname.includes('stats') ? b : ''}`)).catch(() => net.push(`${Date.now() - t0} < ${r.status()} ${u.pathname.slice(0, 14)}`)); });
      (globalThis as unknown as { __net?: string[] }).__net = net;
      const sample = setInterval(() => { void toast.allTextContents().then((texts) => { const text = texts.join(' | '); if (seen.at(-1)?.text !== text) seen.push({ at: Date.now() - t0, text }); }).catch(() => undefined); }, 100);
      try {
        await open(page, subject);
        await expect(toast).toContainText('Loading images', { timeout: 10000 });
        await page.screenshot({ path: join(SHOTS, `${info.project.name}-1-toast-loading.png`) });
        // HANGING timed out + 1 missing.
        await expect(toast).toContainText(`${HANGING + 1} images couldn`, { timeout: 30000 });
        await expect(toast.getByRole('button', { name: 'Retry' })).toBeVisible();
        await page.screenshot({ path: join(SHOTS, `${info.project.name}-1-toast-failed.png`) });
        const resign = page.waitForRequest((r) => r.url().endsWith('/api/rc/sign') && r.method() === 'POST' && (JSON.parse(r.postData() ?? '{}') as { retry?: boolean }).retry === true, { timeout: 15000 });
        await toast.getByRole('button', { name: 'Retry' }).click();
        await resign;
        await expect(toast).toContainText(`${HANGING + 1} images couldn`, { timeout: 30000 });
        await page.waitForTimeout(3000);
        await expect(toast).not.toContainText('Loading images');
        await page.screenshot({ path: join(SHOTS, `${info.project.name}-1-toast-after-retry.png`) });
      } finally { clearInterval(sample); }
    } finally {
      console.log('T67-TOAST', info.project.name, JSON.stringify(seen));
      if (process.env['MW_T67_NET']) console.log('T67-NET', ((globalThis as unknown as { __net?: string[] }).__net ?? []).join(' ; '));
      await restore(); await userSetting(page, 'privacy.remoteContent', 'ask').catch(() => undefined);
    }
  });
});

test('2. a toast stays clear of the bulk-selection bar in a 1440 px folder tab', async ({ page }, info) => {
  await signIn(page);
  const restoreLayout = await useTabbedWorkspace(page);
  // A folder with a long name: starring its tab raises a toast as wide as the
  // column allows (28rem), whose start reaches under the right end of the bar.
  const session = await (await page.request.get('/jmap/session')).json() as { primaryAccounts: Record<string, string> };
  const accountId = session.primaryAccounts['urn:ietf:params:jmap:mail'];
  const call = async (name: string, args: Record<string, unknown>): Promise<Record<string, unknown>> => {
    const r = await postJmap(page.request, { data: { using: ['urn:ietf:params:jmap:core', 'urn:ietf:params:jmap:mail'], methodCalls: [[name, { accountId, ...args }, 'c']] } });
    expect(r.ok(), await r.text()).toBe(true);
    return ((await r.json()) as { methodResponses: [string, Record<string, unknown>][] }).methodResponses[0]![1];
  };
  const folder = `T67 a folder with a long name for a wide toast ${info.project.name} ${Date.now()}`;
  const created = await call('Mailbox/set', { create: { f: { name: folder } } }) as { created?: { f?: { id: string } } };
  const folderId = created.created?.f?.id; expect(folderId).toBeTruthy();
  let moveBack = async (): Promise<void> => undefined;
  try {
    const inbox = ((await call('Mailbox/get', {})) as { list: { id: string; role: string | null }[] }).list.find((b) => b.role === 'inbox')!;
    const ids = ((await call('Email/query', { filter: { inMailbox: inbox.id }, limit: 3 })) as { ids: string[] }).ids;
    const moved = await call('Email/set', { update: Object.fromEntries(ids.map((id) => [id, { mailboxIds: { [folderId!]: true } }])) }) as { updated?: Record<string, unknown> };
    expect(Object.keys(moved.updated ?? {}).length, JSON.stringify(moved)).toBe(ids.length);
    moveBack = async () => { const now = ((await call('Email/query', { filter: { inMailbox: folderId } })) as { ids: string[] }).ids; await call('Email/set', { update: Object.fromEntries(now.map((id) => [id, { mailboxIds: { [inbox.id]: true } }])) }); };
    await page.reload(); await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 30000 });
    await page.getByRole('navigation', { name: 'Mailboxes' }).getByRole('button', { name: new RegExp(`^${folder}`) }).click({ button: 'right' });
    await page.getByRole('menuitem', { name: 'Open in new tab' }).click();
    const panel = page.locator('[role="tabpanel"]:visible').filter({ has: page.locator('[data-folder-message]') }).last();
    await expect(panel.locator('[data-folder-message]').nth(1)).toBeVisible({ timeout: 20000 });
    await panel.getByRole('button', { name: 'Select this page' }).click();
    const bar = page.getByTestId('bulk-bar').filter({ visible: true }).first();
    await expect(bar).toBeVisible();
    const folderTab = page.getByRole('tab', { selected: true });
    await folderTab.click({ button: 'right' });
    await page.getByRole('menuitem', { name: 'Star', exact: true }).click();
    const toast = page.getByTestId('toast-region').locator('[data-toast-id]').first();
    await expect(toast).toBeVisible({ timeout: 10000 });
    await expect(bar).toBeVisible();
    await page.waitForTimeout(400);
    const b = (await bar.boundingBox())!; const tb = (await toast.boundingBox())!;
    await page.screenshot({ path: join(SHOTS, `${info.project.name}-2-toast-bar.png`) });
    const horizontal = tb.x < b.x + b.width && tb.x + tb.width > b.x;
    const overlaps = horizontal && tb.y < b.y + b.height && tb.y + tb.height > b.y;
    console.log('T67-BAR', info.project.name, JSON.stringify({ bar: b, toast: tb, horizontal, overlaps }));
    await folderTab.click({ button: 'right' });
    await page.getByRole('menuitem', { name: /^Unstar/ }).click().catch(() => undefined);
    expect(horizontal, 'the toast column reaches under the bar in this layout').toBe(true);
    expect(overlaps).toBe(false);
    expect(tb.y + tb.height).toBeLessThanOrEqual(b.y);
  } finally {
    await moveBack().catch(() => undefined);
    await call('Mailbox/set', { destroy: [folderId], onDestroyRemoveEmails: false }).catch(() => undefined);
    await restoreLayout();
  }
});

test('3. Settings editors ask in the in-app dialog; a bulk close asks once', async ({ page }, info) => {
  const native: string[] = []; page.on('dialog', (d) => { native.push(d.message()); void d.dismiss(); });
  await signIn(page);
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByRole('button', { name: 'Time zones', exact: true }).click();
  const comparison = page.getByRole('combobox', { name: 'Comparison time zone' });
  await comparison.click(); await comparison.fill('Lisbon');
  await page.getByRole('option', { name: /Lisbon/ }).first().click();
  await page.getByRole('button', { name: 'Labels', exact: true }).click();
  const dialog = page.getByTestId('settings-leave-confirm');
  await expect(dialog).toBeVisible();
  await expect(dialog).toContainText('Discard unsaved time zone changes?');
  await page.screenshot({ path: join(SHOTS, `${info.project.name}-3-leave-dialog.png`) });
  await dialog.getByRole('button', { name: 'Keep editing' }).click();
  await expect(dialog).toHaveCount(0);
  await page.getByRole('tab', { name: 'Mail' }).click({ button: 'right' });
  await page.getByRole('menuitem', { name: 'Close all tabs' }).click();
  const bulk = page.getByTestId('workspace-tabs-close-dirty');
  await expect(bulk).toBeVisible();
  await expect(page.getByRole('alertdialog')).toHaveCount(1);
  await page.screenshot({ path: join(SHOTS, `${info.project.name}-3-bulk-close.png`) });
  await bulk.getByRole('button', { name: 'Discard and close' }).click();
  await expect(page.getByRole('tab', { name: 'Settings' })).toHaveCount(0, { timeout: 10000 });
  await expect(page.getByRole('alertdialog')).toHaveCount(0);
  expect(native).toEqual([]);
});

test('5. an account preference changed just before a reload survives it', async ({ page }, info) => {
  await signIn(page);
  const read = async (): Promise<unknown> => {
    const me = (await (await page.request.get('/api/me')).json()) as { storageAuthority: string };
    return ((await (await page.request.get('/api/account/preferences', { headers: { 'x-mailwoman-storage-authority': me.storageAuthority } })).json()) as { preferences: Record<string, unknown> }).preferences['folderSidebarMode'];
  };
  const results: { want: string; got: unknown }[] = [];
  const compact = page.getByRole('button', { name: 'Compact folder icons' });
  for (let round = 0; round < 6; round++) {
    await expect(compact).toBeVisible({ timeout: 20000 });
    const want = (await compact.getAttribute('aria-pressed')) === 'true' ? 'expanded' : 'compact';
    await compact.click();
    await page.reload();   // well inside the 400 ms debounce
    await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 30000 });
    let got: unknown;
    await expect.poll(async () => (got = await read()), { timeout: 8000 }).toBe(want).catch(() => undefined);
    results.push({ want, got });
  }
  console.log('T67-PREFS', info.project.name, JSON.stringify(results));
  expect(results.filter((r) => r.got !== r.want)).toEqual([]);
});
