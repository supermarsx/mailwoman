// t39-e4: composer tools against the live receipts stack (no route mocks).
//   - L12 Dark background (Format -> View): the surface turns dark or white,
//     unreadable user colours are shown in a readable shade, and the HTML that
//     reaches SMTP is byte-for-byte the same with the button on or off; the
//     choice is stored per user and survives a reload;
//   - Check Names resolves a contact name and reports an unknown one, the From
//     button shows and hides the sender, and High importance reaches the wire
//     as `Importance: high` + `X-Priority: 1`;
//   - format painter, paragraph alignment and the clipboard buttons act on the
//     document;
//   - D4: below 600 px the composer ribbon starts in auto-hide while nobody has
//     chosen, an explicit pinned stays pinned, the body keeps most of the
//     height, and every ribbon tab has 0 geometry violations at 1440, 900 and
//     390 px.
// Runs in the receipts config (chromium, firefox, phone) as testuser.
import { test, expect, type Page, type Locator } from '../csrf.ts';
import { readFile } from 'node:fs/promises';
import path from 'node:path';

const capture = process.env['MW_DSN_SMTP_CAPTURE'] ?? 'test-results/receipts-smtp';
const shots = process.env['MW_T39_E4_SHOTS'];
type Accepted = { commands: string[]; sha256: string };
async function accepted(): Promise<Accepted[]> {
  try {
    return (await readFile(path.join(capture, 'accepted.jsonl'), 'utf8')).trim().split('\n').filter(Boolean).map((line) => JSON.parse(line) as Accepted);
  } catch {
    return [];
  }
}
async function shot(page: Page, name: string): Promise<void> {
  const info = test.info();
  await page.screenshot({ path: shots ? path.join(shots, `${info.project.name}-${name}.png`) : info.outputPath(`${name}.png`), fullPage: true });
}

const narrow = (page: Page): boolean => (page.viewportSize()?.width ?? 1280) < 600;

async function signIn(page: Page): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill('testuser');
  await page.getByLabel('Password', { exact: true }).fill('testpass');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('region', { name: 'Messages', exact: true })).toBeVisible();
  await page.waitForLoadState('networkidle');
}
async function openCompose(page: Page): Promise<Locator> {
  if (narrow(page)) await page.locator('header.shell__bar').getByRole('button', { name: 'Compose', exact: true }).click();
  else await page.locator('.sidebar__compose').click();
  const compose = page.locator('form.compose').last();
  await expect(compose.locator('.compose-rt__surface .ProseMirror')).toBeVisible();
  return compose;
}
const ribbon = (page: Page): Locator => page.locator('div[data-bar="composerRibbon"]').last();
// Shows a ribbon tab (opening an auto-hide ribbon), then unfolds `group` when `control` is folded into it.
async function reach(page: Page, tab: string, group: string, control: Locator): Promise<void> {
  const tabButton = ribbon(page).getByRole('tab', { name: tab, exact: true });
  if ((await tabButton.getAttribute('aria-selected')) !== 'true' || (await ribbon(page).getAttribute('data-expanded')) !== 'true') await tabButton.click();
  await expect(ribbon(page)).toHaveAttribute('data-expanded', 'true');
  await expect(tabButton).toHaveAttribute('aria-selected', 'true');
  const more = ribbon(page).getByRole('button', { name: `More ${group} commands`, exact: true });
  await expect(control.or(more).first()).toBeVisible();
  if (await control.isVisible()) return;
  await more.click();
  await expect(control).toBeVisible();
}
async function closeFolded(page: Page): Promise<void> {
  if (await page.locator('[data-ribbon-popover]:visible').count()) await page.keyboard.press('Escape');
}
async function setSource(page: Page, compose: Locator, html: string): Promise<void> {
  const toggle = compose.getByTestId('format-toggle');
  await reach(page, 'Format text', 'Message type', toggle);
  await toggle.selectOption('html');
  await compose.getByLabel('HTML source', { exact: true }).fill(html);
  await closeFolded(page);
  await reach(page, 'Format text', 'Message type', toggle);
  await toggle.selectOption('rich');
  await closeFolded(page);
}
async function send(_page: Page, compose: Locator): Promise<string> {
  const start = (await accepted()).length;
  await compose.getByRole('button', { name: 'Send', exact: true }).click();
  await expect.poll(async () => (await accepted()).length, { timeout: 35000 }).toBe(start + 1);
  const sent = (await accepted())[start]!;
  return readFile(path.join(capture, `${sent.sha256}.eml`), 'utf8');
}

/** The decoded `text/html` part of a raw message. */
function htmlPart(raw: string): string {
  const text = raw.replace(/\r\n/g, '\n');
  const at = text.search(/^Content-Type:\s*text\/html/im);
  expect(at, 'an HTML part').toBeGreaterThanOrEqual(0);
  const rest = text.slice(at);
  const split = rest.indexOf('\n\n');
  const headers = rest.slice(0, split);
  let body = rest.slice(split + 2);
  const boundary = body.search(/^--/m);
  if (boundary >= 0) body = body.slice(0, boundary);
  const encoding = /Content-Transfer-Encoding:\s*([\w-]+)/i.exec(headers)?.[1]?.toLowerCase();
  if (encoding === 'base64') return Buffer.from(body.replace(/\s+/g, ''), 'base64').toString('utf8');
  if (encoding === 'quoted-printable') {
    const bytes = body.replace(/=\n/g, '').replace(/=([0-9A-F]{2})/gi, (_, h: string) => String.fromCharCode(parseInt(h, 16)));
    return Buffer.from(bytes, 'latin1').toString('utf8');
  }
  return body;
}

// Mirrors components/bars/geometry.ts (assertRibbonGeometry), as bars-layout-live does.
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
    if (el.scrollHeight > el.clientHeight) out.push(`overflow ${el.tagName.toLowerCase()}.${String(el.className).split(' ')[0]} ${el.scrollHeight}/${el.clientHeight}`);
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

async function storageHeaders(page: Page): Promise<Record<string, string>> {
  const me = (await (await page.request.get('/api/me')).json()) as { storageAuthority: string };
  return { 'x-mailwoman-storage-authority': me.storageAuthority };
}
async function serverRibbon(page: Page): Promise<{ value: string; source: string }> {
  const body = (await (await page.request.get('/api/settings')).json()) as { settings?: Record<string, unknown> } & Record<string, unknown>;
  return ((body.settings ?? body) as Record<string, { value: string; source: string }>)['ui.bar.composerRibbon']!;
}
async function clearRibbon(page: Page): Promise<void> {
  await page.request.delete('/api/settings/ui.bar.composerRibbon', { headers: await storageHeaders(page) });
}

test.describe.configure({ mode: 'serial' });
test.beforeEach(async ({ page }) => {
  page.setDefaultTimeout(15000);
  await signIn(page);
  await clearRibbon(page);
  await page.reload();
  await expect(page.getByRole('region', { name: 'Messages', exact: true })).toBeVisible();
});
test.afterEach(async ({ page }) => {
  await clearRibbon(page).catch(() => undefined);
});

test('the dark background is view only: the sent HTML is the same with it on or off, and the choice is kept', async ({ page }, info) => {
  const body = '<p>Plain words</p><p><span style="color: #000000">ink</span> and <span style="color: #ffffff">chalk</span></p>';
  const button = (): Locator => ribbon(page).getByTestId('rt-dark-background');
  const surface = (compose: Locator): Locator => compose.locator('.compose-rt__surface');
  const tone = (compose: Locator) => surface(compose).evaluate((el) => {
    const [r, g, b] = getComputedStyle(el).backgroundColor.match(/\d+/g)!.map(Number) as [number, number, number];
    return { dark: 0.2126 * r + 0.7152 * g + 0.0722 * b < 100, adjusted: [...el.querySelectorAll('[data-mw-view-color]')].map((n) => n.textContent) };
  });
  const write = async (dark: boolean): Promise<string> => {
    const compose = await openCompose(page);
    await compose.getByLabel('To', { exact: true }).fill('dark-background@example.org');
    await compose.getByLabel('Subject', { exact: true }).fill(`Dark background ${dark ? 'on' : 'off'} ${info.project.name} ${Date.now()}`);
    await setSource(page, compose, body);
    await reach(page, 'Format text', 'View', button());
    if ((await button().getAttribute('aria-pressed')) !== String(dark)) await button().click();
    await expect(button()).toHaveAttribute('aria-pressed', String(dark));
    // data-tone is what the surface shows; data-surface is the stored choice, which
    // is 'theme' (follow the app, light here) until the first press on a new account.
    await expect(surface(compose)).toHaveAttribute('data-tone', dark ? 'dark' : 'light');
    await expect(surface(compose)).toHaveAttribute('data-surface', dark ? 'dark' : /^(light|theme)$/);
    await closeFolded(page);
    // Black text is the unreadable one on dark, white text on white; both are shown in a readable shade.
    await expect.poll(() => tone(compose)).toEqual({ dark, adjusted: [dark ? 'ink' : 'chalk'] });
    await shot(page, `dark-background-${dark ? 'on' : 'off'}`);
    return htmlPart(await send(page, compose));
  };
  // Off first, so both states are explicit choices.
  const off = await write(false);
  const on = await write(true);
  expect(on).toBe(off);
  expect(on).toContain('ink');
  expect(on).toMatch(/color:\s*(#ffffff|rgb\(255, 255, 255\))/i);
  expect(on).not.toContain('data-mw-view-color');
  expect(on).not.toMatch(/1e1f22|background(-color)?:\s*(#1e1f22|rgb\(30, 31, 34\))/i);
  // Stored per user: the appearance blob on the server, and the button after a reload.
  await expect.poll(async () => ((await (await page.request.get('/api/account/appearance')).json()) as { appearance?: { composeBackground?: string } }).appearance?.composeBackground).toBe('dark');
  await page.reload();
  await expect(page.getByRole('region', { name: 'Messages', exact: true })).toBeVisible();
  await openCompose(page);
  await reach(page, 'Format text', 'View', button());
  await expect(button()).toHaveAttribute('aria-pressed', 'true');
  await closeFolded(page);
});

test('Check Names, the From button and High importance', async ({ page }, info) => {
  const name = `Checked Name ${info.project.name} ${Date.now()}`;
  const email = `checked-${info.project.name}@example.org`;
  await page.goto('/#/contacts');
  await page.getByRole('button', { name: 'New contact', exact: true }).click();
  await page.getByLabel('Full name', { exact: true }).fill(name);
  await page.getByLabel('Email 1', { exact: true }).fill(email);
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByRole('article', { name: `Contact ${name}`, exact: true })).toBeVisible();
  await page.goto('/#/mail');
  await expect(page.getByRole('region', { name: 'Messages', exact: true })).toBeVisible();

  const compose = await openCompose(page);
  const to = compose.getByLabel('To', { exact: true });
  const check = ribbon(page).getByRole('button', { name: 'Check names', exact: true });
  // An unknown name is reported and left as typed.
  await to.fill('Nobody Known At All');
  await reach(page, 'Format text', 'Recipients', check);
  await check.click();
  await expect(compose.getByRole('alert').filter({ hasText: 'Choose a full email address' })).toContainText('Nobody Known At All');
  await expect(to).toHaveValue('Nobody Known At All');
  // A contact's name resolves to "Name" <address>.
  await closeFolded(page);
  await to.fill(name);
  await reach(page, 'Format text', 'Recipients', check);
  await check.click();
  await expect(to).toHaveValue(`"${name}" <${email}>`);
  await expect(page.getByText('1 contact name(s) resolved', { exact: false })).toBeVisible();

  // From: hidden by default, shown and hidden again by the button.
  const fromButton = ribbon(page).getByRole('button', { name: 'From', exact: true });
  const fromField = compose.locator('label.compose__from');
  await expect(fromField).toHaveCount(0);
  await reach(page, 'Format text', 'Recipients', fromButton);
  await fromButton.click();
  await expect(fromButton).toHaveAttribute('aria-pressed', 'true');
  await expect(fromField.first()).toBeVisible();
  await shot(page, 'from-shown');
  await reach(page, 'Format text', 'Recipients', fromButton);
  await fromButton.click();
  await expect(fromButton).toHaveAttribute('aria-pressed', 'false');
  await expect(fromField).toHaveCount(0);
  await closeFolded(page);

  // High importance reaches the wire.
  const subject = `Importance ${info.project.name} ${Date.now()}`;
  await compose.getByLabel('Subject', { exact: true }).fill(subject);
  await compose.locator('.ProseMirror').fill('Important body');
  const importance = ribbon(page).getByRole('combobox', { name: 'Importance', exact: true });
  await reach(page, 'Delivery', 'Delivery', importance);
  await importance.selectOption('high');
  await closeFolded(page);
  await shot(page, 'importance-high');
  const raw = await send(page, compose);
  expect(raw).toContain(subject);
  expect(raw).toMatch(/^Importance:\s*high\s*$/im);
  expect(raw).toMatch(/^X-Priority:\s*1\b/im);
  expect(raw).toMatch(new RegExp(`^To:.*${email.replace(/[.]/g, '\\.')}`, 'im'));
});

test('format painter, alignment and the clipboard buttons act on the document', async ({ page, context, browserName }, info) => {
  if (browserName === 'chromium') await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  // These legs are about the tools, not auto-hide: pin the ribbon (390 px starts in auto, D4).
  const pinned = await page.request.put('/api/settings/ui.bar.composerRibbon', { data: { value: 'pinned' }, headers: await storageHeaders(page) });
  expect(pinned.status()).toBe(200);
  await page.reload();
  await expect(page.getByRole('region', { name: 'Messages', exact: true })).toBeVisible();
  const compose = await openCompose(page);
  await expect(ribbon(page)).toHaveAttribute('data-mode', 'pinned');
  const editor = compose.locator('.ProseMirror');
  await editor.click();
  await page.keyboard.type('alpha beta');
  const select = async (from: number, to: number): Promise<void> => {
    await editor.evaluate((el, [a, b]) => {
      const text = el.querySelector('p')!.firstChild ?? el.querySelector('p')!;
      const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
      const nodes: Text[] = [];
      for (let n = walker.nextNode(); n; n = walker.nextNode()) nodes.push(n as Text);
      const locate = (offset: number): [Node, number] => {
        let left = offset;
        for (const n of nodes) {
          if (left <= n.length) return [n, left];
          left -= n.length;
        }
        return [text, 0];
      };
      const range = document.createRange();
      range.setStart(...locate(a!));
      range.setEnd(...locate(b!));
      const sel = getSelection()!;
      sel.removeAllRanges();
      sel.addRange(range);
    }, [from, to]);
    await page.waitForTimeout(100);
  };
  // Painter: bold "alpha", copy its formatting, apply it to "beta".
  await select(0, 5);
  const bold = ribbon(page).getByRole('button', { name: 'Bold', exact: true });
  await reach(page, 'Format text', 'Font', bold);
  await bold.click();
  await closeFolded(page);
  await expect(editor.locator('strong')).toHaveText('alpha');
  await select(1, 3);
  const copyFormat = ribbon(page).getByRole('button', { name: 'Copy formatting', exact: true });
  await reach(page, 'Format text', 'Clipboard', copyFormat);
  await copyFormat.click();
  await closeFolded(page);
  await select(6, 10);
  const applyFormat = ribbon(page).getByRole('button', { name: 'Apply formatting', exact: true });
  await reach(page, 'Format text', 'Clipboard', applyFormat);
  await expect(applyFormat).toBeEnabled();
  await applyFormat.click();
  await closeFolded(page);
  await expect(editor.locator('strong')).toHaveText(['alpha', 'beta']);

  // Alignment: centre the paragraph.
  const options = ribbon(page).getByRole('button', { name: 'Paragraph options', exact: true });
  await reach(page, 'Format text', 'Paragraph', options);
  await options.click();
  await page.getByLabel('Alignment', { exact: true }).selectOption('center');
  await expect(editor.locator('p').first()).toHaveCSS('text-align', 'center');
  await page.keyboard.press('Escape');
  await closeFolded(page);

  // Clipboard: copy "alpha", cut "beta", paste at the end.
  const button = (name: string): Locator => ribbon(page).getByRole('button', { name, exact: true });
  await select(0, 5);
  await reach(page, 'Format text', 'Clipboard', button('Copy'));
  await button('Copy').click();
  await closeFolded(page);
  await select(6, 10);
  await reach(page, 'Format text', 'Clipboard', button('Cut'));
  await button('Cut').click();
  await closeFolded(page);
  await expect(editor).toHaveText('alpha ');
  if (browserName === 'chromium') {
    await select(6, 6);
    await reach(page, 'Format text', 'Clipboard', button('Paste'));
    await button('Paste').click();
    await closeFolded(page);
    await expect(editor).toHaveText('alpha beta');
    await expect(editor.locator('strong')).toHaveText(['alpha', 'beta']);
  } else {
    info.annotations.push({ type: 'note', description: 'Paste from the page needs a clipboard-read grant, which Playwright gives only to Chromium.' });
  }
  await expect(compose.locator('.compose-rt__error')).toHaveCount(0);
  await shot(page, 'painter-align-clipboard');
});

test('D4: the ribbon starts in auto-hide below 600 px, an explicit pinned stays, and no tab has geometry violations', async ({ page }, info) => {
  const widths = info.project.name === 'phone' ? [390] : [1440, 900, 390];
  const height = page.viewportSize()!.height;
  const tabsChecked = async (where: string): Promise<void> => {
    const tabs = ribbon(page).locator('[data-bar-header] [role=tab]');
    const count = await tabs.count();
    expect(count).toBe(5);
    for (let i = 0; i < count; i++) {
      await tabs.nth(i).click();
      await expect(ribbon(page)).toHaveAttribute('data-expanded', 'true');
      await page.waitForTimeout(300);
      expect(await ribbon(page).evaluate(ribbonViolations), `${where} tab ${i + 1}`).toEqual([]);
    }
  };
  const measure = () => page.evaluate(() => {
    const bar = document.querySelectorAll('div[data-bar="composerRibbon"]');
    const r = bar[bar.length - 1]!.getBoundingClientRect();
    const field = document.querySelector('.compose > .field--grow')!.getBoundingClientRect();
    return { ribbon: Math.round(r.height), body: Math.round(field.height), viewport: innerHeight };
  });
  for (const width of widths) {
    await page.setViewportSize({ width, height: width < 600 ? 844 : height });
    await page.reload();
    await expect(page.getByRole('region', { name: 'Messages', exact: true })).toBeVisible();
    await openCompose(page);
    expect(await serverRibbon(page)).toMatchObject({ value: 'pinned', source: 'default' });
    await expect(ribbon(page)).toHaveAttribute('data-mode', width < 600 ? 'auto' : 'pinned');
    if (width < 600) {
      await page.locator('.ProseMirror').last().click();
      await expect(ribbon(page)).toHaveAttribute('data-expanded', 'false');
      const collapsed = await measure();
      info.annotations.push({ type: 'measure', description: `${width}px auto: ${JSON.stringify(collapsed)}` });
      expect(collapsed.body).toBeGreaterThan(collapsed.viewport / 2);
      await shot(page, `ribbon-${width}-auto`);
    }
    await tabsChecked(`${width}px ${width < 600 ? 'auto' : 'pinned'}`);
    if (width < 600) {
      // An explicit pinned wins over the narrow default, after a reload too.
      const saved = page.waitForResponse((r) => r.request().method() === 'PUT' && r.url().includes('/api/settings/ui.bar.composerRibbon'));
      await page.locator('[data-bar-pin="composerRibbon"]').last().click();
      expect((await saved).status()).toBe(200);
      await expect(ribbon(page)).toHaveAttribute('data-mode', 'pinned');
      expect(await serverRibbon(page)).toMatchObject({ value: 'pinned', source: 'user' });
      await page.reload();
      await expect(page.getByRole('region', { name: 'Messages', exact: true })).toBeVisible();
      await openCompose(page);
      await expect(ribbon(page)).toHaveAttribute('data-mode', 'pinned');
      const pinned = await measure();
      info.annotations.push({ type: 'measure', description: `${width}px pinned: ${JSON.stringify(pinned)}` });
      await tabsChecked(`${width}px pinned (explicit)`);
      await shot(page, `ribbon-${width}-pinned-explicit`);
      await clearRibbon(page);
    }
  }
});
