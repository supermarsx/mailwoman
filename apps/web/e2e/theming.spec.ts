import { test, expect, type Page } from '@playwright/test';
import { engineLogin, resetAccountAppearance } from './helpers.ts';
import { THEME_LIST } from '../src/theme/registry.ts';

/**
 * V2 design-token theming through the REAL Settings UI (mounted by ab25315).
 * The sidebar gear opens the Settings dialog; picking "Grove Dark" calls
 * app.setTheme -> the theme slice flips :root[data-theme] + the vanilla-extract
 * theme's CSS custom properties (incl. the legacy `--bg`/`--accent` bridge) and
 * persists to localStorage (mw.theme.prefs). No localStorage seeding — this
 * drives the genuine picker.
 */

// Appearance is ALSO synced per account (`/api/account/appearance`), and every
// engine-mode spec signs in as the same account — so it is shared mutable state
// across these tests, and a fresh browser context does not clear it. Without this
// reset, the theme picked by whichever theming test ran first is re-adopted on the
// next test's boot and after every reload, overwriting that test's own pick:
// `theming.spec.ts:55` and `:158` failed with `data-theme="grove-dark"` (plus
// `data-density="compact"`) where they expected `amoled` / `ocean-light` — exactly
// the state the first test in this file sets. See `resetAccountAppearance` for the
// reconcile rule that makes it happen and why resetting fixes the isolation rather
// than the ordering.
test.beforeEach(async ({ request }) => {
  await resetAccountAppearance(request);
});

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

// ─────────────────────────────────────────────────────────────────────────────
// t19-e12. Everything below drives the theme engine t19-e6 shipped in `b7812e1`
// (13 packs, the tri-state mode, live OS-follow) through a REAL browser.
//
// Why these belong in e2e rather than the unit suite: `contrast.test.ts` measures
// `tokens.ts`, but what a user sees is painted by `themes.css.ts` through
// vanilla-extract, under `:root[data-theme="…"]`, resolved by a real CSS engine.
// A pack present in the token table but never bound to a selector would pass the
// entire numeric matrix and render as the default Light theme. Only a browser can
// tell the difference, and that is the one thing these specs exist to prove.

/** Seed `mw.theme.prefs` before the app boots, the way a returning user arrives. */
async function seedPrefs(page: Page, prefs: Record<string, unknown>): Promise<void> {
  await page.addInitScript((p) => {
    localStorage.setItem('mw.theme.prefs', JSON.stringify(p));
  }, prefs);
}

/** Read a legacy-bridge custom property off `<html>` (see themes.css.ts). */
async function legacyVar(page: Page, name: string): Promise<string> {
  return page.evaluate(
    (n) => getComputedStyle(document.documentElement).getPropertyValue(n).trim(),
    name,
  );
}

/** Boot the SPA far enough that the theme slice has applied root attributes. */
async function bootShell(page: Page): Promise<void> {
  await page.goto('/');
  // The theme slice runs when the app state is created, which happens before the
  // login screen paints — so an unauthenticated boot is enough, and much cheaper
  // than a login per theme.
  await expect(page.locator('html')).toHaveAttribute('data-theme', /.+/);
}

test.describe('every built-in pack is bound to real CSS', () => {
  for (const entry of THEME_LIST) {
    test(`${entry.id} paints its own token values, not the default palette`, async ({ page }) => {
      await seedPrefs(page, { mode: 'fixed', theme: entry.id });
      await bootShell(page);

      await expect(page.locator('html')).toHaveAttribute('data-theme', entry.id);
      // The rendered value must be the value the contrast matrix measured. If
      // `themes.css.ts` never bound this pack, the browser falls back to the
      // Light defaults bound at `:root` and these diverge.
      expect(await legacyVar(page, '--bg')).toBe(entry.palette.bg);
      expect(await legacyVar(page, '--text')).toBe(entry.palette.text);
      expect(await legacyVar(page, '--text-dim')).toBe(entry.palette.textDim);
      expect(await legacyVar(page, '--accent-text')).toBe(entry.palette.accentText);

      // `data-appearance` + the inline `color-scheme` drive the UA-painted
      // surfaces (scrollbars, form controls, the overscroll canvas), so a dark
      // pack does not flash a white gutter.
      await expect(page.locator('html')).toHaveAttribute('data-appearance', entry.appearance);
      expect(
        await page.evaluate(() => document.documentElement.style.getPropertyValue('color-scheme')),
      ).toBe(entry.appearance);
    });
  }
});

test.describe('appearance mode tri-state', () => {
  test('system mode follows an OS scheme flip live, with no reload', async ({ page }) => {
    // The live watcher is the load-bearing half: `watchColorScheme()` subscribes
    // to `matchMedia('(prefers-color-scheme: dark)')` for the page's lifetime, so
    // a user flipping their OS theme sees the app follow immediately. A reload
    // would pass even if the listener were never wired.
    await seedPrefs(page, { mode: 'system', lightTheme: 'ocean-light', darkTheme: 'plum-dark' });
    await page.emulateMedia({ colorScheme: 'light' });
    await bootShell(page);
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'ocean-light');

    await page.emulateMedia({ colorScheme: 'dark' });
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'plum-dark');
    await expect(page.locator('html')).toHaveAttribute('data-appearance', 'dark');

    // And back — a one-way listener would pass the first half alone.
    await page.emulateMedia({ colorScheme: 'light' });
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'ocean-light');
    await expect(page.locator('html')).toHaveAttribute('data-appearance', 'light');
  });

  test('fixed mode ignores the OS entirely', async ({ page }) => {
    await seedPrefs(page, { mode: 'fixed', theme: 'grove-light' });
    await page.emulateMedia({ colorScheme: 'dark' });
    await bootShell(page);
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'grove-light');
    await expect(page.locator('html')).toHaveAttribute('data-appearance', 'light');
  });

  test('an explicit pick leaves system mode and survives a reload', async ({ page }) => {
    await seedPrefs(page, { mode: 'system', lightTheme: 'light', darkTheme: 'dark' });
    await page.emulateMedia({ colorScheme: 'dark' });
    await engineLogin(page);
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');

    await page.getByRole('button', { name: 'Settings' }).click();
    const dialog = page.getByRole('dialog', { name: 'Settings' });
    await dialog.getByRole('button', { name: 'Ocean Light' }).click();

    // Picking a light pack while the OS says dark must stick: `setTheme()`
    // switches the mode to `fixed`, otherwise the next OS event would stamp it out.
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'ocean-light');
    await page.reload();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'ocean-light');
  });

  test('a corrupt stored preference degrades instead of failing to boot', async ({ page }) => {
    // `parseAppearancePrefs` falls back field by field. The shell must still
    // render — a bad localStorage payload is not a reason to show a blank page.
    await page.addInitScript(() => {
      localStorage.setItem('mw.theme.prefs', '{"mode":"sideways","theme":"not-a-theme"');
    });
    await bootShell(page);
    await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
    expect(await legacyVar(page, '--bg')).not.toBe('');
  });
});
