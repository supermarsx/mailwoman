import { test, expect, type APIRequestContext, type Page } from './csrf.ts';
import AxeBuilder from '@axe-core/playwright';
import {
  disableSigninCooldown,
  engineLogin,
  injectViaSmtp,
  messageRow,
  resetAccountAppearance,
  setAccountAppearance,
  waitForInboxMessage,
} from './helpers.ts';

/**
 * axe-core WCAG 2.2 AA gate (1.0 hardening, SPEC §24 / ROADMAP-1.0 L21).
 *
 * Drives the REAL app against the live engine-mode stack (mw-server in MW_MODE=engine
 * over Greenmail, :8090 — the same backend as the `e2e-engine`/`e2e-pim`/`e2e-crypto`
 * projects) and runs @axe-core/playwright over the KEY screens the a11y audit (e1–e4)
 * hardened:
 *   • login (pre-auth)              • OAuth consent screen
 *   • admin sign-in gate            • mailbox + Ribbon (WAI-ARIA tablist)
 *   • Settings (workspace tab)      • Calendar month GRID (the §24 flagship)
 *   • Security panel (verdict badges + Received chain)
 *
 * FAIL on ANY WCAG 2.2 AA violation. This is a REAL gate, not a rubber stamp: a
 * residual violation reds the run and prints the exact rule + node so it can be
 * routed to the owning web area (e1 mail / e2 pim / e3 shell-settings / e4 crypto).
 *
 * This spec is run by .github/workflows/a11y.yml via a dedicated Playwright config
 * (the shared playwright.config.ts projects are owned elsewhere); its `baseURL`
 * points at the engine stack. Locally: boot the stack and run with a config whose
 * baseURL is http://localhost:8090 (see a11y.yml for the generated config).
 */

// The WCAG 2.2 AA rule set (axe-core tag families). `wcag22aa` carries the new
// 2.2 success criteria (e.g. 2.5.8 target size, 2.4.11 focus not obscured) that the
// token contract (touch targets) + focus primitives were built to satisfy.
const WCAG_22_AA = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];

// Every test signs in again as the same engine account from the same address.
// The stack's per-IP sign-in pacing (`signin.retryCooldownSecs`, 3 s by default
// since d94472d7) would answer most of those 429, so it is switched off for the
// test stack once per file.
test.beforeAll(async ({ playwright }, testInfo) => {
  await disableSigninCooldown(playwright.request, String(testInfo.project.use.baseURL));
});

// Appearance is stored per account on the server and the browser adopts it on
// sign-in (api/prefs.ts), so a theme one test picks (or the Dark default the first
// sign-in pushes) would carry into the next test. Start each test from "nothing
// stored"; see `resetAccountAppearance`.
test.beforeEach(async ({ request }) => {
  await resetAccountAppearance(request);
});

/** The Settings surface: a workspace tab whose panel is a region named "Settings". */
function settingsSurface(page: Page) {
  return page.getByRole('tabpanel', { name: 'Settings' }).getByRole('region', { name: 'Settings' });
}

interface ScanOptions {
  /** Restrict the scan to this selector (e.g. a dialog) instead of the whole page. */
  include?: string;
  /** Elements to exclude — e.g. the sandboxed message-body iframe (untrusted email HTML). */
  exclude?: string[];
}

/**
 * Run axe over `page` (or a sub-tree), fail on any WCAG 2.2 AA violation, and print
 * a per-screen summary either way so the CI log shows the GREEN baseline explicitly.
 */
async function scanAxe(page: Page, screen: string, opts: ScanOptions = {}): Promise<void> {
  let builder = new AxeBuilder({ page }).withTags(WCAG_22_AA);
  if (opts.include) builder = builder.include(opts.include);
  for (const ex of opts.exclude ?? []) builder = builder.exclude(ex);

  const results = await builder.analyze();
  const v = results.violations;

  if (v.length > 0) {
    const lines = v.map((rule) => {
      const nodes = rule.nodes
        .slice(0, 4)
        .map((n) => `        - ${n.target.join(' ')}`)
        .join('\n');
      return `  ✗ [${rule.impact ?? 'n/a'}] ${rule.id}: ${rule.help}\n${nodes}\n      ${rule.helpUrl}`;
    });
    console.error(`\naxe [${screen}] — ${v.length} WCAG 2.2 AA violation(s):\n${lines.join('\n')}`);
  } else {
    console.log(`axe [${screen}] — GREEN (0 WCAG 2.2 AA violations)`);
  }

  expect(v, `axe found WCAG 2.2 AA violation(s) on screen "${screen}" (see log above)`).toEqual([]);
}

test.describe('axe WCAG 2.2 AA gate', () => {
  test('login screen (pre-auth)', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toBeVisible();
    await scanAxe(page, 'login');
  });

  test('admin sign-in gate', async ({ page }) => {
    // The /admin route lazily mounts the admin panel, which renders a sign-in gate
    // (a real form) before any admin session exists — axe-scannable without creds.
    await page.goto('/admin');
    await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toBeVisible();
    await scanAxe(page, 'admin-signin');
  });

  test('OAuth consent screen', async ({ page }) => {
    // The /oauth/authorize route mounts the resource-owner consent screen. Without
    // valid authorize params it renders its own (accessible) error/empty state; we
    // scan whatever the screen presents — the chrome is the owned surface.
    await page.goto('/oauth/authorize');
    // Wait for the consent screen to settle out of its loading state.
    await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toHaveCount(0);
    await page.waitForLoadState('networkidle');
    await scanAxe(page, 'consent');
  });

  test('mailbox + Ribbon', async ({ page }) => {
    await engineLogin(page);
    // The Ribbon is a WAI-ARIA tablist in the mailbox chrome; scan the whole shell.
    await expect(page.getByRole('tablist')).toBeVisible();
    await scanAxe(page, 'mailbox+ribbon');
  });

  test('Settings (workspace tab)', async ({ page }) => {
    await engineLogin(page);
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    // Settings opens as a workspace tab (a labelled region inside the tab panel),
    // not a modal dialog, since 24fb6149.
    await expect(settingsSurface(page)).toBeVisible();
    // Scan the whole page (the tab strip + the panel) so the tab/tabpanel wiring
    // and the menu/group semantics inside Settings are covered.
    await scanAxe(page, 'settings');
  });

  test('Calendar month grid', async ({ page }) => {
    await engineLogin(page);
    await page.getByTestId('nav-calendar').click();
    await expect(page.locator('main.module-pane[data-surface="calendar"]')).toBeVisible();
    await expect(page.locator('[data-module="calendar"]')).toBeVisible();
    // The month view is a role=grid; make sure it rendered before scanning.
    await expect(page.getByRole('grid').first()).toBeVisible();
    await scanAxe(page, 'calendar-grid');
  });

  test('Security panel (verdict badges + Received chain)', async ({ page }) => {
    const subject = `A11y security ${Date.now()}`;
    await injectViaSmtp({
      from: 'Sender <sender@remote-example.net>',
      subject,
      text: 'security panel a11y scan body',
    });
    await engineLogin(page);
    await waitForInboxMessage(page, subject);
    await messageRow(page, subject).click();

    // Expand the security chip -> the verdict/Received-chain region.
    // The chip is the reader's only dialog trigger; the view-mode and More menus also
    // carry aria-expanded and come first while the verdict is still loading.
    const chip = page.locator('.reader button[aria-expanded][aria-haspopup="dialog"]').first();
    await expect(chip).toBeVisible({ timeout: 15_000 });
    await chip.click();
    await expect(page.getByRole('region', { name: 'Message security details' })).toBeVisible();

    // Exclude the sandboxed message-body iframe: it carries UNTRUSTED email HTML,
    // not Mailwoman's own UI, so scanning into it would be noise/flake, not a gate.
    await scanAxe(page, 'security-panel', { exclude: ['iframe[title="Message body"]'] });
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// t19-e12 — theming a11y. `src/theme/contrast.test.ts` proves the token TABLE is
// contrast-viable; these scans prove the RENDERED result is, which is a
// different claim. axe's `color-contrast` rule reads computed styles off the
// real DOM, so it catches what a numeric table over `tokens.ts` cannot: a
// component that hard-codes a colour instead of reading a token, a pack that was
// never bound to a CSS selector, or a surface painted by a token pair the
// requirement table does not describe.

/**
 * Store a fixed pack as the account's appearance so the next sign-in starts on
 * it. (A `localStorage` seed no longer survives sign-in: the client resets to the
 * defaults on an account change and adopts the account's stored set; see
 * `setAccountAppearance`.)
 */
async function seedTheme(request: APIRequestContext, theme: string): Promise<void> {
  await setAccountAppearance(request, { mode: 'fixed', theme, chosen: true });
}

test.describe('theme contrast in a real browser', () => {
  // One pack per family rather than all 13: the families are what differ in
  // palette, the light/dark variants of a family share their structure, and a
  // full-shell axe scan is not cheap. The numeric matrix covers all 13.
  for (const theme of ['light', 'dark', 'slate-dark', 'ocean-light', 'plum-dark', 'grove-light', 'amoled']) {
    test(`mailbox chrome on ${theme}`, async ({ page, request }) => {
      await seedTheme(request, theme);
      await engineLogin(page);
      await expect(page.locator('html')).toHaveAttribute('data-theme', theme);
      await expect(page.getByRole('tablist')).toBeVisible();
      await scanAxe(page, `mailbox/${theme}`);
    });
  }

  test('Settings on the high-contrast light pack', async ({ page, request }) => {
    await seedTheme(request, 'hc-light');
    await engineLogin(page);
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'hc-light');
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    await expect(settingsSurface(page)).toBeVisible();
    await scanAxe(page, 'settings/hc-light');
  });

  test('Settings on the high-contrast dark pack', async ({ page, request }) => {
    await seedTheme(request, 'hc-dark');
    await engineLogin(page);
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'hc-dark');
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    await expect(settingsSurface(page)).toBeVisible();
    await scanAxe(page, 'settings/hc-dark');
  });
});

test.describe('forced colours and reduced motion', () => {
  // Windows High Contrast / `forced-colors: active` replaces the author palette
  // with the user's system colours. The failure mode it exposes is not low
  // contrast but INVISIBILITY: anything conveyed by a background image, a
  // box-shadow, or a `background-color` alone disappears, because forced-colors
  // does not repaint those. axe's rules for it are limited, so these tests pair
  // the scan with a direct check that the shell still renders its structure.

  test('mailbox chrome under forced-colors: active', async ({ page }) => {
    await page.emulateMedia({ forcedColors: 'active' });
    await engineLogin(page);
    await expect(page.getByRole('tablist')).toBeVisible();
    // The ribbon tabs must remain distinguishable as controls, not just as text:
    // under forced colours a tab styled only by background colour reads as plain
    // text to a user who cannot see the (now discarded) fill.
    const tabs = page.getByRole('tab');
    expect(await tabs.count()).toBeGreaterThan(0);
    await expect(tabs.first()).toBeVisible();
    await scanAxe(page, 'mailbox/forced-colors');
  });

  test('Settings under forced-colors: active', async ({ page }) => {
    await page.emulateMedia({ forcedColors: 'active' });
    await engineLogin(page);
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    const settings = settingsSurface(page);
    await expect(settings).toBeVisible();
    // The theme picker marks the active pack with `aria-pressed`, not colour
    // alone — which is exactly what keeps it usable when the fill is discarded.
    await expect(settings.locator('button[aria-pressed="true"]').first()).toBeVisible();
    await scanAxe(page, 'settings/forced-colors');
  });

  test('login screen under forced-colors: active', async ({ page }) => {
    await page.emulateMedia({ forcedColors: 'active' });
    await page.goto('/');
    await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toBeVisible();
    await scanAxe(page, 'login/forced-colors');
  });

  test('prefers-reduced-motion reaches the root and survives a theme change', async ({ page }) => {
    // `themes.css.ts` gates its transitions on `:root[data-reduced-motion]`, and
    // the flag is set by the same `matchMedia` helper the colour-scheme watcher
    // uses — a refactor that broke one would plausibly break the other.
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await engineLogin(page);
    await expect(page.locator('html')).toHaveAttribute('data-reduced-motion', '');

    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    const settings = settingsSurface(page);
    await settings.getByRole('button', { name: 'AMOLED' }).click();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'amoled');
    await expect(page.locator('html')).toHaveAttribute('data-reduced-motion', '');
  });
});
