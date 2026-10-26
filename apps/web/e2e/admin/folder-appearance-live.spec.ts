import {test,expect} from '../csrf.ts';
import {signInManually,drawerButton} from '../helpers.ts';
test('folder icons edit on a dedicated page and persist with the account @phone',async({page,context})=>{
 await signInManually(page,{username:'testuser',password:'testpass'});
 await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 const initial=await page.request.get('/api/account/preferences');const saved=await initial.json();
 const csrf=(await context.cookies()).find(cookie=>cookie.name==='mw_csrf')!.value;
 try{
  if(test.info().project.name==='phone')await drawerButton(page).click();
  await page.locator('.sidebar__settings').click();
  await page.getByRole('button',{name:'Folders and sidebar',exact:true}).click();
  const region=page.getByRole('region',{name:'Folders and sidebar',exact:true});
  await expect(region.getByRole('table')).toBeVisible();await expect(region.getByRole('combobox')).toHaveCount(0);
  await region.getByRole('button',{name:'Edit Inbox appearance',exact:true}).click();
  const select=page.getByRole('combobox',{name:'Icon for Inbox'});
  await expect(select.locator('option')).toHaveCount(16);
  const chosen=(await select.inputValue())==='flag'?'calendar':'flag';await select.selectOption(chosen);
  await page.getByRole('button',{name:'Save folder appearance'}).click();
  await expect(region.getByRole('table')).toBeVisible();
  await expect.poll(async()=>{const body=await(await page.request.get('/api/account/preferences')).json();return Object.values(body.preferences?.folderIcons??{}).includes(chosen);}).toBe(true);
  await page.reload();
  if(test.info().project.name==='phone')await drawerButton(page).click();
  await page.locator('.sidebar__settings').click();await page.getByRole('button',{name:'Folders and sidebar',exact:true}).click();
  await page.getByRole('button',{name:'Edit Inbox appearance',exact:true}).click();await expect(select).toHaveValue(chosen);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
  await page.screenshot({path:test.info().outputPath('folder-appearance.png'),fullPage:true});
 }finally{
  const current=await page.request.get('/api/account/preferences');
  const response=await page.request.put('/api/account/preferences',{headers:{'X-CSRF-Token':csrf,'If-Match':current.headers()['etag']!},data:saved});
  expect(response.ok(),await response.text()).toBe(true);
 }
});
