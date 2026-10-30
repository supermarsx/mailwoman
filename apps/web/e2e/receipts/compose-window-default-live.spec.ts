import {test,expect} from '../csrf.ts';
import {withUserSetting} from '../helpers.ts';

test('compose window preference opens the live editor and preserves edits when reattached',async({page,playwright,baseURL},info)=>{
 const restoreDraftMode=await withUserSetting(playwright.request,baseURL!,'mail.draftSaveMode','manual');
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
 const original=await (await page.request.get('/api/settings')).json() as {settings:Record<string,{userValue?:unknown}>};
 const key='mail.composeWindow',entry=original.settings[key];
 const authority=(await(await page.request.get('/api/me')).json() as {storageAuthority:string}).storageAuthority;
 const headers={'x-mailwoman-storage-authority':authority};
 try{
  expect((await page.request.put(`/api/settings/${key}`,{headers,data:{value:'window'}})).ok()).toBe(true);
  await page.reload();await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
  const pending=page.waitForEvent('popup');
  if((page.viewportSize()?.width??1280)<600)await page.locator('header.shell__bar').getByRole('button',{name:'Compose',exact:true}).click();else await page.locator('.sidebar__compose').click();
  const popup=await pending;
  const editor=popup.locator('.ProseMirror');await expect(editor).toBeVisible();
  await editor.fill('Window preference preserves this draft');
  await popup.screenshot({path:info.outputPath('compose-window-preference.png'),fullPage:true});
  await popup.getByRole('button',{name:'Reattach to main window',exact:true}).click();
  await expect(page.locator('.ProseMirror')).toHaveText('Window preference preserves this draft');
  await expect.poll(()=>popup.isClosed()).toBe(true);
  const compose=page.locator('form.compose').last();await compose.getByRole('button',{name:'Discard account draft',exact:true}).click();await page.getByTestId('compose-discard-confirm').getByRole('button',{name:'Discard',exact:true}).click();await expect(compose).toHaveCount(0);
 }finally{
  const response=entry&&Object.prototype.hasOwnProperty.call(entry,'userValue')?await page.request.put(`/api/settings/${key}`,{headers,data:{value:entry.userValue}}):await page.request.delete(`/api/settings/${key}`,{headers});
  expect(response.ok()).toBe(true);
  await restoreDraftMode();
 }
});
