import {test,expect,type Page} from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import {signInManually} from '../helpers.ts';
async function openSecurity(page:Page):Promise<void>{
 const sections=page.getByRole('navigation',{name:'Settings sections'});
 if(!await sections.isVisible()){
  const show=page.getByRole('button',{name:'Show folders sidebar'});if(await show.isVisible())await show.click();
  const drawer=page.getByRole('button',{name:'Open folders and apps'});if((page.viewportSize()?.width??1280)<600)await drawer.click();else if(await drawer.isVisible())await drawer.click();
  await page.getByRole('button',{name:'Settings',exact:true}).click();
 }
 await sections.getByRole('button',{name:'Security centre',exact:true}).click();
}
test('passkey enrollment is available in personal settings and respects effective policy @phone',async({page,browser,baseURL})=>{
 test.setTimeout(120_000);await new Promise(resolve=>setTimeout(resolve,3200));
 await page.goto(ADMIN);await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect.poll(async()=>(await page.request.get(`${ADMIN}/session`)).status()).toBe(200);
 await page.goto(`${ADMIN}#/preferences`);await expect(page.getByRole('button',{name:'Add administrator passkey'})).toBeEnabled();
 const path=`${ADMIN}/settings/twofa.passkeysEnabled`;const original=await page.request.get(path);expect(original.status()).toBe(200);const previous=(await original.json()).globalValue;
 const set=async(value:unknown):Promise<void>=>{const row=await page.request.get(path);const saved=await page.request.put(path,{headers:{'if-match':row.headers()['etag']!},data:{globalValue:value}});expect(saved.status(),await saved.text()).toBe(200);};
 const user=await browser.newPage({baseURL:baseURL!,viewport:page.viewportSize()!});
 try{
  await set(true);await new Promise(resolve=>setTimeout(resolve,3200));await user.goto('/');await signInManually(user,{username:'testuser',password:'testpass'});await openSecurity(user);
  await expect(user.getByRole('button',{name:'Add a passkey',exact:true})).toBeEnabled();await user.getByRole('button',{name:'Add a passkey',exact:true}).scrollIntoViewIfNeeded();await user.screenshot({path:test.info().outputPath('user-passkeys-available.png'),fullPage:true});
  await set(false);await user.reload();await openSecurity(user);await expect(user.getByText(/administrator has disabled passkey/)).toBeVisible();await expect(user.getByRole('button',{name:'Add a passkey',exact:true})).toHaveCount(0);
  await user.getByText(/administrator has disabled passkey/).scrollIntoViewIfNeeded();await user.screenshot({path:test.info().outputPath('user-passkeys-policy-disabled.png'),fullPage:true});
  await page.reload();await expect(page.getByRole('button',{name:'Add administrator passkey'})).toBeEnabled();await page.getByRole('button',{name:'Add administrator passkey'}).scrollIntoViewIfNeeded();await page.screenshot({path:test.info().outputPath('admin-passkeys-available.png'),fullPage:true});
 }finally{await set(previous);await user.close();}
});
