// Remote image timeouts, live (t56). Runs on the receipts stack
// (scripts/e2e-receipts-stack.mjs): the HEAD-built server, Greenmail, and an
// egress-proxy fixture that tunnels `1.2.3.4` (the privacy-reader fixture
// pattern, as in remote-images-banner-live.spec.ts). Behind the tunnel,
// /hang*.png accepts the request and never answers; /ok*.png answers at once.
//
// For Chromium and Firefox:
//   - with privacy.fetchTotalTimeoutMs = 3000 (admin), a message whose images
//     hang shows a placeholder per image once ~3 s pass, in the image's own
//     300x120 box, while a fast image in the same message renders inside the
//     sandboxed frame (no CORP console error, t56's CORP fix);
//   - the reader stays responsive while images hang: the list scrolls, the
//     search box takes input and the banner menu opens, each promptly;
//   - another message with the same image URLs answers the failed images from
//     the 60 s negative cache: placeholders at once, no new upstream request;
//   - raising the timeout in admin to 6000 delays the placeholders accordingly.
// Screenshots: MW_T56_SHOTS (default test-results/t56).
import { test, expect, type Page, type APIRequestContext, type FrameLocator } from '../csrf.ts';
import { createServer as createHttpServer } from 'node:http';
import { deflateSync } from 'node:zlib';
import { connect, type Socket } from 'node:net';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';

const SHOTS = process.env['MW_T56_SHOTS'] ?? 'test-results/t56';
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
function body(page: Page): FrameLocator {
  return page.locator('iframe.reader__frame').filter({ visible: true }).first().contentFrame().frameLocator('iframe');
}
type ImageState = { alt: string; complete: boolean; natural: number; w: number; h: number };
async function images(page: Page): Promise<ImageState[]> {
  // The reader may replace the nested frame while a message settles (a re-post
  // of the same body); read again from the live frame when that happens.
  for (let attempt = 0; ; attempt++) {
    try { return await readImages(page); } catch (error) { if (attempt >= 5 || !/detached/i.test(String(error))) throw error; await page.waitForTimeout(200); }
  }
}
async function readImages(page: Page): Promise<ImageState[]> {
  return body(page).locator('img').evaluateAll((nodes) => nodes.map((n) => { const i = n as HTMLImageElement; const r = i.getBoundingClientRect(); return { alt: i.alt, complete: i.complete, natural: i.naturalWidth, w: Math.round(r.width), h: Math.round(r.height) }; }));
}
/** ms from `since` until every hanging image is settled as broken. */
async function untilPlaceholders(page: Page, since: number, limit: number): Promise<{ elapsed: number; states: ImageState[] }> {
  let states: ImageState[] = [];
  await expect(async () => {
    states = await images(page);
    const hanging = states.filter((s) => s.alt.startsWith('Hanging'));
    expect(hanging).toHaveLength(HANGING);
    expect(hanging.every((s) => s.complete && s.natural === 0)).toBe(true);
  }).toPass({ timeout: limit, intervals: [100] });
  return { elapsed: Date.now() - since, states };
}

test.describe.configure({ mode: 'serial' });

test('t56 signed remote images render in the sandboxed frame; failures carry CORP and log no CORP error', async ({ page, playwright, baseURL }, info) => {
  test.skip(info.project.name === 'phone', 'desktop engines only');
  test.setTimeout(3 * 60_000);
  mkdirSync(SHOTS, { recursive: true });
  const admin = await playwright.request.newContext({ baseURL: baseURL ?? '' });
  const fixture = await proxyFixture();
  const id = `t56c-${info.project.name}-${Date.now()}`; let previous: string | undefined;
  const corp: string[] = []; page.on('console', (m) => { if (/Cross-Origin-Resource-Policy|ERR_BLOCKED_BY_RESPONSE|blocked due to its/i.test(m.text())) corp.push(m.text()); });
  const rc: { path: string; status: number; corp: string | null; type: string | null; nosniff: string | null; cookie: boolean }[] = [];
  page.on('response', (r) => { const u = new URL(r.url()); if (/\/api\/rc\/[ip]\//.test(u.pathname)) rc.push({ path: u.pathname.slice(0, 10), status: r.status(), corp: r.headers()['cross-origin-resource-policy'] ?? null, type: r.headers()['content-type'] ?? null, nosniff: r.headers()['x-content-type-options'] ?? null, cookie: 'set-cookie' in r.headers() }); });
  try {
    await adminLogin(admin);
    previous = ((await (await admin.get(`${adminPath}/egress/proxies`)).json()) as { proxies: { id: string; active: boolean }[] }).proxies.find((p) => p.active)?.id;
    expect((await admin.post(`${adminPath}/egress/proxies`, { data: { id, scheme: 'http', host: '127.0.0.1', port: fixture.port, username: '', allowPlaintext: true, trafficClasses: ['remote_content'], scope: 'global' } })).ok()).toBe(true);
    expect((await admin.post(`${adminPath}/egress/proxies/${id}/activate`)).ok()).toBe(true);
    const run = `${info.project.name}-${Date.now()}`; const subject = `T56 corp ${run}`;
    await seed(subject, `corp-${run}@example.test`, `<p>CORP check ${run}</p><p><img alt="Fast logo" width="48" height="48" src="http://1.2.3.4/ok.png?${run}"></p><p><img alt="Missing image" width="200" height="80" src="http://1.2.3.4/missing.png?${run}"></p>`);
    await page.waitForTimeout(3100);
    await signIn(page);
    await userSetting(page, 'privacy.remoteContent', 'proxy');
    await page.reload(); await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 30000 });
    await open(page, subject);
    await expect(body(page).getByText(`CORP check ${run}`)).toBeVisible({ timeout: 20000 });
    await expect(async () => {
      const st = await images(page);
      expect(st.find((s) => s.alt === 'Fast logo')?.natural).toBe(48);
      const missing = st.find((s) => s.alt === 'Missing image');
      expect(missing?.complete && missing.natural === 0).toBe(true);
    }).toPass({ timeout: 20000 });
    const st = await images(page);
    expect(st.find((s) => s.alt === 'Missing image')).toMatchObject({ w: 200, h: 80 });
    await page.screenshot({ path: join(SHOTS, `${info.project.name}-corp-render.png`) });
    // Playwright does not report every nested opaque-frame response in Chromium,
    // so the headers are read by requesting each image's own signed link again
    // (no cookie: the token is the authorisation, as in the frame).
    const srcs = await body(page).locator('img').evaluateAll((nodes) => nodes.map((i) => ({ alt: (i as HTMLImageElement).alt, src: (i as HTMLImageElement).src })));
    const anon = await playwright.request.newContext();
    for (const { alt, src } of srcs) {
      const r = await anon.get(src); const h = r.headers();
      expect(h['cross-origin-resource-policy'], alt).toBe('cross-origin');
      expect(h['x-content-type-options'], alt).toBe('nosniff');
      expect('set-cookie' in h, alt).toBe(false);
      if (alt === 'Fast logo') { expect(r.status()).toBe(200); expect(h['content-type']).toBe('image/png'); }
      else { expect(r.status()).toBe(502); expect(h['x-mw-remote-content']).toBe('failed'); }
    }
    await anon.dispose();
    for (const r of rc) expect(r.corp, `${r.status} ${r.path}`).toBe('cross-origin');
    expect(corp, 'no CORP console errors').toEqual([]);
    console.log('T56-CORP', info.project.name, JSON.stringify(rc));
  } finally {
    await admin.post(`${adminPath}/egress/proxies/${id}/deactivate`).catch(() => undefined);
    if (previous) await admin.post(`${adminPath}/egress/proxies/${previous}/activate`).catch(() => undefined);
    await admin.delete(`${adminPath}/egress/proxies/${id}`).catch(() => undefined);
    await userSetting(page, 'privacy.remoteContent', 'ask').catch(() => undefined);
    await fixture.close();
  }
});

test('t56 hanging remote images become placeholders after the configured timeout', async ({ page, playwright, baseURL }, info) => {
  test.skip(info.project.name === 'phone', 'desktop engines only');
  test.setTimeout(6 * 60_000);
  const shots = SHOTS; mkdirSync(shots, { recursive: true });
  const admin = await playwright.request.newContext({ baseURL: baseURL ?? '' });
  const fixture = await proxyFixture();
  const id = `t56-${info.project.name}-${Date.now()}`; let previous: string | undefined; const restore: (() => Promise<void>)[] = [];
  const corp: string[] = []; page.on('console', (m) => { if (/Cross-Origin-Resource-Policy|ERR_BLOCKED_BY_RESPONSE/i.test(m.text())) corp.push(m.text()); });
  const failed: { status: number; marker: string | null; cache: string | null; corp: string | null }[] = [];
  page.on('response', (r) => { if (r.url().includes('/api/rc/i/') && r.status() !== 200 && r.status() !== 304) failed.push({ status: r.status(), marker: r.headers()['x-mw-remote-content'] ?? null, cache: r.headers()['cache-control'] ?? null, corp: r.headers()['cross-origin-resource-policy'] ?? null }); });
  const report: Record<string, unknown> = { project: info.project.name };
  try {
    await adminLogin(admin);
    previous = ((await (await admin.get(`${adminPath}/egress/proxies`)).json()) as { proxies: { id: string; active: boolean }[] }).proxies.find((p) => p.active)?.id;
    const created = await admin.post(`${adminPath}/egress/proxies`, { data: { id, scheme: 'http', host: '127.0.0.1', port: fixture.port, username: '', allowPlaintext: true, trafficClasses: ['remote_content'], scope: 'global' } });
    expect(created.ok(), await created.text()).toBe(true);
    expect((await admin.post(`${adminPath}/egress/proxies/${id}/activate`)).ok()).toBe(true);
    restore.push(await setGlobal(admin, 'privacy.fetchTotalTimeoutMs', 3000));

    const run = `${info.project.name}-${Date.now()}`;
    const first = `T56 hang 3s ${run}`, second = `T56 hang 6s ${run}`;
    await seed(first, `news3-${run}@example.test`, hangingHtml(`a${run}`));
    await seed(second, `news6-${run}@example.test`, hangingHtml(`b${run}`));
    const dup = `T56 same images ${run}`;
    await seed(dup, `news3b-${run}@example.test`, hangingHtml(`a${run}`).replace('Hanging newsletter', 'Same images'));

    await page.waitForTimeout(3100); // sign-in cooldown
    await signIn(page);
    await userSetting(page, 'privacy.remoteContent', 'proxy');
    await page.reload(); await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 30000 });

    // ── 3 s ────────────────────────────────────────────────────────────────
    const t0 = await open(page, first);
    await expect(body(page).getByText(`Hanging newsletter a${run}`)).toBeVisible({ timeout: 20000 });
    // Still pending well before the deadline, and the fast image is already in.
    await page.waitForTimeout(1200);
    const early = await images(page);
    report.pendingAt1200ms = early.filter((s) => s.alt.startsWith('Hanging') && !s.complete).length;
    expect(early.find((s) => s.alt === 'Fast logo')?.natural, 'a granted image renders inside the sandboxed frame').toBe(48);
    await page.screenshot({ path: join(shots, `${info.project.name}-1-pending.png`) });

    // Responsive while images hang, without leaving the message (leaving would
    // abort the frame's requests): the list scrolls, the search box takes input,
    // and the banner's menu opens and closes, each promptly.
    const list = page.locator('.list__row').first();
    await list.hover(); await page.mouse.wheel(0, 300); await page.mouse.wheel(0, -300);
    const search = page.getByPlaceholder(/Search mail/);
    let started = Date.now();
    await search.fill('t56 responsiveness'); await expect(search).toHaveValue('t56 responsiveness');
    report.searchInputMs = Date.now() - started;
    await search.fill('');
    started = Date.now();
    await page.getByRole('button', { name: 'More image options' }).filter({ visible: true }).first().click();
    await expect(page.getByRole('menu').filter({ visible: true }).first()).toBeVisible({ timeout: 3000 });
    report.menuOpenMs = Date.now() - started;
    await page.keyboard.press('Escape');
    expect(report.searchInputMs as number).toBeLessThan(1500);
    expect(report.menuOpenMs as number).toBeLessThan(1500);

    const settled = await untilPlaceholders(page, t0, 15000);
    report.placeholdersWith3sMs = settled.elapsed;
    expect(settled.elapsed).toBeGreaterThanOrEqual(2500);
    expect(settled.elapsed).toBeLessThan(7000);
    for (const s of settled.states.filter((x) => x.alt.startsWith('Hanging'))) {
      expect([s.w, s.h], `${s.alt} keeps its own box`).toEqual([300, 120]);
    }
    expect(settled.states.find((x) => x.alt === 'Fast logo')?.natural).toBe(48);
    await page.screenshot({ path: join(shots, `${info.project.name}-2-placeholders-3s.png`) });
    await body(page).locator('img[alt="Hanging banner 0"]').screenshot({ path: join(shots, `${info.project.name}-placeholder-closeup.png`) });

    // Another message with the same image URLs: the timed-out fetches are
    // remembered for 60 s, so its placeholders come at once and the origin sees
    // no new request.
    const hangHits = (): number => [...fixture.hits].filter(([p]) => p.startsWith('/hang')).reduce((a, [, n]) => a + n, 0);
    const before = hangHits();
    const t3 = await open(page, dup);
    await expect(body(page).getByText(`Same images a${run}`)).toBeVisible({ timeout: 20000 });
    const cached = await untilPlaceholders(page, t3, 15000);
    report.negativeCachePlaceholdersMs = cached.elapsed;
    expect(cached.elapsed).toBeLessThan(2500);
    expect(hangHits() - before, 'no upstream refetch inside the negative-cache window').toBe(0);
    report.hangUpstreamHits = hangHits();

    // ── 6 s, set in admin ──────────────────────────────────────────────────
    restore.push(await setGlobal(admin, 'privacy.fetchTotalTimeoutMs', 6000));
    const t2 = await open(page, second);
    await expect(body(page).getByText(`Hanging newsletter b${run}`)).toBeVisible({ timeout: 20000 });
    await page.waitForTimeout(Math.max(0, 4200 - (Date.now() - t2)));
    const at4s = await images(page);
    report.pendingAt4200msWith6s = at4s.filter((s) => s.alt.startsWith('Hanging') && !s.complete).length;
    expect(report.pendingAt4200msWith6s, 'not yet at 4.2 s with a 6 s timeout').toBe(HANGING);
    const slow = await untilPlaceholders(page, t2, 20000);
    report.placeholdersWith6sMs = slow.elapsed;
    expect(slow.elapsed).toBeGreaterThanOrEqual(5500);
    expect(slow.elapsed).toBeLessThan(10000);
    await page.screenshot({ path: join(shots, `${info.project.name}-3-placeholders-6s.png`) });

    // The failures the frame received: a 504 marked failed, briefly cacheable,
    // and readable by the opaque frame (CORP cross-origin), with no CORP console noise.
    // Chromium does not report every nested opaque-frame response to Playwright,
    // so the signed links are also requested again directly (the negative cache
    // answers them at once, exactly as the frame saw them).
    const anon = await playwright.request.newContext();
    for (const { alt, src } of await body(page).locator('img').evaluateAll((nodes) => nodes.map((i) => ({ alt: (i as HTMLImageElement).alt, src: (i as HTMLImageElement).src })))) {
      if (!alt.startsWith('Hanging')) continue;
      const r = await anon.get(src); const h = r.headers();
      failed.push({ status: r.status(), marker: h['x-mw-remote-content'] ?? null, cache: h['cache-control'] ?? null, corp: h['cross-origin-resource-policy'] ?? null });
    }
    await anon.dispose();
    expect(failed.length).toBeGreaterThan(0);
    for (const f of failed) {
      expect(f.status).toBe(504);
      expect(f.marker).toBe('failed');
      expect(f.cache).toBe('private, max-age=60');
      expect(f.corp).toBe('cross-origin');
    }
    expect(corp, 'no CORP console errors').toEqual([]);
    report.failedResponses = failed.length;
  } finally {
    info.annotations.push({ type: 't56', description: JSON.stringify(report) });
    console.log('T56-REPORT', JSON.stringify(report));
    for (const r of restore.reverse()) await r().catch(() => undefined);
    await admin.post(`${adminPath}/egress/proxies/${id}/deactivate`).catch(() => undefined);
    if (previous) await admin.post(`${adminPath}/egress/proxies/${previous}/activate`).catch(() => undefined);
    await admin.delete(`${adminPath}/egress/proxies/${id}`).catch(() => undefined);
    await userSetting(page, 'privacy.remoteContent', 'ask').catch(() => undefined);
    await fixture.close();
  }
});

test('t56 the admin console shows the fetch limits and saves a new total timeout', async ({ page }, info) => {
  test.skip(info.project.name === 'phone', 'desktop engines only');
  // The page's own request context shares its cookies, so this signs the console in.
  await adminLogin(page.request);
  let restore: (() => Promise<void>) | undefined;
  try {
    restore = await setGlobal(page.request, 'privacy.fetchTotalTimeoutMs', 20000);
    await page.goto(`${adminPath}#/privacy`);
    const section = page.locator('section').filter({ has: page.getByRole('heading', { name: 'Remote image fetching' }) }).last();
    await expect(section).toBeVisible({ timeout: 20000 });
    for (const label of ['Remote image connect timeout', 'Remote image response timeout', 'Remote image stall timeout', 'Remote image total timeout', 'Remote image redirects', 'Remote image downloads per message', 'Remote image downloads per account']) await expect(section.getByText(label, { exact: true })).toBeVisible();
    await expect(section).toContainText('20 sec');
    await page.screenshot({ path: join(SHOTS, `${info.project.name}-admin-privacy.png`), fullPage: true });
    await section.locator('a[href$="privacy.fetchTotalTimeoutMs"]').click();
    await expect(page.getByRole('heading', { name: 'Remote image total timeout' })).toBeVisible();
    const form = page.locator('form[aria-labelledby="setting-global-heading"]');
    const value = form.getByRole('spinbutton', { name: /Remote image total timeout/ });
    await value.fill('200000');
    await form.getByRole('button', { name: 'Save' }).click();
    await expect(page.getByRole('alert').or(page.locator('[aria-invalid="true"]')).first()).toBeVisible();
    await value.fill('6000');
    await form.getByRole('button', { name: 'Save' }).click();
    await expect.poll(async () => ((await (await page.request.get(`${adminPath}/settings/privacy.fetchTotalTimeoutMs`)).json()) as { globalValue: unknown }).globalValue, { timeout: 10000 }).toBe(6000);
    await page.screenshot({ path: join(SHOTS, `${info.project.name}-admin-total-timeout-saved.png`), fullPage: true });
    // The out-of-range value never reached the server.
    const refused = await page.request.put(`${adminPath}/settings/privacy.fetchTotalTimeoutMs`, { data: { globalValue: 500, locked: false }, headers: { 'if-match': (await page.request.get(`${adminPath}/settings/privacy.fetchTotalTimeoutMs`)).headers()['etag']! } });
    expect(refused.status()).toBe(422);
  } finally {
    await restore?.().catch(() => undefined);
  }
});
