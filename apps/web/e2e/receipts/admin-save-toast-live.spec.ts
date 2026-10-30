import {test,expect} from '../csrf.ts';
import {adminLogin} from '../admin/admin-login.ts';
import {ADMIN} from '../admin/admin-path.ts';

test('personal preferences save only announces progress and completion in corner toasts',async({page},info)=>{
 await adminLogin(page);
 await page.goto(`${ADMIN}#/preferences/layout`);
 const density=page.getByLabel('Interface density');
 await expect(density).toBeVisible();
 const original=await density.inputValue();
 const region=page.getByTestId('toast-region');
 const panel=page.getByRole('tabpanel',{name:'Reading and layout',exact:true});
 await expect(region.getByText('Preferences saved to your account.',{exact:true})).toHaveCount(0);
 let release!:()=>void;
 const held=new Promise<void>(resolve=>{release=resolve;});
 const path=`${ADMIN}/me/preferences`;
 await page.route(url=>url.pathname===path,async route=>{if(route.request().method()==='PUT')await held;await route.continue();});
 try {
  const response=page.waitForResponse(r=>new URL(r.url()).pathname===path&&r.request().method()==='PUT');
  await density.selectOption(original==='compact'?'cozy':'compact');
  await expect(region.getByText('Saving changes…',{exact:true})).toBeVisible();
  await expect(panel.getByText(/Saving changes|Preferences saved/)).toHaveCount(0);
  await page.screenshot({path:info.outputPath('preferences-saving-toast.png'),fullPage:true});
  release();expect((await response).status()).toBe(200);
  await expect(region.getByText('Preferences saved to your account.',{exact:true})).toBeVisible();
  await expect(region.getByText('Saving changes…',{exact:true})).toHaveCount(0);
  await expect(panel.getByText(/Saving changes|Preferences saved/)).toHaveCount(0);
  await page.screenshot({path:info.outputPath('preferences-saved-toast.png'),fullPage:true});
 } finally {
  release();await page.unrouteAll({behavior:'wait'});
  if(await density.inputValue()!==original){const restored=page.waitForResponse(r=>new URL(r.url()).pathname===path&&r.request().method()==='PUT');await density.selectOption(original);expect((await restored).status()).toBe(200);}
 }
});
