// t96: Security > Country database against a live server built from the tree.
// For every leg (chromium and firefox, 1440 and 390 wide) the spec starts its own
// mailwoman server on a fresh database, so each leg begins with no country
// database: the console shows the fail-closed warning and a mailbox sign-in under
// the active country policy is refused (503). "Download now" then fetches a
// synthetic DB-IP-style file from a local mock provider, the status turns to
// Loaded, the test lookup answers, and the same sign-in succeeds.
//
// The mock provider is an HTTPS origin whose certificate is issued for 8.8.8.8 by
// a CA made here with openssl; the server reaches https://8.8.8.8/ through an
// HTTP CONNECT egress route for the geoip class (the address policy is exercised,
// as in the Nextcloud live spec), and the CA is the custom source's extra CA.
//
// Needs the receipts stack (Greenmail for the mailbox sign-in) and:
//   MW_T96_BIN    the mailwoman binary built from the tree
//   MW_WEB_DIR    the built web app (apps/web/dist)
//   MW_T96_SHOTS  optional screenshot directory
import { test, expect } from '../csrf.ts';
import type { APIRequestContext, Page, TestInfo } from '@playwright/test';
import { spawn, execFileSync, type ChildProcess } from 'node:child_process';
import { createServer as httpsServer, type Server as HttpsServer } from 'node:https';
import { createServer as httpServer, type Server as HttpServer } from 'node:http';
import { connect, createServer as netServer } from 'node:net';
import { mkdtempSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { randomBytes } from 'node:crypto';
import { gzipSync } from 'node:zlib';

const bin = process.env['MW_T96_BIN'];
const web = process.env['MW_WEB_DIR'];
const imapPort = process.env['MW_E2E_IMAP_PORT'];
const shots = process.env['MW_T96_SHOTS'];
const ADMIN = '/admin';
test.describe.configure({ timeout: 420000 });

// ── a tiny MaxMind DB writer (the same format as mw_engine's geoip_fixture) ──
function control(out: number[], kind: number, size: number): void {
  const [first, extra] = size < 29 ? [size, [] as number[]] : size < 285 ? [29, [size - 29]] : [30, [((size - 285) >> 8) & 255, (size - 285) & 255]];
  if (kind <= 7) out.push((kind << 5) | first); else out.push(first, kind - 7);
  out.push(...extra);
}
function str(out: number[], s: string): void { const b = Buffer.from(s, 'utf8'); control(out, 2, b.length); out.push(...b); }
function uint(out: number[], kind: number, v: bigint): void {
  const bytes: number[] = []; let x = v; while (x > 0n) { bytes.unshift(Number(x & 255n)); x >>= 8n; }
  control(out, kind, bytes.length); out.push(...bytes);
}
function mmdb(entries: { bits: bigint; prefix: number; country: string }[]): Buffer {
  type Rec = { t: 'e' } | { t: 'n'; i: number } | { t: 'd'; off: number };
  const nodes: [Rec, Rec][] = [[{ t: 'e' }, { t: 'e' }]];
  const data: number[] = [];
  for (const e of entries) {
    const off = data.length;
    control(data, 7, 1); str(data, 'country'); control(data, 7, 1); str(data, 'iso_code'); str(data, e.country);
    let node = 0;
    for (let i = 0; i < e.prefix; i++) {
      const bit = Number((e.bits >> BigInt(127 - i)) & 1n);
      if (i + 1 === e.prefix) { nodes[node]![bit] = { t: 'd', off }; break; }
      const r: Rec = nodes[node]![bit]!;
      if (r.t === 'n') { node = r.i; continue; }
      nodes.push([{ t: 'e' }, { t: 'e' }]); nodes[node]![bit] = { t: 'n', i: nodes.length - 1 }; node = nodes.length - 1;
    }
  }
  const n = nodes.length; const tree = Buffer.alloc(n * 8);
  nodes.forEach((pair, i) => pair.forEach((r, j) => tree.writeUInt32BE(r.t === 'e' ? n : r.t === 'n' ? r.i : n + 16 + r.off, i * 8 + j * 4)));
  const meta: number[] = [];
  control(meta, 7, 9);
  str(meta, 'binary_format_major_version'); uint(meta, 5, 2n); str(meta, 'binary_format_minor_version'); uint(meta, 5, 0n);
  str(meta, 'build_epoch'); uint(meta, 9, BigInt(Math.floor(Date.now() / 1000))); str(meta, 'database_type'); str(meta, 'DBIP-Country-Lite-Test');
  str(meta, 'description'); control(meta, 7, 1); str(meta, 'en'); str(meta, 'Synthetic test database');
  str(meta, 'ip_version'); uint(meta, 5, 6n); str(meta, 'languages'); control(meta, 11, 1); str(meta, 'en');
  str(meta, 'node_count'); uint(meta, 6, BigInt(n)); str(meta, 'record_size'); uint(meta, 5, 32n);
  return Buffer.concat([tree, Buffer.alloc(16), Buffer.from(data), Buffer.from([0xab, 0xcd, 0xef]), Buffer.from('MaxMind.com'), Buffer.from(meta)]);
}
const v4 = (cidr: string, country: string) => { const [a, l] = cidr.split('/') as [string, string]; const n = a.split('.').reduce((acc, o) => (acc << 8n) | BigInt(Number(o)), 0n); return { bits: n, prefix: 96 + Number(l), country }; };
// The browser and the sign-in come from loopback: 127.0.0.0/8 and ::1 are PT here.
const database = gzipSync(mmdb([v4('127.0.0.0/8', 'PT'), { bits: 1n, prefix: 128, country: 'PT' }, v4('81.2.69.0/24', 'GB'), v4('89.114.0.0/16', 'PT')]));

// ── the mock provider: TLS origin for 8.8.8.8 and a CONNECT proxy to it ──────
type Provider = { ca: string; proxyPort: number; hits: string[]; pause: () => () => void; close: () => Promise<void> };
async function startProvider(): Promise<Provider> {
  const dir = mkdtempSync(join(tmpdir(), 'mw-t96-ca-'));
  const ssl = (...args: string[]) => execFileSync('openssl', args, { cwd: dir, stdio: 'pipe' });
  ssl('req', '-x509', '-newkey', 'ec', '-pkeyopt', 'ec_paramgen_curve:P-256', '-nodes', '-keyout', 'ca.key', '-out', 'ca.pem', '-days', '2', '-subj', '/CN=t96 test CA',
    '-addext', 'basicConstraints=critical,CA:TRUE', '-addext', 'keyUsage=critical,keyCertSign,cRLSign');
  ssl('req', '-newkey', 'ec', '-pkeyopt', 'ec_paramgen_curve:P-256', '-nodes', '-keyout', 'leaf.key', '-out', 'leaf.csr', '-subj', '/CN=8.8.8.8');
  writeFileSync(join(dir, 'ext.cnf'), 'subjectAltName=IP:8.8.8.8\nbasicConstraints=CA:FALSE\nextendedKeyUsage=serverAuth\nkeyUsage=digitalSignature\n');
  ssl('x509', '-req', '-in', 'leaf.csr', '-CA', 'ca.pem', '-CAkey', 'ca.key', '-CAcreateserial', '-out', 'leaf.pem', '-days', '2', '-extfile', 'ext.cnf');
  const hits: string[] = [];
  let held: Promise<void> | undefined;
  const origin: HttpsServer = httpsServer({ key: readFileSync(join(dir, 'leaf.key')), cert: readFileSync(join(dir, 'leaf.pem')) }, (req, res) => {
    hits.push(req.url ?? '');
    if (req.url === '/dbip-country-lite.mmdb.gz') {
      res.writeHead(200, { 'content-type': 'application/gzip', 'content-length': database.length });
      const waiting = held; held = undefined;
      if (waiting) {
        const half = Math.floor(database.length / 2); res.write(database.subarray(0, half));
        void waiting.then(() => res.end(database.subarray(half)));
      } else res.end(database);
      return;
    }
    res.writeHead(404); res.end();
  });
  await new Promise<void>(done => origin.listen(0, '127.0.0.1', done));
  const originPort = (origin.address() as { port: number }).port;
  const proxy: HttpServer = httpServer();
  const sockets = new Set<import('node:net').Socket>();
  proxy.on('connect', (req, socket, head) => {
    sockets.add(socket as import('node:net').Socket);
    if (req.url !== '8.8.8.8:443') { socket.end('HTTP/1.1 403 Forbidden\r\n\r\n'); return; }
    const upstream = connect(originPort, '127.0.0.1', () => { socket.write('HTTP/1.1 200 Connection Established\r\n\r\n'); if (head.length) upstream.write(head); socket.pipe(upstream); upstream.pipe(socket); });
    sockets.add(upstream);
    upstream.on('error', () => socket.destroy()); socket.on('error', () => upstream.destroy());
  });
  await new Promise<void>(done => proxy.listen(0, '127.0.0.1', done));
  const ca = readFileSync(join(dir, 'ca.pem'), 'utf8');
  return {
    pause: () => { let release!: () => void; held = new Promise<void>(resolve => { release = resolve; }); return release; },
    ca, hits, proxyPort: (proxy.address() as { port: number }).port,
    close: async () => { for (const s of sockets) s.destroy(); await Promise.all([new Promise(d => proxy.close(() => d(null))), new Promise(d => origin.close(() => d(null)))]); rmSync(dir, { recursive: true, force: true }); },
  };
}

// ── one fresh server per leg ─────────────────────────────────────────────────
async function freePort(): Promise<number> {
  const s = netServer(); await new Promise<void>(d => s.listen(0, '127.0.0.1', d));
  const port = (s.address() as { port: number }).port; await new Promise(d => s.close(() => d(null))); return port;
}
type Server = { base: string; proc: ChildProcess; data: string; stop: () => Promise<void> };
async function startServer(): Promise<Server> {
  const port = await freePort();
  const data = mkdtempSync(join(tmpdir(), 'mw-t96-server-'));
  const base = `http://127.0.0.1:${port}`;
  const proc = spawn(bin!, ['serve'], {
    stdio: ['ignore', 'pipe', 'pipe'],
    env: {
      ...process.env, MW_MODE: 'engine', MW_BIND: `127.0.0.1:${port}`, MW_DB_PATH: join(data, 'mw.db'), MW_UPLOAD_DIR: join(data, 'uploads'),
      MW_SERVER_KEY: randomBytes(32).toString('hex'), MW_WEB_DIR: web!, MW_UPSTREAM_DEFAULT: `imap://127.0.0.1:${imapPort}`, MW_ENGINE_TLS: 'plaintext',
      MW_COOKIE_SECURE: 'false', MW_CSRF_STRICT: 'true', MW_ADMIN_USER: 'root', MW_ADMIN_PASSWORD: 'hunter2', MW_ADMIN_PATH: ADMIN,
      MW_PUBLIC_URL: base, RUST_LOG: 'warn', MW_GEOIP_DB: '', MW_GEOIP_DIR: '', MW_GEOIP_SEED: '',
    },
  });
  let log = ''; proc.stderr?.on('data', c => { log += String(c); }); proc.stdout?.on('data', c => { log += String(c); });
  const deadline = Date.now() + 60000;
  for (;;) {
    try { if ((await fetch(`${base}/healthz`)).ok) break; } catch { /* starting */ }
    if (Date.now() > deadline || proc.exitCode !== null) throw new Error(`server did not start: ${log.slice(-2000)}`);
    await new Promise(r => setTimeout(r, 250));
  }
  return { base, proc, data, stop: async () => { proc.kill(); await new Promise(r => proc.once('exit', r)); rmSync(data, { recursive: true, force: true }); } };
}

async function admin(playwright: { request: { newContext: (o: { baseURL: string }) => Promise<APIRequestContext> } }, base: string): Promise<APIRequestContext> {
  const api = await playwright.request.newContext({ baseURL: base });
  expect((await api.post(`${ADMIN}/login`, { data: { username: 'root', password: 'hunter2' } })).status()).toBe(200);
  expect((await api.post(`${ADMIN}/step-up`, { data: { password: 'hunter2' } })).status()).toBe(200);
  return api;
}
async function put(api: APIRequestContext, key: string, value: unknown): Promise<void> {
  const etag = (await api.get(`${ADMIN}/settings/${key}`)).headers()['etag'];
  const r = await api.put(`${ADMIN}/settings/${key}`, { data: { globalValue: value }, headers: etag ? { 'if-match': etag } : {} });
  expect(r.ok(), `${key}: ${await r.text()}`).toBe(true);
}
async function consoleSignIn(page: Page, base: string): Promise<void> {
  await page.goto(`${base}${ADMIN}`);
  await page.getByLabel('Admin username').fill('root'); await page.getByLabel('Password', { exact: true }).fill('hunter2');
  await page.waitForTimeout(3100);
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByLabel('Admin username')).toHaveCount(0, { timeout: 30000 });
}
const shot = (info: TestInfo, name: string) => shots ? join(shots, `${info.project.name}-${name}.png`) : info.outputPath(`${name}.png`);
const noSideScroll = (page: Page) => page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1);

test('country database: missing → Download now → loaded, test lookup, sign-in under an active country policy', async ({ page, playwright }, info) => {
  test.skip(info.project.name === 'phone', 'the 390 legs run inside the chromium and firefox projects');
  test.skip(!bin || !web || !imapPort, 'set MW_T96_BIN and MW_WEB_DIR, and run with the receipts stack env (MW_E2E_IMAP_PORT)');
  const provider = await startProvider();
  try {
    for (const width of [1440, 390]) {
      const server = await startServer();
      try {
        await page.setViewportSize({ width, height: width > 500 ? 900 : 844 });
        const api = await admin(playwright, server.base);
        // The policy: mailbox sign-ins from PT only; unmapped addresses refused.
        await put(api, 'signin.retryCooldownSecs', 0);
        await put(api, 'security.signin.geoCountries', ['PT']);
        await put(api, 'security.signin.geoMode', 'allow');
        await put(api, 'security.signin.geoUnknown', 'deny');
        await put(api, 'security.signin.geoScope', 'mailbox');
        // The egress route for the geoip class and the source (custom, with its CA).
        expect((await api.post(`${ADMIN}/egress/proxies`, { data: { id: 't96', scheme: 'http', host: '127.0.0.1', port: provider.proxyPort, scope: 'global', trafficClasses: ['geoip'] } })).ok()).toBe(true);
        expect((await api.post(`${ADMIN}/egress/proxies/t96/activate`)).ok()).toBe(true);
        const etag = (await api.get(`${ADMIN}/security/geoip`)).headers()['etag'];
        const configured = await api.put(`${ADMIN}/security/geoip/config`, { headers: { 'if-match': etag ?? '' }, data: { provider: 'custom', customUrl: 'https://8.8.8.8/dbip-country-lite.mmdb.gz', customCaPem: provider.ca, autoUpdate: true, intervalDays: 7 } });
        expect(configured.status(), await configured.text()).toBe(200);

        // Fail closed while missing, even though nothing else is wrong.
        const mailbox = await playwright.request.newContext({ baseURL: server.base });
        const refused = await mailbox.post('/api/login', { data: { username: 'testuser', password: 'testpass' } });
        expect(refused.status()).toBe(503);
        expect((await refused.json()).code).toBe('signinGeographyUnavailable');

        await consoleSignIn(page, server.base);
        await page.goto(`${server.base}${ADMIN}#/security/geoip`);
        const region = page.getByRole('region', { name: 'Country database' });
        await expect(region).toBeVisible({ timeout: 30000 });
        const status = page.getByRole('table', { name: 'Country database status' });
        await expect(status).toContainText('Missing');
        await expect(page.getByText(/An active country policy refuses sign-in in this state/)).toBeVisible();
        expect(await noSideScroll(page), 'no horizontal scroll').toBe(true);
        await page.screenshot({ path: shot(info, `${width}-1-missing`), fullPage: true });

        // Download now, in place: the region node survives the action.
        const regionHandle = await region.elementHandle();
        const release = provider.pause();
        let progressId: string | null = null;
        try {
          await page.getByRole('button', { name: 'Download now', exact: true }).click();
          const progress = page.getByTestId('geoip-download-toast');
          await expect(progress).toHaveAttribute('data-busy', 'true');
          await expect(progress).toContainText(/Downloading the country database.* of /, {timeout: 15000});
          await expect(progress).toHaveCount(1);
          expect(await noSideScroll(page), 'progress toast fits the viewport').toBe(true);
          const snapshot = await (await api.get(`${ADMIN}/security/geoip`)).json();
          expect(snapshot.downloadProgress.receivedBytes).toBe(Math.floor(database.length / 2));
          expect(snapshot.downloadProgress.totalBytes).toBe(database.length);
          expect(snapshot.downloadProgress.phase).toBe('downloading');
          await expect(region).not.toContainText('Downloading the country database');
          progressId = await progress.getAttribute('data-toast-id');
          await page.screenshot({path: shot(info, `${width}-download-progress`), fullPage: true, animations: 'disabled'});
        } finally { release(); }
        await expect(page.getByText('The new database is installed and in use.')).toBeVisible({ timeout: 60000 });
        await expect(page.getByTestId('geoip-download-toast')).toHaveAttribute('data-toast-id', progressId!);
        await expect(page.getByTestId('geoip-download-toast')).not.toHaveAttribute('data-busy', 'true');
        await expect(status).toContainText('Loaded');
        await expect(status).toContainText('DBIP-Country-Lite-Test');
        await expect(page.getByText(/An active country policy refuses sign-in/)).toHaveCount(0);
        expect(await regionHandle!.evaluate(el => el.isConnected), 'the page stayed mounted').toBe(true);
        expect(provider.hits).toContain('/dbip-country-lite.mmdb.gz');
        expect(await noSideScroll(page), 'no horizontal scroll').toBe(true);
        await page.screenshot({ path: shot(info, `${width}-2-loaded`), fullPage: true });

        // The test lookup.
        await page.getByLabel('IP address', { exact: true }).fill('81.2.69.160');
        await page.getByRole('button', { name: 'Look up', exact: true }).click();
        await expect(page.getByTestId('geoip-lookup-result')).toHaveText(/81\.2\.69\.160 is in GB\./);
        await page.getByLabel('IP address', { exact: true }).fill('89.114.7.1');
        await page.getByRole('button', { name: 'Look up', exact: true }).click();
        await expect(page.getByTestId('geoip-lookup-result')).toHaveText(/89\.114\.7\.1 is in PT\./);
        await page.getByTestId('geoip-lookup-result').scrollIntoViewIfNeeded();
        await page.screenshot({ path: shot(info, `${width}-3-lookup`), fullPage: true });

        // The mailbox sign-in now succeeds under the same active policy (loopback is PT here).
        const mail = await page.context().browser()!.newContext({ viewport: { width, height: width > 500 ? 900 : 844 } });
        try {
          const m = await mail.newPage();
          await m.goto(`${server.base}/`);
          await m.getByLabel('Email or username', { exact: true }).fill('testuser');
          await m.getByLabel('Password', { exact: true }).fill('testpass');
          await m.getByRole('button', { name: 'Sign in', exact: true }).click();
          await expect(m.getByLabel('Email or username', { exact: true })).toHaveCount(0, { timeout: 60000 });
          await expect(m.locator('.mail-pane')).toBeVisible({ timeout: 60000 });
          await m.screenshot({ path: shot(info, `${width}-4-signed-in`) });
        } finally { await mail.close(); }
        await mailbox.dispose(); await api.dispose();
      } finally { await server.stop(); }
    }
  } finally { await provider.close(); }
});
