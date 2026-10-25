import { test, expect } from '../csrf.ts';
import { signInManually } from '../helpers.ts';

test('font choices load bundled files without third-party requests @phone', async ({ page }) => {
  const fonts: string[] = [];
  page.on('request', request => { if (request.resourceType() === 'font') fonts.push(request.url()); });
  await page.goto('/');
  await signInManually(page, { username: 'testuser', password: 'testpass' });
  const show = page.getByRole('button', { name: 'Show folders sidebar' });
  if (await show.isVisible()) await show.click();
  if (test.info().project.name === 'phone') await page.getByRole('button', { name: 'Open folders and apps' }).click();
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByRole('navigation', { name: 'Settings sections' }).getByRole('button', { name: 'Appearance', exact: true }).click();
  const group = page.getByRole('group', { name: 'Interface font', exact: true });
  for (const [label, family] of [['Inter', 'Inter Variable'], ['Roboto', 'Roboto Variable'], ['Open Sans', 'Open Sans Variable'], ['Serif', 'Newsreader Variable'], ['Mono', 'JetBrains Mono Variable'], ['Jost', 'Jost']]) {
    await group.getByRole('button', { name: label!, exact: true }).click();
    await expect(group.getByRole('button', { name: label!, exact: true })).toHaveAttribute('aria-pressed', 'true');
    expect(await page.evaluate(async name => { await document.fonts.load(`16px "${name}"`); return document.fonts.check(`16px "${name}"`); }, family!)).toBe(true);
  }
  expect(fonts.length).toBeGreaterThanOrEqual(6);
  expect(fonts.every(url => new URL(url).origin === new URL(page.url()).origin)).toBe(true);
  await group.getByRole('button', { name: 'Default', exact: true }).click();
});
