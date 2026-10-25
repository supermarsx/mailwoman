import AxeBuilder from '@axe-core/playwright';
import { test, expect, type Page } from '@playwright/test';

/**
 * The admin console's own appearance and the UX seam's shell parts in a real browser
 * (t32-cfg §4.7, t32-e71a; UX1, UX12-UX15, DC5).
 *
 *  - A choice in the header's Appearance dialog (theme, accent, mode) changes the console
 *    at once, survives a reload (`mw.admin.appearance`), and leaves the webmail's stored
 *    appearance alone.
 *  - Every built-in theme (18) renders the console shell with zero axe WCAG 2.2 AA
 *    violations, colour contrast included, at 1440x900 and at 390x844, with no horizontal
 *    page scroll on the phone.
 *  - The Appearance dialog itself: axe-clean in a dark, a light and a high-contrast theme,
 *    under forced colours, and right-to-left on the phone without horizontal scroll;
 *    Escape returns focus to the button.
 *
 * Runs against an engine-mode server whose admin panel accepts MW_ADMIN_USER /
 * MW_ADMIN_PASSWORD (default root / hunter2, the dev and CI stacks' values; DEV/CI ONLY).
 * On a stack that refuses them the tests skip and say so.
 */

const ADMIN_USER = process.env['MW_ADMIN_USER'] ?? 'root';
const ADMIN_PASSWORD = process.env['MW_ADMIN_PASSWORD'] ?? 'hunter2';
const WCAG_22_AA = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];

const THEMES = [
  'light',
  'dark',
  'hc-light',
  'hc-dark',
  'amoled',
  'grove-light',
  'grove-dark',
  'slate-light',
  'slate-dark',
  'ocean-light',
  'ocean-dark',
  'plum-light',
  'plum-dark',
  'bauhaus-black',
  'paper',
  'blueprint',
  'terminal',
  'primary',
] as const;

/** The packs that paint a light page (the rest are dark), so each is stored with its own mode. */
const LIGHT = new Set<string>(['light', 'hc-light', 'grove-light', 'slate-light', 'ocean-light', 'plum-light', 'paper', 'primary']);

const DESKTOP = { width: 1440, height: 900 };
const PHONE = { width: 390, height: 844 };

async function signIn(page: Page): Promise<void> {
  const res = await page.request.post('/admin/login', { data: { username: ADMIN_USER, password: ADMIN_PASSWORD } });
  test.skip(res.status() === 401, 'the admin panel refuses MW_ADMIN_USER/MW_ADMIN_PASSWORD on this stack');
  expect(res.status(), 'admin login').toBe(200);
}

async function openConsole(page: Page): Promise<void> {
  await page.goto('/admin');
  await expect(page.locator('[data-screen="admin"]')).toBeVisible();
  await expect(page.getByRole('heading', { level: 2, name: 'Dashboard' })).toBeVisible();
}

/** The Appearance button that is on screen (header on desktop, top bar on a phone). */
function appearanceButton(page: Page) {
  return page.locator('[data-testid="admin-appearance-button"]:visible');
}

async function axe(page: Page, label: string, include?: string): Promise<void> {
  let builder = new AxeBuilder({ page }).withTags(WCAG_22_AA);
  if (include) builder = builder.include(include);
  const v = (await builder.analyze()).violations;
  const lines = v.map((r) => `  [${r.impact ?? 'n/a'}] ${r.id}: ${r.nodes.slice(0, 4).map((n) => n.target.join(' ')).join(' | ')}`);
  if (v.length > 0) console.error(`axe [${label}] ${v.length} violation(s):\n${lines.join('\n')}`);
  expect(lines, `axe violations on ${label}`).toEqual([]);
}

async function noHorizontalScroll(page: Page, label: string): Promise<void> {
  const { scroll, width } = await page.evaluate(() => ({ scroll: document.documentElement.scrollWidth, width: window.innerWidth }));
  expect(scroll, `horizontal page scroll on ${label}`).toBeLessThanOrEqual(width);
}

test.beforeEach(async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await signIn(page);
});

test.describe('console appearance (DC5)', () => {
  test('a choice applies at once, survives a reload and leaves the webmail appearance alone', async ({ page }) => {
    await page.setViewportSize(DESKTOP);
    await openConsole(page);
    await page.evaluate(() => localStorage.removeItem('mw.admin.appearance'));
    const webmailBefore = await page.evaluate(() => localStorage.getItem('mw.theme.prefs'));

    await appearanceButton(page).click();
    const dialog = page.getByRole('dialog', { name: 'Console appearance' });
    await expect(dialog).toBeVisible();
    await dialog.locator('[data-theme-option="paper"]').click();
    await dialog.getByRole('button', { name: 'Teal' }).click();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'paper');
    await expect(page.locator('html')).toHaveAttribute('data-accent', 'teal');
    await dialog.getByRole('radio', { name: 'Dark' }).check();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'blueprint');
    await dialog.getByRole('button', { name: 'Close' }).click();
    await expect(dialog).toBeHidden();
    await expect(appearanceButton(page)).toBeFocused();

    await page.reload();
    await expect(page.getByRole('heading', { level: 2, name: 'Dashboard' })).toBeVisible();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'blueprint');
    await expect(page.locator('html')).toHaveAttribute('data-accent', 'teal');
    expect(await page.evaluate(() => JSON.parse(localStorage.getItem('mw.admin.appearance') ?? 'null'))).toEqual({
      theme: 'paper',
      accent: 'teal',
      mode: 'dark',
    });
    expect(await page.evaluate(() => localStorage.getItem('mw.theme.prefs'))).toBe(webmailBefore);

    // "Use the webmail theme" forgets the choice: back to the default (Dark here).
    await appearanceButton(page).click();
    await dialog.getByRole('button', { name: 'Use the webmail theme' }).click();
    await page.keyboard.press('Escape');
    await expect(appearanceButton(page)).toBeFocused();
    expect(await page.evaluate(() => localStorage.getItem('mw.admin.appearance'))).toBeNull();
  });

  test('"Match the system" follows the browser colour scheme', async ({ page }) => {
    await page.setViewportSize(DESKTOP);
    await page.emulateMedia({ colorScheme: 'light' });
    await page.goto('/admin');
    await page.evaluate(() => localStorage.setItem('mw.admin.appearance', JSON.stringify({ theme: 'grove-dark', accent: '', mode: 'system' })));
    await openConsole(page);
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'grove-light');
    await page.emulateMedia({ colorScheme: 'dark' });
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'grove-dark');
  });
});

for (const [label, viewport] of [
  ['desktop', DESKTOP],
  ['phone', PHONE],
] as const) {
  test.describe(`every theme renders the shell without axe violations (${label})`, () => {
    test(`18 themes at ${viewport.width}x${viewport.height}`, async ({ page }) => {
      test.setTimeout(180_000);
      await page.setViewportSize(viewport);
      await page.goto('/admin');
      for (const theme of THEMES) {
        const choice = { theme, accent: '', mode: LIGHT.has(theme) ? 'light' : 'dark' };
        await page.evaluate((c) => localStorage.setItem('mw.admin.appearance', JSON.stringify(c)), choice);
        await openConsole(page);
        await expect(page.locator('html')).toHaveAttribute('data-theme', theme);
        await axe(page, `${label} ${theme}`);
        if (label === 'phone') await noHorizontalScroll(page, `${label} ${theme}`);
      }
    });
  });
}

test.describe('the Appearance dialog', () => {
  for (const theme of ['dark', 'light', 'hc-dark'] as const) {
    test(`axe-clean with the dialog open in ${theme}`, async ({ page }) => {
      await page.setViewportSize(DESKTOP);
      await page.goto('/admin');
      await page.evaluate((t) => localStorage.setItem('mw.admin.appearance', JSON.stringify({ theme: t, accent: 'magenta', mode: t === 'light' ? 'light' : 'dark' })), theme);
      await openConsole(page);
      await appearanceButton(page).click();
      await expect(page.getByRole('dialog', { name: 'Console appearance' })).toBeVisible();
      await axe(page, `dialog ${theme}`, '[data-testid="admin-appearance"]');
    });
  }

  test('under forced colours the dialog keeps its borders and is axe-clean', async ({ page }) => {
    await page.setViewportSize(DESKTOP);
    await page.emulateMedia({ forcedColors: 'active' });
    await openConsole(page);
    await appearanceButton(page).click();
    const card = page.locator('[data-theme-option="dark"]');
    await expect(card).toBeVisible();
    const border = await card.evaluate((el) => getComputedStyle(el).borderTopStyle);
    expect(border).not.toBe('none');
    await axe(page, 'dialog forced colours', '[data-testid="admin-appearance"]');
  });

  test('phone, right to left: the top-bar button opens it, keyboard only, no horizontal scroll', async ({ page }) => {
    await page.setViewportSize(PHONE);
    await openConsole(page);
    await page.evaluate(() => {
      document.documentElement.setAttribute('dir', 'rtl');
      document.documentElement.setAttribute('lang', 'ar');
    });
    const button = appearanceButton(page);
    await expect(button).toBeVisible();
    const box = await button.boundingBox();
    expect(box && box.width >= 24 && box.height >= 24, 'target at least 24x24').toBe(true);
    await button.focus();
    await page.keyboard.press('Enter');
    const dialog = page.getByRole('dialog', { name: 'Console appearance' });
    await expect(dialog).toBeVisible();
    await noHorizontalScroll(page, 'phone rtl dialog');
    const fits = await dialog.evaluate((el) => {
      const r = el.getBoundingClientRect();
      return r.left >= 0 && r.right <= window.innerWidth;
    });
    expect(fits, 'dialog inside the viewport').toBe(true);
    await page.keyboard.press('Escape');
    await expect(dialog).toBeHidden();
    await expect(button).toBeFocused();
  });
});
