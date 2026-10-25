import { test, expect } from '../csrf.ts';

test('loading stays steady, resolves immediately and respects reduced motion @phone', async ({ page }) => {
  let release!: () => void;
  const pendingResponse = new Promise<void>(resolve => { release = resolve; });
  await page.route('**/api/me', async route => {
    await pendingResponse;
    await route.fulfill({ status: 401, contentType: 'application/json', body: '{}' });
  });
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  await page.goto('/');
  const pending = page.getByTestId('async-pending');
  await expect(pending).toBeVisible();
  const indicator = pending.locator('span');
  const box = await pending.boundingBox();
  expect(box?.height).toBeGreaterThanOrEqual(44);
  expect(await indicator.evaluate(node => getComputedStyle(node).animationDelay)).toBe('0.12s');
  await expect.poll(() => indicator.evaluate(node => getComputedStyle(node).opacity)).toBe('1');
  expect(await pending.boundingBox()).toEqual(box);
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await expect.poll(() => indicator.evaluate(node => getComputedStyle(node).animationName)).toBe('none');
  expect(await indicator.evaluate(node => getComputedStyle(node).opacity)).toBe('1');
  release();
  await expect(pending).toHaveCount(0);
  await expect(page.locator('main.login')).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
});
