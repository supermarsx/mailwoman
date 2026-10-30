// t38: the four collapsible bars (mail top bar, workspace tabs, mail ribbon,
// composer ribbon) against a live server, with no route mocks:
//   - each bar starts pinned; unpinning stores `ui.bar.<id>=auto` on the server,
//     which a reload and a second browser context (another device) both read;
//   - an administrator's default applies, and a locked bar cannot be changed
//     (the pin is disabled, the settings route answers 403, Settings shows it);
//   - every mail and composer ribbon tab keeps each group exactly three rows
//     high, with no element taller than its box (no vertical overflow);
//   - the signed-in address appears once, in the sidebar, never in the top bar;
//   - an auto-hide bar opens on focus or tap, Esc closes it and returns focus;
//   - the composer body still fills its height in every mode.
// Runs in the receipts config (chromium, firefox, phone) against
// MW_RECEIPTS_BASE_URL as parent@example.org; the admin test needs the server's
// admin credentials (MW_ADMIN_USER / MW_ADMIN_PASSWORD, default root / hunter2).
import { test, expect, type Page } from '../csrf.ts';

const USER = { name: 'parent@example.org', pass: 'parent-pass' };
const BARS = ['mailTopBar', 'workspaceTabs', 'mailRibbon', 'composerRibbon'] as const;
type BarId = (typeof BARS)[number];
const ADMIN = { username: process.env['MW_ADMIN_USER'] ?? 'root', password: process.env['MW_ADMIN_PASSWORD'] ?? 'hunter2' };

const touch = (page: Page): boolean => (page.viewportSize()?.width ?? 1280) < 600;
const bar = (page: Page, id: BarId) => page.locator(`div[data-bar="${id}"]`);

async function signIn(page: Page): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill(USER.name);
  await page.getByLabel('Password', { exact: true }).fill(USER.pass);
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(bar(page, 'mailTopBar')).toBeAttached();
  await page.waitForLoadState('networkidle');
}

async function settingsHeaders(page: Page): Promise<Record<string, string>> {
  const me = (await (await page.request.get('/api/me')).json()) as { storageAuthority: string };
  return { 'x-mailwoman-storage-authority': me.storageAuthority };
}
async function serverBar(page: Page, id: BarId): Promise<{ value: string; source: string; locked: boolean }> {
  const body = (await (await page.request.get('/api/settings')).json()) as { settings?: Record<string, unknown> } & Record<string, unknown>;
  return ((body.settings ?? body) as Record<string, { value: string; source: string; locked: boolean }>)[`ui.bar.${id}`]!;
}
async function clearBars(page: Page): Promise<void> {
  const headers = await settingsHeaders(page);
  for (const id of BARS) await page.request.delete(`/api/settings/ui.bar.${id}`, { headers });
}

async function openDrawer(page: Page): Promise<void> {
  const menu = page.locator('header.shell__bar button[aria-controls="shell-nav"]').first();
  if ((await menu.isVisible()) && (await menu.getAttribute('aria-expanded')) !== 'true') await menu.click();
}
async function setLayout(page: Page, layout: 'Ribbon' | 'Default'): Promise<void> {
  if (touch(page)) await openDrawer(page);
  await page.locator('.sidebar, #shell-nav').getByRole('button', { name: 'Settings', exact: true }).first().click();
  await page.getByRole('navigation', { name: 'Settings sections' }).getByRole('button', { name: 'Appearance', exact: true }).click();
  await page.getByRole('group', { name: 'Layout' }).getByRole('button', { name: layout, exact: true }).click();
  await backToMail(page);
}
async function backToMail(page: Page): Promise<void> {
  const mail = bar(page, 'workspaceTabs').getByRole('tab', { name: 'Mail', exact: true });
  if (await mail.isVisible()) await mail.click();
  else {
    await openDrawer(page);
    await page.locator('#shell-nav').getByRole('button', { name: /^Inbox/ }).first().click();
  }
  if (touch(page)) await page.keyboard.press('Escape');
}
async function openCompose(page: Page): Promise<void> {
  if (touch(page)) await page.locator('header.shell__bar').getByRole('button', { name: 'Compose', exact: true }).click();
  else await page.locator('.sidebar__compose').click();
  await expect(page.locator('.compose-rt__surface .ProseMirror')).toBeVisible();
}
async function gotoBar(page: Page, id: BarId): Promise<void> {
  if (id === 'composerRibbon') await openCompose(page);
}
async function press(page: Page, locator: ReturnType<Page['locator']>): Promise<void> {
  if (touch(page)) await locator.tap();
  else await locator.click();
}
// Reveals an auto-hide bar, so its pin is reachable: the slim handle, or a ribbon tab.
async function reveal(page: Page, id: BarId): Promise<void> {
  if ((await bar(page, id).getAttribute('data-expanded')) === 'true') return;
  const handle = page.locator(`[data-bar-handle="${id}"]`);
  await press(page, (await handle.count()) ? handle : bar(page, id).locator('[data-bar-header] [role=tab]').first());
  await expect(bar(page, id)).toHaveAttribute('data-expanded', 'true');
}
async function togglePin(page: Page, id: BarId, to: 'pinned' | 'auto'): Promise<void> {
  await reveal(page, id);
  const saved = page.waitForResponse((r) => r.request().method() === 'PUT' && r.url().includes(`/api/settings/ui.bar.${id}`));
  await press(page, page.locator(`[data-bar-pin="${id}"]`));
  expect((await saved).status()).toBe(200);
  await expect(bar(page, id)).toHaveAttribute('data-mode', to);
}
async function leave(page: Page): Promise<void> {
  const size = page.viewportSize()!;
  await page.mouse.move(size.width / 2, size.height - 10);
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur?.());
}

// Mirrors components/bars/geometry.ts (assertRibbonGeometry), plus the strict
// rule: no rendered element under the ribbon may be taller inside than its box.
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

test.describe.configure({ mode: 'serial' });

test.beforeEach(async ({ page }) => {
  page.setDefaultTimeout(15000);
  await signIn(page);
  await clearBars(page);
});

test('each bar starts pinned, and pin or auto-hide is stored on the server for every device', async ({ page, browser }, info) => {
  await setLayout(page, 'Ribbon');
  await page.reload();
  for (const id of BARS) expect(await serverBar(page, id)).toMatchObject({ value: 'pinned', source: 'default', locked: false });
  const other = await browser.newContext({ baseURL: String(info.project.use.baseURL), viewport: page.viewportSize(), hasTouch: info.project.use.hasTouch === true, locale: 'en-US', serviceWorkers: 'block' });
  const device = await other.newPage();
  await signIn(device);
  for (const id of BARS) {
    await gotoBar(page, id);
    // t39 D4 (9dab0fe7): below 600 px the composer ribbon starts auto-hidden while
    // its value is still the registry default; an explicit choice wins. Pin it first.
    if (id === 'composerRibbon' && touch(page)) {
      await expect(bar(page, id)).toHaveAttribute('data-mode', 'auto');
      await togglePin(page, id, 'pinned');
    }
    await expect(bar(page, id)).toHaveAttribute('data-mode', 'pinned');
    await togglePin(page, id, 'auto');
    expect(await serverBar(page, id)).toMatchObject({ value: 'auto', source: 'user' });
    await leave(page);
    await expect(bar(page, id)).toHaveAttribute('data-expanded', 'false');
    await page.reload();
    await gotoBar(page, id);
    await expect(bar(page, id)).toHaveAttribute('data-mode', 'auto');
    await device.reload();
    await gotoBar(device, id);
    await expect(bar(device, id)).toHaveAttribute('data-mode', 'auto');
    await togglePin(device, id, 'pinned');
    await page.reload();
    await gotoBar(page, id);
    await expect(bar(page, id)).toHaveAttribute('data-mode', 'pinned');
    await expect(bar(page, id)).toHaveAttribute('data-expanded', 'true');
  }
  await other.close();
  await setLayout(page, 'Default');
});

test('an administrator default applies, and a locked bar cannot be changed', async ({ page, playwright }) => {
  const admin = await playwright.request.newContext({ baseURL: String(test.info().project.use.baseURL) });
  const login = await admin.post('/admin/login', { data: ADMIN });
  expect(login.status(), 'configured admin fixture must sign in').toBe(200);
  const put = async (key: string, data: Record<string, unknown>): Promise<number> => {
    const etag = (await admin.get(`/admin/settings/${key}`)).headers()['etag'];
    return (await admin.put(`/admin/settings/${key}`, { data, headers: etag ? { 'if-match': etag } : {} })).status();
  };
  try {
    expect(await put('ui.bar.workspaceTabs', { globalValue: 'auto' })).toBe(200);
    expect(await put('ui.bar.mailTopBar', { globalValue: 'auto', locked: true })).toBe(200);
    await page.reload();
    await leave(page);
    await expect(bar(page, 'workspaceTabs')).toHaveAttribute('data-mode', 'auto');
    await expect(bar(page, 'mailTopBar')).toHaveAttribute('data-mode', 'auto');
    expect(await serverBar(page, 'mailTopBar')).toMatchObject({ value: 'auto', locked: true });
    // The unlocked default can still be overridden.
    await togglePin(page, 'workspaceTabs', 'pinned');
    // The locked bar: a disabled pin that says why, and the route refuses a write.
    await reveal(page, 'mailTopBar');
    const pin = page.locator('[data-bar-pin="mailTopBar"]');
    await expect(pin).toBeDisabled();
    await expect(pin).toHaveAttribute('data-tooltip', 'Set by your administrator');
    const refused = await page.request.put('/api/settings/ui.bar.mailTopBar', { data: { value: 'pinned' }, headers: await settingsHeaders(page) });
    expect(refused.status()).toBe(403);
    await leave(page);
    if (touch(page)) await openDrawer(page);
    await page.locator('.sidebar, #shell-nav').getByRole('button', { name: 'Settings', exact: true }).first().click();
    await page.getByRole('navigation', { name: 'Settings sections' }).getByRole('button', { name: 'Appearance', exact: true }).click();
    const locked = page.getByTestId('settings-bars').getByRole('group', { name: 'Mail toolbar' });
    await expect(locked.getByRole('radio', { name: 'Auto-hide' })).toBeChecked();
    await expect(locked.getByRole('radio', { name: 'Auto-hide' })).toBeDisabled();
    await expect(locked.getByRole('radio', { name: 'Always shown' })).toBeDisabled();
    await expect(locked.getByText('Set by your administrator')).toBeVisible();
    await expect(page.getByTestId('settings-bars').getByRole('group', { name: 'Workspace tabs' }).getByRole('radio', { name: 'Always shown' })).toBeEnabled();
  } finally {
    await put('ui.bar.mailTopBar', { globalValue: null, locked: false });
    await put('ui.bar.workspaceTabs', { globalValue: null });
    await clearBars(page);
    await admin.dispose();
  }
});

// t39 C5: a domain-scoped default. `auto` + lock for example.org reaches
// parent@example.org (auto, locked, write refused) and not child@child.test,
// which keeps the global default (pinned, unlocked, its own write accepted).
test('a domain default and lock apply to that domain only', async ({ page, browser, playwright }, info) => {
  const admin = await playwright.request.newContext({ baseURL: String(info.project.use.baseURL) });
  const login = await admin.post('/admin/login', { data: ADMIN });
  expect(login.status(), 'configured admin fixture must sign in').toBe(200);
  const key = 'ui.bar.mailRibbon';
  const other = await browser.newContext({ baseURL: String(info.project.use.baseURL), viewport: page.viewportSize(), hasTouch: info.project.use.hasTouch === true, locale: 'en-US', serviceWorkers: 'block' });
  const child = await other.newPage();
  child.setDefaultTimeout(15000);
  try {
    const set = await admin.put(`/admin/settings/${key}/domains/example.org`, { data: { value: 'auto', locked: true } });
    expect(set.status(), await set.text()).toBe(200);
    await setLayout(page, 'Ribbon');
    await page.reload();
    await leave(page);
    expect(await serverBar(page, 'mailRibbon')).toMatchObject({ value: 'auto', locked: true });
    expect((await serverBar(page, 'mailRibbon')).source).not.toBe('default');
    await expect(bar(page, 'mailRibbon')).toHaveAttribute('data-mode', 'auto');
    await reveal(page, 'mailRibbon');
    await expect(page.locator('[data-bar-pin="mailRibbon"]')).toBeDisabled();
    const refused = await page.request.put(`/api/settings/${key}`, { data: { value: 'pinned' }, headers: await settingsHeaders(page) });
    expect(refused.status()).toBe(403);
    expect(await refused.text()).toContain('setting_locked');
    await leave(page);

    await child.goto('/');
    await child.getByLabel('Email or username', { exact: true }).fill('child@child.test');
    await child.getByLabel('Password', { exact: true }).fill('child-pass');
    await child.getByRole('button', { name: 'Sign in', exact: true }).click();
    await expect(bar(child, 'mailTopBar')).toBeAttached();
    await child.waitForLoadState('networkidle');
    expect(await serverBar(child, 'mailRibbon')).toMatchObject({ value: 'pinned', source: 'default', locked: false });
    await setLayout(child, 'Ribbon');
    await child.reload();
    await expect(bar(child, 'mailRibbon')).toHaveAttribute('data-mode', 'pinned');
    await expect(child.locator('[data-bar-pin="mailRibbon"]')).toBeEnabled();
    // A context from browser.newContext() is not covered by the csrf.ts fixture: repeat mw_csrf by hand.
    const csrf = decodeURIComponent((await other.cookies()).find((c) => c.name === 'mw_csrf')?.value ?? '');
    const childHeaders = { ...(await settingsHeaders(child)), 'x-csrf-token': csrf };
    const allowed = await child.request.put(`/api/settings/${key}`, { data: { value: 'auto' }, headers: childHeaders });
    expect(allowed.status(), await allowed.text()).toBe(200);
    expect(await serverBar(child, 'mailRibbon')).toMatchObject({ value: 'auto', source: 'user', locked: false });
    expect((await child.request.delete(`/api/settings/${key}`, { headers: childHeaders })).status()).toBe(200);
    await setLayout(child, 'Default');
  } finally {
    await admin.delete(`/admin/settings/${key}/domains/example.org`);
    await setLayout(page, 'Default');
    await clearBars(page);
    await other.close();
    await admin.dispose();
  }
});

test('every mail and composer ribbon tab keeps three-row groups with no vertical overflow', async ({ page }) => {
  await setLayout(page, 'Ribbon');
  const check = async (id: BarId): Promise<number> => {
    const tabs = bar(page, id).locator('[data-bar-header] [role=tab], [role=tablist] [role=tab]');
    const count = await tabs.count();
    expect(count).toBeGreaterThan(0);
    for (let i = 0; i < count; i++) {
      await press(page, tabs.nth(i));
      await expect(tabs.nth(i)).toHaveAttribute('aria-selected', 'true');
      await page.waitForTimeout(300); // the fold pass settles in one animation frame or two
      expect(await bar(page, id).evaluate(ribbonViolations), `${id} tab ${i + 1}`).toEqual([]);
      // Every folded group's dropdown holds the same three-row grid.
      const more = bar(page, id).locator('[data-ribbon-more]').filter({ visible: true });
      for (let m = 0; m < (await more.count()); m++) {
        await more.nth(m).focus();
        await page.keyboard.press('Enter');
        await expect(more.nth(m)).toHaveAttribute('aria-expanded', 'true');
        expect(await bar(page, id).evaluate(ribbonViolations), `${id} tab ${i + 1} dropdown ${m + 1}`).toEqual([]);
        await page.keyboard.press('Escape');
        await expect(more.nth(m)).toBeFocused();
      }
    }
    return count;
  };
  expect(await check('mailRibbon')).toBe(3);
  await openCompose(page);
  expect(await check('composerRibbon')).toBe(5);
  await backToMail(page);
  await setLayout(page, 'Default');
});

test('the signed-in address shows once, in the sidebar, never in the top bar', async ({ page }) => {
  const visible = (address: string) => page.evaluate((address) => {
    const shown = (e: Element): boolean => {
      const r = e.getBoundingClientRect();
      return r.width > 0 && r.height > 0 && getComputedStyle(e).visibility !== 'hidden' && !e.closest('[hidden],[inert]') && r.right > 0 && r.left < innerWidth;
    };
    const hits: string[] = [];
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    for (let n = walker.nextNode(); n; n = walker.nextNode()) {
      const el = n.parentElement;
      if (el && n.textContent?.includes(address) && shown(el) && !el.closest('option, select')) hits.push(el.closest('.sidebar__head') ? 'sidebar' : el.closest('[data-bar]') ? 'bar' : 'other');
    }
    for (const s of document.querySelectorAll('select')) if (shown(s) && s.selectedOptions[0]?.textContent?.includes(address)) hits.push(s.closest('.sidebar__head') ? 'sidebar' : 'other');
    return hits;
  }, address);
  await expect(page.getByRole('toolbar', { name: 'Workspace layout' })).not.toContainText('@');
  if (touch(page)) {
    expect(await visible(USER.name)).toEqual([]);
    await openDrawer(page);
  }
  await expect.poll(() => visible(USER.name)).toEqual(['sidebar']);
});

test('an auto-hide bar opens on focus or tap, and Esc closes it and returns focus', async ({ page }) => {
  for (const id of ['mailTopBar', 'workspaceTabs'] as const) {
    await togglePin(page, id, 'auto');
    await leave(page);
    await expect(bar(page, id)).toHaveAttribute('data-expanded', 'false');
    const handle = page.locator(`[data-bar-handle="${id}"]`);
    if (touch(page)) {
      await handle.tap();
      await expect(bar(page, id)).toHaveAttribute('data-expanded', 'true');
      await page.touchscreen.tap(195, (page.viewportSize()?.height ?? 844) - 40);
      await expect(bar(page, id)).toHaveAttribute('data-expanded', 'false');
    } else {
      await handle.focus();
      await expect(bar(page, id)).toHaveAttribute('data-expanded', 'true');
      await page.keyboard.press('Tab');
      await expect(bar(page, id).locator(':focus')).toHaveCount(1);
      await page.keyboard.press('Escape');
      await expect(bar(page, id)).toHaveAttribute('data-expanded', 'false');
      await expect(handle).toBeFocused();
    }
    await togglePin(page, id, 'pinned');
  }
});

test('the composer body fills its height pinned, collapsed and opened', async ({ page }) => {
  await openCompose(page);
  // t39 D4: on a phone the composer ribbon starts auto-hidden; pin it to measure the pinned case.
  if (touch(page)) await togglePin(page, 'composerRibbon', 'pinned');
  const measure = () => page.evaluate(() => {
    const ribbon = document.querySelector('div[data-bar="composerRibbon"]')!.getBoundingClientRect();
    const compose = document.querySelector('form.compose')!;
    const field = document.querySelector('.compose > .field--grow')!.getBoundingClientRect();
    const end = compose.getBoundingClientRect().bottom - (parseFloat(getComputedStyle(compose).paddingBottom) || 0);
    return { fills: Math.abs(field.bottom - end) <= 1, covered: ribbon.bottom > field.top + 0.5, height: field.height };
  });
  const pinned = await measure();
  await togglePin(page, 'composerRibbon', 'auto');
  await page.locator('.compose-rt__surface .ProseMirror').click();
  await leave(page);
  await expect(bar(page, 'composerRibbon')).toHaveAttribute('data-expanded', 'false');
  const collapsed = await measure();
  await press(page, bar(page, 'composerRibbon').getByRole('tab', { name: 'Insert', exact: true }));
  await expect(bar(page, 'composerRibbon')).toHaveAttribute('data-expanded', 'true');
  const opened = await measure();
  await togglePin(page, 'composerRibbon', 'pinned');
  for (const m of [pinned, collapsed, opened]) expect(m).toMatchObject({ fills: true, covered: false });
  expect(collapsed.height).toBeGreaterThan(pinned.height);
});
