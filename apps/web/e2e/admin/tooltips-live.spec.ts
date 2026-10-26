import { test, expect } from '../csrf.ts';

test('app tooltips follow keyboard focus, theme, viewport and reduced motion without blocking touch @phone', async ({ page }, testInfo) => {
  await page.goto('/admin');
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  const button = page.locator(testInfo.project.name === 'phone' ? 'button[aria-controls="admin-drawer"]' : 'button[aria-controls="admin-sidebar"]');
  await expect(button).toBeVisible();
  const before = await button.boundingBox();
  await button.focus();
  const tooltip = page.getByRole('tooltip');
  await expect(tooltip).toBeVisible();
  expect(await button.boundingBox()).toEqual(before);
  await expect(button).toHaveAttribute('aria-describedby', /tooltip-/);
  const geometry = await tooltip.evaluate(node => {
    const rect = node.getBoundingClientRect();
    const style = getComputedStyle(node);
    return { left: rect.left, right: rect.right, top: rect.top, bottom: rect.bottom, width: innerWidth, height: innerHeight, position: style.position, background: style.backgroundColor };
  });
  expect(geometry.position).toBe('fixed');
  expect(geometry.left).toBeGreaterThanOrEqual(0);
  expect(geometry.top).toBeGreaterThanOrEqual(0);
  expect(geometry.right).toBeLessThanOrEqual(geometry.width);
  expect(geometry.bottom).toBeLessThanOrEqual(geometry.height);
  expect(geometry.background).not.toBe('rgba(0, 0, 0, 0)');
  await page.emulateMedia({ reducedMotion: 'reduce' });
  expect(await tooltip.evaluate(node => getComputedStyle(node).animationName)).toBe('none');
  await page.screenshot({ path: testInfo.outputPath('themed-tooltip.png'), fullPage: true });
  await page.keyboard.press('Escape');
  await expect(tooltip).toHaveCount(0);
  const expanded = await button.getAttribute('aria-expanded');
  if (testInfo.project.name === 'phone') await button.tap();
  else await button.click();
  await expect(button).toHaveAttribute('aria-expanded', expanded === 'true' ? 'false' : 'true');
  await expect(tooltip).toHaveCount(0);
});
