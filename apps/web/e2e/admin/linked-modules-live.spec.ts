import {test,expect} from '../csrf.ts';
import {postJmap} from '../helpers.ts';

test('linked mailbox modules use child data and preserve parent data across real account switches @phone',async({page},info)=>{
 test.setTimeout(240_000);
 const tag=`linked-pim-${info.project.name}-${Date.now()}`;
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('parent@example.org');await page.getByLabel('Password',{exact:true}).fill('parent-pass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('searchbox',{name:'Search mail'})).toBeVisible();
 const parent=await (await page.request.get('/api/me')).json() as {accountId:string;storageAuthority:string};
 const owner={'x-mailwoman-storage-authority':parent.storageAuthority};
 expect((await page.request.post('/api/security/step-up',{data:{password:'parent-pass'}})).ok()).toBe(true);
 const linked=await page.request.post('/api/accounts/link',{headers:owner,data:{username:'child@child.test',password:'child-pass',displayName:tag}});expect(linked.ok(),await linked.text()).toBe(true);
 const list=await (await page.request.get('/api/accounts',{headers:owner})).json() as {accounts:{id:string;address:string;revision:string}[]};const link=list.accounts.find(row=>row.address==='child@child.test')!;expect(link).toBeTruthy();
 const child=await (await page.request.get('/api/me',{headers:{'x-mailwoman-link':link.id}})).json() as {accountId:string;storageAuthority:string};expect(child.accountId).not.toBe(parent.accountId);
 const using=['urn:ietf:params:jmap:core','urn:mailwoman:calendars','urn:mailwoman:tasks','urn:mailwoman:notes','urn:mailwoman:contacts','urn:mailwoman:crypto'];
 const call=async(childScope:boolean,method:string,args:Record<string,unknown>)=>{
  const who=childScope?child:parent;
  const response=await postJmap(page.request,{headers:{'x-mailwoman-storage-authority':who.storageAuthority,...(childScope?{'x-mailwoman-link':link.id}:{})},data:{using,methodCalls:[[method,{accountId:who.accountId,...args},'pim']]}});
  expect(response.ok()).toBe(true);const body=await response.json() as {methodResponses:[string,Record<string,unknown>,string][]};expect(body.methodResponses[0]?.[0],JSON.stringify(body)).toBe(method);return body.methodResponses[0]![1];
 };
 const created:Array<{child:boolean;kind:string;id:string}>=[];
 const create=async(childScope:boolean,kind:string,value:Record<string,unknown>)=>{const result=await call(childScope,`${kind}/set`,{create:{fixture:value}});const id=(result['created'] as Record<string,{id:string}>)?.['fixture']?.id;expect(id,JSON.stringify(result)).toBeTruthy();created.push({child:childScope,kind,id:id!});return id!;};
 const note=(title:string)=>({notebookId:'default',title,tags:[],color:'#94a3b8',pinned:false,bodyHtml:'',bodyText:'Mailbox-scoped note',links:[]});
 const drawer=async()=>{const button=page.getByRole('button',{name:'Open folders and apps',exact:true});if(await button.isVisible()&&await button.getAttribute('aria-expanded')!=='true')await button.click();};
 const navigate=async(module:string)=>{await drawer();await page.getByTestId(`nav-${module.toLowerCase()}`).click();await expect(page.getByRole('tab',{name:module,exact:true})).toHaveAttribute('aria-selected','true');};
 const choose=async(id:string)=>{await drawer();const trigger=page.locator('button[data-account-indicator="picker"]');await trigger.click();await page.getByRole('menu',{name:'Mailbox account',exact:true}).locator(`[data-mailbox-item][data-link="${id}"]`).click();await expect(page.getByRole('navigation',{name:'Mailboxes',includeHidden:true})).toHaveAttribute('aria-busy','false');};
 try{
  const parentNote=await create(false,'Note',note(`Parent ${tag}`));
  const childNote=await create(true,'Note',note(`Child ${tag}`));
  const contact=await create(true,'ContactCard',{name:{full:`Contact ${tag}`},emails:[{context:'work',value:'fixture@example.org',pref:1}]});
  const taskLists=(await call(true,'TaskList/get',{ids:null}))['list'] as {id:string}[];expect(taskLists.length).toBeGreaterThan(0);
  const task=await create(true,'Task',{listId:taskLists[0]!.id,title:`Task ${tag}`,description:'Mailbox scoped',progress:'needs-action'});
  const calendars=(await call(true,'Calendar/get',{ids:null}))['list'] as {id:string}[];expect(calendars.length).toBeGreaterThan(0);
  const event=await create(true,'CalendarEvent',{calendarId:calendars[0]!.id,title:`Event ${tag}`,description:'',start:`${new Date().toISOString().slice(0,10)}T12:00:00`,duration:'PT1H',timeZone:null,showWithoutTime:false,locations:[],recurrenceRules:[],excludedRecurrenceDates:[],participants:{},alerts:{},status:'confirmed',freeBusyStatus:'busy'});
  await page.reload();await expect(page.getByRole('searchbox',{name:'Search mail'})).toBeVisible();
  await choose(link.id);
  for(const module of ['Calendar','Tasks','Notes','Contacts','Keys'])await navigate(module);
  await navigate('Notes');await expect(page.locator('[data-surface="notes"]').filter({visible:true})).toContainText(`Child ${tag}`);await expect(page.locator('[data-surface="notes"]').filter({visible:true})).not.toContainText(`Parent ${tag}`);
  await navigate('Tasks');await expect(page.locator('[data-surface="tasks"]').filter({visible:true})).toContainText(`Task ${tag}`);
  await navigate('Contacts');await expect(page.locator('[data-surface="contacts"]').filter({visible:true})).toContainText(`Contact ${tag}`);
  await navigate('Keys');await page.getByRole('button',{name:'Generate key',exact:true}).click();
  const dialog=page.getByRole('dialog',{name:'Generate a key'});await dialog.getByLabel('Key type').selectOption('pgp');await dialog.getByLabel('Email',{exact:true}).fill('child@child.test');await dialog.getByLabel('Key passphrase',{exact:true}).fill('linked-fixture-key-passphrase');await dialog.getByRole('button',{name:'Generate',exact:true}).click();await expect(dialog).toBeHidden({timeout:60_000});
  const keys=(await call(true,'CryptoKey/get',{ids:null}))['list'] as {id:string;isOwn:boolean;addresses:string[]}[];const key=keys.find(row=>row.isOwn&&row.addresses.includes('child@child.test'));expect(key).toBeTruthy();created.push({child:true,kind:'CryptoKey',id:key!.id});
  for(const [kind,id] of [['Note',childNote],['ContactCard',contact],['Task',task],['CalendarEvent',event],['CryptoKey',key!.id]]){
   const read=await call(false,`${kind}/get`,{ids:[id]});expect(read['list']).toEqual([]);expect(read['notFound']).toContain(id);
  }
  await page.screenshot({path:info.outputPath('linked-child-modules.png'),fullPage:true});
  await choose('');await navigate('Notes');await expect(page.locator('[data-surface="notes"]').filter({visible:true})).toContainText(`Parent ${tag}`);await expect(page.locator('[data-surface="notes"]').filter({visible:true})).not.toContainText(`Child ${tag}`);
  await call(true,'Note/set',{update:{[childNote]:{title:`Updated child ${tag}`}}});
  await choose(link.id);await navigate('Notes');await expect(page.locator('[data-surface="notes"]').filter({visible:true})).toContainText(`Updated child ${tag}`);
  expect(((await call(false,'Note/get',{ids:[parentNote]}))['list'] as {title:string}[])[0]?.title).toBe(`Parent ${tag}`);
 }finally{
  for(const item of created.reverse()){const result=await call(item.child,`${item.kind}/set`,{destroy:[item.id]});expect(result['destroyed']).toContain(item.id);}
  const fresh=(await (await page.request.get('/api/accounts',{headers:owner})).json()).accounts as {id:string;revision:string}[];const current=fresh.find(row=>row.id===link.id);
  if(current){expect((await page.request.post('/api/security/step-up',{data:{password:'parent-pass'}})).ok()).toBe(true);expect((await page.request.delete(`/api/accounts/${link.id}`,{headers:{...owner,'if-match':JSON.stringify(current.revision)}})).ok()).toBe(true);}
 }
});
