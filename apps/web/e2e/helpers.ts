import { expect, type Page } from '@playwright/test';
import net from 'node:net';

/**
 * Credentials the mock JMAP backend accepts. The jmapUrl uses the compose
 * service name `mock` because mw-SERVER (not the browser) dials it; the browser
 * only ever talks to the same-origin proxy. Overridable for non-compose runs.
 */
export const CREDS = {
  jmapUrl: process.env['MW_E2E_JMAP_URL'] ?? 'http://mock:8181/.well-known/jmap',
  username: process.env['MW_E2E_USERNAME'] ?? 'testuser@example.org',
  password: process.env['MW_E2E_PASSWORD'] ?? 'testpass',
} as const;

/**
 * The Inbox mailbox button in the sidebar nav. Scoped to the "Mailboxes"
 * navigation so it matches neither the "Focused inbox" toggle (InboxTabs, which
 * substring-matches "inbox") NOR breaks when an unread badge makes the button's
 * accessible name "Inbox 3" instead of "Inbox".
 */
export function sidebarInbox(page: Page) {
  return page.getByRole('navigation', { name: 'Mailboxes' }).getByRole('button', { name: 'Inbox' });
}

/**
 * Log in through the real UI and wait until the mailbox shell is ready.
 * Each spec calls this so tests stay independent (fresh session per test).
 */
export async function login(page: Page): Promise<void> {
  await page.goto('/');
  // The app boots (checks /api/me) before rendering the login form.
  await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();

  await page.getByLabel('JMAP server URL').fill(CREDS.jmapUrl);
  await page.getByLabel('Username', { exact: true }).fill(CREDS.username);
  await page.getByLabel('Password', { exact: true }).fill(CREDS.password);
  await page.getByRole('button', { name: 'Sign in' }).click();

  // Mailbox shell is up once the sidebar and Inbox mailbox render.
  await expect(page.getByRole('button', { name: 'Compose' })).toBeVisible();
  await expect(sidebarInbox(page)).toBeVisible();
}

/** A message row in the list, located by (a substring of) its subject. */
export function messageRow(page: Page, subject: string) {
  return page.locator('.list__row').filter({ hasText: subject });
}

// ── Engine-mode helpers (shared by the V2 specs that need the real engine) ──
//
// The V2 modern-mail specs drive the SAME unmodified UI against mw-server in
// MW_MODE=engine over a real IMAP/SMTP account (Greenmail) on :8090 — the
// `engine` Playwright project. The server (not the browser) dials Greenmail, so
// the login "JMAP server URL" field carries the in-network `imap://greenmail:3143`
// and Greenmail's login name is the bare local part `testuser`.

export const ENGINE_CREDS = {
  imapUrl: process.env['MW_E2E_ENGINE_IMAP_URL'] ?? 'imap://greenmail:3143',
  username: process.env['MW_E2E_ENGINE_USERNAME'] ?? 'testuser',
  password: process.env['MW_E2E_ENGINE_PASSWORD'] ?? 'testpass',
  /** Full address used as the SMTP RCPT TO so a send loops back to this account. */
  selfAddress: process.env['MW_E2E_ENGINE_SELF'] ?? 'testuser@example.org',
} as const;

/** Log in through the real UI against the engine stack; waits for the shell. */
export async function engineLogin(page: Page): Promise<void> {
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
  await page.getByLabel('JMAP server URL').fill(ENGINE_CREDS.imapUrl);
  await page.getByLabel('Username', { exact: true }).fill(ENGINE_CREDS.username);
  await page.getByLabel('Password', { exact: true }).fill(ENGINE_CREDS.password);
  await page.getByRole('button', { name: 'Sign in' }).click();
  await expect(page.getByRole('button', { name: 'Compose' })).toBeVisible();
  await expect(sidebarInbox(page)).toBeVisible();
}

/** The whole list slot (row button + its action cluster), located by subject. */
export function messageSlot(page: Page, subject: string) {
  return page.locator('.list__slot').filter({ hasText: subject });
}

/**
 * Compose a self-addressed message and click Send. Does NOT wait for delivery —
 * the engine holds the submission for its undo-send window (~10 s) before it
 * dials SMTP, so callers use {@link waitForInboxMessage} to await arrival.
 * Optionally schedules the send for later (fills the datetime-local field).
 */
export async function composeSelf(
  page: Page,
  subject: string,
  body: string,
  opts: { sendLater?: string } = {},
): Promise<void> {
  await page.getByRole('button', { name: 'Compose' }).click();
  const dialog = page.getByRole('dialog', { name: 'Compose message' });
  await expect(dialog).toBeVisible();
  await dialog.getByLabel('To', { exact: true }).fill(ENGINE_CREDS.selfAddress);
  await dialog.getByLabel('Subject', { exact: true }).fill(subject);
  await dialog.getByLabel('Body', { exact: true }).fill(body);
  if (opts.sendLater !== undefined) {
    await dialog.getByLabel('Send later', { exact: true }).fill(opts.sendLater);
    await expect(dialog.getByRole('button', { name: 'Schedule' })).toBeVisible();
    await dialog.getByRole('button', { name: 'Schedule' }).click();
  } else {
    await dialog.getByRole('button', { name: 'Send' }).click();
  }
  await expect(dialog).toBeHidden();
}

/**
 * Poll the Inbox (engine sync is poll-based) until a message with `subject`
 * shows up. Generous timeout: an undo-send hold (~10 s) plus SMTP loopback plus
 * the engine's watch-loop resync must all elapse first.
 */
export async function waitForInboxMessage(page: Page, subject: string, timeout = 45_000): Promise<void> {
  await expect(async () => {
    await sidebarInbox(page).click();
    await expect(messageRow(page, subject)).toBeVisible({ timeout: 3_000 });
  }).toPass({ timeout });
}

/**
 * Deliver a message straight into the Greenmail account over SMTP (host
 * localhost:3025), bypassing the app's compose+undo-hold. Lets a spec inject a
 * NEW delivery from an arbitrary sender (and optionally with an attachment) so
 * realtime-push / search-operator / multi-window specs don't depend on the
 * 10 s undo-send hold. The engine's watch loop ingests it like any other mail.
 */
export interface InjectAttachment {
  filename: string;
  /** MIME type (e.g. application/pdf, image/png, video/mp4). */
  contentType: string;
  /** Raw bytes as a binary string. Provide EITHER this OR `base64`. */
  content?: string;
  /** Pre-encoded base64 payload (for real binary like a PNG). */
  base64?: string;
}

export async function injectViaSmtp(opts: {
  from: string;
  subject: string;
  text: string;
  to?: string;
  /** Shorthand for a single octet-stream attachment. */
  withAttachment?: { filename: string; content: string };
  /** One or more typed attachments (image/pdf/video/…) for the viewer specs. */
  attachments?: InjectAttachment[];
}): Promise<void> {
  const host = process.env['MW_E2E_SMTP_HOST'] ?? '127.0.0.1';
  const port = Number(process.env['MW_E2E_SMTP_PORT'] ?? 3025);
  const to = opts.to ?? ENGINE_CREDS.selfAddress;

  const parts: InjectAttachment[] = opts.attachments ? [...opts.attachments] : [];
  if (opts.withAttachment !== undefined) {
    parts.push({
      filename: opts.withAttachment.filename,
      contentType: 'application/octet-stream',
      content: opts.withAttachment.content,
    });
  }

  let body: string;
  if (parts.length > 0) {
    const bound = `mwbound${Date.now()}`;
    const attachmentParts = parts
      .map((p) => {
        const b64 = p.base64 ?? Buffer.from(p.content ?? '', 'binary').toString('base64');
        // Fold the base64 into 76-char lines (RFC 2045) so long payloads parse.
        const folded = b64.replace(/(.{76})/g, '$1\r\n');
        return (
          `--${bound}\r\nContent-Type: ${p.contentType}; name="${p.filename}"\r\n` +
          `Content-Disposition: attachment; filename="${p.filename}"\r\n` +
          `Content-Transfer-Encoding: base64\r\n\r\n${folded}\r\n`
        );
      })
      .join('');
    body =
      `MIME-Version: 1.0\r\n` +
      `Content-Type: multipart/mixed; boundary="${bound}"\r\n\r\n` +
      `--${bound}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n${opts.text}\r\n` +
      attachmentParts +
      `--${bound}--\r\n`;
  } else {
    body = `MIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n${opts.text}\r\n`;
  }

  const message =
    `From: ${opts.from}\r\nTo: ${to}\r\nSubject: ${opts.subject}\r\n` +
    `Date: ${new Date().toUTCString()}\r\n${body}`;

  const steps = [
    `EHLO mailwoman-e2e`,
    `MAIL FROM:<${extractAddr(opts.from)}>`,
    `RCPT TO:<${to}>`,
    `DATA`,
    `${message}\r\n.`,
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
      // Act only on a complete final reply line ("NNN <text>", space after code).
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

/** Pull the bare address out of `Name <addr>` or a bare `addr`. */
function extractAddr(input: string): string {
  const m = input.match(/<([^>]+)>/);
  return m ? m[1]! : input.trim();
}
