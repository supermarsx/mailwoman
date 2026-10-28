import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import AxeBuilder from '@axe-core/playwright';
import { test, expect, type APIRequestContext, type Page } from './csrf.ts';
import { engineLogin, resetAccountAppearance, setAccountAppearance, settingsSurface } from './helpers.ts';
import { THEME_LIST, accentShade } from '../src/theme/registry.ts';
import { ACCENT_IDS, type AccentId } from '../src/theme/accentMath.ts';
import type { ThemeName } from '../src/theme/contract.css.ts';

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

/**
 * Resolves once the account appearance holding `field: value` has been stored.
 *
 * A pick is kept on the ACCOUNT (24fb6149): the client writes it with a 600 ms
 * debounce (`createAppearanceSync`) and, after a reload, starts from the defaults
 * and adopts what the account holds. A reload before that write lands shows the
 * defaults, so a spec that reloads waits for the write first. Create this before
 * the click that makes the pick, and await it before `page.reload()`.
 */
function appearanceSaved(page: Page, field: string, value: string): Promise<unknown> {
  return page.waitForResponse(
    (r) =>
      r.request().method() === 'PUT' &&
      new URL(r.url()).pathname.endsWith('/api/account/appearance') &&
      r.ok() &&
      (r.request().postData() ?? '').includes(`"${field}":"${value}"`),
  );
}

test.describe('V2 theming via the Settings dialog', () => {
  test('picking Grove Dark flips data-theme + token vars; density switches too', async ({ page }) => {
    await engineLogin(page);
    const root = page.locator('html');

    // Baseline before switching (the Dark default for a user who never chose).
    const beforeBg = await bgVar(page);
    const beforeTheme = await root.getAttribute('data-theme');

    // Open Settings from the sidebar gear.
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    const dialog = settingsSurface(page);
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
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    const dialog = settingsSurface(page);
    const saved = appearanceSaved(page, 'theme', 'amoled');
    await dialog.getByRole('button', { name: 'AMOLED' }).click();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'amoled');

    // Reload: the account's stored set is adopted and re-applied.
    await saved;
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

/**
 * Store `prefs` as the engine account's appearance, the way a returning user
 * arrives: the next sign-in adopts it.
 *
 * Seeding `localStorage` before boot stopped doing this in 24fb6149: on every
 * account change, the first one at boot included, the web client resets the
 * local set to the defaults and adopts the ACCOUNT's stored set
 * (`bindAppearanceToAccount` in src/api/prefs.ts), so one account never inherits
 * another's look and the login screen shows the defaults. The server blob goes
 * through the same `parseAppearancePrefs` merge a local one did, legacy shapes
 * (no `chosen`, a colour accent) included, so these tests keep their meaning;
 * they now sign in (`engineLogin`) instead of stopping at the login screen.
 */
async function seedPrefs(request: APIRequestContext, prefs: Record<string, unknown>): Promise<void> {
  await setAccountAppearance(request, prefs);
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
    test(`${entry.id} paints its own token values, not the default palette`, async ({ page, request }) => {
      await seedPrefs(request, { mode: 'fixed', theme: entry.id, chosen: true });
      await engineLogin(page);

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
  test('system mode follows an OS scheme flip live, with no reload', async ({ page, request }) => {
    // The live watcher is the load-bearing half: `watchColorScheme()` subscribes
    // to `matchMedia('(prefers-color-scheme: dark)')` for the page's lifetime, so
    // a user flipping their OS theme sees the app follow immediately. A reload
    // would pass even if the listener were never wired.
    await seedPrefs(request, {
      mode: 'system',
      lightTheme: 'ocean-light',
      darkTheme: 'plum-dark',
      chosen: true,
    });
    await page.emulateMedia({ colorScheme: 'light' });
    await engineLogin(page);
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'ocean-light');

    await page.emulateMedia({ colorScheme: 'dark' });
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'plum-dark');
    await expect(page.locator('html')).toHaveAttribute('data-appearance', 'dark');

    // And back — a one-way listener would pass the first half alone.
    await page.emulateMedia({ colorScheme: 'light' });
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'ocean-light');
    await expect(page.locator('html')).toHaveAttribute('data-appearance', 'light');
  });

  test('fixed mode ignores the OS entirely', async ({ page, request }) => {
    await seedPrefs(request, { mode: 'fixed', theme: 'grove-light', chosen: true });
    await page.emulateMedia({ colorScheme: 'dark' });
    await engineLogin(page);
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'grove-light');
    await expect(page.locator('html')).toHaveAttribute('data-appearance', 'light');
  });

  test('an explicit pick leaves system mode and survives a reload', async ({ page, request }) => {
    // `chosen: true`: a user who picked "Follow the system". Without the flag
    // this exact set is the pre-t32 untouched default, which boots Dark anyway.
    await seedPrefs(request, { mode: 'system', lightTheme: 'light', darkTheme: 'dark', chosen: true });
    await page.emulateMedia({ colorScheme: 'dark' });
    await engineLogin(page);
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');

    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    const dialog = settingsSurface(page);
    const saved = appearanceSaved(page, 'theme', 'ocean-light');
    await dialog.getByRole('button', { name: 'Ocean Light' }).click();

    // Picking a light pack while the OS says dark must stick: `setTheme()`
    // switches the mode to `fixed`, otherwise the next OS event would stamp it out.
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'ocean-light');
    await saved;
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
    await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toBeVisible();
    expect(await legacyVar(page, '--bg')).not.toBe('');
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// t32: Dark is the default for a user who has not chosen a theme, whatever the
// OS reports. Light applies when the user picks a light pack, or picks "Follow
// the system" while the OS is light. Each case reads `data-theme` off a real
// boot, so a default that only changed in the unit-tested parser would fail.

test.describe('the Dark default', () => {
  test('a fresh profile under a light OS boots Dark', async ({ page }) => {
    await page.emulateMedia({ colorScheme: 'light' });
    await bootShell(page);
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
    await expect(page.locator('html')).toHaveAttribute('data-appearance', 'dark');
  });

  test('picking "Follow the system" under a light OS gives Light and survives a reload', async ({
    page,
  }) => {
    await page.emulateMedia({ colorScheme: 'light' });
    await engineLogin(page);
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');

    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    const dialog = settingsSurface(page);
    const saved = appearanceSaved(page, 'mode', 'system');
    await dialog.getByRole('button', { name: 'Follow the system' }).click();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');

    await saved;
    await page.reload();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
  });

  test('an old untouched default in storage (no chosen flag) boots Dark', async ({ page, request }) => {
    // What a build before t32 stored for a user who never chose.
    await seedPrefs(request, {
      mode: 'system',
      theme: 'light',
      lightTheme: 'light',
      darkTheme: 'dark',
      density: 'compact',
    });
    await page.emulateMedia({ colorScheme: 'light' });
    await engineLogin(page);
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
    // The fields that are not the theme decision survive.
    await expect(page.locator('html')).toHaveAttribute('data-density', 'compact');
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// t32 V: the named accents. `themes.css.ts` emits one rule per theme × accent
// from the generated `accents.ts`; the unit suite measures the table, and only a
// browser shows that each rule exists in the build and wins over the pack.

/** The legacy colours an accent rule sets, read off `<html>`. */
async function accentVars(page: Page): Promise<{ accent: string; accentText: string; focus: string }> {
  return page.evaluate(() => {
    const cs = getComputedStyle(document.documentElement);
    const get = (n: string) => cs.getPropertyValue(n).trim();
    return { accent: get('--accent'), accentText: get('--accent-text'), focus: get('--focus') };
  });
}

test.describe('named accents', () => {
  test('every theme × accent cell is bound to real CSS', async ({ page }) => {
    await bootShell(page);
    const wrong: string[] = [];
    for (const entry of THEME_LIST) {
      const focusIsAccent = entry.tokens.color.focus === entry.tokens.color.accent;
      for (const id of ACCENT_IDS) {
        const shade = accentShade(entry.id, id);
        await page.evaluate(
          ([theme, accent]) => {
            document.documentElement.setAttribute('data-theme', theme);
            document.documentElement.setAttribute('data-accent', accent);
          },
          [entry.id, id] as const,
        );
        const got = await accentVars(page);
        const want = {
          accent: shade.accent,
          accentText: shade.accentText,
          focus: focusIsAccent ? shade.accent : entry.palette.focus,
        };
        if (JSON.stringify(got) !== JSON.stringify(want)) {
          wrong.push(`${entry.id}/${id}: ${JSON.stringify(got)} != ${JSON.stringify(want)}`);
        }
      }
    }
    expect(wrong).toEqual([]);
  });

  test('without an accent each pack paints its own', async ({ page, request }) => {
    await seedPrefs(request, { mode: 'fixed', theme: 'blueprint', chosen: true });
    await engineLogin(page);
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'blueprint');
    expect(await page.locator('html').getAttribute('data-accent')).toBeNull();
    expect((await accentVars(page)).accent).toBe(accentShade('blueprint', '').accent);
  });

  test('picking a swatch applies it, follows a theme change and survives a reload', async ({
    page,
  }) => {
    await engineLogin(page);
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    const dialog = settingsSurface(page);
    const accents = dialog.getByRole('group', { name: 'Accent' });

    await accents.getByRole('button', { name: 'Teal' }).click();
    await expect(page.locator('html')).toHaveAttribute('data-accent', 'teal');
    await expect(accents.getByRole('button', { name: 'Teal' })).toHaveAttribute('aria-pressed', 'true');
    expect((await accentVars(page)).accent).toBe(accentShade('dark', 'teal').accent);

    const saved = appearanceSaved(page, 'theme', 'paper');
    await dialog.getByRole('button', { name: 'Paper' }).click();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'paper');
    expect((await accentVars(page)).accent).toBe(accentShade('paper', 'teal').accent);

    await saved;
    await page.reload();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'paper');
    await expect(page.locator('html')).toHaveAttribute('data-accent', 'teal');
  });

  test('a colour accent stored by an older build arrives as the nearest named accent', async ({
    page,
    request,
  }) => {
    await seedPrefs(request, { mode: 'fixed', theme: 'light', chosen: true, accent: '#7c3aed' });
    await engineLogin(page);
    await expect(page.locator('html')).toHaveAttribute('data-accent', 'violet');
    expect((await accentVars(page)).accent).toBe(accentShade('light', 'violet').accent);
  });
});

// The five Bauhaus packs, each with two accents, as the next two groups use them.
const SHOT_CELLS: ReadonlyArray<readonly [ThemeName, readonly [AccentId, AccentId]]> = [
  ['bauhaus-black', ['yellow', 'magenta']],
  ['paper', ['red', 'teal']],
  ['blueprint', ['orange', 'green']],
  ['terminal', ['violet', 'graphite']],
  ['primary', ['red', 'indigo']],
];

/** axe `color-contrast` over the page, the untrusted message body excluded. */
async function contrastViolations(page: Page): Promise<string[]> {
  const result = await new AxeBuilder({ page })
    .withRules(['color-contrast'])
    .exclude('iframe.reader__frame')
    .analyze();
  return result.violations.flatMap((v) =>
    v.nodes.map((n) => `${n.target.join(' ')}: ${n.failureSummary ?? v.help}`),
  );
}

test.describe('the Bauhaus packs under axe', () => {
  for (const [theme, pair] of SHOT_CELLS) {
    for (const accent of pair) {
      test(`${theme} with ${accent}: no colour-contrast violation on the mailbox or Settings`, async ({
        page,
    request,
      }) => {
        await seedPrefs(request, { mode: 'fixed', theme, accentId: accent, chosen: true });
        await engineLogin(page);
        await expect(page.locator('html')).toHaveAttribute('data-accent', accent);
        expect(await contrastViolations(page), `${theme}/${accent} mailbox`).toEqual([]);

        await page.getByRole('button', { name: 'Settings', exact: true }).click();
        await expect(settingsSurface(page)).toBeVisible();
        expect(await contrastViolations(page), `${theme}/${accent} settings`).toEqual([]);
      });
    }
  }
});

// ─────────────────────────────────────────────────────────────────────────────
// Screenshots of the same cells on the mailbox, an open message and Settings →
// Appearance. Runs only when T32_THEME_SHOTS names an output directory, against
// a stack with mail in the inbox (the restyle harness stack,
// `e2e/restyle/stack.sh`).

const SHOT_DIR = process.env.T32_THEME_SHOTS ?? '';

test.describe('theme screenshots', () => {
  test.skip(SHOT_DIR === '', 'set T32_THEME_SHOTS to an output directory to capture');

  for (const [theme, pair] of SHOT_CELLS) {
    for (const accent of pair) {
      test(`${theme} with ${accent}`, async ({ page, request }) => {
        await page.setViewportSize({ width: 1280, height: 800 });
        await seedPrefs(request, { mode: 'fixed', theme, accentId: accent, chosen: true });
        await engineLogin(page);
        await expect(page.locator('html')).toHaveAttribute('data-theme', theme);
        await expect(page.locator('html')).toHaveAttribute('data-accent', accent);
        const dir = join(SHOT_DIR, theme);
        mkdirSync(dir, { recursive: true });

        const firstRow = page.locator('.list__row').first();
        await expect(firstRow).toBeVisible({ timeout: 45_000 });
        await page.screenshot({ path: join(dir, `${accent}-mailbox.png`) });

        await firstRow.click();
        await expect(page.locator('iframe.reader__frame')).toBeVisible();
        await page.screenshot({ path: join(dir, `${accent}-reader.png`) });

        await page.getByRole('button', { name: 'Settings', exact: true }).click();
        const dialog = settingsSurface(page);
        const accents = dialog.getByRole('group', { name: 'Accent' });
        await expect(dialog).toBeVisible();
        await page.screenshot({ path: join(dir, `${accent}-settings.png`) });
        await accents.scrollIntoViewIfNeeded();
        await page.screenshot({ path: join(dir, `${accent}-settings-accents.png`) });
      });
    }
  }
});
