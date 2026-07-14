import { test, expect, type Page } from '@playwright/test';
import { engineLogin } from './helpers.ts';

/**
 * Pseudolocale + RTL smoke (1.0 hardening, SPEC §24 / ROADMAP-1.0 L21).
 *
 * Two failure modes translation introduces that unit tests never see:
 *   1. RTL mirroring — the UI must mirror under `dir=rtl` without breaking layout
 *      (the CSS logical properties e1–e4 used should mirror "for free"). The
 *      Calendar month GRID is the sharp case (§24): its columns/navigation mirror.
 *   2. Expansion clipping — real translations (de/ru/…) run ~30-40% longer than
 *      English; fixed-width chrome must not clip or force horizontal page scroll.
 *
 * The app's locale set is a closed list (src/i18n/locales.ts), so a synthetic
 * pseudolocale cannot be registered without a source change; this smoke therefore
 * works at the DOM level — it forces `dir=rtl` and applies an accent+expand text
 * transform, then asserts the layout survives. (UN-wrapped strings are caught
 * separately + deterministically by scripts/i18n/no-hardcoded-strings.mjs.)
 *
 * Runs against the engine-mode stack (:8090) via .github/workflows/a11y.yml.
 */

/** Force right-to-left on the document root (mirrors what a real RTL locale does). */
async function forceRtl(page: Page): Promise<void> {
  await page.evaluate(() => {
    document.documentElement.setAttribute('dir', 'rtl');
    document.documentElement.setAttribute('lang', 'ar');
  });
}

/**
 * Accent + expand every visible text node to ~1.4× length (the classic
 * pseudolocale expansion). Catches fixed-width containers that would clip a longer
 * translation. A one-shot DOM snapshot transform — we measure immediately after.
 */
async function pseudoExpand(page: Page): Promise<void> {
  await page.evaluate(() => {
    const MAP: Record<string, string> = {
      a: 'á', e: 'é', i: 'í', o: 'ó', u: 'ú', n: 'ñ', c: 'ç', s: 'š', y: 'ý',
      A: 'Á', E: 'É', I: 'Í', O: 'Ó', U: 'Ú', N: 'Ñ', C: 'Ç', S: 'Š',
    };
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    const nodes: Text[] = [];
    for (let n = walker.nextNode(); n; n = walker.nextNode()) {
      const text = n.nodeValue ?? '';
      if (/[A-Za-z]{2,}/.test(text)) nodes.push(n as Text);
    }
    for (const node of nodes) {
      const src = node.nodeValue ?? '';
      const accented = [...src].map((ch) => MAP[ch] ?? ch).join('');
      // Pad by ~40% with visible filler so expansion actually stresses the box.
      const pad = '·'.repeat(Math.ceil(src.replace(/\s/g, '').length * 0.4));
      node.nodeValue = pad.length > 0 ? `${accented}${pad}` : accented;
    }
  });
}

/**
 * Assert the page does not scroll horizontally at the document level — the tell
 * for clipped/overflowing chrome under mirroring or expansion. (Inner regions may
 * legitimately scroll; the PAGE must not.) A small tolerance absorbs sub-pixel /
 * scrollbar rounding.
 */
async function assertNoPageOverflow(page: Page, label: string): Promise<void> {
  const overflow = await page.evaluate(() => {
    const el = document.documentElement;
    return { scroll: el.scrollWidth, client: el.clientWidth };
  });
  expect(
    overflow.scroll,
    `${label}: page overflows horizontally (scrollWidth ${overflow.scroll} > clientWidth ${overflow.client}) — clipping/mirroring bug`,
  ).toBeLessThanOrEqual(overflow.client + 2);
}

/**
 * Open the Calendar and switch to the Month view — the flagship WAI-ARIA month
 * grid (grid > row > gridcell, 6×7). The default (Week) view renders a different
 * "Time grid", so the structural mirroring check explicitly targets Month.
 */
async function gotoCalendar(page: Page): Promise<void> {
  await page.getByTestId('nav-calendar').click();
  await expect(page.locator('main.module-pane[data-surface="calendar"]')).toBeVisible();
  await page.getByRole('tab', { name: 'Month', exact: true }).click();
  // The month grid carries a "… <Month Year>" aria-label; wait for its 6×7 cells.
  await expect
    .poll(async () => page.getByRole('gridcell').count())
    .toBeGreaterThanOrEqual(28);
}

test.describe('RTL mirroring smoke', () => {
  test('mailbox + Ribbon mirror under dir=rtl without page overflow', async ({ page }) => {
    await engineLogin(page);
    await forceRtl(page);
    // The Ribbon (tablist) + Compose stay present and reachable after mirroring.
    await expect(page.getByRole('tablist')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Compose' })).toBeVisible();
    await assertNoPageOverflow(page, 'mailbox RTL');
  });

  test('calendar month grid mirrors under dir=rtl, structure intact', async ({ page }) => {
    await engineLogin(page);
    await gotoCalendar(page);
    await forceRtl(page);
    // The month grid mirrors; its structure must survive: whole weeks of 7 cells.
    const grid = page.getByRole('grid');
    await expect(grid).toBeVisible();
    const cells = await grid.getByRole('gridcell').count();
    expect(cells, 'calendar month grid lost cells under RTL').toBeGreaterThanOrEqual(28);
    expect(cells % 7, 'calendar month grid is not whole 7-day weeks under RTL').toBe(0);
    await assertNoPageOverflow(page, 'calendar RTL');
  });
});

test.describe('pseudolocale expansion smoke', () => {
  test('mailbox survives ~1.4× text expansion without page overflow', async ({ page }) => {
    await engineLogin(page);
    // Prove the chrome was present BEFORE expansion (Compose button by name), then
    // expand — the transform mangles visible text nodes, so post-expansion we assert
    // via text-independent roles (the Ribbon tablist) + the page-overflow guard.
    await expect(page.getByRole('button', { name: 'Compose' })).toBeVisible();
    await pseudoExpand(page);
    await expect(page.getByRole('tablist').first()).toBeVisible();
    await assertNoPageOverflow(page, 'mailbox pseudo');
  });

  test('calendar grid survives text expansion without page overflow', async ({ page }) => {
    await engineLogin(page);
    await gotoCalendar(page);
    await pseudoExpand(page);
    await expect(page.getByRole('grid').first()).toBeVisible();
    await assertNoPageOverflow(page, 'calendar pseudo');
  });
});
