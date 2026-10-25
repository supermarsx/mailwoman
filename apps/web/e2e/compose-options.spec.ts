import { test, expect, type Locator, type Page } from '@playwright/test';
import net from 'node:net';
import { engineLogin } from './helpers.ts';
import { uid } from './pim-helpers.ts';

/**
 * Compose send options and inline images, end to end (26.20 t29-e12).
 *
 * Each case composes in the real app against the engine stack and then reads
 * what was DELIVERED straight from Greenmail over IMAP, in the recipient's own
 * mailbox (Greenmail makes an account for any address it receives mail for,
 * with the address as login and password). Nothing about the sent message is
 * taken from the app.
 *
 * The receipt request and inline parts need a From address with a domain; the
 * Greenmail login is the bare name `testuser`, so the stack advertises
 * `testuser@example.org` through `MW_ALLOWED_FROMS` and each case sends as that
 * identity. The open-tracking cases depend on the stack: with `MW_PUBLIC_URL`
 * the image is sent and an open is counted; without it the option is absent.
 */

test.describe.configure({ mode: 'serial' });

const IMAP_HOST = process.env['MW_E2E_IMAP_HOST'] ?? '127.0.0.1';
const IMAP_PORT = Number(process.env['MW_E2E_IMAP_PORT'] ?? 3143);
const FROM = process.env['MW_E2E_ENGINE_FROM'] ?? 'testuser@example.org';

/** A 1x1 PNG. */
const PNG = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==',
  'base64',
);

// ── The mail server, directly ────────────────────────────────────────────────

/** The raw source of every INBOX message of `address` whose Subject contains
 *  `subject`. An account that does not exist yet reads as no messages. */
function imapFetch(address: string, subject: string): Promise<string[]> {
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
        send(`LOGIN "${address}" "${address}"`);
        return;
      }
      const done = new RegExp(`(^|\\r\\n)a${step} (OK|NO|BAD)[^\\r\\n]*\\r\\n$`).exec(buf);
      if (done === null) return;
      if (done[2] !== 'OK') {
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
        if (lit !== null) out.push(reply.slice(lit.index + lit[0].length, lit.index + lit[0].length + Number(lit[1])));
      }
      const next = ids[step - 3];
      if (next !== undefined) return send(`FETCH ${next} BODY.PEEK[]`);
      sock.end();
      resolve(out);
    });
  });
}

/** Wait until exactly one message with `subject` is in `address`'s INBOX and
 *  return its source. Covers the undo-send hold (~10 s) and the SMTP hop. */
async function deliveredTo(address: string, subject: string, timeout = 60_000): Promise<string> {
  let found: string[] = [];
  await expect(async () => {
    found = await imapFetch(address, subject);
    expect(found.length, `messages for ${address} with subject "${subject}"`).toBeGreaterThan(0);
  }).toPass({ timeout });
  expect(found, `one send delivers one message to ${address}`).toHaveLength(1);
  return found[0]!;
}

interface Part {
  headers: string;
  body: Buffer;
}

/** A message's top-level headers (unfolded) and its leaf parts, decoded. */
function readMessage(raw: string): { headers: string; parts: Part[] } {
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
  return { headers: split(raw)[0], parts: walk(raw) };
}

/** Every value of one header in an unfolded header block. */
function headerValues(headers: string, name: string): string[] {
  return [...headers.matchAll(new RegExp(`^${name}:[ \\t]*(.*)$`, 'gim'))].map((m) => m[1]!.trim());
}

/** The decoded HTML of a message. */
function htmlOf(raw: string): string {
  const part = readMessage(raw).parts.find((p) => /^content-type:\s*text\/html/im.test(p.headers));
  return part?.body.toString('utf8') ?? '';
}

// ── The app ─────────────────────────────────────────────────────────────────

/** Open the composer, send as `FROM`, address it, give it a subject and body. */
async function compose(page: Page, to: string, subject: string, body: string): Promise<Locator> {
  await page.getByRole('button', { name: 'Compose' }).click();
  const dialog = page.getByRole('dialog', { name: 'Compose message' });
  await expect(dialog).toBeVisible();
  const from = dialog.getByRole('combobox', { name: 'From', exact: true });
  // The identity whose address is FROM (option text: "<name> <address>").
  let value: string | undefined;
  await expect(async () => {
    const options = await from
      .locator('option')
      .evaluateAll((els) => els.map((o) => ({ value: (o as HTMLOptionElement).value, text: o.textContent ?? '' })));
    value = options.find((o) => o.text.includes(`<${FROM}>`))?.value;
    expect(value, `the stack advertises ${FROM} (MW_ALLOWED_FROMS): ${JSON.stringify(options)}`).toBeTruthy();
  }).toPass({ timeout: 10_000 });
  await from.selectOption(value!);
  await dialog.getByLabel('To', { exact: true }).fill(to);
  await dialog.getByLabel('Subject', { exact: true }).fill(subject);
  const editor = dialog.getByTestId('compose-richtext');
  await editor.click();
  await editor.pressSequentially(body);
  return dialog;
}

async function send(dialog: Locator): Promise<void> {
  await dialog.getByRole('button', { name: 'Send' }).click();
  await expect(dialog).toBeHidden();
}

async function openTrackingOn(page: Page): Promise<boolean> {
  const res = await page.request.get('/api/policy');
  expect(res.ok()).toBe(true);
  return ((await res.json()) as { openTracking?: boolean }).openTracking === true;
}

test.describe('compose: read receipt, inline images, open tracking (engine mode)', () => {
  test('a read receipt request is sent only when asked for, addressed to the sender', async ({ page }) => {
    test.slow();
    const tag = uid();
    const rcpt = `rr-${tag}@example.org`;
    await engineLogin(page);

    // Precondition: the option is offered, off, and without it no header is sent.
    let dialog = await compose(page, rcpt, `Plain ${tag}`, 'no receipt asked');
    const receipt = dialog.getByTestId('compose-request-receipt');
    await expect(receipt).not.toBeChecked();
    await send(dialog);
    const plain = readMessage(await deliveredTo(rcpt, `Plain ${tag}`));
    expect(headerValues(plain.headers, 'Disposition-Notification-To')).toEqual([]);

    dialog = await compose(page, rcpt, `Receipt ${tag}`, 'receipt asked');
    await dialog.getByTestId('compose-request-receipt').check();
    await send(dialog);
    const asked = readMessage(await deliveredTo(rcpt, `Receipt ${tag}`));
    const dnt = headerValues(asked.headers, 'Disposition-Notification-To');
    expect(dnt).toHaveLength(1);
    expect(dnt[0]!.replace(/^.*<|>.*$/g, '')).toBe(FROM);
    expect(headerValues(asked.headers, 'From')[0]).toContain(FROM);
  });

  test('an image placed in the body is delivered as a multipart/related part the HTML names', async ({ page }) => {
    test.slow();
    const tag = uid();
    const rcpt = `img-${tag}@example.org`;
    const subject = `Inline ${tag}`;
    await engineLogin(page);

    const dialog = await compose(page, rcpt, subject, 'see the image: ');
    await dialog.getByLabel('Insert an image').setInputFiles({ name: 'dot.png', mimeType: 'image/png', buffer: PNG });
    // Ours only: ProseMirror adds a separator `img` after an inline node at a line end.
    const shown = dialog.getByTestId('compose-richtext').locator('img.compose-rt__img');
    await expect(shown).toHaveCount(1);
    expect(await shown.getAttribute('src')).toMatch(/^blob:/);
    await send(dialog);

    const raw = await deliveredTo(rcpt, subject);
    const msg = readMessage(raw);
    expect(headerValues(msg.headers, 'Content-Type')[0]).toMatch(/^multipart\/related/i);
    const image = msg.parts.find((p) => /^content-type:\s*image\/png/im.test(p.headers));
    expect(image, 'an image/png part').toBeDefined();
    expect(image!.body.equals(PNG), 'the part bytes are the uploaded file').toBe(true);
    const cid = /^content-id:\s*<([^>]+)>/im.exec(image!.headers)?.[1];
    expect(cid, 'the part has a Content-ID').toBeDefined();
    const html = htmlOf(raw);
    expect(html).toContain(`src="cid:${cid}"`);
    expect(html).not.toContain('blob:');
  });

  test('with open tracking on, the image URL is in the delivered HTML and loading it counts one open', async ({
    page,
    playwright,
  }) => {
    test.slow();
    await engineLogin(page);
    test.skip(!(await openTrackingOn(page)), 'this stack has no MW_PUBLIC_URL (see the next case)');
    const tag = uid();
    const rcpt = `trk-${tag}@example.org`;
    const subject = `Tracked ${tag}`;

    const dialog = await compose(page, rcpt, subject, 'tracked message');
    const track = dialog.getByTestId('compose-track-opens');
    await expect(track).not.toBeChecked();
    await track.check();
    const bind = page.waitForRequest((r) => r.method() === 'PUT' && r.url().includes('/api/track/open/'));
    await send(dialog);
    const emailId = (JSON.parse((await bind).postData() ?? '{}') as { emailId?: string }).emailId;
    expect(emailId).toBeTruthy();

    const html = htmlOf(await deliveredTo(rcpt, subject));
    const url = /<img src="(https?:\/\/[^"]+\/t\/[A-Za-z0-9_-]+\.gif)"/.exec(html)?.[1];
    expect(url, 'an absolute image URL in the delivered HTML').toBeDefined();

    // Precondition: nothing is counted before the image is loaded.
    const count = async (): Promise<number> => {
      const res = await page.request.get(`/api/track/open?emailId=${encodeURIComponent(emailId!)}`);
      expect(res.status()).toBe(200);
      return ((await res.json()) as { count: number }).count;
    };
    expect(await count()).toBe(0);

    // The recipient's mail program, without the sender's session.
    const reader = await playwright.request.newContext();
    const img = await reader.get(url!);
    expect(img.status()).toBe(200);
    expect(img.headers()['content-type']).toBe('image/gif');
    await reader.dispose();
    await expect.poll(count).toBe(1);
  });

  test('without MW_PUBLIC_URL the open-tracking option is absent', async ({ page }) => {
    await engineLogin(page);
    test.skip(await openTrackingOn(page), 'this stack has MW_PUBLIC_URL (see the previous case)');
    const dialog = await compose(page, `none-${uid()}@example.org`, 'unused', 'x');
    await expect(dialog.getByTestId('compose-request-receipt')).toBeVisible();
    await expect(dialog.getByTestId('compose-track-opens')).toHaveCount(0);
  });
});
