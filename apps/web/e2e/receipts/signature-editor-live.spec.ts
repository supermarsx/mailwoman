// t49: Settings -> Signatures is the composer's editor. On a live server over a
// real IMAP/SMTP store (Greenmail), a rich signature with a font, a colour, a
// link, an embedded image and a table is saved, survives a reload unchanged,
// goes into a composed message through the composer's signature picker, and
// arrives with its HTML intact and its image as an inline cid: part. Plain mode
// is checked on both ends, and the signature ribbon passes the 3-row geometry
// check on every tab.
//
//   MW_RECEIPTS_BASE_URL  the server (engine mode over Greenmail, auth disabled)
//   MW_T49_IMAP           Greenmail IMAP host:port, to read what was delivered
//   MW_T49_SHOTS          where screenshots go (default: the test output dir)
import { type Page } from '@playwright/test';
import { test, expect } from '../csrf.ts';
import net from 'node:net';
import path from 'node:path';

const imap = process.env['MW_T49_IMAP'] ?? '127.0.0.1:3143';
const PNG = 'iVBORw0KGgoAAAANSUhEUgAAAAgAAAAICAIAAABLbSncAAAAEUlEQVR4nGOQ96vEihiGlgQAYZI5gapHhb8AAAAASUVORK5CYII=';

/** Every message in `user`'s INBOX, raw (Greenmail IMAP, auth disabled). */
async function inbox(user: string): Promise<string[]> {
  const [host, port] = imap.split(':');
  return new Promise((resolve, reject) => {
    const socket = net.connect(Number(port), host!);
    let buffer = '';
    let step = 0;
    const out: string[] = [];
    const send = (line: string): void => void socket.write(`${line}\r\n`);
    socket.setEncoding('utf8');
    socket.on('error', reject);
    socket.on('data', (chunk: string) => {
      buffer += chunk;
      if (step === 0 && buffer.includes('* OK')) { step = 1; buffer = ''; send(`a1 LOGIN "${user}" "x"`); }
      else if (step === 1 && /a1 (OK|NO|BAD)/.test(buffer)) { step = 2; buffer = ''; send('a2 SELECT INBOX'); }
      else if (step === 2 && /a2 (OK|NO|BAD)/.test(buffer)) {
        const exists = Number(/\* (\d+) EXISTS/.exec(buffer)?.[1] ?? 0);
        buffer = '';
        if (exists === 0) { send('a4 LOGOUT'); socket.end(); resolve([]); return; }
        step = 3; send(`a3 FETCH 1:${exists} BODY.PEEK[]`);
      } else if (step === 3 && /a3 (OK|NO|BAD)/.test(buffer)) {
        for (const m of buffer.matchAll(/\* \d+ FETCH \(BODY\[\] \{(\d+)\}\r\n/g)) {
          const start = m.index! + m[0].length;
          out.push(buffer.slice(start, start + Number(m[1])));
        }
        send('a4 LOGOUT'); socket.end(); resolve(out);
      }
    });
  });
}

/** Mirrors components/bars/geometry.ts (assertRibbonGeometry), as bars-layout-live does. */
function ribbonViolations(root: Element): string[] {
  const cs = getComputedStyle(root);
  const row = parseFloat(cs.getPropertyValue('--mw-ribbon-row')) || 28;
  const gap = parseFloat(cs.getPropertyValue('--mw-ribbon-gap')) || 2;
  const expected = 3 * row + 2 * gap;
  const hidden = (el: Element): boolean => {
    for (let e: Element | null = el; e && e !== root.parentElement; e = e.parentElement) {
      if (e.hasAttribute('hidden')) return true;
      const p = e.parentElement;
      if (p instanceof HTMLDetailsElement && !p.open && e.tagName !== 'SUMMARY') return true;
    }
    return false;
  };
  const out: string[] = [];
  for (const el of root.querySelectorAll('[data-ribbon-body], [data-ribbon-controls]')) {
    if (hidden(el)) continue;
    const h = el.getBoundingClientRect().height;
    if (Math.abs(h - expected) > 0.5) out.push(`height ${el.closest('[data-ribbon-group]')?.getAttribute('aria-label')} ${h} != ${expected}`);
  }
  for (const el of root.querySelectorAll('*')) {
    if (!(el instanceof HTMLElement) || hidden(el) || el.clientHeight === 0) continue;
    if (!el.checkVisibility({ visibilityProperty: true }) || /rect\(0(px)?,? 0(px)?,? 0(px)?,? 0(px)?\)/.test(getComputedStyle(el).clip)) continue;
    if (el.scrollHeight > el.clientHeight + 1) out.push(`overflow ${el.tagName.toLowerCase()}.${String(el.className).split(' ')[0]} ${el.scrollHeight}/${el.clientHeight}`);
  }
  for (const g of root.querySelectorAll('[data-ribbon-group]')) {
    if (hidden(g)) continue;
    const b = g.getBoundingClientRect();
    for (const c of g.querySelectorAll('[data-ribbon-controls] > *, [data-ribbon-more]')) {
      if (hidden(c) || c.closest('[data-ribbon-popover]')) continue;
      const r = c.getBoundingClientRect();
      if (r.top < b.top - 0.5 || r.bottom > b.bottom + 0.5 || r.left < b.left - 0.5 || r.right > b.right + 0.5) out.push(`outside ${g.getAttribute('aria-label')}`);
    }
  }
  if (document.documentElement.scrollWidth > innerWidth) out.push(`page scrolls sideways ${document.documentElement.scrollWidth} > ${innerWidth}`);
  return out;
}

/** A ribbon group's control, through its folded dropdown when the width folds it. */
async function control(page: Page, scope: string, group: string, find: (page: Page) => ReturnType<Page['getByRole']>): Promise<ReturnType<Page['getByRole']>> {
  const target = find(page);
  if (await target.isVisible()) return target;
  await page.locator(scope).getByRole('button', { name: `More ${group} commands` }).click();
  await expect(target).toBeVisible();
  return target;
}

async function signIn(page: Page): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill('testuser');
  await page.getByLabel('Password', { exact: true }).fill('testpass');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Compose', exact: true })).toBeVisible();
}
async function openSignatures(page: Page, phone: boolean): Promise<void> {
  if (phone) await page.getByRole('button', { name: 'Open folders and apps' }).click();
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByRole('button', { name: 'Signatures', exact: true }).click();
}

test('the signature editor is the composer editor, and a rich signature survives save, reload and delivery', async ({ page }, info) => {
  const shots = process.env['MW_T49_SHOTS'] ?? info.outputDir;
  const shot = (name: string): Promise<Buffer> => page.screenshot({ path: path.join(shots, `${info.project.name}-${name}.png`) });
  const phone = (page.viewportSize()?.width ?? 1440) < 600;
  const scope = '[data-testid="signature-editor"]';
  const errors: string[] = [];
  page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
  page.on('pageerror', (e) => errors.push(String(e)));
  await signIn(page);
  await openSignatures(page, phone);
  await page.getByRole('button', { name: 'New signature', exact: true }).click();
  const name = `Rich ${info.project.name} ${Date.now()}`;
  await page.getByLabel('Name', { exact: true }).fill(name);
  await (await control(page, scope, 'Signature format', (p) => p.getByRole('combobox', { name: 'Signature format' }))).selectOption('rich');
  const editor = page.getByRole('textbox', { name: 'Rich signature' });
  await editor.click();
  await page.keyboard.type('Kind regards, Mariana');
  // The composer's ribbon, with the signature groups only.
  const ribbon = page.locator(`${scope} [data-bar]`);
  await expect(ribbon.getByRole('tab')).toHaveText(['Format text', 'Insert', 'Table']);
  for (const id of ['compose-group-recipients', 'compose-attach', 'compose-crypto', 'compose-send-options', 'rt-group-editing']) await expect(page.locator(`${scope} [data-testid="${id}"]`)).toHaveCount(0);
  await expect(page.locator(scope).getByRole('button', { name: 'Send' })).toHaveCount(0);
  // Font and colour on the whole line.
  await page.keyboard.press('ControlOrMeta+a');
  await (await control(page, scope, 'Font', (p) => p.locator(scope).getByRole('combobox', { name: 'Font', exact: true }))).selectOption({ label: 'Georgia' });
  await editor.click();
  await page.keyboard.press('ControlOrMeta+a');
  await (await control(page, scope, 'Font', (p) => p.locator(scope).getByLabel('Font color', { exact: true }))).fill('#1f4e79');
  // A link on the name.
  await editor.click();
  await page.keyboard.press('End');
  for (let i = 0; i < 'Mariana'.length; i++) await page.keyboard.press('Shift+ArrowLeft');
  await ribbon.getByRole('tab', { name: 'Insert' }).click();
  await (await control(page, scope, 'Insert', (p) => p.locator(scope).getByTestId('rt-link'))).click();
  await page.getByRole('textbox', { name: 'Link' }).fill('https://example.org/mariana');
  await page.getByTestId('rt-link-apply').click();
  // An embedded image, then a table, at the end.
  await editor.click();
  await page.keyboard.press('ControlOrMeta+End');
  await page.keyboard.press('Enter');
  const image = await control(page, scope, 'Insert', (p) => p.locator(scope).getByTestId('rt-image'));
  await image.locator('input[type=file]').setInputFiles({ name: 'logo.png', mimeType: 'image/png', buffer: Buffer.from(PNG, 'base64') });
  await expect(editor.locator('img.compose-rt__img')).toHaveCount(1);
  await (await control(page, scope, 'Table', (p) => p.locator(scope).getByRole('button', { name: 'Insert table' }))).click();
  await expect(editor.locator('table')).toHaveCount(1);
  await editor.locator('td').first().click();
  await page.keyboard.type('Mailwoman');
  await page.getByLabel('Plain-text alternative').fill('Kind regards, Mariana <mariana@example.org>');
  // The 3-row contract on every tab of the signature ribbon.
  for (const tab of ['Format text', 'Insert', 'Table']) {
    await ribbon.getByRole('tab', { name: tab }).click();
    await page.mouse.move(1, 1);
    const violations = await ribbon.evaluate(ribbonViolations);
    expect(violations, `${tab}: ${violations.join('; ')}`).toEqual([]);
    await shot(`editor-${tab.split(' ')[0]!.toLowerCase()}`);
  }
  // Save: what the editor sends, and what the server stores.
  const firstSave = page.waitForRequest((r) => r.url().endsWith('/api/account/signatures') && r.method() === 'POST');
  await page.getByTestId('signature-save').click();
  const saveRequest = await firstSave;
  const sent = JSON.parse(saveRequest.postData()!) as { html: string; text: string };
  // The account headers the web client sends with a settings write (account and storage authority).
  const accountHeaders = Object.fromEntries(Object.entries(await saveRequest.allHeaders()).filter(([key]) => key.startsWith('x-mailwoman-')));
  await expect(page.getByTestId('signature-list')).toContainText(name);
  const read = async (): Promise<{ name: string; html: string; text: string }> =>
    ((await (await page.request.get('/api/account/signatures')).json()) as { signatures: { name: string; html: string; text: string }[] }).signatures.find((s) => s.name === name)!;
  const stored = await read();
  for (const want of ['font-family', 'Georgia', 'color', 'href="https://example.org/mariana"', '<img src="data:image/png;base64,', '<table', 'Mailwoman']) expect(stored.html, want).toContain(want);
  expect(stored.text).toBe('Kind regards, Mariana <mariana@example.org>');
  info.annotations.push({ type: 'editor-html', description: sent.html }, { type: 'stored-html', description: stored.html });
  // Reload, reopen: nothing is unsaved, and saving again sends and stores the same HTML.
  await page.reload();
  await openSignatures(page, phone);
  await page.getByTestId('signature-list').getByRole('listitem').filter({ hasText: name }).getByRole('button', { name: 'Edit' }).click();
  await expect(editor.locator('img.compose-rt__img')).toHaveCount(1);
  await expect(editor.locator('table')).toHaveCount(1);
  await expect(editor.locator('a[href="https://example.org/mariana"]')).toHaveText('Mariana');
  await shot('reopened');
  const secondSave = page.waitForRequest((r) => r.url().endsWith('/api/account/signatures') && r.method() === 'POST');
  await page.getByTestId('signature-save').click();
  const resent = JSON.parse((await secondSave).postData()!) as { html: string; text: string };
  expect(resent.html).toBe(sent.html);
  expect(resent.text).toBe(sent.text);
  expect((await read()).html).toBe(stored.html);
  // Opening and leaving asks nothing.
  await page.getByTestId('signature-list').getByRole('listitem').filter({ hasText: name }).getByRole('button', { name: 'Edit' }).click();
  await expect(editor).toBeVisible();
  await page.locator(scope).getByRole('button', { name: 'Cancel' }).click();
  await expect(page.getByRole('alertdialog')).toHaveCount(0);
  await expect(page.getByTestId('signature-list')).toBeVisible();
  // Plain mode in the editor shows the plain alternative.
  await page.getByTestId('signature-list').getByRole('listitem').filter({ hasText: name }).getByRole('button', { name: 'Edit' }).click();
  await (await control(page, scope, 'Signature format', (p) => p.getByRole('combobox', { name: 'Signature format' }))).selectOption('plain');
  await expect(page.getByLabel('Signature', { exact: true })).toHaveValue('Kind regards, Mariana <mariana@example.org>');
  await expect(ribbon.getByRole('tab')).toHaveText(['Format text']);
  expect(await ribbon.evaluate(ribbonViolations)).toEqual([]);
  await shot('plain');
  await page.locator(scope).getByRole('button', { name: 'Cancel' }).click();
  await page.getByRole('alertdialog').getByRole('button').filter({ hasText: /discard|leave/i }).first().click();

  // Into a message, through the composer's picker, and out through SMTP.
  const created = await page.request.post('/api/account/identities', { headers: accountHeaders, data: { name, email: 'testuser@example.org', signatureName: name } });
  expect(created.ok(), await created.text()).toBe(true);
  const identity = (await created.json()) as { id: string };
  await page.reload();
  for (const format of ['rich', 'plain'] as const) {
    const to = `t49-${format}-${info.project.name}-${Date.now()}@example.org`;
    const subject = `${format} ${name}`;
    await page.getByRole('button', { name: 'Compose', exact: true }).click();
    const compose = page.getByRole('region', { name: 'Compose message' });
    if (!(await page.getByRole('combobox', { name: 'From', exact: true }).isVisible())) await (await control(page, '[aria-label="Compose message"]', 'Recipients', (p) => p.getByRole('region', { name: 'Compose message' }).getByRole('button', { name: 'From', exact: true }))).click();
    await page.getByRole('combobox', { name: 'From', exact: true }).selectOption(identity.id);
    await page.getByLabel('To', { exact: true }).fill(to);
    await page.getByLabel('Subject', { exact: true }).fill(subject);
    await (await control(page, '[aria-label="Compose message"]', 'Message type', (p) => p.getByTestId('format-toggle'))).selectOption(format);
    if (format === 'rich') {
      await compose.getByTestId('compose-richtext').click();
      await page.keyboard.type('Hello recipient.');
    } else {
      await page.getByLabel('Body', { exact: true }).fill('Hello recipient.');
    }
    await compose.getByRole('tab', { name: 'Insert' }).click();
    const picker = await control(page, '[aria-label="Compose message"]', 'Signature', (p) => p.getByRole('combobox', { name: 'Signature', exact: true }));
    await picker.selectOption(identity.id);
    if (format === 'rich') await expect(compose.getByTestId('compose-richtext').locator('img.compose-rt__img')).toHaveCount(1);
    else await expect(page.getByLabel('Body', { exact: true })).toHaveValue(/Kind regards, Mariana <mariana@example.org>/);
    await shot(`compose-${format}`);
    await compose.getByRole('button', { name: 'Send', exact: true }).click();
    await expect.poll(async () => (await inbox(to)).length, { timeout: 60000 }).toBe(1);
    const raw = (await inbox(to))[0]!.replace(/=\r\n/g, '');
    expect(raw).toContain(subject);
    if (format === 'rich') {
      for (const want of ['Content-Type: text/html', 'Georgia', 'color', 'https://example.org/mariana', '<table', 'Mailwoman', 'src=3D"cid:', 'Content-Type: image/png', 'Content-ID:']) expect(raw, want).toMatch(new RegExp(want.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i'));
      expect(raw).not.toContain('data:image');
      const cid = /src=3D"cid:([^"]+)"/.exec(raw)?.[1];
      expect(raw).toContain(`<${cid}>`);
      info.annotations.push({ type: 'received-html', description: raw.slice(raw.search(/Content-Type: text\/html/i), raw.search(/Content-Type: text\/html/i) + 4000) });
    } else {
      expect(raw).toMatch(/Content-Type: text\/plain/i);
      expect(raw).not.toMatch(/Content-Type: text\/html/i);
      expect(raw).toContain('Kind regards, Mariana <mariana@example.org>');
    }
  }
  // No refused inline style: stored HTML is parsed with its style attributes renamed and
  // applied through the CSSOM (richtext.ts inertBody, t67; Chromium logged ~65 before).
  // Refused requests (signed-out probes, the sign-in rate limit across runs) are the
  // server's answers, not page errors.
  const csp = errors.filter((e) => /Applying inline style violates|Content Security Policy/.test(e));
  info.annotations.push({ type: 'csp-style-reports', description: String(csp.length) });
  expect(csp).toEqual([]);
  expect(errors.filter((e) => !/favicon|ResizeObserver|Failed to load resource: the server responded with a status of/i.test(e))).toEqual([]);
});
