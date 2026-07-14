import { test, expect, type Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { engineLogin, injectViaSmtp, waitForInboxMessage, messageRow } from './helpers.ts';

/**
 * axe-core WCAG 2.2 AA gate (1.0 hardening, SPEC §24 / ROADMAP-1.0 L21).
 *
 * Drives the REAL app against the live engine-mode stack (mw-server in MW_MODE=engine
 * over Greenmail, :8090 — the same backend as the `e2e-engine`/`e2e-pim`/`e2e-crypto`
 * projects) and runs @axe-core/playwright over the KEY screens the a11y audit (e1–e4)
 * hardened:
 *   • login (pre-auth)              • OAuth consent screen
 *   • admin sign-in gate            • mailbox + Ribbon (WAI-ARIA tablist)
 *   • Settings dialog (aria-modal)  • Calendar month GRID (the §24 flagship)
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
    await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
    await scanAxe(page, 'login');
  });

  test('admin sign-in gate', async ({ page }) => {
    // The /admin route lazily mounts the admin panel, which renders a sign-in gate
    // (a real form) before any admin session exists — axe-scannable without creds.
    await page.goto('/admin');
    await expect(page.getByRole('button', { name: /sign in/i })).toBeVisible();
    await scanAxe(page, 'admin-signin');
  });

  test('OAuth consent screen', async ({ page }) => {
    // The /oauth/authorize route mounts the resource-owner consent screen. Without
    // valid authorize params it renders its own (accessible) error/empty state; we
    // scan whatever the screen presents — the chrome is the owned surface.
    await page.goto('/oauth/authorize');
    // Wait for the consent screen to settle out of its loading state.
    await expect(page.getByRole('button', { name: 'Sign in' })).toHaveCount(0);
    await page.waitForLoadState('networkidle');
    await scanAxe(page, 'consent');
  });

  test('mailbox + Ribbon', async ({ page }) => {
    await engineLogin(page);
    // The Ribbon is a WAI-ARIA tablist in the mailbox chrome; scan the whole shell.
    await expect(page.getByRole('tablist')).toBeVisible();
    await scanAxe(page, 'mailbox+ribbon');
  });

  test('Settings dialog (aria-modal)', async ({ page }) => {
    await engineLogin(page);
    await page.getByRole('button', { name: 'Settings' }).click();
    const dialog = page.getByRole('dialog', { name: 'Settings' });
    await expect(dialog).toBeVisible();
    // Scan the whole page (the modal + its backdrop context) so aria-modal + the
    // menu/group semantics inside the dialog are covered.
    await scanAxe(page, 'settings-dialog');
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
    const chip = page.locator('.reader button[aria-expanded]').first();
    await expect(chip).toBeVisible({ timeout: 15_000 });
    await chip.click();
    await expect(page.getByRole('region', { name: 'Message security details' })).toBeVisible();

    // Exclude the sandboxed message-body iframe: it carries UNTRUSTED email HTML,
    // not Mailwoman's own UI, so scanning into it would be noise/flake, not a gate.
    await scanAxe(page, 'security-panel', { exclude: ['iframe[title="Message body"]'] });
  });
});
