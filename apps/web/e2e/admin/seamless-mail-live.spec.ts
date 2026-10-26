import { test, expect } from '../csrf.ts';
import { signInManually } from '../helpers.ts';

test('folder loading retains its viewport and menus do not blank the page @phone', async ({ page }) => {
  await signInManually(page, { username: 'testuser', password: 'testpass' });
  await expect(page.locator('.list__scroll')).toBeVisible();
  const sessionResponse = await page.request.get('/jmap/session');
  expect(sessionResponse.ok()).toBe(true);
  const session = await sessionResponse.json();
  const accountId = session.primaryAccounts['urn:ietf:params:jmap:mail'];
  const using = ['urn:ietf:params:jmap:core', 'urn:ietf:params:jmap:mail'];
  const existing = await (await page.request.post('/jmap/api', { data: { using, methodCalls: [['Mailbox/get', { accountId }, 'get']] } })).json();
  if (!existing.methodResponses[0][1].list.some((row: { name: string }) => row.name === 'Seamless loading')) {
    const created = await (await page.request.post('/jmap/api', { data: { using, methodCalls: [['Mailbox/set', { accountId, create: { target: { name: 'Seamless loading' } } }, 'create']] } })).json();
    expect(created.methodResponses[0][1].created?.target?.id).toBeTruthy();
  }
  await page.reload();
  const scroller = page.locator('.list__scroll');
  await expect(scroller).toBeVisible();
  await expect(page.locator('.list')).toHaveAttribute('aria-busy', 'false');
  await scroller.evaluate(el => el.setAttribute('data-retained-probe', 'yes'));
  const more = page.getByTestId('msg-more-toggle').first();
  if (await more.isVisible()) await more.click();
  else await page.locator('.list__slot').first().hover();
  await page.getByRole('button', { name: 'Snooze', exact: true }).first().click();
  const menu = page.getByRole('menu').first();
  await expect(menu).toBeVisible();
  expect(await menu.evaluate(el => Number(getComputedStyle(el).opacity))).toBeGreaterThanOrEqual(.96);
  await page.keyboard.press('Escape');
  await expect(menu).toHaveCount(0);
  if (test.info().project.name === 'phone') await page.getByRole('button', { name: 'Open folders and apps' }).click();
  const show = page.getByRole('button', { name: 'Show folders sidebar' });
  if (await show.isVisible()) await show.click();
  let release!: () => void;
  const gate = new Promise<void>(resolve => { release = resolve; });
  let held = false;
  await page.route('**/jmap/api', async route => {
    if (!held && route.request().postData()?.includes('Email/query')) {
      held = true;
      await gate;
    }
    await route.continue();
  });
  try {
    await page.getByRole('navigation', { name: 'Mailboxes' }).getByRole('button', { name: 'Seamless loading', exact: true }).click();
    await expect(page.locator('.list')).toHaveAttribute('aria-busy', 'true');
    await expect(scroller).toHaveAttribute('data-retained-probe', 'yes');
    expect(await scroller.evaluate(el => (el as HTMLElement).inert)).toBe(true);
    expect(await page.locator('.list').evaluate(el => Number(getComputedStyle(el).opacity))).toBe(1);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  } finally { release(); }
  await expect(page.locator('.list')).toHaveAttribute('aria-busy', 'false');
  await page.emulateMedia({ reducedMotion: 'reduce' });
  expect(await page.getByRole('button').first().evaluate(el => parseFloat(getComputedStyle(el).transitionDuration))).toBeLessThanOrEqual(.001);
  if (test.info().project.name === 'phone') await page.getByRole('button', { name: 'Open folders and apps' }).click();
  await page.getByRole('navigation', { name: 'Mailboxes' }).getByRole('button', { name: 'Attachments', exact: true }).click();
  await expect(page.locator('.sidebar__nav').first()).toBeAttached();
  await expect(page.getByRole('heading', { name: 'Attachments', exact: true })).toBeVisible();
});
