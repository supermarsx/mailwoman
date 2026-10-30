import {test,expect} from '../csrf.ts';
test('folder tabs retain independent selection and scroll without replacing the main folder',async({page},info)=>{
 page.setDefaultTimeout(15000);
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
 const me=await (await page.request.get('/api/me')).json() as {accountId:string};
 const folderName=`Folder tabs ${info.project.name} ${Date.now()}`;
 const mailboxSet=async(args:Record<string,unknown>)=>{const response=await page.request.post('/jmap/api',{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Mailbox/set',{accountId:me.accountId,...args},'folders']]}});expect(response.ok()).toBe(true);const body=await response.json() as {methodResponses:[string,{created?:Record<string,{id:string}>;destroyed?:string[]}][]};expect(body.methodResponses[0]?.[0]).toBe('Mailbox/set');return body.methodResponses[0]![1];};
 const created=await mailboxSet({create:{fixture:{name:folderName}}});const folderId=created.created?.fixture?.id;expect(folderId).toBeTruthy();
 try{
 await page.reload();await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
 if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
 const settings=page.locator('.sidebar__footer').getByRole('button',{name:'Settings',exact:true});
 await expect(settings).toHaveText('');await expect(settings).toHaveAttribute('data-tooltip','Settings');
 const settingsBox=await settings.boundingBox(),logoutBox=await page.locator('.sidebar__footer').getByRole('button',{name:'Log out',exact:true}).boundingBox();
 expect(settingsBox).toBeTruthy();expect(logoutBox).toBeTruthy();expect(settingsBox!.x+settingsBox!.width).toBeLessThanOrEqual(logoutBox!.x+1);expect(Math.abs(settingsBox!.y-logoutBox!.y)).toBeLessThan(4);
 if(info.project.name==='phone')await page.keyboard.press('Escape');
 const initial=await page.locator('#shell-nav .sidebar__box--active').first().getAttribute('aria-label');
 async function openFolder(name:string){
  if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
  const folder=page.locator('#shell-nav').getByRole('button',{name,exact:true});await expect(folder).toBeVisible();
  await folder.click({button:'right'});await page.getByRole('menuitem',{name:'Open in new tab',exact:true}).click();
  return name;
 }
 const first=await openFolder('Inbox');const firstTab=page.getByRole('tab',{name:first,exact:true});await expect(firstTab).toHaveAttribute('aria-selected','true');
 const pane=page.getByRole('tabpanel',{name:first,exact:true});await expect(pane.getByRole('checkbox').first()).toBeVisible();await pane.getByRole('checkbox').first().check();
 const scroll=await pane.getByRole('list').evaluate(list=>{const scroller=list.parentElement!;scroller.scrollTop=250;scroller.dispatchEvent(new Event('scroll'));return scroller.scrollTop;});
 expect(scroll).toBeGreaterThan(0);
 const second=await openFolder(folderName);expect(second).not.toBe(first);await expect(page.getByRole('tab',{name:second,exact:true})).toHaveAttribute('aria-selected','true');
 await firstTab.click();await expect(pane.getByRole('checkbox').first()).toBeChecked();
 expect(await pane.getByRole('list').evaluate(list=>list.parentElement!.scrollTop)).toBe(scroll);
 await page.getByRole('tab',{name:'Mail',exact:true}).click();expect(await page.locator('#shell-nav .sidebar__box--active').first().getAttribute('aria-label')).toBe(initial);
 await firstTab.click();await page.reload();await expect(page.getByRole('tab',{name:first,exact:true})).toBeVisible();await expect(page.getByRole('tab',{name:second,exact:true})).toBeVisible();
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
 await page.screenshot({path:info.outputPath('folder-tabs-retained.png'),fullPage:true});
 }finally{if(folderId)expect((await mailboxSet({destroy:[folderId]})).destroyed).toContain(folderId);}
});
