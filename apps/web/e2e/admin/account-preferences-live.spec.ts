import {test,expect,type Page} from '../csrf.ts';
import {signInManually} from '../helpers.ts';
async function folders(page:Page):Promise<void>{
 const show=page.getByRole('button',{name:'Show folders sidebar'});if(await show.isVisible())await show.click();
 await page.getByRole('button',{name:'Settings',exact:true}).click();
 await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Folders and sidebar'}).click();
 await expect(page.getByRole('region',{name:'Folders and sidebar'})).toBeVisible();
}
test('separate devices merge folder icons and workspace layout through account CAS',async({page,browser,baseURL})=>{
 const other=await browser.newContext({...(baseURL ? {baseURL} : {}),viewport:{width:1280,height:900}});const second=await other.newPage();
 try{
  for(const target of [page,second]){await target.goto('/');await signInManually(target,{username:'testuser',password:'testpass'});await expect(target.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();}
  await folders(page);await folders(second);
  const first=page.getByRole('region',{name:'Folders and sidebar'}).getByRole('combobox').first();
  const label=(await first.locator('..').locator('span').textContent())!;
  const next=(await first.inputValue())==='star'?'calendar':'star';
  const firstSaved=page.waitForResponse(response=>response.url().endsWith('/api/account/preferences')&&response.request().method()==='PUT'&&response.status()===200);
  await first.selectOption(next);await firstSaved;
  const expected=(await(await page.request.get('/api/account/preferences')).json()).preferences.folderIcons;
  const layout=second.getByRole('toolbar',{name:'Workspace layout'}).getByRole('button',{name:'Stacked',exact:true});
  const targetLayout=await layout.getAttribute('aria-pressed')==='true'?'tabs':'stack';
  await second.getByRole('toolbar',{name:'Workspace layout'}).getByRole('button',{name:targetLayout==='tabs'?'Tabs':'Stacked',exact:true}).click();
  await expect.poll(async()=> (await(await second.request.get('/api/account/preferences')).json()).preferences.workspaceLayout).toBe(targetLayout);
  expect((await(await second.request.get('/api/account/preferences')).json()).preferences.folderIcons).toEqual(expected);
  await second.reload();await expect(second.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();await folders(second);
  await expect(second.getByRole('combobox',{name:label,exact:true})).toHaveValue(next);
  await second.screenshot({path:test.info().outputPath('account-folders-second-device.png'),fullPage:true});
 }finally{await other.close();}
});
