import {test,expect} from '../csrf.ts';
import {signInManually,drawerButton,storageAuthorityHeaders} from '../helpers.ts';
test('folder icons and labels edit on a dedicated page and persist with the account @phone',async({page,context})=>{
 await signInManually(page,{username:'testuser',password:'testpass'});
 await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 const initial=await page.request.get('/api/account/preferences');const saved=await initial.json();
 const csrf=(await context.cookies()).find(cookie=>cookie.name==='mw_csrf')!.value;const authority=await storageAuthorityHeaders(page.request);
 try{
  const seeded=await page.request.put('/api/account/preferences',{headers:{'X-CSRF-Token':csrf,...authority,'If-Match':initial.headers()['etag']!},data:{...saved,preferences:{...saved.preferences,tags:{...saved.preferences.tags,'folder-project':{name:'Project correspondence',color:'#2563eb',icon:'tag'}}}}});expect(seeded.ok(),await seeded.text()).toBe(true);
  await page.reload();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
  if(test.info().project.name==='phone')await drawerButton(page).click();
  await page.locator('.sidebar__settings').click();
  await page.getByRole('button',{name:'Folders and sidebar',exact:true}).click();
  const region=page.getByRole('region',{name:'Folders and sidebar',exact:true});
  await expect(region.getByRole('table')).toBeVisible();
  // No appearance control inline in the folder table: they live on the dedicated edit page. (The section's own
  // "Folder counts" controls above the table, 12445b55, are not per-folder.)
  await expect(region.getByRole('table').getByRole('combobox')).toHaveCount(0);await expect(region.getByRole('table').getByRole('checkbox')).toHaveCount(0);
  await region.getByRole('button',{name:'Edit Inbox appearance',exact:true}).click();
  // The icon choice is a radio group: "Automatic folder role" plus every FOLDER_ICONS entry (75 in
  // src/api/accountPreferences.ts), grouped by theme and filtered by the search box above it.
  const picker=page.getByRole('radiogroup',{name:'Icon for Inbox'});
  await expect(picker.getByRole('radio')).toHaveCount(76);
  await expect(picker.getByRole('group')).not.toHaveCount(0);
  await page.getByRole('searchbox',{name:'Search icons'}).fill('calend');
  await expect(picker.getByRole('radio',{name:'Calendar',exact:true})).toBeVisible();
  await expect(picker.getByRole('radio',{name:'Flag',exact:true})).toHaveCount(0);
  await page.getByRole('searchbox',{name:'Search icons'}).fill('');
  await expect(picker.getByRole('radio')).toHaveCount(76);
  const chosen=(await picker.getByRole('radio',{name:'Flag',exact:true}).getAttribute('aria-checked'))==='true'?'calendar':'flag';
  const chosenName=chosen==='flag'?'Flag':'Calendar';
  await picker.getByRole('radio',{name:chosenName,exact:true}).click();
  await expect(picker.getByRole('radio',{name:chosenName,exact:true})).toHaveAttribute('aria-checked','true');
  await page.getByRole('checkbox',{name:'Project correspondence',exact:true}).check();
  await page.getByRole('button',{name:'Save folder appearance'}).click();
  await expect(region.getByRole('table')).toBeVisible();
  await expect.poll(async()=>{const body=await(await page.request.get('/api/account/preferences')).json();return Object.values(body.preferences?.folderIcons??{}).includes(chosen);}).toBe(true);
  await expect.poll(async()=>{const body=await(await page.request.get('/api/account/preferences')).json();return Object.values(body.preferences?.folderLabels??{}).some(labels=>Array.isArray(labels)&&labels.includes('folder-project'));}).toBe(true);
  await page.reload();
  if(test.info().project.name==='phone')await drawerButton(page).click();
  await page.locator('.sidebar__settings').click();await page.getByRole('button',{name:'Folders and sidebar',exact:true}).click();
  await page.getByRole('button',{name:'Edit Inbox appearance',exact:true}).click();await expect(picker.getByRole('radio',{name:chosenName,exact:true})).toHaveAttribute('aria-checked','true');
  await expect(page.getByRole('checkbox',{name:'Project correspondence',exact:true})).toBeChecked();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
  await page.screenshot({path:test.info().outputPath('folder-appearance.png'),fullPage:true});
 }finally{
  const current=await page.request.get('/api/account/preferences');
  const response=await page.request.put('/api/account/preferences',{headers:{'X-CSRF-Token':csrf,...authority,'If-Match':current.headers()['etag']!},data:saved});
  expect(response.ok(),await response.text()).toBe(true);
 }
});
