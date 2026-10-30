import {test,expect} from '../csrf.ts';

test('session cache paints a warm folder before refresh and preloading stays opt-in without read side effects',async({page},info)=>{
 test.setTimeout(120000);
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
 const me=await (await page.request.get('/api/me')).json() as {accountId:string;storageAuthority:string};
 const headers={'x-mailwoman-storage-authority':me.storageAuthority}; expect(typeof me.storageAuthority).toBe('string');
 const jmap=async(method:string,args:Record<string,unknown>)=>{const response=await page.request.post('/jmap/api',{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[[method,{accountId:me.accountId,...args},'test']]}});expect(response.ok()).toBe(true);const body=await response.json() as {methodResponses:[string,Record<string,unknown>,string][]};expect(body.methodResponses[0]?.[0]).toBe(method);return body.methodResponses[0]![1];};
 await expect.poll(async()=>(await jmap('Mailbox/get',{}))['mailwomanInitialSync'],{timeout:60000}).toBe('ready');
 const boxes=(await jmap('Mailbox/get',{}))['list'] as {id:string;role:string|null}[];const inbox=boxes.find(box=>box.role==='inbox')!.id;
 await expect.poll(async()=>((await jmap('Email/query',{filter:{inMailbox:inbox},limit:100}))['ids'] as string[]).length,{timeout:60000}).toBeGreaterThanOrEqual(30);
 const name=`Cache fixture ${info.project.name} ${Date.now()}`;const created=await jmap('Mailbox/set',{create:{cache:{name}}});const folder=(created['created'] as Record<string,{id:string}>)['cache']!.id;
 const keys=['enabled','refreshOnOpen','prefetchEnabled','hoverEnabled','focusEnabled','dwellMs','preloadMessages'] as const;
 const original=await (await page.request.get('/api/settings')).json() as {settings:Record<string,{userValue?:unknown}>};
 const put=async(key:string,value:unknown)=>expect((await page.request.put(`/api/settings/mail.cache.${key}`,{headers,data:{value}})).ok()).toBe(true);
 let release:(()=>void)|undefined;
 const openFolders=async()=>{const button=page.getByRole('button',{name:'Open folders and apps',exact:true});if(await button.isVisible()&&await button.getAttribute('aria-expanded')!=='true')await button.click();};
 const choose=async(label:string)=>{await openFolders();await page.locator('#shell-nav').getByRole('button',{name:label,exact:true}).click();};
 const queries:{methods:string[];mailbox?:string}[]=[];
 page.on('request',request=>{if(request.method()!=='POST'||!new URL(request.url()).pathname.endsWith('/jmap/api'))return;const body=request.postDataJSON() as {methodCalls:[string,{filter?:{inMailbox?:string}},string][]};queries.push({methods:body.methodCalls.map(call=>call[0]),...(body.methodCalls.find(call=>call[0]==='Email/query')?.[1].filter?.inMailbox?{mailbox:body.methodCalls.find(call=>call[0]==='Email/query')![1].filter!.inMailbox!}:{})});});
 try{
  await put('enabled',true);await put('refreshOnOpen',true);await put('prefetchEnabled',false);
  await page.reload();await expect(page.locator('section.list [data-email-id]').first()).toBeVisible();
  const firstId=await page.locator('section.list [data-email-id]').first().getAttribute('data-email-id');
  await choose(name);await expect(page.locator('section.list')).toHaveAttribute('aria-busy','false');
  let held=false;const gate=new Promise<void>(resolve=>{release=resolve;});
  await page.route('**/jmap/api',async route=>{const body=route.request().postDataJSON() as {methodCalls:[string,{filter?:{inMailbox?:string}},string][]};if(body.methodCalls.some(call=>call[0]==='Email/query'&&call[1].filter?.inMailbox===inbox)){held=true;await gate;}await route.continue();});
  await choose('Inbox');await expect.poll(()=>held).toBe(true);
  await expect(page.locator(`section.list [data-email-id="${firstId}"]`)).toBeVisible();await expect(page.locator('section.list')).toHaveAttribute('aria-busy','false');await expect(page.locator('.list__scroll')).not.toHaveAttribute('inert');
  await page.screenshot({path:info.outputPath('cached-folder-before-refresh.png'),animations:'disabled'});
  release?.();await page.unrouteAll({behavior:'wait'});
  await openFolders();queries.length=0;await page.locator('#shell-nav').getByRole('button',{name,exact:true}).hover();await page.waitForTimeout(350);expect(queries.filter(q=>q.mailbox===folder)).toHaveLength(0);
  await put('prefetchEnabled',true);await put('hoverEnabled',true);await put('focusEnabled',false);await put('dwellMs',200);await put('preloadMessages',true);
  await page.mouse.move(0,0);await page.reload();await expect(page.locator('section.list [data-email-id]').first()).toBeVisible();await openFolders();queries.length=0;
  const target=page.locator('#shell-nav').getByRole('button',{name,exact:true});await target.hover();await page.waitForTimeout(30);await page.mouse.move(0,0);await page.waitForTimeout(300);expect(queries.filter(q=>q.mailbox===folder)).toHaveLength(0);
  await target.hover();await expect.poll(()=>queries.filter(q=>q.mailbox===folder).length).toBe(1);
  if(info.project.name==='phone')await page.keyboard.press('Escape');
  queries.length=0;await page.locator('section.list [data-email-id]').first().hover();await expect.poll(()=>queries.some(q=>q.methods.length===1&&q.methods[0]==='Email/get')).toBe(true);
  expect(queries.flatMap(q=>q.methods).some(method=>method.endsWith('/set'))).toBe(false);
  await expect(page.locator('section.reader .reader__subject')).toHaveCount(0);
  await expect(page.locator('section.reader iframe.reader__frame')).toHaveCount(0);
 }finally{
  release?.();await page.unrouteAll({behavior:'wait'});
  for(const key of keys){const entry=original.settings[`mail.cache.${key}`];const response=entry&&Object.prototype.hasOwnProperty.call(entry,'userValue')?await page.request.put(`/api/settings/mail.cache.${key}`,{headers,data:{value:entry.userValue}}):await page.request.delete(`/api/settings/mail.cache.${key}`,{headers});expect(response.ok()).toBe(true);}
  expect((await jmap('Mailbox/set',{destroy:[folder]}))['destroyed']).toContain(folder);
 }
});
