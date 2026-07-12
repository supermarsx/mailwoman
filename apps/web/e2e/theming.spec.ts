import { test, expect, type Page } from '@playwright/test';
import { engineLogin } from './helpers.ts';

/**
 * V2 design-token theming through the REAL Settings UI (mounted by ab25315).
 * The sidebar gear opens the Settings dialog; picking "Grove Dark" calls
 * app.setTheme -> the theme slice flips :root[data-theme] + the vanilla-extract
 * theme's CSS custom properties (incl. the legacy `--bg`/`--accent` bridge) and
 * persists to localStorage (mw.theme.prefs). No localStorage seeding — this
 * drives the genuine picker.
 */

async function bgVar(page: Page): Promise<string> {
  return page.evaluate(() =>
    getComputedStyle(document.documentElement).getPropertyValue('--bg').trim(),
  );
}

test.describe('V2 theming via the Settings dialog', () => {
  test('picking Grove Dark flips data-theme + token vars; density switches too', async ({ page }) => {
    await engineLogin(page);
    const root = page.locator('html');

    // Baseline before switching (default light-ish theme).
    const beforeBg = await bgVar(page);
    const beforeTheme = await root.getAttribute('data-theme');

    // Open Settings from the sidebar gear.
    await page.getByRole('button', { name: 'Settings' }).click();
    const dialog = page.getByRole('dialog', { name: 'Settings' });
    await expect(dialog).toBeVisible();

    // Pick Grove Dark.
    const grove = dialog.getByRole('button', { name: 'Grove Dark' });
    await grove.click();
    await expect(grove).toHaveAttribute('aria-pressed', 'true');
    await expect(root).toHaveAttribute('data-theme', 'grove-dark');

    // A real token value changed (unless we were already on grove-dark).
    const afterBg = await bgVar(page);
    expect(afterBg).not.toBe('');
    if (beforeTheme !== 'grove-dark') expect(afterBg).not.toBe(beforeBg);

    // Density control is real too.
    await dialog.getByRole('button', { name: 'Compact' }).click();
    await expect(root).toHaveAttribute('data-density', 'compact');

    await dialog.getByRole('button', { name: 'Close settings' }).click();
    await expect(dialog).toBeHidden();
    // Selection stuck after closing the dialog.
    await expect(root).toHaveAttribute('data-theme', 'grove-dark');
  });

  test('the chosen theme persists across a reload', async ({ page }) => {
    await engineLogin(page);
    await page.getByRole('button', { name: 'Settings' }).click();
    const dialog = page.getByRole('dialog', { name: 'Settings' });
    await dialog.getByRole('button', { name: 'AMOLED' }).click();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'amoled');

    // Reload: the theme slice reloads prefs from localStorage and re-applies.
    await page.reload();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'amoled');
  });
});
