import {test,expect,type Page} from '../csrf.ts';
import {signInManually,storageAuthorityHeaders} from '../helpers.ts';
async function folders(page:Page):Promise<void>{
 const show=page.getByRole('button',{name:'Show folders sidebar'});if(await show.isVisible())await show.click();
 await page.getByRole('button',{name:'Settings',exact:true}).click();
 await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Folders and sidebar'}).click();
 await expect(page.getByRole('region',{name:'Folders and sidebar'})).toBeVisible();
}
test('separate devices merge folder icons and workspace layout through account CAS',async({page,browser,baseURL})=>{
 const other=await browser.newContext({...(baseURL ? {baseURL} : {}),viewport:{width:1280,height:900}});const second=await other.newPage();
 let original:unknown;
 try{
  for(const target of [page,second]){await target.waitForTimeout(3500);await target.goto('/');await signInManually(target,{username:'testuser',password:'testpass'});await expect(target.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();}
  original=(await(await page.request.get('/api/account/preferences')).json()).preferences;
  await folders(page);await folders(second);
  await page.getByRole('button',{name:'Edit Inbox appearance',exact:true}).click();
  const label='Icon for Inbox';
  const first=page.getByRole('combobox',{name:label,exact:true});
  const next=(await first.inputValue())==='star'?'calendar':'star';
  const firstSaved=page.waitForResponse(response=>response.url().endsWith('/api/account/preferences')&&response.request().method()==='PUT'&&response.status()===200);
  await first.selectOption(next);await page.getByRole('button',{name:'Save folder appearance',exact:true}).click();await firstSaved;
  const expected=(await(await page.request.get('/api/account/preferences')).json()).preferences.folderIcons;
  const layout=second.getByRole('toolbar',{name:'Workspace layout'}).getByRole('button',{name:'Stacked',exact:true});
  const targetLayout=await layout.getAttribute('aria-pressed')==='true'?'tabs':'stack';
  await second.getByRole('toolbar',{name:'Workspace layout'}).getByRole('button',{name:targetLayout==='tabs'?'Tabs':'Stacked',exact:true}).click();
  await expect.poll(async()=> (await(await second.request.get('/api/account/preferences')).json()).preferences.workspaceLayout).toBe(targetLayout);
  expect((await(await second.request.get('/api/account/preferences')).json()).preferences.folderIcons).toEqual(expected);
  await second.reload();await expect(second.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();await folders(second);
  await second.getByRole('button',{name:'Edit Inbox appearance',exact:true}).click();
  await expect(second.getByRole('combobox',{name:label,exact:true})).toHaveValue(next);
  await second.screenshot({path:test.info().outputPath('account-folders-second-device.png'),fullPage:true});
 }finally{
  await other.close();
  // The account's layout and folder icons are shared by every later spec: put back what this one found.
  if(original!==undefined){
   const current=await page.request.get('/api/account/preferences');
   const restored=await page.request.put('/api/account/preferences',{headers:{...await storageAuthorityHeaders(page.request),'if-match':current.headers()['etag']!},data:{version:1,preferences:original}});
   expect(restored.ok(),await restored.text()).toBe(true);
  }
 }
});
