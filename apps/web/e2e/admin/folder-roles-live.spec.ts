import {test,expect,type Page} from '../csrf.ts';
import {signInManually,storageAuthorityHeaders} from '../helpers.ts';
async function openRoles(page:Page):Promise<void>{
 await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 const drawer=page.getByRole('button',{name:'Open folders and apps'});if(await drawer.isVisible())await drawer.click();
 const show=page.getByRole('button',{name:'Show folders sidebar'});if(await show.isVisible())await show.click();
 await page.getByRole('button',{name:'Settings',exact:true}).click();
 await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Folders and sidebar'}).click();
 await page.getByText('Special folders',{exact:true}).click();
}
test('folder role selection persists across reload and automatic reset restores discovery @phone',async({page,context})=>{
 await signInManually(page,{username:'testuser',password:'testpass'});
 await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 const csrf=(await context.cookies()).find(cookie=>cookie.name==='mw_csrf')?.value;expect(csrf).toBeTruthy();const headers={'X-CSRF-Token':csrf!,...(await storageAuthorityHeaders(page.request))};
 const key='mail.folderRole.archive';const initial=(await(await page.request.get('/api/settings')).json()).settings[key];expect(initial).toBeTruthy();
 try{
  await openRoles(page);const select=page.getByRole('combobox',{name:'Archive',exact:true});await expect(select).toBeEnabled();
  const option=select.locator('option[value^="id:"]').first();const value=(await option.getAttribute('value'))!;expect(value).toBeTruthy();
  await select.selectOption(value);await expect.poll(async()=> (await(await page.request.get('/api/settings')).json()).settings[key].userValue).toEqual({id:value.slice(3)});
  await expect(select).toBeEnabled();await page.reload();await openRoles(page);await expect(page.getByRole('combobox',{name:'Archive',exact:true})).toHaveValue(value);
  await page.screenshot({path:test.info().outputPath('folder-roles.png'),fullPage:true});await page.getByRole('combobox',{name:'Archive',exact:true}).scrollIntoViewIfNeeded();await page.screenshot({path:test.info().outputPath('folder-role-controls.png'),fullPage:true});expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.getByRole('combobox',{name:'Archive',exact:true}).selectOption('auto');await expect.poll(async()=> (await(await page.request.get('/api/settings')).json()).settings[key].userValue).toBeNull();
 }finally{
  const response=initial.userValue===undefined?await page.request.delete(`/api/settings/${key}`,{headers}):await page.request.put(`/api/settings/${key}`,{headers,data:{value:initial.userValue}});expect(response.ok(),await response.text()).toBe(true);
 }
});
