import {test,expect} from '../csrf.ts';
import {signInManually,drawerButton,storageAuthorityHeaders} from '../helpers.ts';
test('time zones persist on the account and validate before save @phone',async({page,context})=>{
 await signInManually(page,{username:'testuser',password:'testpass'});
 await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 const csrf=(await context.cookies()).find(cookie=>cookie.name==='mw_csrf')!.value;
 const headers={'X-CSRF-Token':csrf,...(await storageAuthorityHeaders(page.request))};
 const initial=(await(await page.request.get('/api/settings')).json()).settings;
 try{
  if(test.info().project.name==='phone')await drawerButton(page).click();
  await page.locator('.sidebar__settings').click();
  await page.getByRole('button',{name:'Time zones',exact:true}).click();
  const field=page.getByLabel('Comparison time zone',{exact:true});
  await field.fill('person@example.test');
  await field.locator('xpath=ancestor::form').getByRole('button',{name:'Save time zone'}).click();
  await expect(page.getByRole('alert')).toContainText('valid IANA');
  await field.fill('America/New_York');
  await field.locator('xpath=ancestor::form').getByRole('button',{name:'Save time zone'}).click();
  await expect(field.locator('xpath=ancestor::form').getByRole('button',{name:'Save time zone'})).toBeDisabled();
  expect((await(await page.request.get('/api/settings')).json()).settings['time.comparisonZone'].userValue).toBe('America/New_York');
  await page.reload();
  if(test.info().project.name==='phone')await drawerButton(page).click();
  await page.locator('.sidebar__settings').click();
  await page.getByRole('button',{name:'Time zones',exact:true}).click();
  await expect(page.getByLabel('Comparison time zone',{exact:true})).toHaveValue('America/New_York');
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
  await page.screenshot({path:test.info().outputPath('time-zones.png'),fullPage:true});
 }finally{
  const value=initial['time.comparisonZone']?.userValue;
  const restored=value===undefined?await page.request.delete('/api/settings/time.comparisonZone',{headers}):await page.request.put('/api/settings/time.comparisonZone',{headers,data:{value}});
  expect(restored.ok(),await restored.text()).toBe(true);
 }
});
