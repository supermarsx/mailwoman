import { test, expect } from '../csrf.ts';
import { ADMIN } from './admin-path.ts';

test('page and dialog entry fade without moving content, and respect reduced motion @phone', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  await page.goto(ADMIN);
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toHaveCount(0);
  await page.goto(`${ADMIN}#/preferences`);
  const route = page.getByTestId('admin-route-transition');
  await expect(route).toHaveAttribute('data-mw-entering', 'true');
  const motion = await route.evaluate(node => {
    const style = getComputedStyle(node);
    return { animation: style.animationName, duration: parseFloat(style.animationDuration), transform: style.transform };
  });
  expect(motion.animation).not.toBe('none');
  expect(motion.duration).toBeGreaterThan(0.001);
  expect(motion.transform).toBe('none');
  await page.keyboard.press('Control+k');
  const dialog = page.getByRole('dialog');
  await expect(dialog).toBeVisible();
  expect(await dialog.evaluate(node => getComputedStyle(node).animationName)).not.toBe('none');
  await page.keyboard.press('Escape');

  // Change the OS preference while mounted: CSS must stop an existing entry too.
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await expect.poll(() => route.evaluate(node => getComputedStyle(node).animationName)).toBe('none');
  await page.keyboard.press('Control+k');
  await expect(dialog).toBeVisible();
  expect(await dialog.evaluate(node => getComputedStyle(node).animationName)).toBe('none');
  await page.keyboard.press('Escape');
  await page.goto(`${ADMIN}#/settings`);
  await expect(route).toHaveAttribute('data-mw-entering', 'false');
  expect(await route.evaluate(node => getComputedStyle(node).animationName)).toBe('none');
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: test.info().outputPath('motion-reduced-settings.png'), fullPage: true });
});
