import { test, expect } from '@playwright/test';

/**
 * V2 design-token theming, end-to-end. The theme slice applies the user's
 * preferences to `:root` at store construction (App boot) — `data-theme` +
 * `data-density` attributes and the vanilla-extract theme's CSS custom
 * properties (incl. the legacy `--bg`/`--accent` bridge) — and persists them to
 * localStorage (`mw.theme.prefs`), the same key `loadPrefs`/`savePrefs` use.
 *
 * NOTE: the in-app Settings/Ribbon theme PICKER is not currently mounted in the
 * render tree, so this spec drives the REAL persistence path (localStorage →
 * loadPrefs → apply) rather than clicking a picker button — it exercises the
 * genuine theme-switch mechanism (attribute flip + token values), not a stub.
 * Theming is applied before auth, so no login is required.
 */

const prefs = (theme: string, density: string) =>
  JSON.stringify({ theme, density, accent: '', font: 'default', layout: 'default', ribbonCollapsed: false });

test.describe('V2 theming (design tokens)', () => {
  test('switching to Grove Dark flips data-theme + density and changes token vars', async ({ page }) => {
    const root = page.locator(':root');
    const bg = () =>
      page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue('--bg').trim());

    // Establish a known baseline (Light / cozy) via the real persisted prefs.
    await page.goto('/');
    await page.evaluate((p) => localStorage.setItem('mw.theme.prefs', p), prefs('light', 'cozy'));
    await page.reload();
    await expect(root).toHaveAttribute('data-theme', 'light');
    await expect(root).toHaveAttribute('data-density', 'cozy');
    const lightBg = await bg();
    expect(lightBg).not.toBe('');

    // Switch to Grove Dark + compact density.
    await page.evaluate((p) => localStorage.setItem('mw.theme.prefs', p), prefs('grove-dark', 'compact'));
    await page.reload();

    // The attribute flip is what the whole token system keys off of.
    await expect(root).toHaveAttribute('data-theme', 'grove-dark');
    await expect(root).toHaveAttribute('data-density', 'compact');

    // A real token value changed: Grove Dark's background differs from Light's.
    const groveBg = await bg();
    expect(groveBg).not.toBe('');
    expect(groveBg).not.toBe(lightBg);
  });

  test('theme preference survives a reload (persisted, not transient)', async ({ page }) => {
    const root = page.locator(':root');
    await page.goto('/');
    await page.evaluate((p) => localStorage.setItem('mw.theme.prefs', p), prefs('amoled', 'relaxed'));
    await page.reload();
    await expect(root).toHaveAttribute('data-theme', 'amoled');

    // A second, independent reload still reflects the saved preference.
    await page.reload();
    await expect(root).toHaveAttribute('data-theme', 'amoled');
    await expect(root).toHaveAttribute('data-density', 'relaxed');
  });
});
