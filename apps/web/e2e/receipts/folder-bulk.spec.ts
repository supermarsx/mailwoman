import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {test,expect} from '../csrf.ts';

// Controlled mail responses; every Email/set is intercepted, never sent to the
// shared server. Native Trash role/state guards have separate actual HTTP gates.
test('folder bulk actions target only its selection and permanent deletion requires confirmation',async({page,context},info)=>{
 page.setDefaultTimeout(15000);
 const dist=path.resolve('../../.orchestration/reports/codex-folder-bulk-web');
 await context.route('**/*',async route=>{const url=new URL(route.request().url());if(url.pathname!=='/'&&!url.pathname.startsWith('/assets/'))return route.continue();const file=path.resolve(dist,url.pathname==='/'?'index.html':url.pathname.slice(1));if(!file.startsWith(dist+path.sep))return route.abort();try{await route.fulfill({body:await readFile(file),contentType:file.endsWith('.html')?'text/html':file.endsWith('.css')?'text/css':file.endsWith('.js')?'text/javascript':'application/octet-stream'});}catch{await route.continue();}});
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
 const session=await(await page.request.get('/jmap/session')).json();const accountId=session.primaryAccounts['urn:ietf:params:jmap:mail'];
 const source='folder-bulk-source',archive='folder-bulk-archive',trash='folder-bulk-trash';
 const names:Record<string,string>={[source]:'Bulk source',[archive]:'Bulk archive',[trash]:'Bulk trash'};
 let rows:{id:string;mailboxIds:Record<string,boolean>;keywords:Record<string,boolean>}[]=[{id:'folder-bulk-one',mailboxIds:{[source]:true},keywords:{} as Record<string,boolean>},{id:'folder-bulk-two',mailboxIds:{[source]:true},keywords:{} as Record<string,boolean>}];
 const writes:{update?:Record<string,unknown>;destroy?:string[]}[]=[];let releaseDelete:(()=>void)|undefined;
 const boxes=()=>Object.entries(names).map(([id,name])=>({id,name,role:id===trash?'trash':id===archive?'archive':null,parentId:null,sortOrder:999,totalEmails:rows.filter(row=>row.mailboxIds[id]).length,unreadEmails:rows.filter(row=>row.mailboxIds[id]&&!row.keywords['$seen']).length}));
 await page.route('**/jmap/api',async route=>{
  const request=route.request().postDataJSON();const calls=request.methodCalls as [string,Record<string,unknown>,string][];
  if(calls.some(([name])=>name==='Mailbox/get')){
   const response=await route.fetch();const body=await response.json();
   for(const call of body.methodResponses)if(call[0]==='Mailbox/get'){const args=calls.find(item=>item[2]===call[2])?.[1],ids=args?.ids as string[]|undefined;call[1].list=ids?.some(id=>id in names)?boxes().filter(box=>ids.includes(box.id)):[...call[1].list.map((box:{role?:string})=>box.role==='archive'||box.role==='trash'?{...box,role:null}:box),...boxes()];call[1].notFound=[];call[1].mailwomanFolderRoleGuard=true;}
   return route.fulfill({response,json:body});
  }
  const controlled=calls.some(([name,args])=>name==='Email/query'&&String((args.filter as {inMailbox?:string})?.inMailbox) in names||name==='Email/get'&&Array.isArray(args.ids)&&args.ids.some(id=>String(id).startsWith('folder-bulk-'))||name==='Email/set');
  if(!controlled)return route.continue();
  const responses=[];let queried:string[]=[];
  for(const [name,args,id] of calls){
   if(name==='Email/query'){const folder=(args.filter as {inMailbox:string}).inMailbox;queried=rows.filter(row=>row.mailboxIds[folder]).map(row=>row.id);responses.push([name,{accountId,queryState:'q',position:0,total:queried.length,ids:queried},id]);}
   else if(name==='Email/get'){const ids=Array.isArray(args.ids)?args.ids:queried;responses.push([name,{accountId,state:'state',list:rows.filter(row=>ids.includes(row.id)).map(row=>({...row,subject:row.id,from:[{email:'controlled@example.org'}],preview:'Controlled folder bulk fixture',receivedAt:'2026-10-26T00:00:00Z'})),notFound:[]},id]);}
   else if(name==='Email/set'){
    const update=args.update as Record<string,Record<string,unknown>>|undefined,destroy=args.destroy as string[]|undefined;expect([...Object.keys(update??{}),...(destroy??[])].every(id=>id.startsWith('folder-bulk-'))).toBe(true);expect(args.ifInState).toBe('state');writes.push({...(update?{update}:{}),...(destroy?{destroy}:{})});
    if(destroy){expect(args.mailwomanRequireMailboxRole).toBe('trash');await new Promise<void>(resolve=>{releaseDelete=resolve;});rows=rows.filter(row=>!destroy.includes(row.id));responses.push([name,{accountId,destroyed:destroy},id]);}
    else{for(const [key,patch] of Object.entries(update??{})){const row=rows.find(row=>row.id===key)!;if(patch.mailboxIds)row.mailboxIds=patch.mailboxIds as Record<string,boolean>;else row.keywords=patch['keywords/$seen']===true?{$seen:true}:{};}responses.push([name,{accountId,updated:Object.fromEntries(Object.keys(update??{}).map(key=>[key,null]))},id]);}
   }
  }
  return route.fulfill({json:{sessionState:'s',methodResponses:responses}});
 });
 await page.reload();await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
 async function open(name:string){if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();const folder=page.locator('#shell-nav').getByRole('button',{name,exact:true});await folder.focus();await folder.press('Shift+F10');await page.getByRole('menuitem',{name:'Open in new tab',exact:true}).click();return page.getByRole('tabpanel',{name,exact:true});}
 const pane=await open('Bulk source');await pane.getByRole('button',{name:'Select this page'}).click();await pane.getByRole('button',{name:'Mark read',exact:true}).click();await expect.poll(()=>rows.every(row=>row.keywords['$seen'])).toBe(true);await expect(pane.getByRole('button',{name:'Select this page'})).toBeEnabled();
 await pane.getByRole('button',{name:'Select this page'}).click();await pane.getByRole('button',{name:'Mark unread',exact:true}).click();await expect.poll(()=>rows.every(row=>!row.keywords['$seen'])).toBe(true);await expect(pane.getByRole('button',{name:'Select this page'})).toBeEnabled();
 await pane.getByRole('button',{name:'Select this page'}).click();await pane.getByRole('button',{name:'Archive',exact:true}).click();await expect(pane.getByRole('checkbox')).toHaveCount(0);
 const archived=await open('Bulk archive');await expect(archived.getByRole('checkbox')).toHaveCount(2);await archived.getByRole('button',{name:'Select this page'}).click();await archived.getByRole('combobox',{name:'Move selected messages to'}).selectOption(source);await archived.getByRole('button',{name:'Move',exact:true}).click();await expect(archived.getByRole('checkbox')).toHaveCount(0);
 await page.getByRole('tab',{name:'Bulk source',exact:true}).click();await expect(pane.getByRole('checkbox')).toHaveCount(2);await pane.getByRole('button',{name:'Select this page'}).click();await pane.getByRole('button',{name:'Delete',exact:true}).click();await expect(pane.getByRole('checkbox')).toHaveCount(0);
 const deleted=await open('Bulk trash');await expect(deleted.getByRole('checkbox')).toHaveCount(2);await deleted.getByRole('checkbox').first().check();await deleted.getByRole('checkbox').first().focus();await page.keyboard.press('Delete');
 const confirm=deleted.getByRole('alertdialog',{name:'Permanently delete selected messages'});await expect(confirm).toBeVisible();expect(writes.filter(write=>write.destroy)).toHaveLength(0);await confirm.getByRole('button',{name:'Cancel',exact:true}).click();expect(writes.filter(write=>write.destroy)).toHaveLength(0);
 await deleted.getByRole('button',{name:'Delete',exact:true}).click();await confirm.getByRole('button',{name:'Delete permanently',exact:true}).click();await expect(confirm).toHaveCount(0);await expect(page.getByTestId('empty-trash-progress')).toBeVisible();await expect.poll(()=>typeof releaseDelete).toBe('function');releaseDelete!();
 await expect(page.getByTestId('empty-trash-result')).toContainText('Deleted 1;');await expect(deleted.getByRole('checkbox')).toHaveCount(1);expect(writes).toHaveLength(6);
 await page.screenshot({path:info.outputPath('folder-bulk-confirmed.png'),fullPage:true});expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
});
