import {test,expect} from '../csrf.ts';

test('user folders collapse persists and hidden folders remain manageable without deleting mailboxes',async({page},info)=>{
 test.setTimeout(180000);page.setDefaultTimeout(20000);
 await page.goto('/');
 await page.getByLabel('Email or username',{exact:true}).fill('testuser');
 await page.getByLabel('Password',{exact:true}).fill('testpass');
 await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
 const me=await (await page.request.get('/api/me')).json() as {accountId:string;storageAuthority:string};
 const headers={'x-mailwoman-storage-authority':me.storageAuthority};
 const readPrefs=async()=>{const response=await page.request.get('/api/account/preferences',{headers});expect(response.ok()).toBe(true);return {etag:response.headers()['etag']!,body:await response.json() as {version:number;preferences:Record<string,unknown>}};};
 const original=(await readPrefs()).body.preferences;
 const writePrefs=async(patch:Record<string,unknown>)=>{const current=await readPrefs();const response=await page.request.put('/api/account/preferences',{headers:{...headers,'if-match':current.etag},data:{version:1,preferences:{...current.body.preferences,...patch}}});expect(response.ok(),await response.text()).toBe(true);};
 const call=async(name:string,args:Record<string,unknown>)=>{
  const response=await page.request.post('/jmap/api',{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[[name,{accountId:me.accountId,...args},'c']]}});
  expect(response.ok()).toBe(true);const body=await response.json() as {methodResponses:[string,Record<string,unknown>][]};expect(body.methodResponses[0]![0]).toBe(name);return body.methodResponses[0]![1];
 };
 const name=`Visibility ${info.project.name} ${Date.now()}`;
 let parent:string|undefined,child:string|undefined;
 const nav=page.locator('#shell-nav');
 const drawer=async()=>{await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();if(info.project.name==='phone'){const button=page.getByRole('button',{name:'Open folders and apps',exact:true});await expect(button).toBeVisible();if(await button.getAttribute('aria-expanded')!=='true')await button.click();}await expect(nav).toBeVisible();};
 try{
  const created=await call('Mailbox/set',{create:{p:{name}}});parent=(created.created as Record<string,{id:string}>|undefined)?.p?.id;expect(parent,JSON.stringify(created)).toBeTruthy();
  const nested=await call('Mailbox/set',{create:{c:{name:`Child ${name}`,parentId:parent}}});child=(nested.created as Record<string,{id:string}>|undefined)?.c?.id;expect(child,JSON.stringify(nested)).toBeTruthy();
  await writePrefs({userFoldersCollapsed:false});await page.reload();await drawer();
  const toggle=nav.getByRole('button',{name:'User folders',exact:true});
  await expect(nav.getByRole('button',{name,exact:true})).toBeVisible();
  await toggle.click();await expect(toggle).toHaveAttribute('aria-expanded','false');
  await expect(nav.getByRole('button',{name,exact:true})).toHaveCount(0);
  await expect(nav.getByRole('button',{name:'Inbox',exact:true})).toBeVisible();
  await expect.poll(async()=>(await readPrefs()).body.preferences.userFoldersCollapsed).toBe(true);
  await page.reload();await drawer();await expect(toggle).toHaveAttribute('aria-expanded','false');await toggle.click();
  const folder=nav.getByRole('button',{name,exact:true});await folder.focus();await folder.press('Shift+F10');
  await page.getByRole('menuitem',{name:'Hide from sidebar',exact:true}).click();
  await expect(folder).toHaveCount(0);await expect(nav.getByRole('button',{name:`Child ${name}`,exact:true})).toHaveCount(0);
  await expect.poll(async()=>((await readPrefs()).body.preferences.folderHidden as string[]|undefined)?.includes(parent!)).toBe(true);
  const boxes=(await call('Mailbox/get',{})).list as {id:string}[];expect(boxes.map(box=>box.id)).toEqual(expect.arrayContaining([parent,child]));
  await drawer();await nav.getByRole('button',{name:'Settings',exact:true}).click();
  await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Folders and sidebar',exact:true}).click();
  await page.getByRole('button',{name:`Show Child ${name} in sidebar`,exact:true}).click();
  await expect.poll(async()=>((await readPrefs()).body.preferences.folderHidden as string[]|undefined)?.includes(parent!)??false).toBe(false);
  await page.getByRole('button',{name:'Close settings',exact:true}).click();await drawer();await expect(folder).toBeVisible();
  await page.screenshot({path:info.outputPath('folder-visibility-restored.png'),fullPage:true,animations:'disabled'});
 }finally{
  const current=await readPrefs();
  for(const key of ['folderHidden','userFoldersCollapsed']){if(key in original)current.body.preferences[key]=original[key];else delete current.body.preferences[key];}
  expect((await page.request.put('/api/account/preferences',{headers:{...headers,'if-match':current.etag},data:current.body})).ok()).toBe(true);
  if(child)await call('Mailbox/set',{destroy:[child]});if(parent)await call('Mailbox/set',{destroy:[parent]});
 }
});
