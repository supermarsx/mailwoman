#!/usr/bin/env node
/* global process, console, fetch, setTimeout */
// Fixture for receipts/linked-renew-live.spec.ts (t48): a native engine-mode
// server built from the current tree over a private Greenmail that CHECKS
// passwords (so a changed password is refused), with mailbox linking on.
//
//   cargo build -p mw-server --bin mailwoman
//   pnpm --dir apps/web build
//   node apps/web/e2e/receipts/linked-renew-fixture.mjs
//   # in another shell, with the values it prints:
//   MW_RECEIPTS_BASE_URL=... MW_T48_DB=... MW_T48_GREENMAIL_API=... npx playwright test \
//     -c apps/web/e2e/receipts/playwright.config.ts linked-renew-live --project chromium --project firefox
//
// Environment (all optional): MW_BIN, MW_WEB_DIR, MW_FIXTURE_PORT (default 46280),
// MW_FIXTURE_IMAP (46743), MW_FIXTURE_API (46780).
// Ctrl-C stops the server it started and removes its Greenmail container.

import { spawn, execFileSync } from 'node:child_process';
import { mkdtempSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve, dirname } from 'node:path';
import { randomBytes } from 'node:crypto';
import { fileURLToPath } from 'node:url';

const repo = resolve(dirname(fileURLToPath(import.meta.url)), '../../../..');
const exe = process.platform === 'win32' ? 'mailwoman.exe' : 'mailwoman';
const bin = process.env.MW_BIN ?? join(process.env.CARGO_TARGET_DIR ?? join(repo, 'target'), 'debug', exe);
const web = process.env.MW_WEB_DIR ?? join(repo, 'apps/web/dist');
const port = Number(process.env.MW_FIXTURE_PORT ?? 46280);
const imap = Number(process.env.MW_FIXTURE_IMAP ?? 46743);
const api = Number(process.env.MW_FIXTURE_API ?? 46780);
const container = `mw-linked-renew-greenmail-${port}`;
const admin = { username: 'root', password: 'linked-renew-admin' };
const base = `http://127.0.0.1:${port}`;

for (const [what, path] of [['server binary', bin], ['web build', join(web, 'index.html')]]) {
  if (!existsSync(path)) {
    console.error(`missing ${what}: ${path}`);
    process.exit(2);
  }
}
function docker(...args) {
  return execFileSync('docker', args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
}
try { docker('rm', '-f', container); } catch { /* not there */ }
docker('run', '-d', '--name', container,
  '-p', `127.0.0.1:${imap}:3143`, '-p', `127.0.0.1:${api}:8080`,
  '-e', 'GREENMAIL_OPTS=-Dgreenmail.setup.test.all -Dgreenmail.hostname=0.0.0.0 -Dgreenmail.users.login=email -Dgreenmail.users=parent:parent-pass@example.org,child:child-pass@child.test',
  'greenmail/standalone:2.1.4');

const data = mkdtempSync(join(tmpdir(), 'mw-linked-renew-'));
const db = join(data, 'mw.db');
const server = spawn(bin, ['serve'], {
  stdio: ['ignore', 'inherit', 'inherit'],
  env: {
    ...process.env,
    MW_MODE: 'engine',
    MW_BIND: `127.0.0.1:${port}`,
    MW_DB_PATH: db,
    MW_SERVER_KEY: randomBytes(32).toString('hex'),
    MW_WEB_DIR: web,
    MW_UPSTREAM_DEFAULT: `imap://127.0.0.1:${imap}`,
    MW_ENGINE_TLS: 'plaintext',
    MW_ADMIN_USER: admin.username,
    MW_ADMIN_PASSWORD: admin.password,
    RUST_LOG: process.env.RUST_LOG ?? 'warn',
  },
});
let stopping = false;
function stop(code = 0) {
  if (stopping) return;
  stopping = true;
  try { server.kill(); } catch { /* gone */ }
  try { docker('rm', '-f', container); } catch { /* gone */ }
  process.exit(code);
}
process.on('SIGINT', () => stop(0));
process.on('SIGTERM', () => stop(0));
server.on('exit', (code) => { if (!stopping) { console.error(`server exited (${code})`); stop(1); } });

async function waitFor(check, what, ms = 120_000) {
  const end = Date.now() + ms;
  for (;;) {
    try { if (await check()) return; } catch { /* not yet */ }
    if (Date.now() > end) { console.error(`timed out waiting for ${what}`); stop(1); }
    await new Promise((r) => setTimeout(r, 500));
  }
}
await waitFor(async () => (await fetch(`${base}/`)).ok, 'the server');
await waitFor(async () => (await fetch(`http://127.0.0.1:${api}/api/user`)).ok, 'greenmail');

const login = await fetch(`${base}/admin/login`, {
  method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(admin),
});
if (!login.ok) { console.error(`admin login: ${login.status} ${await login.text()}`); stop(1); }
const cookie = login.headers.getSetCookie().map((c) => c.split(';')[0]).join('; ');
const csrf = (await login.clone().json().catch(() => ({}))).csrfToken;
for (const [key, value] of [['accounts.linkingEnabled', true], ['signin.retryCooldownSecs', 0]]) {
  const put = await fetch(`${base}/admin/settings/${key}`, {
    method: 'PUT',
    headers: { 'content-type': 'application/json', cookie, ...(csrf ? { 'x-csrf-token': csrf } : {}) },
    body: JSON.stringify({ globalValue: value }),
  });
  if (!put.ok) { console.error(`PUT ${key}: ${put.status} ${await put.text()}`); stop(1); }
}
console.log(`READY MW_RECEIPTS_BASE_URL=${base} MW_T48_DB=${db} MW_T48_GREENMAIL_API=http://127.0.0.1:${api} (server pid ${server.pid})`);
