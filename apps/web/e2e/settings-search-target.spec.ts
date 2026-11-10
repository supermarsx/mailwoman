import {test,expect} from '@playwright/test';
import {login,settingsSurface} from './helpers.ts';
test.use({serviceWorkers:'block'});test.setTimeout(90000);
for(const reducedMotion of ['no-preference','reduce'] as const)test(`settings search scrolls and highlights the exact row (${reducedMotion})`,async({page})=>{
 await page.setViewportSize({width:1280,height:720});await page.emulateMedia({reducedMotion});
 await page.route(/\/api\/settings(?:\?.*)?$/,route=>route.fulfill({json:{settings:{'reader.fontSize':{value:16,source:'default',locked:false},'notifications.askPermission':{value:false,source:'global',locked:true}}}}));
 await login(page);await page.getByRole('button',{name:'Settings',exact:true}).click();
 const screen=settingsSurface(page),search=screen.getByRole('searchbox',{name:'Search settings'});
 await search.fill('Reading font size');
 await screen.locator('[data-settings-hit="reader-fontSize"]').click();
 const control=screen.locator('#reader-fontSize'),row=control.locator('xpath=ancestor::*[@data-setting-row][1]');
 await expect(control).toBeFocused();await expect(row).toHaveAttribute('data-card-flash','');await expect(row).toBeInViewport({ratio:1});
 expect(await row.evaluate(element=>getComputedStyle(element).outlineStyle)).toBe('solid');
 await expect(search).toHaveValue('');
 await expect(row).not.toHaveAttribute('data-card-flash','');
 await search.fill('Reading font size');await screen.locator('[data-settings-hit="reader-fontSize"]').click();
 await expect(control).toBeFocused();await expect(row).toHaveAttribute('data-card-flash','');
});
