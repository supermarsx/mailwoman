import { test, expect, type Page } from '@playwright/test';
import net from 'node:net';
import { ENGINE_CREDS, engineLogin, composeSelf, sidebarInbox, messageRow } from './helpers.ts';
import { gotoModule, uid } from './pim-helpers.ts';

/**
 * V2 offline queue + replay (plan §1.2, §2.5), end-to-end. Compose while offline
 * -> the mutation is captured in the IndexedDB outbound queue instead of sent;
 * on reconnect the queue drains FIFO and the send actually dispatches.
 *
 * Runtime note: app.online() is request-failure-driven (client.onNetwork), NOT
 * navigator.onLine — so we force a real request to fail first (click Inbox)
 * before it flips to offline. The browser 'online' event on reconnect is what
 * triggers the replay.
 *
 * What was delivered is read back from Greenmail over IMAP by this file, not
 * through the app: a row in the app's list shows that a subject arrived, and
 * says nothing about the body or the envelope.
 */

test.describe.configure({ mode: 'serial' });

// ── Reading the mailbox directly ─────────────────────────────────────────────

const IMAP_HOST = process.env['MW_E2E_IMAP_HOST'] ?? '127.0.0.1';
const IMAP_PORT = Number(process.env['MW_E2E_IMAP_PORT'] ?? 3143);

/**
 * Fetch the raw RFC 5322 source of every INBOX message whose Subject contains
 * `subject`, straight from Greenmail's IMAP listener (the port
 * docker-compose.dev.yml publishes). Subjects in this file are ASCII.
 */
function imapFetchBySubject(subject: string): Promise<string[]> {
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
        send(`LOGIN ${ENGINE_CREDS.username} ${ENGINE_CREDS.password}`);
        return;
      }
      const done = new RegExp(`(^|\\r\\n)a${step} (OK|NO|BAD)[^\\r\\n]*\\r\\n$`).exec(buf);
      if (done === null) return;
      if (done[2] !== 'OK') return fail(new Error(`IMAP step ${step} failed: ${buf.slice(-200)}`));
      const reply = buf;
      if (step === 1) return send('SELECT INBOX');
      if (step === 2) return send(`SEARCH SUBJECT "${subject}"`);
      if (step === 3) {
        ids = (/\* SEARCH([^\r\n]*)/.exec(reply)?.[1] ?? '').trim().split(/\s+/).filter((s) => s.length > 0);
      } else {
        // `* n FETCH (BODY[] {size}\r\n<size octets>)`
        const lit = /\{(\d+)\}\r\n/.exec(reply);
        if (lit !== null) out.push(reply.slice(lit.index + lit[0].length, lit.index + lit[0].length + Number(lit[1])));
      }
      const next = ids[step - 3];
      if (next !== undefined) return send(`FETCH ${next} BODY.PEEK[]`);
      sock.end();
      resolve(out);
    });
  });
}

/** Poll Greenmail until exactly the expected message has been delivered. The
 *  wait covers the undo-send hold (~10 s) plus the SMTP loopback. */
async function deliveredMessage(subject: string, timeout = 60_000): Promise<string> {
  let found: string[] = [];
  await expect(async () => {
    found = await imapFetchBySubject(subject);
    expect(found.length, `messages in the Greenmail INBOX with subject "${subject}"`).toBeGreaterThan(0);
  }).toPass({ timeout });
  expect(found, 'one send produces one delivered message').toHaveLength(1);
  return found[0]!;
}

/** A message's headers (unfolded) and its text content with the transfer
 *  encoding of each part undone — enough for the single-part and
 *  multipart/alternative messages the composer produces. */
function readMessage(raw: string): { headers: string; text: string } {
  const split = (block: string): [string, string] => {
    const at = block.indexOf('\r\n\r\n');
    return at < 0 ? [block, ''] : [block.slice(0, at).replace(/\r\n[ \t]+/g, ' '), block.slice(at + 4)];
  };
  const decode = (headers: string, body: string): string => {
    const cte = /^content-transfer-encoding:\s*(\S+)/im.exec(headers)?.[1]?.toLowerCase();
    if (cte === 'base64') return Buffer.from(body.replace(/\s+/g, ''), 'base64').toString('utf8');
    if (cte === 'quoted-printable') {
      const bytes = body
        .replace(/=\r\n/g, '')
        .replace(/=([0-9A-Fa-f]{2})/g, (_m, hex: string) => String.fromCharCode(parseInt(hex, 16)));
      return Buffer.from(bytes, 'latin1').toString('utf8');
    }
    return Buffer.from(body, 'latin1').toString('utf8');
  };
  const walk = (block: string): string => {
    const [headers, body] = split(block);
    const boundary = /^content-type:\s*multipart\/[^;]+;.*?boundary="?([^";\s]+)"?/im.exec(headers)?.[1];
    if (boundary === undefined) return decode(headers, body);
    return body
      .split(`--${boundary}`)
      .slice(1)
      .filter((p) => !p.startsWith('--'))
      .map((p) => walk(p.replace(/^\r\n/, '')))
      .join('\n');
  };
  return { headers: split(raw)[0], text: walk(raw) };
}

test.describe('V2 offline queue + replay (engine mode)', () => {
  test('compose while offline queues, then replays and sends on reconnect', async ({ page, context }) => {
    test.slow();
    await engineLogin(page);

    // Go offline, then force a request to fail so app.online() flips to false.
    await context.setOffline(true);
    await sidebarInbox(page).click();
    await expect(page.locator('.sidebar__offline')).toBeVisible({ timeout: 15_000 });

    // Composing now takes the offline path: queued, not sent.
    const subject = `Offline ${Date.now()}`;
    const body = `queued while offline ${subject}`;
    await composeSelf(page, subject, body);
    await expect(page.getByText('Queued — will send when back online')).toBeVisible();
    // Precondition for the delivery assertions below: nothing has been sent yet.
    await context.setOffline(false);
    expect(await imapFetchBySubject(subject)).toEqual([]);

    // Reconnect. The idle app won't issue a request on its own, so click Inbox:
    // that first successful request fires onNetwork(up) -> drains the queue, and
    // its replayed send self-delivers back to the Inbox. The DELIVERED message
    // (durable) is the proof the queued send actually dispatched — stronger than
    // the transient replay toast.
    await expect(async () => {
      await sidebarInbox(page).click();
      await expect(messageRow(page, subject).first()).toBeVisible({ timeout: 3_000 });
    }).toPass({ timeout: 45_000 });

    // And it carries the BODY that was typed. Compose was first opened while
    // offline, so the rich editor's chunk could not be fetched and the text went
    // into the fallback textarea; a send built from the absent editor's HTML
    // delivered this subject over an empty body.
    const delivered = readMessage(await deliveredMessage(subject));
    expect(delivered.text).toContain(body);
  });
});

test.describe('what a send delivers (engine mode)', () => {
  // No service worker in these two: a cached copy of the editor chunk would be
  // served without the request ever reaching the route that blocks it.
  test.use({ serviceWorkers: 'block' });

  /** Open Compose and return its dialog. */
  async function openCompose(page: Page) {
    await page.getByRole('button', { name: 'Compose' }).click();
    const dialog = page.getByRole('dialog', { name: 'Compose message' });
    await expect(dialog).toBeVisible();
    return dialog;
  }

  test('with the rich editor chunk blocked, the typed body is what arrives', async ({ page }) => {
    test.slow();
    let blocked = 0;
    await page.route(/\/assets\/RichTextEditor-[^/]*\.js(\?.*)?$/, (route) => {
      blocked += 1;
      return route.abort();
    });
    await engineLogin(page);

    const dialog = await openCompose(page);
    // Precondition: the chunk really was requested and refused, the editor is
    // not there, and Body is the plain fallback while the composer is still in
    // rich mode (its toggle offers "Plain text").
    await expect.poll(() => blocked).toBeGreaterThan(0);
    const bodyField = dialog.getByLabel('Body', { exact: true });
    await expect(bodyField).toBeVisible();
    await expect(dialog.getByTestId('compose-richtext')).toHaveCount(0);
    expect(await bodyField.evaluate((el) => el.tagName)).toBe('TEXTAREA');
    await expect(dialog.getByTestId('format-toggle')).toHaveText('Plain text');

    const subject = `Fallback body ${uid()}`;
    const marker = `typed-into-the-fallback-${uid()}`;
    await dialog.getByLabel('To', { exact: true }).fill(ENGINE_CREDS.selfAddress);
    await dialog.getByLabel('Subject', { exact: true }).fill(subject);
    await bodyField.fill(`first line ${marker}\nsecond line`);
    await dialog.getByRole('button', { name: 'Send' }).click();
    await expect(dialog).toBeHidden();

    const delivered = readMessage(await deliveredMessage(subject));
    expect(delivered.text).toContain(`first line ${marker}`);
    expect(delivered.text).toContain('second line');
  });

  test('a contact picked with its display name is delivered at the bare address', async ({ page }) => {
    test.slow();
    await engineLogin(page);

    // A contact whose address is this account's own, so the send loops back.
    const tag = uid();
    const name = `Loop Back ${tag}`;
    await gotoModule(page, 'contacts');
    const contacts = page.locator('[data-module="contacts"]');
    await contacts.getByRole('button', { name: 'New contact' }).click();
    const form = contacts.getByRole('form', { name: 'New contact' });
    await form.getByLabel('Full name').fill(name);
    await form.getByLabel('Email 1', { exact: true }).fill(ENGINE_CREDS.selfAddress);
    await form.getByRole('button', { name: 'Save' }).click();
    await expect(contacts.getByRole('list', { name: 'Contact list' }).getByText(name)).toBeVisible();

    await sidebarInbox(page).click();
    const dialog = await openCompose(page);
    const to = dialog.getByLabel('To', { exact: true });
    await to.pressSequentially(`Loop Back ${tag}`.slice(0, 14 + tag.length));
    const suggestion = dialog.getByTestId('contact-suggestion').filter({ hasText: name });
    await expect(suggestion).toBeVisible();
    // mousedown is what the composer listens for (it must land before the blur).
    await suggestion.dispatchEvent('mousedown');
    // Precondition: the field now holds the display-name form, not a bare address.
    await expect(to).toHaveValue(`${name} <${ENGINE_CREDS.selfAddress}>, `);

    const subject = `Display name ${tag}`;
    await dialog.getByLabel('Subject', { exact: true }).fill(subject);
    await dialog.getByLabel('Body', { exact: true }).fill(`to a named contact ${tag}`);
    await dialog.getByRole('button', { name: 'Send' }).click();
    // Not refused: the composer closes and no error is shown.
    await expect(dialog).toBeHidden();

    // Delivered to the account, which Greenmail does only for an envelope
    // recipient that is the bare address; and the To header names the contact
    // with the address in angle brackets.
    const delivered = readMessage(await deliveredMessage(subject));
    const toHeader = /^To:\s*(.*)$/im.exec(delivered.headers)?.[1] ?? '';
    expect(toHeader).toContain(`<${ENGINE_CREDS.selfAddress}>`);
    expect(toHeader).toContain('Loop Back');
    expect(toHeader).not.toContain(`<${name}`);
  });
});
