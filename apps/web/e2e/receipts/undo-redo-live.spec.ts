// t80: the editor's Undo and Redo buttons, live, at 1440 and 390 px, light and
// dark, in the composer, the signature editor and the detached composer.
//
// Each of Undo and Redo is a split button in the ribbon header: the action
// (icon, and its label at desktop widths) and the history disclosure (chevron).
// Measured from the boxes the browser lays out:
//   - the icon is centred in its button (block axis, and inline axis when the
//     label is hidden) and in the chevron part, within 1 px;
//   - the padding before the icon equals the padding after the label (or after
//     the icon when the label is hidden), and above equals below, within 1 px;
//   - a visible label sits 4-8 px from the icon;
//   - both halves of a split, and both splits, are one ribbon row high;
//   - the buttons are disabled when there is nothing to undo or redo;
//   - each tooltip names its shortcut (Ctrl+Z, Ctrl+Y on this platform);
//   - under dir=rtl the icons are mirrored;
//   - the ribbon body stays exactly three rows high with nothing scrolling.
//
//   MW_RECEIPTS_BASE_URL  the server (scripts/e2e-receipts-stack.mjs)
//   MW_DETACHED_DIST      optional web build served over the server's shell
//   MW_T80_SHOTS          screenshot directory (default: the test output dir)
//   MW_T80_PHASE          screenshot name prefix (before / after)
import type { Locator, Page } from '@playwright/test';
import { test, expect } from '../csrf.ts';
import { readFile } from 'node:fs/promises';
import path from 'node:path';

test.beforeEach(async ({ context }) => {
  const configured = process.env['MW_DETACHED_DIST'];
  if (!configured) return;
  const dist = path.resolve(configured);
  await context.route('**/*', async (route) => {
    const pathname = new URL(route.request().url()).pathname;
    if (pathname !== '/' && pathname !== '/workspace-popup.html' && !pathname.startsWith('/assets/')) return route.continue();
    const file = path.resolve(dist, pathname === '/' ? 'index.html' : pathname.slice(1));
    if (!file.startsWith(dist + path.sep)) return route.abort();
    const body = await readFile(file);
    const response = await route.fetch();
    const type = file.endsWith('.html') ? 'text/html' : file.endsWith('.js') ? 'text/javascript' : file.endsWith('.css') ? 'text/css' : 'application/octet-stream';
    await route.fulfill({ response, status: 200, body, contentType: type });
  });
});
// The fixture keeps the production three-second initial-login cooldown.
test.beforeEach(async () => {
  await new Promise((resolve) => setTimeout(resolve, 3100));
});

const shots = (info: { outputDir: string }): string => process.env['MW_T80_SHOTS'] ?? info.outputDir;
const phase = process.env['MW_T80_PHASE'] ?? 'run';
const WIDTHS = [
  { width: 1440, height: 900 },
  { width: 390, height: 844 },
] as const;

interface Split {
  action: string;
  label: boolean;
  btn: DOMRect;
  icon: DOMRect;
  text: DOMRect | null;
  summary: DOMRect;
  chevron: DOMRect;
  borderL: number;
  borderR: number;
  borderT: number;
  borderB: number;
  disabled: boolean;
  tooltip: string | null;
  ariaLabel: string | null;
  transform: string;
  rtl: boolean;
}

/** The two splits under `root` (a `.compose-rt__history`), measured. */
async function measure(history: Locator): Promise<Split[]> {
  return history.evaluate((node) => {
    const out: unknown[] = [];
    for (const split of node.querySelectorAll(':scope > *')) {
      const btn = split.querySelector(':scope > button') as HTMLButtonElement | null;
      const summary = split.querySelector('summary');
      if (!btn || !summary) continue;
      const icon = btn.querySelector('svg')!;
      // The label: a visible element, or the bare text node after the icon.
      let text: DOMRect | null = null;
      const span = [...btn.children].find((c) => c.tagName !== 'svg') as HTMLElement | undefined;
      if (span) {
        const r = span.getBoundingClientRect();
        if (r.width > 1 && getComputedStyle(span).visibility !== 'hidden') text = r;
      } else {
        const node = [...btn.childNodes].find((c) => c.nodeType === Node.TEXT_NODE && c.textContent!.trim());
        if (node) {
          const range = document.createRange();
          range.selectNodeContents(node);
          const r = range.getBoundingClientRect();
          if (r.width > 1) text = r;
        }
      }
      const cs = getComputedStyle(btn);
      out.push({
        action: btn.getAttribute('aria-label'),
        label: text !== null,
        btn: btn.getBoundingClientRect().toJSON(),
        icon: icon.getBoundingClientRect().toJSON(),
        text: text ? text.toJSON() : null,
        summary: summary.getBoundingClientRect().toJSON(),
        chevron: summary.querySelector('svg')!.getBoundingClientRect().toJSON(),
        borderL: parseFloat(cs.borderInlineStartWidth),
        borderR: parseFloat(cs.borderInlineEndWidth),
        borderT: parseFloat(cs.borderTopWidth),
        borderB: parseFloat(cs.borderBottomWidth),
        disabled: btn.disabled,
        tooltip: btn.getAttribute('data-tooltip'),
        ariaLabel: btn.getAttribute('aria-label'),
        transform: getComputedStyle(icon).transform,
        rtl: cs.direction === 'rtl',
      });
    }
    return out as never;
  });
}

const mid = (r: DOMRect, axis: 'x' | 'y'): number => (axis === 'x' ? r.left + r.width / 2 : r.top + r.height / 2);

/** The geometry contract for one measured history. Soft, so one run reports every miss. */
function checkGeometry(splits: Split[], where: string): Record<string, number>[] {
  expect.soft(splits.length, `${where}: Undo and Redo splits`).toBe(2);
  const rows: Record<string, number>[] = [];
  for (const s of splits) {
    const tag = `${where} ${s.action}`;
    // Logical sides: under RTL the icon is on the right and the label on its left.
    const padStart = s.rtl ? s.btn.right - s.icon.right : s.icon.left - s.btn.left;
    const padEnd = s.rtl ? (s.text ? s.text.left : s.icon.left) - s.btn.left : s.btn.right - (s.text ? s.text.right : s.icon.right);
    const padTop = s.icon.top - s.btn.top;
    const padBottom = s.btn.bottom - s.icon.bottom;
    const row = {
      btnW: s.btn.width,
      btnH: s.btn.height,
      summaryW: s.summary.width,
      summaryH: s.summary.height,
      iconDy: mid(s.icon, 'y') - mid(s.btn, 'y'),
      iconDx: s.text ? 0 : mid(s.icon, 'x') - mid(s.btn, 'x'),
      padStart,
      padEnd,
      padTop,
      padBottom,
      gap: s.text ? (s.rtl ? s.icon.left - s.text.right : s.text.left - s.icon.right) : -1,
      textDy: s.text ? mid(s.text, 'y') - mid(s.btn, 'y') : 0,
      chevronDx: mid(s.chevron, 'x') - mid(s.summary, 'x'),
      chevronDy: mid(s.chevron, 'y') - mid(s.summary, 'y'),
    };
    rows.push(row);
    expect.soft(Math.abs(row.iconDy), `${tag}: icon centred vertically`).toBeLessThanOrEqual(1);
    expect.soft(Math.abs(row.iconDx), `${tag}: icon centred horizontally (icon only)`).toBeLessThanOrEqual(1);
    expect.soft(Math.abs(row.padStart - row.padEnd), `${tag}: inline padding symmetric (${row.padStart} / ${row.padEnd})`).toBeLessThanOrEqual(1);
    expect.soft(Math.abs(row.padTop - row.padBottom), `${tag}: block padding symmetric`).toBeLessThanOrEqual(1);
    expect.soft(row.padStart, `${tag}: room before the icon`).toBeGreaterThanOrEqual(4);
    if (s.text) {
      expect.soft(row.gap, `${tag}: icon-label gap`).toBeGreaterThanOrEqual(4);
      expect.soft(row.gap, `${tag}: icon-label gap`).toBeLessThanOrEqual(8);
      expect.soft(Math.abs(row.textDy), `${tag}: label centred vertically`).toBeLessThanOrEqual(1);
    }
    expect.soft(Math.abs(row.chevronDx), `${tag}: chevron centred horizontally`).toBeLessThanOrEqual(1);
    expect.soft(Math.abs(row.chevronDy), `${tag}: chevron centred vertically`).toBeLessThanOrEqual(1);
    expect.soft(Math.abs(row.btnH - row.summaryH), `${tag}: split halves one height`).toBeLessThanOrEqual(0.5);
    expect.soft(Math.abs(s.btn.top - s.summary.top), `${tag}: split halves aligned`).toBeLessThanOrEqual(0.5);
    expect.soft(s.ariaLabel, `${tag}: accessible name`).toMatch(/^(Undo|Redo)$/);
  }
  if (splits.length === 2) {
    const [a, b] = splits as [Split, Split];
    expect.soft(Math.abs(a.btn.height - b.btn.height), `${where}: Undo and Redo one height`).toBeLessThanOrEqual(0.5);
    expect.soft(Math.abs(a.btn.top - b.btn.top), `${where}: Undo and Redo on one line`).toBeLessThanOrEqual(0.5);
  }
  return rows;
}

/** The t38 rule: every visible ribbon body is exactly three rows; nothing in the bar scrolls. */
async function checkThreeRows(bar: Locator, where: string): Promise<void> {
  const g = await bar.evaluate((node) => {
    const cs = getComputedStyle(node);
    const expected = 3 * parseFloat(cs.getPropertyValue('--mw-ribbon-row')) + 2 * parseFloat(cs.getPropertyValue('--mw-ribbon-gap'));
    const hs = [...node.querySelectorAll('[data-ribbon-body],[data-ribbon-controls]')].filter((e) => !e.closest('[hidden]') && !e.closest('[data-ribbon-popover]')).map((e) => e.getBoundingClientRect().height);
    const over = [...node.querySelectorAll('*')].filter((e) => !e.closest('[hidden]') && !e.closest('details:not([open])') && e.clientHeight > 0 && e.scrollHeight > e.clientHeight + 1).map((e) => e.className.toString());
    return { expected, hs, over, pageOverflow: document.documentElement.scrollWidth - innerWidth };
  });
  expect.soft(g.hs.length, `${where}: ribbon bodies`).toBeGreaterThan(0);
  for (const h of g.hs) expect.soft(Math.abs(h - g.expected), `${where}: three rows`).toBeLessThanOrEqual(0.5);
  expect.soft(g.over, `${where}: nothing scrolls vertically`).toEqual([]);
  expect.soft(g.pageOverflow, `${where}: no page scroll`).toBeLessThanOrEqual(1);
}

/** Disabled states, the tooltip's shortcut, and the RTL mirror. */
async function checkStates(page: Page, history: Locator, editor: Locator, where: string, rtlShot: () => Promise<void>): Promise<void> {
  const undo = history.getByRole('button', { name: 'Undo', exact: true });
  const redo = history.getByRole('button', { name: 'Redo', exact: true });
  await expect.soft(redo, `${where}: nothing to redo`).toBeDisabled();
  await editor.click();
  await page.keyboard.type('Undo me');
  await expect.soft(undo, `${where}: something to undo`).toBeEnabled();
  const s = await measure(history);
  expect.soft(s[0]?.tooltip ?? '', `${where}: Undo tooltip shortcut`).toContain('Ctrl+Z');
  expect.soft(s[1]?.tooltip ?? '', `${where}: Redo tooltip shortcut`).toContain('Ctrl+Y');
  await undo.click();
  await expect.soft(redo, `${where}: redo after an undo`).toBeEnabled();
  // RTL: both icons mirrored (scaleX(-1) -> matrix(-1, 0, 0, 1, 0, 0)).
  await page.evaluate(() => document.documentElement.setAttribute('dir', 'rtl'));
  const rtl = await measure(history);
  for (const split of rtl) expect.soft(split.transform, `${where} ${split.action}: mirrored under RTL`).toMatch(/^matrix\(-1, 0, 0, 1/);
  checkGeometry(rtl, `${where} rtl`);
  await rtlShot();
  await page.evaluate(() => document.documentElement.setAttribute('dir', 'ltr'));
}

async function signIn(page: Page): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill('testuser');
  await page.getByLabel('Password', { exact: true }).fill('testpass');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Compose', exact: true })).toBeVisible();
}
async function navigate(page: Page, name: string): Promise<void> {
  if ((page.viewportSize()?.width ?? 1440) < 600) await page.getByRole('button', { name: 'Open folders and apps' }).click();
  await page.locator('#shell-nav').getByRole('button', { name, exact: true }).click();
}
/** The core light or dark pack on :root (state/slices/theme.ts sets the same attributes). */
async function setTheme(page: Page, theme: 'light' | 'dark'): Promise<void> {
  await page.emulateMedia({ colorScheme: theme });
  await page.evaluate((value) => {
    document.documentElement.setAttribute('data-theme', value);
    document.documentElement.setAttribute('data-appearance', value);
  }, theme);
}
/** An auto-hide ribbon (narrow widths) is opened on its current tab so its body is measured. */
async function openRibbon(bar: Locator): Promise<void> {
  const tab = bar.getByRole('tab', { selected: true });
  if ((await tab.getAttribute('aria-expanded')) === 'false') await tab.click();
}
/** A control folded into a group's dropdown, or behind a hidden ribbon, made visible. */
async function reveal(page: Page, bar: Locator, target: Locator): Promise<void> {
  if (await target.isVisible()) return;
  await openRibbon(bar);
  for (const more of await bar.getByRole('button', { name: /^More .* commands$/ }).all()) {
    if (await target.isVisible()) return;
    if (!(await more.isVisible())) continue;
    await more.click();
    if (await target.isVisible()) return;
    await page.keyboard.press('Escape');
  }
  await expect(target).toBeVisible();
}
/** A tight shot of the ribbon header (the history and the tabs), and the whole page. */
async function shoot(page: Page, history: Locator, info: { outputDir: string; project: { name: string } }, name: string): Promise<void> {
  const base = `${phase}-${info.project.name}-${name}`;
  const box = await history.boundingBox();
  if (box) {
    const vp = page.viewportSize()!;
    const x = Math.max(0, box.x - 8);
    const y = Math.max(0, box.y - 8);
    await page.screenshot({ path: path.join(shots(info), `${base}-history.png`), clip: { x, y, width: Math.min(vp.width - x, box.width + 16), height: box.height + 16 } });
  }
  await page.screenshot({ path: path.join(shots(info), `${base}.png`) });
}

test.describe('t80 undo and redo', () => {
  // eslint-disable-next-line no-empty-pattern -- Playwright passes fixtures first; only the test info is needed.
  test.beforeEach(({}, info) => {
    test.skip(info.project.name === 'phone', 'Chromium and Firefox, each at 1440 and 390');
  });

  test('composer: centred icons, symmetric padding, states, tooltips, RTL, three rows', async ({ page }, info) => {
    test.setTimeout(240000);
    await page.setViewportSize(WIDTHS[0]);
    await signIn(page);
    for (const size of WIDTHS) {
      await page.setViewportSize(size);
      await navigate(page, 'Compose');
      const compose = page.locator('.workspace-compose').last();
      await compose.getByLabel('Subject', { exact: true }).fill(`Undo ${info.project.name} ${size.width}`);
      const history = compose.locator('.compose-ribbon__heading > .compose-rt__history');
      await expect(history).toBeVisible();
      const editor = compose.locator('.ProseMirror');
      const fresh = await measure(history);
      expect.soft(fresh.map((s) => s.disabled), `composer ${size.width}: nothing to undo or redo yet`).toEqual([true, true]);
      await checkStates(page, history, editor, `composer ${size.width}`, () => shoot(page, history, info, `composer-${size.width}-rtl`));
      for (const theme of ['light', 'dark'] as const) {
        await setTheme(page, theme);
        await editor.click();
        await openRibbon(compose.locator('[data-bar="composerRibbon"]'));
        const rows = checkGeometry(await measure(history), `composer ${size.width} ${theme}`);
        info.annotations.push({ type: `composer-${size.width}-${theme}`, description: JSON.stringify(rows) });
        await checkThreeRows(compose.locator('[data-bar="composerRibbon"]'), `composer ${size.width} ${theme}`);
        await shoot(page, history, info, `composer-${size.width}-${theme}`);
      }
      // Hover and keyboard focus: visible, and the box does not move.
      await setTheme(page, 'light');
      // Redo is the enabled one after checkStates' single undo.
      const undo = history.getByRole('button', { name: 'Redo', exact: true });
      await expect.soft(undo).toBeEnabled();
      const before = await undo.boundingBox();
      await undo.hover();
      await shoot(page, history, info, `composer-${size.width}-hover`);
      await undo.focus();
      await page.keyboard.press('Shift+Tab');
      await page.keyboard.press('Tab');
      await expect.soft(undo).toBeFocused();
      const ring = await undo.evaluate((n) => getComputedStyle(n).outlineStyle);
      expect.soft(ring, `composer ${size.width}: focus ring`).not.toBe('none');
      await shoot(page, history, info, `composer-${size.width}-focus`);
      const after = await undo.boundingBox();
      expect.soft(Math.abs((after?.width ?? 0) - (before?.width ?? 0)), 'hover/focus keep the size').toBeLessThanOrEqual(0.5);
      await page.mouse.move(0, 0);
    }
  });

  test('signature editor: the same buttons and geometry', async ({ page }, info) => {
    test.setTimeout(240000);
    await page.setViewportSize(WIDTHS[0]);
    await signIn(page);
    for (const size of WIDTHS) {
      await page.setViewportSize(size);
      await page.goto('/');
      await expect(page.getByRole('button', { name: 'Compose', exact: true })).toBeVisible();
      if (size.width < 600) await page.getByRole('button', { name: 'Open folders and apps' }).click();
      await page.getByRole('button', { name: 'Settings', exact: true }).click();
      await page.getByRole('button', { name: 'Signatures', exact: true }).click();
      await page.getByRole('button', { name: 'New signature', exact: true }).click();
      await page.getByLabel('Name', { exact: true }).fill(`Undo ${info.project.name} ${size.width}`);
      const scope = page.locator('[data-testid="signature-editor"]');
      const format = page.getByRole('combobox', { name: 'Signature format' });
      await reveal(page, scope.locator('[data-bar]').first(), format);
      await format.selectOption('rich');
      await page.keyboard.press('Escape');
      const history = scope.locator('.compose-ribbon__heading > .compose-rt__history');
      await expect(history).toBeVisible();
      const editor = page.getByRole('textbox', { name: 'Rich signature' });
      await checkStates(page, history, editor, `signature ${size.width}`, () => shoot(page, history, info, `signature-${size.width}-rtl`));
      for (const theme of ['light', 'dark'] as const) {
        await setTheme(page, theme);
        await openRibbon(scope.locator('[data-bar]').first());
        const rows = checkGeometry(await measure(history), `signature ${size.width} ${theme}`);
        info.annotations.push({ type: `signature-${size.width}-${theme}`, description: JSON.stringify(rows) });
        await checkThreeRows(scope.locator('[data-bar]').first(), `signature ${size.width} ${theme}`);
        await shoot(page, history, info, `signature-${size.width}-${theme}`);
      }
      await setTheme(page, 'light');
    }
  });

  test('detached composer: the same buttons and geometry', async ({ page }, info) => {
    test.setTimeout(240000);
    await page.setViewportSize(WIDTHS[0]);
    await signIn(page);
    await navigate(page, 'Compose');
    const subject = `Undo popup ${info.project.name}`;
    await page.getByLabel('Subject', { exact: true }).fill(subject);
    const pending = page.waitForEvent('popup');
    await page.getByRole('tab', { name: subject, exact: true }).press('Shift+F10');
    await page.getByRole('menuitem', { name: `Open ${subject} in a new window`, exact: true }).click();
    const popup = await pending;
    for (const size of WIDTHS) {
      await popup.setViewportSize(size);
      const history = popup.locator('.compose-ribbon__heading > .compose-rt__history');
      await expect(history).toBeVisible();
      const fresh = await measure(history);
      if (size.width === 1440) expect.soft(fresh.map((s) => s.disabled), 'detached: nothing to undo or redo yet').toEqual([true, true]);
      for (const theme of ['light', 'dark'] as const) {
        await setTheme(popup, theme);
        await openRibbon(popup.locator('[data-bar="composerRibbon"]'));
        const rows = checkGeometry(await measure(history), `detached ${size.width} ${theme}`);
        info.annotations.push({ type: `detached-${size.width}-${theme}`, description: JSON.stringify(rows) });
        await checkThreeRows(popup.locator('[data-bar="composerRibbon"]'), `detached ${size.width} ${theme}`);
        await shoot(popup, history, info, `detached-${size.width}-${theme}`);
      }
      await setTheme(popup, 'light');
    }
    const editor = popup.locator('.ProseMirror');
    await editor.click();
    await popup.keyboard.type('Popup text');
    const undo = popup.locator('.compose-rt__history').getByRole('button', { name: 'Undo', exact: true });
    await expect(undo).toBeEnabled();
    await undo.click();
    await expect(editor).not.toContainText('Popup text');
    await popup.close();
  });
});
