// t39 R5: the reading-pane loader and its two saved preferences against a live
// server. Settings > Message display stores reader.loading and
// reader.loadingDelayMs through /api/settings; a reload reads them back; while a
// message body is on its way the reader shows the chosen loader (placeholder
// lines, a text indicator, or an announcement only). The server's responses are
// real: the policy and grants requests are only held back for a moment
// (route.continue after a delay) so the loader is on screen long enough to inspect.
import { test, expect, type Page } from '../csrf.ts';

async function signIn(page: Page): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill('testuser');
  await page.getByLabel('Password', { exact: true }).fill('testpass');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.locator('.list__row').first()).toBeVisible();
}
const touch = (page: Page): boolean => (page.viewportSize()?.width ?? 1280) < 600;
async function headers(page: Page): Promise<Record<string, string>> {
  const me = (await (await page.request.get('/api/me')).json()) as { storageAuthority: string };
  return { 'x-mailwoman-storage-authority': me.storageAuthority };
}
async function stored(page: Page, key: string): Promise<{ value: unknown; source: string }> {
  const body = (await (await page.request.get('/api/settings')).json()) as { settings: Record<string, { value: unknown; source: string }> };
  return body.settings[key]!;
}
async function openSettings(page: Page): Promise<void> {
  if (touch(page)) {
    const menu = page.locator('header.shell__bar button[aria-controls="shell-nav"]').first();
    if ((await menu.getAttribute('aria-expanded')) !== 'true') await menu.click();
  }
  await page.locator('.sidebar, #shell-nav').getByRole('button', { name: 'Settings', exact: true }).first().click();
  await page.getByRole('navigation', { name: 'Settings sections' }).getByRole('button', { name: 'Message display', exact: true }).click();
}
async function backToMail(page: Page): Promise<void> {
  await expect(page.locator('div[data-bar="mailTopBar"]')).toBeAttached();
  if (touch(page)) {
    const menu = page.locator('header.shell__bar button[aria-controls="shell-nav"]').first();
    if ((await menu.getAttribute('aria-expanded')) !== 'true') await menu.click();
  }
  await page.locator('.sidebar, #shell-nav').getByRole('button', { name: /^Inbox/ }).first().click();
  if (touch(page)) await page.keyboard.press('Escape');
  await expect(page.locator('.list__row').first()).toBeVisible();
}
/** Opens the n-th list row with its policy and grants requests held back; returns the loader. */
async function openSlowly(page: Page, n: number) {
  let held = 0;
  await page.route('**/*', async (route) => {
    const req = route.request();
    // The reader shows its loader while the opened message's privacy policy and
    // image grants load (Reader.tsx readerBody waits on both); hold those two.
    if (req.method() === 'GET' && /\/api\/(rc\/policy|remote-images\/grants)$/.test(new URL(req.url()).pathname)) {
      held++;
      await new Promise((resolve) => setTimeout(resolve, 2500));
    }
    await route.continue();
  });
  await page.locator('.list__row').nth(n).click();
  const loader = page.getByRole('status').filter({ hasText: 'Sanitizing…' }).first();
  await expect(loader).toBeAttached({ timeout: 2000 });
  return { loader, held: () => held, done: () => page.unrouteAll({ behavior: 'wait' }) };
}

test('loader preferences are stored, read back after a reload, and drive the reader loader', async ({ page }) => {
  page.setDefaultTimeout(15000);
  await signIn(page);
  const h = await headers(page);
  try {
    await openSettings(page);
    const loading = page.locator('#reader-loading');
    const delay = page.locator('#reader-loadingDelayMs');
    await expect(loading).toHaveValue('skeleton');
    await expect(delay).toHaveValue('120');
    const savedLoading = page.waitForResponse((r) => r.request().method() === 'PUT' && r.url().endsWith('/api/settings/reader.loading'));
    await loading.selectOption('indicator');
    expect((await savedLoading).status()).toBe(200);
    const savedDelay = page.waitForResponse((r) => r.request().method() === 'PUT' && r.url().endsWith('/api/settings/reader.loadingDelayMs'));
    await delay.fill('0');
    await delay.press('Tab');
    expect((await savedDelay).status()).toBe(200);
    expect(await stored(page, 'reader.loading')).toMatchObject({ value: 'indicator', source: 'user' });
    expect(await stored(page, 'reader.loadingDelayMs')).toMatchObject({ value: 0, source: 'user' });
    await page.reload();
    await expect(page.locator('div[data-bar="mailTopBar"]')).toBeAttached();
    await openSettings(page);
    await expect(page.locator('#reader-loading')).toHaveValue('indicator');
    await expect(page.locator('#reader-loadingDelayMs')).toHaveValue('0');

    // Text indicator: visible text, no placeholder lines, delay 0.
    await backToMail(page);
    let slow = await openSlowly(page, 1);
    await expect(slow.loader).toBeVisible();
    await expect(slow.loader.locator('[aria-hidden="true"]')).toHaveCount(0);
    expect(await slow.loader.evaluate((el) => (el as HTMLElement).style.getPropertyValue('--reader-loading-delay'))).toBe('0ms');
    expect(slow.held()).toBeGreaterThan(0);
    await expect(page.locator('.reader__subject').filter({ visible: true }).first()).toBeVisible({ timeout: 15000 });
    await slow.done();

    // None: announced to assistive technology only (visually hidden), no lines.
    expect((await page.request.put('/api/settings/reader.loading', { data: { value: 'none' }, headers: h })).status()).toBe(200);
    await page.reload();
    await backToMail(page);
    slow = await openSlowly(page, 2);
    await expect(slow.loader.locator('[aria-hidden="true"]')).toHaveCount(0);
    const box = await slow.loader.boundingBox();
    expect(box === null || box.width <= 1 || box.height <= 1).toBe(true);
    await slow.done();

    // Skeleton (the default once the user value is removed): three placeholder lines.
    expect((await page.request.delete('/api/settings/reader.loading', { headers: h })).status()).toBe(200);
    await page.reload();
    await backToMail(page);
    slow = await openSlowly(page, 3);
    await expect(slow.loader.locator('[aria-hidden="true"] > span')).toHaveCount(3);
    await slow.done();
    expect(await stored(page, 'reader.loading')).toMatchObject({ value: 'skeleton', source: 'default' });
  } finally {
    await page.unrouteAll({ behavior: 'ignoreErrors' });
    await page.request.delete('/api/settings/reader.loading', { headers: h });
    await page.request.delete('/api/settings/reader.loadingDelayMs', { headers: h });
  }
});
