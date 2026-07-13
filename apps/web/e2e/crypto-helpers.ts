import { expect, type APIRequestContext, type Page } from '@playwright/test';
import net from 'node:net';
import { engineLogin, ENGINE_CREDS, messageRow, sidebarInbox } from './helpers.ts';

/**
 * Shared helpers for the V4 crypto/security live E2E (plan §3 e10). These specs
 * drive the REAL crypto UI (key management, security panel, compose crypto + DLP,
 * max-security switch, decrypt-on-receipt) against the engine-mode server (:8090),
 * backed by the REAL WASM crypto worker (mw-crypto + mw-sanitize, embedded in the
 * runtime image by scripts/build-wasm) and the real engine security surface (e6).
 *
 * Login is the same engine-mode login the V1 `imap-engine` + V3 `pim-*` specs use
 * (the server dials Greenmail; the browser only ever talks to :8090). Re-export it
 * so the crypto specs import from a single module.
 */
export { engineLogin, ENGINE_CREDS, messageRow, sidebarInbox } from './helpers.ts';

/** JMAP capability URNs (mirrors apps/web/src/api). */
export const CAP_CORE = 'urn:ietf:params:jmap:core';
export const CAP_CRYPTO = 'urn:mailwoman:crypto';
export const CAP_SECURITY = 'urn:mailwoman:security';

/** A short unique token so assertions are immune to leftover/looped-back mail. */
export function uid(): string {
  return `${Date.now().toString(36)}${Math.random().toString(36).slice(2, 6)}`;
}

/**
 * Navigate to the key-management module via its nav-rail entry and wait until the
 * module root (`[data-module="keys"]` inside `main.module-pane[data-surface="keys"]`)
 * is visible — the lazy chunk resolves on first activation (same pattern as PIM).
 */
export async function gotoKeys(page: Page): Promise<void> {
  await page.getByTestId('nav-keys').click();
  await expect(page.locator('main.module-pane[data-surface="keys"]')).toBeVisible();
  await expect(page.locator('[data-module="keys"]')).toBeVisible();
}

/**
 * Generate an OpenPGP key through the real UI + real WASM worker. Fills the
 * generate dialog (Type=OpenPGP by default) and waits for the new key to appear
 * under "Your keys". Returns nothing — assertions live in the caller. The key's
 * address MUST match the recipient for the encrypt round-trip (self-send), so the
 * caller passes the account's own address.
 */
export async function generatePgpKey(
  page: Page,
  opts: { email: string; passphrase: string; name?: string },
): Promise<void> {
  await page.getByRole('button', { name: 'Generate key', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Generate a key' });
  await expect(dialog).toBeVisible();
  // Type defaults to OpenPGP; set it explicitly for clarity.
  await dialog.getByLabel('Key type').selectOption('pgp');
  if (opts.name !== undefined) await dialog.getByLabel('Name', { exact: true }).fill(opts.name);
  await dialog.getByLabel('Email', { exact: true }).fill(opts.email);
  await dialog.getByLabel('Key passphrase', { exact: true }).fill(opts.passphrase);
  await dialog.getByRole('button', { name: 'Generate', exact: true }).click();
  // Real WASM keygen + CryptoKey/set; the dialog closes and the key row appears.
  await expect(dialog).toBeHidden({ timeout: 30_000 });
  await expect(
    page.getByRole('list', { name: 'Your keys' }).getByText(opts.email),
  ).toBeVisible({ timeout: 30_000 });
}

// ── Raw SMTP injection (arbitrary headers/body) ──────────────────────────────
//
// helpers.injectViaSmtp only sends text/plain bodies; the crypto specs need
// custom headers (a DKIM-Signature) and HTML / multipart/alternative bodies. This
// speaks the same minimal SMTP dialog against Greenmail's plaintext listener but
// takes a fully-formed RFC 822 message (headers + blank line + body).

/** Pull the bare address out of `Name <addr>` or a bare `addr`. */
function extractAddr(input: string): string {
  const m = input.match(/<([^>]+)>/);
  return m ? m[1]! : input.trim();
}

/** Deliver a complete raw RFC-822 message straight into the Greenmail account. */
export async function sendRawSmtp(opts: {
  from: string;
  to?: string;
  /** The full message: headers, a blank line, then the body. */
  message: string;
}): Promise<void> {
  const host = process.env['MW_E2E_SMTP_HOST'] ?? '127.0.0.1';
  const port = Number(process.env['MW_E2E_SMTP_PORT'] ?? 3025);
  const to = opts.to ?? ENGINE_CREDS.selfAddress;
  const steps = [
    `EHLO mailwoman-e2e`,
    `MAIL FROM:<${extractAddr(opts.from)}>`,
    `RCPT TO:<${to}>`,
    `DATA`,
    `${opts.message}\r\n.`,
    `QUIT`,
  ];
  await new Promise<void>((resolve, reject) => {
    const sock = net.createConnection({ host, port });
    let step = -1; // -1 = waiting for the 220 greeting
    let buf = '';
    const fail = (e: Error) => {
      sock.destroy();
      reject(e);
    };
    sock.setTimeout(15_000, () => fail(new Error('SMTP inject timed out')));
    sock.on('error', fail);
    sock.on('data', (chunk) => {
      buf += chunk.toString('utf8');
      const lines = buf.split('\r\n').filter((l) => /^\d{3} /.test(l));
      if (lines.length === 0) return;
      buf = '';
      const code = Number(lines[lines.length - 1]!.slice(0, 3));
      if (code >= 400) return fail(new Error(`SMTP error: ${lines[lines.length - 1]}`));
      step += 1;
      if (step >= steps.length) {
        sock.end();
        return resolve();
      }
      sock.write(steps[step]! + '\r\n');
    });
  });
}

/** Build + inject a multipart/alternative (text/plain + text/html) message. */
export async function injectHtmlViaSmtp(opts: {
  from: string;
  subject: string;
  text: string;
  html: string;
  to?: string;
  /** Extra header lines (e.g. a DKIM-Signature), each without a trailing CRLF. */
  extraHeaders?: string[];
}): Promise<void> {
  const to = opts.to ?? ENGINE_CREDS.selfAddress;
  const bound = `mwalt${Date.now()}`;
  const headers = [
    `From: ${opts.from}`,
    `To: ${to}`,
    `Subject: ${opts.subject}`,
    `Date: ${new Date().toUTCString()}`,
    `MIME-Version: 1.0`,
    ...(opts.extraHeaders ?? []),
    `Content-Type: multipart/alternative; boundary="${bound}"`,
  ].join('\r\n');
  const body =
    `--${bound}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n${opts.text}\r\n` +
    `--${bound}\r\nContent-Type: text/html; charset=utf-8\r\n\r\n${opts.html}\r\n` +
    `--${bound}--\r\n`;
  await sendRawSmtp({ from: opts.from, to, message: `${headers}\r\n\r\n${body}` });
}

// ── JMAP over the browser session (for MailRule/get assertions) ──────────────
//
// SenderControl "block" must materialize a REAL MailRule/Sieve row (plan §1.9),
// not localStorage. We assert that by querying `MailRule/get` over the SAME
// cookie-authed session the UI uses. `page.request` shares the page's cookies.

interface JmapResponse {
  methodResponses: [string, Record<string, unknown>, string][];
}

/** The account id the crypto surface uses (crypto primary, or the first account). */
export async function cryptoAccountId(request: APIRequestContext): Promise<string> {
  const res = await request.get('/jmap/session');
  expect(res.ok()).toBeTruthy();
  const session = (await res.json()) as {
    primaryAccounts: Record<string, string>;
    accounts: Record<string, unknown>;
  };
  return session.primaryAccounts[CAP_CRYPTO] ?? Object.keys(session.accounts)[0]!;
}

/** POST a JMAP request over the browser session; returns the named method result. */
export async function jmapCall<T = Record<string, unknown>>(
  request: APIRequestContext,
  methodCalls: [string, Record<string, unknown>, string][],
  callId: string,
): Promise<T> {
  const res = await request.post('/jmap/api', {
    headers: { 'content-type': 'application/json' },
    data: { using: [CAP_CORE, CAP_CRYPTO, CAP_SECURITY], methodCalls },
  });
  expect(res.ok()).toBeTruthy();
  const body = (await res.json()) as JmapResponse;
  const found = body.methodResponses.find((r) => r[2] === callId);
  if (found === undefined) throw new Error(`no method response for callId ${callId}`);
  return found[1] as T;
}

/** Poll `MailRule/get` until a rule referencing `address` exists (or time out). */
export async function waitForMailRule(
  request: APIRequestContext,
  accountId: string,
  address: string,
  timeout = 15_000,
): Promise<Record<string, unknown>> {
  let last: Record<string, unknown>[] = [];
  await expect(async () => {
    const got = await jmapCall<{ list: Record<string, unknown>[] }>(
      request,
      [['MailRule/get', { accountId }, 'mr']],
      'mr',
    );
    last = got.list ?? [];
    const hit = last.find((r) => JSON.stringify(r).includes(address));
    expect(hit, `a MailRule referencing ${address}`).toBeTruthy();
  }).toPass({ timeout, intervals: [500, 1000, 2000] });
  return last.find((r) => JSON.stringify(r).includes(address))!;
}

/** Poll the Inbox until a message with `subject` shows (engine sync is poll-based). */
export async function waitForInboxMessage(page: Page, subject: string, timeout = 60_000): Promise<void> {
  await expect(async () => {
    await sidebarInbox(page).click();
    await expect(messageRow(page, subject)).toBeVisible({ timeout: 3_000 });
  }).toPass({ timeout });
}

/** Open a message from the Inbox list by (a substring of) its subject. */
export async function openMessage(page: Page, subject: string): Promise<void> {
  await sidebarInbox(page).click();
  await messageRow(page, subject).first().click();
  await expect(page.locator('.reader')).toBeVisible();
}
