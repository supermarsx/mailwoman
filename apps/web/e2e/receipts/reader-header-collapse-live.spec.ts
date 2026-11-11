import {test,expect} from '../csrf.ts';
import {login,messageRow} from '../helpers.ts';

test('opt-in collapses message headers, keeps actions available, and resets on the next message',async({page})=>{
 await page.route('**/api/settings',async route=>{
  const response=await route.fetch();const body=await response.json();
  body.settings={...body.settings,'reader.autoHideHeader':{value:true,source:'user',locked:false}};
  await route.fulfill({response,json:body});
 });
 await login(page);await messageRow(page,'Lunch tomorrow?').click();
 const details=page.getByTestId('reader-header-details');
 await expect(details).toBeHidden();await expect(page.getByRole('heading',{name:'Lunch tomorrow?',exact:true})).toBeVisible();
 await expect(page.getByRole('toolbar',{name:'Message actions'})).toBeVisible();
 const show=page.getByRole('button',{name:'Show header',exact:true});await expect(show).toHaveAttribute('aria-expanded','false');await show.click();
 await expect(details).toBeVisible();await expect(page.getByRole('button',{name:'Hide header',exact:true})).toHaveAttribute('aria-expanded','true');
 await page.locator('.list__subject').filter({hasNotText:'Lunch tomorrow?'}).first().click();
 await expect(show).toBeVisible();await expect(details).toBeHidden();
});
