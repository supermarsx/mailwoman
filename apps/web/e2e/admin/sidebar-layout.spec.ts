import { test, expect } from '../csrf.ts';

test('admin sidebar resizes, collapses and restores from account preferences @phone', async ({ page }, info) => {
  await page.goto('/admin');
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.locator('#admin-main')).toBeVisible();
  const originalResponse = await page.request.get('/admin/me/preferences');
  const original = await originalResponse.json();
  try {
    const baseline = await page.request.put('/admin/me/preferences', { headers: { 'If-Match': originalResponse.headers()['etag']! }, data: { ...original, shellLayout: { sidebarWidth: 220, sidebarCollapsed: false } } });
    expect(baseline.ok()).toBe(true);
    await page.reload();
    if (info.project.name === 'phone') {
      await expect(page.locator('#admin-sidebar')).toBeHidden();
      await expect(page.getByRole('separator', { name: 'Resize navigation sidebar' })).toBeHidden();
      await page.getByRole('button', { name: 'Menu', exact: true }).click();
      await expect(page.locator('#admin-drawer')).toBeVisible();
    } else {
      const splitter = page.getByRole('separator', { name: 'Resize navigation sidebar' });
      await expect(splitter).toHaveAttribute('aria-valuenow', '220');
      await splitter.focus();
      await page.keyboard.press('Shift+ArrowRight');
      await expect.poll(async () => (await (await page.request.get('/admin/me/preferences')).json()).shellLayout?.sidebarWidth).toBe(260);
      const box = await splitter.boundingBox();
      await page.mouse.move(box!.x + box!.width / 2, box!.y + 50);
      await page.mouse.down();
      await page.mouse.move(box!.x + box!.width / 2 + 60, box!.y + 50, { steps: 4 });
      await page.mouse.up();
      await expect.poll(async () => (await (await page.request.get('/admin/me/preferences')).json()).shellLayout?.sidebarWidth).toBe(320);
      await page.getByRole('button', { name: 'Hide navigation sidebar' }).click();
      await expect(page.locator('#admin-sidebar')).toBeHidden();
      await expect.poll(async () => (await (await page.request.get('/admin/me/preferences')).json()).shellLayout?.sidebarCollapsed).toBe(true);
      await page.reload();
      await expect(page.locator('#admin-sidebar')).toBeHidden();
      await page.getByRole('button', { name: 'Show navigation sidebar' }).click();
      await expect(page.locator('#admin-sidebar')).toBeVisible();
      await expect(splitter).toHaveAttribute('aria-valuenow', '320');
    }
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: info.outputPath('sidebar-layout.png'), fullPage: true });
  } finally {
    const current = await page.request.get('/admin/me/preferences');
    expect((await page.request.put('/admin/me/preferences', { headers: { 'If-Match': current.headers()['etag']! }, data: original })).ok()).toBe(true);
  }
});
