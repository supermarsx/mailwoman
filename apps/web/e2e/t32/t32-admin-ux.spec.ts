import AxeBuilder from '@axe-core/playwright';
import { test, expect, type Page } from '../csrf.ts';

/** Account-scoped admin preferences, responsive shell themes and accessibility.
 * Uses the dedicated page and the server preferences API, never browser-global defaults.
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

async function openPreferences(page: Page): Promise<void> {
  await page.goto('/admin#/preferences');
  await expect(page.getByRole('heading', { level: 2, name: 'My preferences' })).toBeVisible();
  await expect(page.getByRole('button', {name:'Reset appearance',exact:true})).toBeEnabled();
}
async function saveAppearance(page: Page, appearance: unknown): Promise<void> {
  const current = await page.request.get('/admin/me/preferences');
  expect(current.ok()).toBe(true);
  const response = await page.request.put('/admin/me/preferences', {
    headers: { 'If-Match': current.headers()['etag'] ?? '' },
    data: { ...await current.json(), appearance },
  });
  expect(response.ok(), `preferences PUT ${response.status()}: ${response.ok() ? '' : await response.text()}`).toBe(true);
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

// These tests intentionally share the fixture admin account and mutate its preferences.
test.describe.configure({ mode: 'serial' });

test.beforeEach(async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await signIn(page);
});

test('personal choices survive reload and controls are on a dedicated page', async ({ page }) => {
  await page.setViewportSize(DESKTOP);
  await openPreferences(page);
  await expect(page.locator('[data-testid="admin-appearance-button"]')).toHaveCount(0);
  await page.locator('[data-theme-option="paper"]').click();
  await page.locator('[data-accent-option="teal"]').click();
  await page.getByRole('tab',{name:'Reading and layout',exact:true}).click();
  await page.getByLabel('Interface density').selectOption('compact');
  await expect(page.getByText('Preferences saved to your account.', { exact: true })).toBeVisible();
  await page.reload();
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'paper');
  await expect(page.locator('html')).toHaveAttribute('data-accent', 'teal');
  await expect(page.locator('html')).toHaveAttribute('data-density', 'compact');
  await expect(page.getByLabel('Interface density')).toHaveValue('compact');
  await page.getByRole('tab',{name:'Appearance',exact:true}).click();
  await page.getByRole('button', { name: 'Reset appearance', exact: true }).click();
  await expect(page.getByText('Preferences saved to your account.', { exact: true })).toBeVisible();
});

test('system mode follows the browser colour scheme', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'light' });
  await saveAppearance(page, { theme: 'grove-dark', accent: '', mode: 'system' });
  await openConsole(page);
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'grove-light');
  await page.emulateMedia({ colorScheme: 'dark' });
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'grove-dark');
});

for (const [label, viewport] of [['desktop', DESKTOP], ['phone', PHONE]] as const) {
  test(`all themes are accessible on ${label}`, async ({ page }) => {
    test.setTimeout(180_000);
    await page.setViewportSize(viewport);
    for (const theme of THEMES) {
      await saveAppearance(page, { theme, accent: '', mode: LIGHT.has(theme) ? 'light' : 'dark' });
      await openConsole(page);
      await expect(page.locator('html')).toHaveAttribute('data-theme', theme);
      await axe(page, `${label} ${theme}`);
      if (label === 'phone') await noHorizontalScroll(page, `${label} ${theme}`);
    }
  });
  test(`preferences fit ${label} with intrinsic action buttons`, async ({ page }) => {
    await page.setViewportSize(viewport); await openPreferences(page);
    await axe(page, `preferences ${label}`);
    await noHorizontalScroll(page, `preferences ${label}`);
    const button = page.getByRole('button', { name: 'Reset appearance', exact: true });
    const box = await button.boundingBox();
    expect(box?.width).toBeLessThan(viewport.width * 0.8);
  });
}

for (const theme of ['dark', 'light', 'hc-dark'] as const) {
  test(`preferences axe check in ${theme}`, async ({ page }) => {
    await saveAppearance(page, { theme, accent: 'magenta', mode: theme === 'light' ? 'light' : 'dark' });
    await openPreferences(page); await axe(page, `preferences ${theme}`);
  });
}
test('preferences under forced colours and RTL phone layout', async ({ page }) => {
  await page.setViewportSize(PHONE); await page.emulateMedia({ forcedColors: 'active' });
  await openPreferences(page);
  await page.evaluate(() => { document.documentElement.dir = 'rtl'; });
  await noHorizontalScroll(page, 'preferences RTL');
  await axe(page, 'preferences forced colours');
});
