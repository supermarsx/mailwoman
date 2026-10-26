import {test,expect} from '../csrf.ts';

// Controlled API responses exercise UI cancellation without destroying the
// shared fixture's real Trash. Native membership/CAS deletion has separate tests.
test('trash confirmation closes while a persistent notification owns cancellation',async({page},info)=>{
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 const session=await(await page.request.get('/jmap/session')).json();const accountId=session.primaryAccounts['urn:ietf:params:jmap:mail'];
 const folder='trash-progress-fixture',ids=Array.from({length:101},(_,i)=>`trash-progress-${i}`);
 let mutations=0,release:(()=>void)|undefined;
 await page.route('**/jmap/api',async route=>{
  const request=route.request().postDataJSON();const calls=request.methodCalls as [string,Record<string,unknown>,string][];
  const fixture=calls.some(([name,args])=>name==='Email/query'&&(args.filter as {inMailbox?:string}|undefined)?.inMailbox===folder||name==='Email/get'&&Array.isArray(args.ids)&&args.ids.some(id=>String(id).startsWith('trash-progress-'))||name==='Email/set'&&Array.isArray(args.destroy)&&args.destroy.some(id=>String(id).startsWith('trash-progress-')));
  if(fixture){
   const responses=[];
   for(const [name,args,callId] of calls){
    if(name==='Email/query')responses.push([name,{accountId,queryState:'q',position:args.position??0,total:101,ids:ids.slice(Number(args.position??0),Number(args.position??0)+Number(args.limit??50))},callId]);
    else if(name==='Email/get'){const selected=Array.isArray(args.ids)?args.ids:ids.slice(0,50);responses.push([name,{accountId,state:'s',list:selected.map(id=>({id,mailboxIds:{[folder]:true},keywords:{},subject:`Disposable ${id}`,receivedAt:'2026-10-26T00:00:00Z',from:[{email:'fixture@example.org'}],preview:''})),notFound:[]},callId]);}
    else if(name==='Email/set'){mutations++;await new Promise<void>(resolve=>{release=resolve;});responses.push([name,{accountId,destroyed:args.destroy},callId]);}
   }
   await route.fulfill({json:{methodResponses:responses,sessionState:'s'}});return;
  }
  if(calls.some(call=>call[0]==='Mailbox/get')){
   const response=await route.fetch();const body=await response.json();
   for(const call of body.methodResponses)if(call[0]==='Mailbox/get'){
    const args=calls.find(item=>item[2]===call[2])?.[1];
    const box={id:folder,name:'Fixture Trash',role:'trash',parentId:null,sortOrder:999,totalEmails:101,unreadEmails:101};
    call[1].list=Array.isArray(args?.ids)&&args.ids.includes(folder)?[box]:[...call[1].list.map((item:{role:string})=>item.role==='trash'?{...item,role:null}:item),box];call[1].notFound=[];
   }
   await route.fulfill({response,json:body});return;
  }
  await route.continue();
 });
 await page.reload();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
 await page.locator('.sidebar__box').filter({hasText:'Fixture Trash'}).click();
 if(info.project.name==='phone'){const close=page.getByRole('button',{name:'Close folders and apps'});if(await close.isVisible())await close.click();}
 await page.getByRole('button',{name:'Empty trash',exact:true}).click();const dialog=page.getByRole('alertdialog',{name:'Empty trash'});
 await expect(dialog.getByRole('button',{name:'Delete permanently'})).toBeEnabled();await dialog.getByRole('button',{name:'Delete permanently'}).click();await expect(dialog).toHaveCount(0);
 const progress=page.getByTestId('empty-trash-progress');await expect(progress).toBeVisible();await expect.poll(()=>mutations).toBe(1);
 await progress.getByRole('button',{name:'Stop after the current request'}).click();await expect(progress).toContainText('Stopping after the current request');
 await expect(progress).toHaveCSS('opacity','1');await page.screenshot({path:info.outputPath('trash-progress-cancelling.png')});release!();
 const result=page.getByTestId('empty-trash-result');await expect(result).toContainText('Confirmed deleted: 100');await expect(result).toContainText('remaining: 1');await expect(result).toContainText('outcome unknown: 0');expect(mutations).toBe(1);
 await expect(result).toHaveCSS('opacity','1');await page.screenshot({path:info.outputPath('trash-progress-stopped.png')});expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
});
