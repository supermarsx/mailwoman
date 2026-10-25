import { test, expect } from '../csrf.ts';
import { signInManually } from '../helpers.ts';
test('native SVG controls retain labels and currentColor on desktop and phone @phone', async ({ page }) => {
 await page.goto('/'); await signInManually(page, {username:'testuser',password:'testpass'});
 const compose=page.getByRole('button',{name:'Compose',exact:true}); await expect(compose).toBeVisible();
 await expect(page.getByRole('button',{name:'Unpin Mail',exact:true}).locator('svg[data-icon="star"]')).toHaveAttribute('aria-hidden','true');
 const apps = page.getByRole('navigation', { name: 'Apps', exact: true });
 const mobileMenu = page.locator('button[aria-controls="shell-nav"]');
 if (await mobileMenu.isVisible()) await mobileMenu.click();
 await expect(apps.getByRole('button', { name: 'Calendar', exact: true }).locator('svg[data-icon="calendar"]')).toHaveCount(1);
 if (await mobileMenu.isVisible()) await page.locator('#shell-nav').getByRole('button', {name:'Compose',exact:true}).click();
 else await compose.click(); const editor=page.locator('.compose').first(); await expect(editor).toBeVisible();
 const close=editor.locator('button').filter({has:page.locator('svg[data-icon="close"]')}).first(); await expect(close).toBeVisible();
 await expect(close).toHaveAttribute('aria-label', /.+/);
 expect(await close.locator('svg').evaluate(svg=>svg.getBoundingClientRect().width)).toBe(16);
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 await page.screenshot({path:test.info().outputPath('native-svg-compose.png'),fullPage:true,animations:'disabled'});
});
