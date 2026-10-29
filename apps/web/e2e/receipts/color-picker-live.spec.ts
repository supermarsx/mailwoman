// t81: the app colour picker against a HEAD-built server, Chromium and Firefox
// at 1440 and 390 px. It opens from the composer's font colour, the custom
// accent in Settings and the admin console's custom accent; it stays inside the
// viewport next to every edge; it is usable from the keyboard alone; and no
// Content-Security-Policy violation is reported on the way.
import { test, expect } from '../csrf.ts';
import type { Locator, Page } from '@playwright/test';
import { colorTrigger } from '../colorPicker.ts';

const SHOTS = process.env['MW_T81_SHOTS'];

async function cspWatch(page: Page): Promise<string[]> {
  const seen: string[] = [];
  await page.addInitScript(() => {
    document.addEventListener('securitypolicyviolation', (event) => {
      const w = window as unknown as { __csp?: string[] };
      (w.__csp ??= []).push(`${event.violatedDirective} ${event.blockedURI}`);
    });
  });
  page.on('console', (message) => { if (/content.security.policy/i.test(message.text())) seen.push(message.text()); });
  return seen;
}
async function cspReported(page: Page, console: string[]): Promise<string[]> {
  const inPage = await page.evaluate(() => (window as unknown as { __csp?: string[] }).__csp ?? []);
  return [...inPage, ...console];
}

async function shot(page: Page, name: string): Promise<void> {
  const project = test.info().project.name;
  await page.screenshot({ path: SHOTS ? `${SHOTS}/${project}-${name}.png` : test.info().outputPath(`${name}.png`) });
}

/** The open popover of `trigger` is wholly inside the viewport. */
async function insideViewport(page: Page, trigger: Locator): Promise<{ side: string | null }> {
  const popover = page.locator(`#${await trigger.getAttribute('aria-controls')}`);
  await expect(popover).toBeVisible();
  await expect.poll(async () => popover.evaluate((el) => {
    const r = el.getBoundingClientRect();
    return r.left >= 0 && r.top >= 0 && r.right <= innerWidth + 0.5 && r.bottom <= innerHeight + 0.5 && r.width > 100;
  })).toBe(true);
  return { side: await popover.getAttribute('data-side') };
}

/** Pin a colour field to a viewport corner (CSSOM) and check where its popover goes. */
async function cornerChecks(page: Page, trigger: Locator): Promise<void> {
  const field = trigger.locator('xpath=ancestor::*[@data-color-field][1]');
  for (const corner of [{ top: '4px', left: '4px', side: 'below' }, { bottom: '4px', right: '4px', side: 'above' }, { top: '4px', right: '4px', side: 'below' }, { bottom: '4px', left: '4px', side: 'above' }]) {
    await field.evaluate((el, c) => {
      const s = (el as HTMLElement).style;
      for (const p of ['top', 'left', 'right', 'bottom']) s.removeProperty(p);
      s.setProperty('position', 'fixed');
      s.setProperty('z-index', '10000');
      for (const [k, v] of Object.entries(c)) if (k !== 'side') s.setProperty(k, v as string);
    }, corner);
    await trigger.click();
    const { side } = await insideViewport(page, trigger);
    expect(side, JSON.stringify(corner)).toBe(corner.side);
    await page.keyboard.press('Escape');
    await expect(trigger).toHaveAttribute('aria-expanded', 'false');
  }
  await field.evaluate((el) => { for (const p of ['position', 'z-index', 'top', 'left', 'right', 'bottom']) (el as HTMLElement).style.removeProperty(p); });
}

async function signIn(page: Page): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill('testuser');
  await page.getByLabel('Password', { exact: true }).fill('testpass');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Compose', exact: true })).toBeVisible();
}
const narrow = (page: Page): boolean => (page.viewportSize()?.width ?? 1440) < 700;
async function openNav(page: Page): Promise<void> {
  const drawer = page.getByRole('button', { name: 'Open folders and apps' });
  if (narrow(page) && await drawer.isVisible()) await drawer.click();
}

test.beforeEach(async ({}, info) => {
  test.skip(info.project.name === 'phone', 'widths are set per test for Chromium and Firefox');
  await new Promise((resolve) => setTimeout(resolve, 3100));
});

for (const width of [1440, 390]) {
  test.describe(`${width}px`, () => {
    test.use({ viewport: { width, height: width > 700 ? 900 : 844 } });

    test('composer font colour: keyboard only, live, applied to the selection', async ({ page }) => {
      const csp = await cspWatch(page);
      await signIn(page);
      await openNav(page);
      await page.locator('#shell-nav').getByRole('button', { name: 'Compose', exact: true }).click();
      const compose = page.locator('.workspace-compose').last();
      const editor = compose.locator('.ProseMirror');
      await editor.fill('Coloured words');
      await editor.press('ControlOrMeta+a');
      const trigger = colorTrigger(compose, 'Font color');
      // A phone-width ribbon may be folded: open its tab, then the folded Font group.
      const more = compose.getByRole('button', { name: 'More Font commands', exact: true });
      if (!(await trigger.isVisible())) {
        await compose.getByRole('tab', { name: 'Format text', exact: true }).click();
        await expect(trigger.or(more).first()).toBeVisible();
        // Keep the ribbon open so it does not fold back while the picker is used.
        const pin = compose.getByRole('button', { name: 'Keep the editor ribbon open', exact: true });
        if (await pin.isVisible()) await pin.click();
      }
      if (!(await trigger.isVisible())) await expect(async () => { if (!(await trigger.isVisible())) await more.click(); await expect(trigger).toBeVisible({ timeout: 2000 }); }).toPass({ timeout: 20000 });
      // Keyboard only from here.
      await trigger.focus();
      await page.keyboard.press('Enter');
      const dialog = compose.getByRole('dialog', { name: 'Font color', exact: true });
      await expect(dialog).toBeVisible();
      await insideViewport(page, trigger);
      const area = dialog.getByRole('slider', { name: 'Saturation and brightness' });
      await expect(area).toBeFocused();
      await page.keyboard.press('End');
      await page.keyboard.press('ArrowUp');
      for (let i = 0; i < 5; i++) await page.keyboard.press('Shift+ArrowUp');
      await page.keyboard.press('Tab');
      await expect(dialog.getByRole('slider', { name: 'Hue' })).toBeFocused();
      for (let i = 0; i < 12; i++) await page.keyboard.press('Shift+ArrowRight');
      await expect(dialog.getByRole('slider', { name: 'Hue' })).toHaveAttribute('aria-valuenow', '120');
      await expect(dialog.locator('[data-color-contrast]')).toBeVisible();
      // Tab stays in the popover: Shift+Tab from the area goes to Done.
      await area.focus();
      await page.keyboard.press('Shift+Tab');
      await expect(dialog.getByRole('button', { name: 'Done', exact: true })).toBeFocused();
      await shot(page, `${width}-compose-open`);
      await area.focus();
      await page.keyboard.press('Enter');
      await expect(dialog).toBeHidden();
      await expect(trigger).toBeFocused();
      await expect(trigger).toHaveAttribute('data-color-value', /^#[0-9a-f]{6}$/);
      const color = await trigger.getAttribute('data-color-value');
      await expect.poll(() => editor.evaluate((el) => [...el.querySelectorAll<HTMLElement>('[style]')].map((n) => n.style.color).join(' '))).toMatch(/rgb\(/);
      expect(color).not.toBe('#000000');
      // Escape keeps the value.
      await page.keyboard.press('Enter');
      await expect(dialog).toBeVisible();
      await page.keyboard.press('Escape');
      await expect(dialog).toBeHidden();
      await expect(trigger).toBeFocused();
      await expect(trigger).toHaveAttribute('data-color-value', color!);
      await cornerChecks(page, trigger);
      expect(await cspReported(page, csp)).toEqual([]);
    });

    test('settings custom accent: live preview, hex entry, near the edges', async ({ page }) => {
      const csp = await cspWatch(page);
      await signIn(page);
      await openNav(page);
      await page.locator('#shell-nav').getByRole('button', { name: 'Settings', exact: true }).click();
      await page.getByRole('navigation', { name: 'Settings sections' }).getByRole('button', { name: 'Appearance', exact: true }).click();
      const trigger = colorTrigger(page, 'Custom accent colour');
      await trigger.scrollIntoViewIfNeeded();
      const before = await trigger.getAttribute('data-color-value');
      await trigger.focus();
      await page.keyboard.press('Enter');
      const dialog = page.getByRole('dialog', { name: 'Custom accent colour', exact: true });
      await insideViewport(page, trigger);
      await page.keyboard.press('Tab');
      await page.keyboard.press('Tab');
      const hex = dialog.getByRole('textbox', { name: 'Hex', exact: true });
      await expect(hex).toBeFocused();
      await hex.fill('#12');
      await expect(hex).toHaveAttribute('aria-invalid', 'true');
      await expect(dialog.getByRole('alert')).toContainText('#RGB or #RRGGBB');
      await hex.fill('#2a9d8f');
      await expect(hex).toHaveAttribute('aria-invalid', 'false');
      // Live: the page accent follows before anything is saved.
      await expect.poll(() => page.evaluate(() => [...document.documentElement.style].filter((p) => p.startsWith('--')).map((p) => document.documentElement.style.getPropertyValue(p)).join(' '))).toMatch(/#/);
      await shot(page, `${width}-accent-open`);
      await page.keyboard.press('Enter');
      await expect(trigger).toHaveAttribute('data-color-value', '#2a9d8f');
      await expect.poll(async () => (await (await page.request.get('/api/account/appearance')).json()).appearance?.customAccent).toBe('#2a9d8f');
      await cornerChecks(page, trigger);
      // Put it back: the Default choice clears the custom accent.
      await trigger.click();
      await dialog.getByRole('button', { name: 'Default', exact: true }).click();
      await dialog.getByRole('button', { name: 'Done', exact: true }).click();
      await expect(trigger).toHaveAttribute('data-color-value', before ?? '');
      await expect.poll(async () => (await (await page.request.get('/api/account/appearance')).json()).appearance?.customAccent ?? '').toBe(before ?? '');
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true);
      expect(await cspReported(page, csp)).toEqual([]);
    });

    test('admin console custom accent: swatches, recents, edges', async ({ page }) => {
      const csp = await cspWatch(page);
      await page.goto('/admin');
      await page.getByLabel('Admin username').fill('root');
      await page.getByLabel('Password', { exact: true }).fill('hunter2');
      await page.getByRole('button', { name: 'Sign in', exact: true }).click();
      await expect(page.locator('#admin-main')).toBeVisible();
      await page.goto('/admin#/preferences');
      const trigger = colorTrigger(page, 'Custom accent colour');
      await trigger.scrollIntoViewIfNeeded();
      const before = await trigger.getAttribute('data-color-value');
      await trigger.click();
      const dialog = page.getByRole('dialog', { name: 'Custom accent colour', exact: true });
      await insideViewport(page, trigger);
      const swatch = dialog.getByRole('group', { name: 'Theme colours' }).getByRole('button', { name: /^Teal \(#/ });
      const teal = (await swatch.getAttribute('data-color-swatch'))!;
      await swatch.click();
      await expect(dialog.getByRole('textbox', { name: 'Hex', exact: true })).toHaveValue(teal);
      await expect(dialog.locator('[data-color-contrast]')).toBeVisible();
      await shot(page, `${width}-admin-open`);
      await dialog.getByRole('button', { name: 'Done', exact: true }).click();
      await expect(trigger).toHaveAttribute('data-color-value', teal);
      await trigger.click();
      await expect(dialog.getByRole('group', { name: 'Recent colours' }).locator(`[data-color-swatch="${teal}"]`)).toBeVisible();
      await page.keyboard.press('Escape');
      await expect(trigger).toBeFocused();
      await cornerChecks(page, trigger);
      await trigger.click();
      await dialog.getByRole('button', { name: 'Default', exact: true }).click();
      await dialog.getByRole('button', { name: 'Done', exact: true }).click();
      await expect(trigger).toHaveAttribute('data-color-value', before ?? '');
      expect(await cspReported(page, csp)).toEqual([]);
    });
  });
}
