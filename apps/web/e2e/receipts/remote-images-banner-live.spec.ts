// Remote-images banner, loading toast and stats reads, live (t55). Runs on the
// receipts stack (scripts/e2e-receipts-stack.mjs): a real server, Greenmail, and
// an egress proxy fixture that serves the message's "remote" images slowly from
// 1.2.3.4 (CONNECT-tunnelled, the privacy-reader fixture pattern).
//
// For Chromium and Firefox, at 1440 and 390 px, in dark and light:
//   - a plain message shows no banner;
//   - a message with remote images and trackers shows ONE line (measured);
//   - Show images -> one in-place toast, then "Images shown · Undo"; the
//     banner height and the body's top do not move while images load;
//   - Undo -> blocked again; menu "Always show images from <sender>" -> shown;
//   - /api/rc/stats: at most 2 reads for the open message, 0 once it is left.
// Proxy mode: the banner text settles (never "0"), the body does not jump.
// privacy.remoteImagesToast off (admin, locked): no toast.
//
// MW_T55_PHASE=before runs only the screenshots/flicker probe against an older
// build (the old multi-line bar), recording what changes; it asserts nothing
// about the new banner.
import { test, expect, type Page, type APIRequestContext } from '../csrf.ts';
import { createServer as createHttpServer } from 'node:http';
import { deflateSync } from 'node:zlib';
import { connect, type Socket } from 'node:net';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';

const PHASE = process.env['MW_T55_PHASE'] === 'before' ? 'before' : 'after';
const SHOTS = process.env['MW_T55_SHOTS'] ?? 'test-results/t55';
const adminPath = process.env['MW_E2E_ADMIN_PATH'] ?? '/admin';
const IMAGES = 12;

function png(size = 8): Buffer {
  const crc = (b: Buffer): number => { let c = 0xffffffff; for (const v of b) { c ^= v; for (let i = 0; i < 8; i++) c = c & 1 ? (c >>> 1) ^ 0xedb88320 : c >>> 1; } return (c ^ 0xffffffff) >>> 0; };
  const chunk = (name: string, data: Buffer): Buffer => { const bytes = Buffer.concat([Buffer.from(name), data]), len = Buffer.alloc(4), sum = Buffer.alloc(4); len.writeUInt32BE(data.length); sum.writeUInt32BE(crc(bytes)); return Buffer.concat([len, bytes, sum]); };
  const head = Buffer.alloc(13); head.writeUInt32BE(size); head.writeUInt32BE(size, 4); head[8] = 8; head[9] = 6;
  const data = Buffer.alloc(size * (1 + size * 4)); for (let y = 0; y < size; y++) for (let x = 0; x < size; x++) { const p = y * (1 + size * 4) + 1 + x * 4; data[p] = 200; data[p + 1] = 90; data[p + 2] = 40; data[p + 3] = 255; }
  return Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk('IHDR', head), chunk('IDAT', deflateSync(data)), chunk('IEND', Buffer.alloc(0))]);
}

async function proxyFixture(): Promise<{ port: number; close: () => Promise<void> }> {
  const sockets = new Set<Socket>();
  const origin = createHttpServer((request, response) => {
    const path = new URL(request.url ?? '/', 'http://1.2.3.4').pathname;
    const send = (body: Buffer): void => { response.writeHead(200, { 'content-type': 'image/png', 'cache-control': 'no-store' }); response.end(body); };
    if (/^\/img\d+\.png$/.test(path)) { setTimeout(() => send(png(24)), 700); return; }
    if (path === '/pixel.png') { send(png(1)); return; }
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
  return { port: (proxy.address() as { port: number }).port, close: async () => { for (const s of sockets) s.destroy(); await new Promise<void>((done) => proxy.close(() => done())); origin.close(); } };
}

async function smtp(from: string, message: string): Promise<void> {
  const port = Number(process.env['MW_E2E_SMTP_PORT'] ?? 42225);
  const steps = ['EHLO t55', `MAIL FROM:<${from}>`, 'RCPT TO:<testuser@example.org>', 'DATA', `${message}\r\n.`, 'QUIT'];
  await new Promise<void>((resolve, reject) => {
    const socket = connect(port, '127.0.0.1'); let i = 0; let buffer = '';
    socket.on('data', (chunk) => { buffer += chunk.toString(); if (!/\r\n$/.test(buffer)) return; const code = Number(buffer.slice(0, 3)); buffer = ''; if (code >= 400) { reject(new Error(`SMTP ${code}`)); socket.destroy(); return; } if (i < steps.length) socket.write(`${steps[i++]}\r\n`); else socket.end(); });
    socket.on('close', () => resolve()); socket.on('error', reject);
  });
}

async function seed(subject: string, from: string, html: string): Promise<void> {
  await smtp(from, `From: Sender <${from}>\r\nTo: testuser@example.org\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <${Date.now()}.${Math.random()}@example.test>\r\nSubject: ${subject}\r\nMIME-Version: 1.0\r\nContent-Type: text/html; charset=utf-8\r\n\r\n${html}`);
}

const imagesHtml = (tag: string): string => `<p>Newsletter ${tag}</p>` + Array.from({ length: IMAGES }, (_, n) => `<img alt="remote ${n}" width="24" height="24" src="http://1.2.3.4/img${n}.png?${tag}">`).join('') + `<img alt="pixel" src="http://1.2.3.4/pixel.png?${tag}"><img alt="beacon" width="1" height="1" src="http://1.2.3.4/pixel.png?b=${tag}"><p>${'Body text. '.repeat(40)}</p>`;

async function adminLogin(api: APIRequestContext): Promise<void> {
  expect((await api.post(`${adminPath}/login`, { data: { username: 'root', password: 'hunter2' } })).status()).toBe(200);
  expect((await api.post(`${adminPath}/step-up`, { data: { password: 'hunter2' } })).status()).toBe(200);
}
async function setGlobal(api: APIRequestContext, key: string, value: unknown, locked = false): Promise<() => Promise<void>> {
  const before = await api.get(`${adminPath}/settings/${key}`); expect(before.status(), `${key} exists`).toBe(200);
  const saved = await before.json() as { globalValue: unknown; globalLocked: boolean };
  const put = await api.put(`${adminPath}/settings/${key}`, { data: { globalValue: value, locked }, headers: { 'if-match': before.headers()['etag']! } }); expect(put.ok(), await put.text()).toBe(true);
  return async () => { const cur = await api.get(`${adminPath}/settings/${key}`); await api.put(`${adminPath}/settings/${key}`, { data: { globalValue: saved.globalValue ?? null, locked: saved.globalLocked }, headers: { 'if-match': cur.headers()['etag']! } }); };
}
async function authority(page: Page): Promise<Record<string, string>> {
  const me = await (await page.request.get('/api/me')).json() as { storageAuthority: string };
  return { 'x-mailwoman-storage-authority': me.storageAuthority };
}
async function userSetting(page: Page, key: string, value: unknown): Promise<void> {
  const r = await page.request.put(`/api/settings/${key}`, { data: { value }, headers: await authority(page) }); expect(r.ok(), await r.text()).toBe(true);
}

async function signIn(page: Page): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill('testuser');
  await page.getByLabel('Password', { exact: true }).fill('testpass');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 30000 });
}
async function open(page: Page, subject: string): Promise<void> {
  const row = page.locator('.list__row').filter({ hasText: subject });
  await expect(async () => {
    if (!(await row.isVisible())) {
      if ((page.viewportSize()?.width ?? 1440) < 761) { await page.getByRole('button', { name: 'Open folders and apps' }).click(); }
      await page.getByRole('navigation', { name: 'Mailboxes' }).getByRole('button', { name: 'Inbox' }).click();
    }
    await expect(row).toBeVisible({ timeout: 3000 });
  }).toPass({ timeout: 60000 });
  await row.click();
  await expect(page.locator('iframe.reader__frame').filter({ visible: true }).first()).toBeVisible({ timeout: 20000 });
}

/** Samples the banner and the body frame every animation frame. */
async function startProbe(page: Page): Promise<void> {
  await page.evaluate(() => {
    const w = window as unknown as { __t55: { heights: number[]; tops: number[]; texts: string[]; toasts: number[]; decoded: boolean[]; stop: boolean } };
    w.__t55 = { heights: [], tops: [], texts: [], toasts: [], decoded: [], stop: false };
    const visible = (sel: string): HTMLElement | undefined => [...document.querySelectorAll<HTMLElement>(sel)].find((e) => e.offsetParent !== null);
    const tick = (): void => {
      const s = w.__t55; if (s.stop) return;
      const bar = visible('[data-testid="remote-content-bar"]'); const frame = visible('iframe.reader__frame');
      s.heights.push(bar ? Math.round(bar.getBoundingClientRect().height) : -1);
      s.tops.push(frame ? Math.round(frame.getBoundingClientRect().top) : -1);
      const text = bar?.querySelector('[role="status"]')?.textContent ?? bar?.textContent ?? '';
      if (s.texts.at(-1) !== text) s.texts.push(text);
      s.toasts.push(document.querySelectorAll('[data-testid="remote-images-toast"]').length);
      const d = [...document.querySelectorAll('p')].some((p) => /recently decoded/.test(p.textContent ?? '')); if (s.decoded.at(-1) !== d) s.decoded.push(d);
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });
}
async function stopProbe(page: Page): Promise<{ heights: number[]; tops: number[]; texts: string[]; toasts: number[]; decoded: boolean[] }> {
  return page.evaluate(() => { const w = window as unknown as { __t55: { heights: number[]; tops: number[]; texts: string[]; toasts: number[]; decoded: boolean[]; stop: boolean } }; w.__t55.stop = true; return w.__t55; });
}
const distinct = (xs: number[]): number[] => [...new Set(xs.filter((x) => x >= 0))];

async function oneLine(page: Page): Promise<{ height: number; summaryLines: number; tops: number; overflow: number; rowOverflow: number }> {
  const bar = page.getByTestId('remote-content-bar').filter({ visible: true });
  return bar.evaluate((node) => {
    const row = node.firstElementChild as HTMLElement; const summary = node.querySelector('[role="status"]') as HTMLElement;
    const kids = [...row.children].map((k) => k.getBoundingClientRect()).filter((r) => r.width > 0);
    const lh = parseFloat(getComputedStyle(summary).lineHeight) || 16;
    return {
      height: node.getBoundingClientRect().height,
      summaryLines: Math.round(summary.getBoundingClientRect().height / lh),
      tops: Math.max(...kids.map((k) => k.top + k.height / 2)) - Math.min(...kids.map((k) => k.top + k.height / 2)),
      overflow: document.documentElement.scrollWidth - innerWidth,
      rowOverflow: row.scrollHeight - row.clientHeight,
    };
  });
}

test.describe.configure({ mode: 'serial' });

test('t55 remote-images banner, toast and stats reads', async ({ page, playwright, baseURL }, info) => {
  test.skip(info.project.name === 'phone', 'widths are looped inside the chromium/firefox projects');
  test.setTimeout(15 * 60_000);
  const shots = join(SHOTS, PHASE); mkdirSync(shots, { recursive: true });
  const admin = await playwright.request.newContext({ baseURL: baseURL ?? '' });
  const fixture = await proxyFixture();
  const id = `t55-${info.project.name}-${Date.now()}`; let previous: string | undefined; const restore: (() => Promise<void>)[] = [];
  const stats: string[] = []; page.on('request', (r) => { if (r.url().includes('/api/rc/stats')) stats.push(new URL(r.url()).searchParams.get('emailId') ?? ''); });
  const report: Record<string, unknown> = { phase: PHASE, project: info.project.name };
  try {
    await adminLogin(admin);
    previous = ((await (await admin.get(`${adminPath}/egress/proxies`)).json()) as { proxies: { id: string; active: boolean }[] }).proxies.find((p) => p.active)?.id;
    const created = await admin.post(`${adminPath}/egress/proxies`, { data: { id, scheme: 'http', host: '127.0.0.1', port: fixture.port, username: '', allowPlaintext: true, trafficClasses: ['remote_content'], scope: 'global' } });
    expect(created.ok(), await created.text()).toBe(true);
    expect((await admin.post(`${adminPath}/egress/proxies/${id}/activate`)).ok()).toBe(true);

    const run = `${info.project.name}-${Date.now()}`;
    const plain = `T55 plain ${run}`;
    await seed(plain, `plain-${run}@example.test`, '<p>Just text, nothing remote.</p>');
    const combos = PHASE === 'before' ? [[1440, 'dark'], [390, 'light']] as const : [[1440, 'dark'], [1440, 'light'], [390, 'dark'], [390, 'light']] as const;
    const subjects = combos.map(([w, theme]) => ({ w, theme, subject: `T55 images ${w} ${theme} ${run}`, from: `news-${w}-${theme}-${run}@example.test` }));
    for (const s of subjects) await seed(s.subject, s.from, imagesHtml(`${s.w}${s.theme}`));
    const proxySubject = `T55 proxy ${run}`; await seed(proxySubject, `proxy-${run}@example.test`, imagesHtml('proxy'));
    const toastOffSubject = `T55 toast off ${run}`; await seed(toastOffSubject, `off-${run}@example.test`, imagesHtml('off'));

    await page.waitForTimeout(3100); // sign-in cooldown
    await signIn(page);
    await userSetting(page, 'privacy.remoteContent', 'ask');
    await userSetting(page, 'reader.colorMode', 'original');
    await page.reload(); await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 30000 });

    for (const s of subjects) {
      const tag = `${info.project.name}-${s.w}-${s.theme}`;
      await page.setViewportSize({ width: s.w, height: s.w < 761 ? 844 : 900 });
      await page.emulateMedia({ colorScheme: s.theme, reducedMotion: 'reduce' });
      await page.evaluate((t) => { document.documentElement.setAttribute('data-theme', t); document.documentElement.setAttribute('data-appearance', t); }, s.theme);

      await open(page, plain);
      await expect(page.getByTestId('remote-content-bar').filter({ visible: true })).toHaveCount(0);

      await open(page, s.subject);
      const bar = page.getByTestId('remote-content-bar').filter({ visible: true });
      await expect(bar).toBeVisible({ timeout: 20000 });
      await page.waitForTimeout(500);
      await bar.screenshot({ path: join(shots, `${tag}-banner-blocked.png`) });
      await page.screenshot({ path: join(shots, `${tag}-reader-blocked.png`) });
      if (PHASE === 'before') {
        report[`${tag}-height`] = (await bar.boundingBox())?.height;
        continue;
      }
      const geometry = await oneLine(page); report[`${tag}-blocked`] = geometry;
      expect(geometry.summaryLines, 'summary is one line').toBe(1);
      expect(geometry.tops, 'all row parts on one line').toBeLessThan(6);
      expect(geometry.height, 'banner stays compact').toBeLessThanOrEqual(40);
      expect(geometry.rowOverflow).toBeLessThanOrEqual(1);
      expect(geometry.overflow).toBeLessThanOrEqual(1);
      await expect(bar.getByRole('status')).toHaveText(/^Remote images blocked( · \d+ trackers? removed)?$/);
      // Focus did not move into the banner when the message opened (t51).
      expect(await bar.evaluate((n) => n.contains(document.activeElement))).toBe(false);
      if (s.w < 761) {
        // Narrow: icon buttons carry their names and tooltips.
        await expect(bar.getByRole('button', { name: 'Show images' })).toHaveAttribute('data-tooltip', /.+/);
        expect(await bar.getByRole('button', { name: 'Show images' }).evaluate((b) => (b.querySelector('span') as HTMLElement).getBoundingClientRect().width)).toBeLessThanOrEqual(1);
      }

      // Show images: one toast, banner + body do not move, few stats reads.
      const email = await page.evaluate(() => location.href);
      const before = stats.length;
      await startProbe(page);
      await bar.getByRole('button', { name: 'Show images' }).click();
      const toast = page.getByTestId('remote-images-toast');
      await expect(toast).toContainText('Loading images…', { timeout: 5000 });
      await page.screenshot({ path: join(shots, `${tag}-loading-toast.png`) });
      await expect(toast).toContainText(/Images loaded|couldn't load/, { timeout: 30000 });
      await expect(bar.getByRole('status')).toHaveText(/^Images shown/);
      await page.waitForTimeout(2500);
      const probe = await stopProbe(page); report[`${tag}-show-probe`] = { heights: distinct(probe.heights), tops: distinct(probe.tops), texts: probe.texts, maxToasts: Math.max(...probe.toasts), statsReads: stats.length - before, email };
      expect(distinct(probe.heights), 'banner height constant while images load').toHaveLength(1);
      expect(distinct(probe.tops), 'body top constant while images load').toHaveLength(1);
      expect(Math.max(...probe.toasts), 'one toast at a time').toBeLessThanOrEqual(1);
      expect(probe.texts.some((t) => /\b0 tracker/.test(t))).toBe(false);
      expect(stats.length - before, 'stats reads for the shown message').toBeLessThanOrEqual(2);
      await bar.screenshot({ path: join(shots, `${tag}-banner-shown.png`) });
      await page.screenshot({ path: join(shots, `${tag}-reader-shown.png`) });
      expect((await oneLine(page)).summaryLines).toBe(1);

      // Undo -> blocked again.
      await bar.getByRole('button', { name: /^Undo/ }).click();
      await expect(bar.getByRole('status')).toHaveText(/^Remote images blocked/, { timeout: 20000 });
      // Always for this sender (menu) -> shown, Undo names the sender scope.
      await bar.getByRole('button', { name: 'More image options' }).click();
      await page.screenshot({ path: join(shots, `${tag}-menu.png`) });
      await page.getByRole('menuitem', { name: new RegExp(`Always show images from .${s.from.replace(/[.]/g, '\\.')}.`) }).click();
      await expect(bar.getByRole('status')).toHaveText(/^Images shown/, { timeout: 20000 });
      await expect(bar.getByRole('button', { name: /^Undo: Stop showing images from/ })).toBeVisible();
      await bar.getByRole('button', { name: /^Undo/ }).click();
      await expect(bar.getByRole('status')).toHaveText(/^Remote images blocked/, { timeout: 20000 });

      // Leave the message: no more stats reads for it.
      await open(page, plain);
      const left = stats.length; await page.waitForTimeout(7000);
      expect(stats.length - left, 'no stats reads after leaving').toBe(0);
    }

    // Proxy mode: images load on open; banner text settles, never "0", body does not jump.
    await page.setViewportSize({ width: 1440, height: 900 });
    await userSetting(page, 'privacy.remoteContent', 'proxy');
    await page.reload(); await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 30000 });
    await startProbe(page);
    const proxyBefore = stats.length;
    await open(page, proxySubject);
    await page.waitForTimeout(12000);
    const proxyProbe = await stopProbe(page);
    report['proxy-probe'] = { heights: distinct(proxyProbe.heights), tops: distinct(proxyProbe.tops), texts: proxyProbe.texts, decodedLineToggles: proxyProbe.decoded.length - 1, statsReads: stats.length - proxyBefore };
    await page.screenshot({ path: join(shots, `${info.project.name}-proxy-after-12s.png`) });
    if (PHASE === 'after') {
      expect(proxyProbe.texts.filter((t) => t !== '').length, 'banner text changes at most twice').toBeLessThanOrEqual(2);
      expect(proxyProbe.texts.some((t) => /\b0 tracker|recently decoded/.test(t))).toBe(false);
      expect(distinct(proxyProbe.tops).length, 'body top settles once').toBeLessThanOrEqual(2);
      expect(stats.length - proxyBefore, 'stats reads in proxy mode').toBeLessThanOrEqual(2);
      await expect(page.getByText(/recently decoded/)).toHaveCount(0);
    }
    await userSetting(page, 'privacy.remoteContent', 'ask');

    if (PHASE === 'after') {
      // Admin turns the toast off (locked): Show images raises no toast.
      restore.push(await setGlobal(admin, 'privacy.remoteImagesToast', false, true));
      await page.reload(); await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 30000 });
      await open(page, toastOffSubject);
      const bar = page.getByTestId('remote-content-bar').filter({ visible: true });
      await bar.getByRole('button', { name: 'Show images' }).click();
      await expect(bar.getByRole('status')).toHaveText(/^Images shown/, { timeout: 30000 });
      await page.waitForTimeout(3000);
      await expect(page.getByTestId('remote-images-toast')).toHaveCount(0);
      report['toast-off'] = 'no toast';
    }
  } finally {
    await info.attach('t55-report', { body: JSON.stringify(report, null, 2), contentType: 'application/json' });
    console.log('T55-REPORT', JSON.stringify(report));
    for (const r of restore.reverse()) await r().catch(() => undefined);
    if (previous) await admin.post(`${adminPath}/egress/proxies/${previous}/activate`).catch(() => undefined);
    await admin.delete(`${adminPath}/egress/proxies/${id}`).catch(() => undefined);
    await admin.dispose(); await fixture.close();
  }
});
