import { test, expect } from '@playwright/test';
import { login } from './helpers.ts';
test.use({ serviceWorkers: 'block' });
test.setTimeout(75000);
const timestamp = '2026-11-08T22:37:48.689600982+00:00';
test('readable activity preserves repeated events, exact details and fits desktop and phone', async ({ page }, info) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.addLocatorHandler(page.getByRole('dialog', { name: 'Allow notifications?', exact: true }), async dialog => { await dialog.getByRole('button', { name: 'Not now', exact: true }).click(); });
  await page.route(/\/api\/settings(?:\?.*)?$/, async route => {
    if (route.request().method() !== 'GET') return route.continue();
    const response = await route.fetch(); const body = await response.json() as { settings: Record<string, unknown> };
    body.settings['notifications.askPermission'] = { value: false, source: 'global', locked: true };
    body.settings['time.displayZone'] = { value: 'America/New_York', source: 'user', locked: false };
    await route.fulfill({ response, json: body });
  });
  await page.route(/\/api\/account\/security\/activity(?:\?.*)?$/, async route => {
    if (route.request().method() !== 'GET') return route.continue();
    await route.fulfill({ status: 200, json: { events: [
      { id: 'event-first', timestamp, action: 'linked-credential-renewed', actor: 'system', target: 'account:owner', detail: { link: 'first-authority', reason: 'x'.repeat(160) } },
      { id: 'event-second', timestamp, action: 'linked-credential-renewed', actor: 'system', target: 'account:owner', detail: { link: 'second-authority' } },
      { id: 'event-failed', timestamp, action: 'linked-credential-refresh-failed', actor: 'system', detail: { reason: 'expired' } },
    ] } });
  });
  await login(page);
  const show = page.getByRole('button', { name: 'Show folders sidebar', exact: true }); if (await show.isVisible()) await show.click();
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByRole('navigation', { name: 'Settings sections' }).getByRole('button', { name: 'Security centre', exact: true }).click();
  await page.getByRole('tab', { name: 'Account activity', exact: true }).click();
  const activity = page.getByTestId('account-activity'); const rows = activity.getByRole('listitem');
  await expect(rows).toHaveCount(3);
  await expect(rows.first().getByText('Linked mailbox sign-in renewed', { exact: true })).toBeVisible();
  await expect(rows.first().locator('time')).toContainText('5:37:48 PM');
  await expect(rows.first().locator('time')).toHaveAttribute('datetime', timestamp);
  await expect(rows.first().getByText(timestamp, { exact: true })).toBeHidden();
  await rows.first().getByText('Event details', { exact: true }).click();
  await expect(rows.first().getByText(timestamp, { exact: true })).toBeVisible();
  await expect(rows.first().locator('pre')).toContainText('first-authority');
  await page.screenshot({ path: info.outputPath('activity-desktop.png'), fullPage: true });
  await page.setViewportSize({ width: 390, height: 844 }); await activity.scrollIntoViewIfNeeded();
  const box = await activity.boundingBox(); expect(box).not.toBeNull(); expect(box!.x).toBeGreaterThanOrEqual(0); expect(box!.x + box!.width).toBeLessThanOrEqual(391);
  await expect(rows.first().locator('pre')).toBeVisible();
  expect(await activity.evaluate(node => node.scrollWidth <= node.clientWidth + 1)).toBe(true);
  await page.screenshot({ path: info.outputPath('activity-phone.png'), fullPage: true });
});
