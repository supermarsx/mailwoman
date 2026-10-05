import { test, expect, type Locator, type Page } from '@playwright/test';
import net from 'node:net';
import { ENGINE_CREDS, engineLogin, messageRow, waitForInboxMessage } from './helpers.ts';
import { uid } from './pim-helpers.ts';

/**
 * Reply, Reply all, Forward, and Bcc, end to end (26.20 t28-e11).
 *
 * Each case puts a message into the engine account over SMTP, acts on it
 * through the reader toolbar, and then reads what was DELIVERED straight from
 * the mail server: the recipient's own mailbox on Greenmail, over IMAP, by this
 * file. Nothing about the sent message is taken from the app.
 *
 * The other mailboxes: Greenmail creates an account for any address it
 * receives mail for, with the address as both login and password. Every case
 * uses addresses made for it (`<role>-<tag>@example.org`), so the cases are
 * independent of each other and of earlier runs.
 */

test.describe.configure({ mode: 'serial' });

const IMAP_HOST = process.env['MW_E2E_IMAP_HOST'] ?? '127.0.0.1';
const IMAP_PORT = Number(process.env['MW_E2E_IMAP_PORT'] ?? 3143);
const SMTP_HOST = process.env['MW_E2E_SMTP_HOST'] ?? '127.0.0.1';
const SMTP_PORT = Number(process.env['MW_E2E_SMTP_PORT'] ?? 3025);

// ── The mail server, directly ────────────────────────────────────────────────

/** Deliver `raw` (a complete RFC 5322 message, CRLF line ends) to `rcpt`. */
function smtpDeliver(mailFrom: string, rcpt: string[], raw: string): Promise<void> {
  const steps = ['EHLO mailwoman-e2e', `MAIL FROM:<${mailFrom}>`, ...rcpt.map((r) => `RCPT TO:<${r}>`), 'DATA', `${raw}\r\n.`, 'QUIT'];
  return new Promise<void>((resolve, reject) => {
    const sock = net.createConnection({ host: SMTP_HOST, port: SMTP_PORT });
    let step = -1; // -1: waiting for the greeting
    let buf = '';
    const fail = (e: Error): void => {
      sock.destroy();
      reject(e);
    };
    sock.setTimeout(15_000, () => fail(new Error('SMTP delivery timed out')));
    sock.on('error', fail);
    sock.on('data', (chunk) => {
      buf += chunk.toString('utf8');
      const finals = buf.split('\r\n').filter((l) => /^\d{3} /.test(l));
      if (finals.length === 0) return;
      buf = '';
      const last = finals[finals.length - 1]!;
      if (Number(last.slice(0, 3)) >= 400) return fail(new Error(`SMTP refused step ${step}: ${last}`));
      step += 1;
      if (step >= steps.length) {
        sock.end();
        return resolve();
      }
      sock.write(`${steps[step]!}\r\n`);
    });
  });
}

/**
 * The raw source of every INBOX message of `login` whose Subject contains
 * `subject` — or, with `what: 'FLAGS'`, each such message's FETCH FLAGS reply.
 * An account that does not exist yet (nothing was delivered to it) reads as no
 * messages.
 */
function imapFetch(
  login: { user: string; pass: string },
  subject: string,
  what: 'BODY.PEEK[]' | 'FLAGS' = 'BODY.PEEK[]',
): Promise<string[]> {
  return new Promise<string[]>((resolve, reject) => {
    const sock = net.createConnection({ host: IMAP_HOST, port: IMAP_PORT });
    sock.setEncoding('latin1');
    let buf = '';
    let step = 0;
    let ids: string[] = [];
    const out: string[] = [];
    const fail = (e: Error): void => {
      sock.destroy();
      reject(e);
    };
    const send = (line: string): void => {
      step += 1;
      buf = '';
      sock.write(`a${step} ${line}\r\n`);
    };
    sock.setTimeout(15_000, () => fail(new Error('IMAP read timed out')));
    sock.on('error', fail);
    sock.on('data', (chunk: string) => {
      buf += chunk;
      if (step === 0) {
        if (!buf.includes('\r\n')) return;
        send(`LOGIN "${login.user}" "${login.pass}"`);
        return;
      }
      const done = new RegExp(`(^|\\r\\n)a${step} (OK|NO|BAD)[^\\r\\n]*\\r\\n$`).exec(buf);
      if (done === null) return;
      if (done[2] !== 'OK') {
        // No such account yet: nothing has been delivered to it.
        if (step === 1) {
          sock.end();
          return resolve([]);
        }
        return fail(new Error(`IMAP step ${step} failed: ${buf.slice(-200)}`));
      }
      const reply = buf;
      if (step === 1) return send('SELECT INBOX');
      if (step === 2) return send(`SEARCH SUBJECT "${subject}"`);
      if (step === 3) {
        ids = (/\* SEARCH([^\r\n]*)/.exec(reply)?.[1] ?? '').trim().split(/\s+/).filter((s) => s.length > 0);
      } else {
        const lit = /\{(\d+)\}\r\n/.exec(reply);
        if (what === 'FLAGS') out.push(/FLAGS \(([^)]*)\)/.exec(reply)?.[1] ?? '');
        else if (lit !== null) out.push(reply.slice(lit.index + lit[0].length, lit.index + lit[0].length + Number(lit[1])));
      }
      const next = ids[step - 3];
      if (next !== undefined) return send(`FETCH ${next} ${what}`);
      sock.end();
      resolve(out);
    });
  });
}

/** The mailbox Greenmail made for `address` (login and password are the address). */
const boxOf = (address: string): { user: string; pass: string } => ({ user: address, pass: address });

/** Wait until exactly one message with `subject` is in `address`'s INBOX and
 *  return its source. Covers the undo-send hold (~10 s) and the SMTP hop. */
async function deliveredTo(address: string, subject: string, timeout = 60_000): Promise<string> {
  let found: string[] = [];
  await expect(async () => {
    found = await imapFetch(boxOf(address), subject);
    expect(found.length, `messages for ${address} with subject "${subject}"`).toBeGreaterThan(0);
  }).toPass({ timeout });
  expect(found, `one send delivers one message to ${address}`).toHaveLength(1);
  return found[0]!;
}

interface Part {
  headers: string;
  /** The part's content with its transfer encoding undone, as bytes. */
  body: Buffer;
}

/** A message's top-level headers (unfolded) and its leaf parts. */
function readMessage(raw: string): { headers: string; parts: Part[]; text: string } {
  const split = (block: string): [string, string] => {
    const at = block.indexOf('\r\n\r\n');
    return at < 0 ? [block, ''] : [block.slice(0, at).replace(/\r\n[ \t]+/g, ' '), block.slice(at + 4)];
  };
  const decode = (headers: string, body: string): Buffer => {
    const cte = /^content-transfer-encoding:\s*(\S+)/im.exec(headers)?.[1]?.toLowerCase();
    if (cte === 'base64') return Buffer.from(body.replace(/\s+/g, ''), 'base64');
    if (cte === 'quoted-printable') {
      return Buffer.from(
        body.replace(/=\r\n/g, '').replace(/=([0-9A-Fa-f]{2})/g, (_m, hex: string) => String.fromCharCode(parseInt(hex, 16))),
        'latin1',
      );
    }
    return Buffer.from(body.replace(/\r\n$/, ''), 'latin1');
  };
  const walk = (block: string): Part[] => {
    const [headers, body] = split(block);
    const boundary = /^content-type:\s*multipart\/[^;]+;.*?boundary="?([^";\s]+)"?/im.exec(headers)?.[1];
    if (boundary === undefined) return [{ headers, body: decode(headers, body) }];
    return body
      .split(`--${boundary}`)
      .slice(1)
      .filter((p) => !p.startsWith('--'))
      .flatMap((p) => walk(p.replace(/^\r\n/, '')));
  };
  const parts = walk(raw);
  const text = parts
    .filter((p) => /^content-type:\s*text\//im.test(p.headers) || !/^content-type:/im.test(p.headers))
    .map((p) => p.body.toString('utf8'))
    .join('\n');
  return { headers: split(raw)[0], parts, text };
}

/** One header's value from an unfolded header block, or `null` when absent. */
function header(headers: string, name: string): string | null {
  return new RegExp(`^${name}:[ \\t]*(.*)$`, 'im').exec(headers)?.[1] ?? null;
}

// ── The app ─────────────────────────────────────────────────────────────────

/** Open the message with `subject` in the reader and return the reader toolbar. */
async function openInReader(page: Page, subject: string): Promise<Locator> {
  await waitForInboxMessage(page, subject);
  await messageRow(page, subject).first().click();
  await expect(page.locator('.reader__subject')).toHaveText(subject);
  // The body has been fetched and sanitised: this is what a quote is built from.
  await expect(page.locator('iframe.reader__frame')).toBeVisible();
  return page.getByRole('toolbar', { name: 'Message actions' });
}

/** Click a reader action and return the composer it opens. */
async function act(page: Page, toolbar: Locator, name: 'Reply' | 'Reply all' | 'Forward'): Promise<Locator> {
  await toolbar.getByRole('button', { name, exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Compose message' });
  await expect(dialog.getByRole('heading', { name, exact: true })).toBeVisible();
  // The quote is in the editor before anything is typed.
  await expect(dialog.getByTestId('compose-richtext')).toBeVisible();
  return dialog;
}

/** Type `text` at the very start of the body, above the quote. */
async function typeAbove(dialog: Locator, text: string): Promise<void> {
  const body = dialog.getByTestId('compose-richtext');
  await body.click();
  await body.press('Control+Home');
  await body.pressSequentially(text);
}

async function send(dialog: Locator): Promise<void> {
  await dialog.getByRole('button', { name: 'Send' }).click();
  await expect(dialog).toBeHidden();
}

test.describe('reply, reply all, forward, Bcc (engine mode)', () => {
  test('a reply is addressed to the author and threaded, with the original quoted and no image', async ({ page }) => {
    test.slow();
    const tag = uid();
    const alice = `alice-${tag}@example.org`;
    const subject = `Quarterly ${tag}`;
    const origId = `orig-${tag}@e2e.example`;
    const rootId = `root-${tag}@e2e.example`;
    const marker = `original-body-${tag}`;
    await smtpDeliver(alice, [ENGINE_CREDS.selfAddress], [
      `From: Alice Example <${alice}>`,
      `To: ${ENGINE_CREDS.selfAddress}`,
      `Subject: ${subject}`,
      `Date: ${new Date().toUTCString()}`,
      `Message-ID: <${origId}>`,
      `References: <${rootId}>`,
      'MIME-Version: 1.0',
      'Content-Type: text/html; charset=utf-8',
      '',
      `<html><body><p>${marker}</p>`,
      `<img src="http://tracker.e2e.example/pixel-${tag}.gif" width="1" height="1">`,
      `<p style="background:url(http://tracker.e2e.example/bg-${tag}.png)">second paragraph</p></body></html>`,
    ].join('\r\n'));

    await engineLogin(page);
    const self = { user: ENGINE_CREDS.username, pass: ENGINE_CREDS.password };
    // Precondition for the answered mark below: the original is not marked yet.
    await waitForInboxMessage(page, subject);
    expect((await imapFetch(self, subject, 'FLAGS')).join(' ')).not.toContain('\\Answered');
    await expect(messageRow(page, subject).getByTestId('row-answered')).toHaveCount(0);

    // Opening the message does not take the mailbox screen out of the document
    // while the reader's own requests are out: nothing is removed from the app
    // root between here and the reader being up.
    await page.evaluate(() => {
      const w = window as unknown as { __rootRemovals: number };
      w.__rootRemovals = 0;
      new MutationObserver((records) => {
        for (const r of records) w.__rootRemovals += r.removedNodes.length;
      }).observe(document.getElementById('root')!, { childList: true });
    });
    const toolbar = await openInReader(page, subject);
    expect(await page.evaluate(() => (window as unknown as { __rootRemovals: number }).__rootRemovals)).toBe(0);
    const dialog = await act(page, toolbar, 'Reply');
    await expect(dialog.getByLabel('To', { exact: true })).toHaveValue(`Alice Example <${alice}>`);
    await expect(dialog.getByLabel('Subject', { exact: true })).toHaveValue(`Re: ${subject}`);
    const answer = `my-answer-${tag}`;
    await typeAbove(dialog, answer);
    await send(dialog);

    const delivered = readMessage(await deliveredTo(alice, subject));
    expect(header(delivered.headers, 'Subject')).toBe(`Re: ${subject}`);
    expect(header(delivered.headers, 'In-Reply-To')).toBe(`<${origId}>`);
    expect(header(delivered.headers, 'References')).toBe(`<${rootId}> <${origId}>`);
    expect(header(delivered.headers, 'To')).toContain(`<${alice}>`);
    expect(delivered.text).toContain(answer);
    expect(delivered.text).toContain(marker);
    expect(delivered.text).toContain('second paragraph');
    expect(delivered.text).toMatch(/Alice Example wrote:/);
    expect(delivered.text).toMatch(/<blockquote>[\s\S]*<\/blockquote>/);
    // Nothing of the original that loads from elsewhere came along.
    expect(delivered.text).not.toMatch(/<img/i);
    expect(delivered.text).not.toContain('tracker.e2e.example');
    expect(delivered.text).not.toMatch(/url\(/i);

    // The original is marked answered: on the IMAP server, and on its row.
    await expect(async () => {
      expect((await imapFetch(self, subject, 'FLAGS')).join(' ')).toContain('\\Answered');
    }).toPass({ timeout: 30_000 });
    await expect(async () => {
      await waitForInboxMessage(page, subject, 5_000);
      await expect(messageRow(page, subject).getByTestId('row-answered')).toBeVisible({ timeout: 3_000 });
    }).toPass({ timeout: 45_000 });

    // The second reply to the same message does not stack the prefix.
    const again = await act(page, await openInReader(page, subject), 'Reply');
    await expect(again.getByLabel('Subject', { exact: true })).toHaveValue(`Re: ${subject}`);
  });

  test('a reply that comes back to this account joins the original’s conversation', async ({ page }) => {
    test.slow();
    const tag = uid();
    const subject = `Conversation ${tag}`;
    const origId = `conv-${tag}@e2e.example`;
    // Reply-To names this account, so the reply is delivered here.
    await smtpDeliver(`list-${tag}@example.org`, [ENGINE_CREDS.selfAddress], [
      `From: List Bot <list-${tag}@example.org>`,
      `Reply-To: ${ENGINE_CREDS.selfAddress}`,
      `To: ${ENGINE_CREDS.selfAddress}`,
      `Subject: ${subject}`,
      `Date: ${new Date().toUTCString()}`,
      `Message-ID: <${origId}>`,
      'MIME-Version: 1.0',
      'Content-Type: text/plain; charset=utf-8',
      '',
      `first message ${tag}`,
    ].join('\r\n'));

    await engineLogin(page);
    const toolbar = await openInReader(page, subject);
    // Precondition: one message, shown as a single row, not a conversation.
    await expect(page.getByTestId('thread-head').filter({ hasText: subject })).toHaveCount(0);

    const dialog = await act(page, toolbar, 'Reply');
    await expect(dialog.getByLabel('To', { exact: true })).toHaveValue(ENGINE_CREDS.selfAddress);
    await typeAbove(dialog, `answer ${tag}`);
    await send(dialog);

    // Both messages are in this account's INBOX on the server…
    await expect(async () => {
      expect(await imapFetch({ user: ENGINE_CREDS.username, pass: ENGINE_CREDS.password }, subject)).toHaveLength(2);
    }).toPass({ timeout: 60_000 });
    // …and the list shows them as one conversation of two.
    const head = page.getByTestId('thread-head').filter({ hasText: subject });
    await expect(async () => {
      await page.getByRole('navigation', { name: 'Mailboxes' }).getByRole('button', { name: 'Inbox' }).click();
      await expect(head).toHaveCount(1, { timeout: 3_000 });
    }).toPass({ timeout: 60_000 });
    await expect(head.getByLabel('2 messages')).toBeVisible();
  });

  test('reply all keeps the other recipients in To and Cc and leaves this account out', async ({ page }) => {
    test.slow();
    const tag = uid();
    const alice = `alice-${tag}@example.org`;
    const bob = `bob-${tag}@example.org`;
    const carol = `carol-${tag}@example.org`;
    const subject = `Planning ${tag}`;
    await smtpDeliver(alice, [ENGINE_CREDS.selfAddress], [
      `From: Alice Example <${alice}>`,
      `To: ${ENGINE_CREDS.selfAddress}, Bob <${bob}>`,
      `Cc: ${carol}, ${ENGINE_CREDS.selfAddress}`,
      `Subject: ${subject}`,
      `Date: ${new Date().toUTCString()}`,
      `Message-ID: <all-${tag}@e2e.example>`,
      'MIME-Version: 1.0',
      'Content-Type: text/plain; charset=utf-8',
      '',
      `to everyone ${tag}`,
    ].join('\r\n'));

    await engineLogin(page);
    const dialog = await act(page, await openInReader(page, subject), 'Reply all');
    await expect(dialog.getByLabel('To', { exact: true })).toHaveValue(`Alice Example <${alice}>, Bob <${bob}>`);
    await expect(dialog.getByLabel('Cc', { exact: true })).toHaveValue(carol);
    await typeAbove(dialog, `to all of you ${tag}`);
    await send(dialog);

    // Each of the three received it; read Bob's copy.
    await deliveredTo(alice, subject);
    await deliveredTo(carol, subject);
    const delivered = readMessage(await deliveredTo(bob, subject));
    const to = header(delivered.headers, 'To') ?? '';
    const cc = header(delivered.headers, 'Cc') ?? '';
    expect(to).toContain(`<${alice}>`);
    expect(to).toContain(`<${bob}>`);
    expect(cc).toContain(carol);
    expect(`${to} ${cc}`).not.toContain(ENGINE_CREDS.selfAddress);
    expect(header(delivered.headers, 'In-Reply-To')).toBe(`<all-${tag}@e2e.example>`);
    // The reply did not come back to the sender's own INBOX: only the original is there.
    expect(await imapFetch({ user: ENGINE_CREDS.username, pass: ENGINE_CREDS.password }, subject)).toHaveLength(1);
  });

  test('a forward carries the original attachment, byte for byte', async ({ page }) => {
    test.slow();
    const tag = uid();
    const dave = `dave-${tag}@example.org`;
    const subject = `Report ${tag}`;
    // Bytes that are not text in any charset, so a re-encoding would show.
    const bytes = Buffer.from([0x00, 0xff, 0x10, 0x80, 0x0d, 0x0a, 0x7f, 0xfe, ...Buffer.from(`payload-${tag}`)]);
    const bound = `b${tag}`;
    await smtpDeliver(`erin-${tag}@example.org`, [ENGINE_CREDS.selfAddress], [
      `From: Erin <erin-${tag}@example.org>`,
      `To: ${ENGINE_CREDS.selfAddress}`,
      `Subject: ${subject}`,
      `Date: ${new Date().toUTCString()}`,
      `Message-ID: <fwd-${tag}@e2e.example>`,
      'MIME-Version: 1.0',
      `Content-Type: multipart/mixed; boundary="${bound}"`,
      '',
      `--${bound}`,
      'Content-Type: text/plain; charset=utf-8',
      '',
      `see the attached report ${tag}`,
      `--${bound}`,
      'Content-Type: application/octet-stream; name="report.bin"',
      'Content-Disposition: attachment; filename="report.bin"',
      'Content-Transfer-Encoding: base64',
      '',
      bytes.toString('base64'),
      `--${bound}--`,
    ].join('\r\n'));

    await engineLogin(page);
    const dialog = await act(page, await openInReader(page, subject), 'Forward');
    await expect(dialog.getByLabel('Subject', { exact: true })).toHaveValue(`Fwd: ${subject}`);
    await expect(dialog.getByLabel('To', { exact: true })).toHaveValue('');
    await expect(dialog.getByTestId('compose-attachments')).toContainText('report.bin');
    await dialog.getByLabel('To', { exact: true }).fill(dave);
    await send(dialog);

    const delivered = readMessage(await deliveredTo(dave, subject));
    expect(header(delivered.headers, 'Subject')).toBe(`Fwd: ${subject}`);
    expect(header(delivered.headers, 'In-Reply-To')).toBeNull();
    expect(delivered.text).toContain(`see the attached report ${tag}`);
    expect(delivered.text).toContain('Forwarded message');
    const attached = delivered.parts.filter((p) => /report\.bin/.test(p.headers));
    expect(attached).toHaveLength(1);
    expect(attached[0]!.body.equals(bytes), 'the attachment bytes at the recipient').toBe(true);
  });

  test('a Bcc recipient receives the message, and no copy names them', async ({ page }) => {
    test.slow();
    const tag = uid();
    const frank = `frank-${tag}@example.org`;
    const grace = `grace-${tag}@example.org`;
    const subject = `Quiet copy ${tag}`;

    await engineLogin(page);
    await page.getByRole('button', { name: 'Compose' }).click();
    const dialog = page.getByRole('dialog', { name: 'Compose message' });
    await expect(dialog).toBeVisible();
    await dialog.getByRole('button', { name: 'Add Cc or Bcc' }).click();
    await dialog.getByLabel('To', { exact: true }).fill(frank);
    await dialog.getByLabel('Bcc', { exact: true }).fill(grace);
    await dialog.getByLabel('Subject', { exact: true }).fill(subject);
    await dialog.getByLabel('Body', { exact: true }).fill(`body ${tag}`);
    await send(dialog);

    // The Bcc recipient was an envelope recipient…
    const atGrace = readMessage(await deliveredTo(grace, subject));
    const atFrank = readMessage(await deliveredTo(frank, subject));
    expect(header(atFrank.headers, 'To')).toContain(frank);
    // …and no delivered copy has a Bcc header. The To recipient's copy does
    // not hold the Bcc address at all. (The Bcc recipient's own copy may: the
    // receiving server can record whom it delivered to.)
    expect(header(atFrank.headers, 'Bcc'), 'Bcc header in the copy delivered to the To recipient').toBeNull();
    expect(header(atGrace.headers, 'Bcc'), 'Bcc header in the copy delivered to the Bcc recipient').toBeNull();
    expect(atFrank.headers, 'headers of the copy delivered to the To recipient').not.toContain(grace);
  });
});
