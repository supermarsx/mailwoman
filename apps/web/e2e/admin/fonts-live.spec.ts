import { test, expect } from '../csrf.ts';
import { signInManually, drawerButton } from '../helpers.ts';

test('bundled font selection persists and loads without external font requests @phone', async ({page,context}) => {
  const fontRequests: string[] = [];
  page.on('request', request => {
    if (/\.(woff2?|ttf)(\?|$)|fonts\.googleapis|fonts\.gstatic/.test(request.url())) fontRequests.push(request.url());
  });
  await signInManually(page,{username:'testuser',password:'testpass'});
  await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
  const before = await (await page.request.get('/api/account/appearance')).json();
  const csrf = (await context.cookies()).find(cookie => cookie.name === 'mw_csrf' && cookie.path === '/')!.value;
  async function openAppearance() {
    if (test.info().project.name === 'phone') await drawerButton(page).click();
    await page.locator('.sidebar__settings').click();
    await page.getByRole('button',{name:'Appearance',exact:true}).click();
  }
  try {
    await openAppearance();
    const group = page.getByRole('group',{name:'Interface font'});
    for (const [label, family] of [['Inter', 'Inter Variable'], ['Roboto', 'Roboto Variable'], ['Open Sans', 'Open Sans Variable'], ['Serif', 'Newsreader Variable'], ['Mono', 'JetBrains Mono Variable'], ['Jost', 'Jost']]) {
      await group.getByRole('button', {name:label!,exact:true}).click();
      await expect(group.getByRole('button',{name:label!,exact:true})).toHaveAttribute('aria-pressed','true');
      expect(await page.evaluate(async name => {
        await document.fonts.load(`16px "${name}"`);
        return document.fonts.check(`16px "${name}"`);
      },family!)).toBe(true);
    }
    await group.getByRole('button',{name:'Inter',exact:true}).click();
    await expect.poll(async () => (await (await page.request.get('/api/account/appearance')).json()).appearance?.font).toBe('inter');
    await page.reload();
    await expect.poll(() => page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue('--mw-ui-font'))).toContain('Inter');
    await openAppearance();
    await expect(group.getByRole('button',{name:'Inter',exact:true})).toHaveAttribute('aria-pressed','true');
    await page.evaluate(() => document.fonts.ready);
    expect(fontRequests.length).toBeGreaterThanOrEqual(6);
    const origin = new URL(page.url()).origin;
    expect(fontRequests.every(url => new URL(url).origin === origin)).toBe(true);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth+1)).toBe(true);
    await group.scrollIntoViewIfNeeded();
    await page.screenshot({path:test.info().outputPath('font-appearance.png'),fullPage:true});
  } finally {
    if (before.appearance) await page.request.put('/api/account/appearance',{headers:{'X-CSRF-Token':csrf},data:{appearance:before.appearance}});
    else await page.request.delete('/api/account/appearance',{headers:{'X-CSRF-Token':csrf}});
  }
});
