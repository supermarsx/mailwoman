import {test,expect,type Page} from './csrf.ts';
import {engineLogin,injectViaSmtp,messageRow,sidebarInbox} from './helpers.ts';

test.describe.configure({retries:0});
async function enableNotifications(page:Page):Promise<void>{
 const me=await (await page.request.get('/api/me')).json() as {accountId:string;storageAuthority:string};
 const saved=await page.request.put('/api/account/notifications',{headers:{'X-Mailwoman-Account':me.accountId,'X-Mailwoman-Storage-Authority':me.storageAuthority},data:{enabled:true,rules:[],quietHours:{enabled:false,start:'22:00',end:'07:00',timezone:'UTC'}}});
 expect(saved.ok(),await saved.text()).toBe(true);
}
async function delivery(page:Page,label:string,timeout:number):Promise<void>{
 let manualRefreshes=0;page.on('request',request=>{if(request.url().includes('/api/mail/refresh'))manualRefreshes++;});
 const subject=`Realtime ${label} ${Date.now()}`;const began=Date.now();
 await injectViaSmtp({from:'Push Bot <pushbot@example.org>',subject,text:subject});
 await expect(page.getByTestId('incoming-mail-toast').filter({hasText:subject})).toBeVisible({timeout});
 await expect(messageRow(page,subject)).toBeVisible({timeout});
 expect(manualRefreshes).toBe(0);
 console.info('Automatic delivery',JSON.stringify({mode:label,elapsedMs:Date.now()-began,manualRefreshes}));
 await test.info().attach(`delivery-${label}`,{body:JSON.stringify({elapsedMs:Date.now()-began,manualRefreshes}),contentType:'application/json'});
}
test('SMTP delivery updates the Inbox and toast over WebSocket without Refresh',async({page})=>{
 await page.addInitScript(()=>Object.defineProperty(globalThis,'SharedWorker',{value:undefined,configurable:true}));
 let socket=false;page.on('websocket',()=>{socket=true;});
 await engineLogin(page);await enableNotifications(page);await expect(sidebarInbox(page)).toBeVisible();
 await expect.poll(()=>socket).toBe(true);await delivery(page,'websocket',20000);
});
test('EventSource fallback still delivers new mail and its toast without Refresh',async({page})=>{
 await page.addInitScript(()=>{Object.defineProperty(globalThis,'SharedWorker',{value:undefined,configurable:true});Object.defineProperty(globalThis,'WebSocket',{value:undefined,configurable:true});});
 await engineLogin(page);await enableNotifications(page);await delivery(page,'eventsource',20000);
});
test('blocked streaming transports automatically catch up mail and notifications',async({page})=>{
 test.setTimeout(100000);
 await page.addInitScript(()=>{for(const key of ['SharedWorker','WebSocket','EventSource'])Object.defineProperty(globalThis,key,{value:undefined,configurable:true});});
 await engineLogin(page);await enableNotifications(page);await delivery(page,'polling',75000);
});
test('shared windows receive the same delivery and the unfocused window can notify the browser',async({page,context})=>{
 test.setTimeout(90000);await context.grantPermissions(['notifications']);
 await page.addInitScript(()=>{
  const Original=globalThis.Notification;const delivered:string[]=[];
  const reads:{method:string;created:number;count:number}[]=[];Object.defineProperty(globalThis,'__incomingReads',{value:reads});
  if(typeof SharedWorker!=='undefined'){
   const Worker=globalThis.SharedWorker;
   globalThis.SharedWorker=new Proxy(Worker,{construct(target,args){const worker=Reflect.construct(target,args,target) as SharedWorker;worker.port.addEventListener('message',(event:MessageEvent)=>{const data=event.data as {result?:{methodResponses?:[string,{created?:string[];list?:unknown[]},string][]}};for(const [method,result,tag] of data.result?.methodResponses??[])if(tag==='incoming')reads.push({method,created:result.created?.length??0,count:result.list?.length??-1});});return worker;}});
  }
  Object.defineProperty(globalThis,'__mailNotificationBodies',{value:delivered});
  // Keep native static getters bound to the native constructor; subclassing
  // Notification makes Chromium's permission getter throw an illegal invocation.
  globalThis.Notification=new Proxy(Original,{construct(target,args){const notification=Reflect.construct(target,args,target);delivered.push((args[1] as NotificationOptions|undefined)?.body??'');return notification;},get(target,key){const value:unknown=Reflect.get(target,key,target);return typeof value==='function'?value.bind(target):value;}});
 });
 let seededOverHttp=false;
 page.on('response',response=>{try{if(!response.url().includes('/jmap/api'))return;const data=response.request().postDataJSON() as {methodCalls:[string,{ids?:string[]},string][]};if(data.methodCalls.some(([method,args,tag])=>method==='Email/get'&&tag==='incoming'&&args.ids?.length===0)&&response.ok())seededOverHttp=true;}catch{/* SharedWorker replies are observed separately. */}});
 await engineLogin(page);await enableNotifications(page);
 await expect.poll(async()=>seededOverHttp||await page.evaluate(()=>(globalThis as unknown as {__incomingReads:{method:string;count:number}[]}).__incomingReads.some(read=>read.method==='Email/get'&&read.count===0)),{timeout:10000}).toBe(true);
 const second=await context.newPage();let secondSeeded=false;
 second.on('response',response=>{try{if(!response.url().includes('/jmap/api'))return;const data=response.request().postDataJSON() as {methodCalls:[string,{ids?:string[]},string][]};if(data.methodCalls.some(([method,args,tag])=>method==='Email/get'&&tag==='incoming'&&args.ids?.length===0)&&response.ok())secondSeeded=true;}catch{/* Worker replies are observed below. */}});
 await second.addInitScript(()=>{
  Object.defineProperty(globalThis,'__incomingSeeded',{value:false,writable:true});
  if(typeof SharedWorker==='undefined')return;const Worker=globalThis.SharedWorker;
  globalThis.SharedWorker=new Proxy(Worker,{construct(target,args){const worker=Reflect.construct(target,args,target) as SharedWorker;worker.port.addEventListener('message',(event:MessageEvent)=>{const data=event.data as {result?:{methodResponses?:[string,{list?:unknown[]},string][]}};if(data.result?.methodResponses?.some(([method,result,tag])=>method==='Email/get'&&tag==='incoming'&&result.list?.length===0))(globalThis as unknown as {__incomingSeeded:boolean}).__incomingSeeded=true;});return worker;}});
 });
 await second.goto('/');await expect(sidebarInbox(second)).toBeVisible();
 await expect.poll(async()=>secondSeeded||await second.evaluate(()=>(globalThis as unknown as {__incomingSeeded:boolean}).__incomingSeeded),{timeout:10000}).toBe(true);
 await second.bringToFront();
 // Headless Chromium gives every target focus. Control only this visibility
 // input; SMTP, push, shared state and the granted native Notification API stay real.
 await page.evaluate(()=>Object.defineProperty(document,'hasFocus',{value:()=>false,configurable:true}));
 const subject=`Realtime shared ${Date.now()}`;const began=Date.now();await injectViaSmtp({from:'Push Bot <pushbot@example.org>',subject,text:subject});
 // Shared toast snapshots preserve accessible content, not local test ids.
 try{await expect(second.getByRole('status').filter({hasText:subject})).toBeVisible({timeout:20000});}finally{
 console.info('Shared delivery state',JSON.stringify(await Promise.all([page,second].map(window=>window.evaluate(()=>({hidden:document.hidden,focused:document.hasFocus(),permission:Notification.permission,seeded:(globalThis as unknown as {__incomingSeeded?:boolean}).__incomingSeeded,reads:(globalThis as unknown as {__incomingReads?:unknown}).__incomingReads,notices:(globalThis as unknown as {__mailNotificationBodies?:unknown}).__mailNotificationBodies}))))));
 }
 await expect(messageRow(second,subject)).toBeVisible({timeout:20000});await expect(messageRow(page,subject)).toBeVisible({timeout:20000});
 console.info('Incoming worker trace',JSON.stringify(await page.evaluate(()=>({reads:(globalThis as unknown as {__incomingReads:unknown}).__incomingReads,notices:(globalThis as unknown as {__mailNotificationBodies:unknown}).__mailNotificationBodies,permission:Notification.permission,focused:document.hasFocus(),hidden:document.hidden}))));
 await expect.poll(()=>page.evaluate(subject=>(globalThis as unknown as {__mailNotificationBodies:string[]}).__mailNotificationBodies.some(body=>body.includes(subject)),subject),{timeout:20000}).toBe(true);
 await test.info().attach('delivery-shared-browser-notification',{body:JSON.stringify({elapsedMs:Date.now()-began}),contentType:'application/json'});await second.close();
});
test('inactive linked mailbox sends an actionable alert without switching or refreshing first',async({page})=>{
 test.skip(!process.env['MW_E2E_LINKED_USERNAME'],'Dedicated linked fixture account not configured');
 test.setTimeout(90000);
 const scopedTrace:unknown[]=[];page.on('response',async response=>{try{const request=response.request();if(!request.headers()['x-mailwoman-link'])return;const path=new URL(response.url()).pathname;if(path==='/jmap/api'){const data=await response.json() as {methodResponses:[string,{created?:string[];list?:unknown[]},string][]};scopedTrace.push({status:response.status(),parts:data.methodResponses.map(([method,result,tag])=>({method,tag,created:result.created?.length,count:result.list?.length}))});}else if(path.endsWith('/notifications')){const data=await response.json() as {enabled?:boolean};scopedTrace.push({policy:response.status(),enabled:data.enabled});}}catch{/* Test diagnostics only. */}});
 await engineLogin(page);await enableNotifications(page);
 const primary=await (await page.request.get('/api/me')).json() as {accountId:string;storageAuthority:string};
 const username=process.env['MW_E2E_LINKED_USERNAME']!;const list=await (await page.request.get('/api/accounts')).json() as {accounts:{id:string;address:string}[]};
 let link=list.accounts.find(account=>account.address===username)?.id;
 if(!link){const confirmed=await page.request.post('/api/security/step-up',{data:{password:process.env['MW_E2E_ENGINE_PASSWORD']??'testpass'}});expect(confirmed.ok(),await confirmed.text()).toBe(true);}
 if(!link){const created=await page.request.post('/api/accounts/link',{headers:{'X-Mailwoman-Account':primary.accountId,'X-Mailwoman-Storage-Authority':primary.storageAuthority},data:{username,password:process.env['MW_E2E_LINKED_PASSWORD']!,displayName:'Realtime linked fixture'}});expect(created.ok(),await created.text()).toBe(true);link=(await created.json() as {account:{id:string}}).account.id;}
 const linked=await (await page.request.get('/api/me',{headers:{'x-mailwoman-link':link}})).json() as {accountId:string;storageAuthority:string};
 const saved=await page.request.put('/api/account/notifications',{headers:{'x-mailwoman-link':link,'X-Mailwoman-Account':linked.accountId,'X-Mailwoman-Storage-Authority':linked.storageAuthority},data:{enabled:true,rules:[],quietHours:{enabled:false,start:'22:00',end:'07:00',timezone:'UTC'}}});expect(saved.ok(),await saved.text()).toBe(true);
 const seeded=page.waitForResponse(response=>{if(!response.url().includes('/jmap/api')||response.request().headers()['x-mailwoman-link']!==link)return false;try{return (response.request().postDataJSON() as {methodCalls:[string,{ids?:string[]},string][]}).methodCalls.some(([method,args,tag])=>method==='Email/get'&&tag==='incoming'&&args.ids?.length===0);}catch{return false;}},{timeout:20000});
 await page.reload();await expect(sidebarInbox(page)).toBeVisible();await seeded;
 let refreshes=0;page.on('request',request=>{if(request.url().includes('/api/mail/refresh'))refreshes++;});
 const subject=`Realtime inactive linked ${Date.now()}`;const began=Date.now();await injectViaSmtp({from:'Push Bot <pushbot@example.org>',to:username,subject,text:subject});
 const alert=page.getByTestId('incoming-mail-toast').filter({hasText:subject});try{await expect(alert).toBeVisible({timeout:20000});}finally{console.info('Inactive mailbox trace',JSON.stringify(scopedTrace));}await expect(messageRow(page,subject)).toHaveCount(0);expect(refreshes).toBe(0);
 await alert.getByRole('button',{name:'Open',exact:true}).click();await expect(page.getByRole('heading',{name:subject,exact:true})).toBeVisible({timeout:20000});
 await test.info().attach('delivery-inactive-linked',{body:JSON.stringify({elapsedMs:Date.now()-began,manualRefreshes:refreshes}),contentType:'application/json'});
});
test('server-filed mail in a new folder is discovered and notified without Refresh',async({page})=>{
 test.setTimeout(110000);await engineLogin(page);await enableNotifications(page);
 const {appendViaImap}=await import('./helpers.ts');const subject=`Realtime server filed ${Date.now()}`;const began=Date.now();let refreshes=0;
 page.on('request',request=>{if(request.url().includes('/api/mail/refresh'))refreshes++;});
 await appendViaImap(`Realtime filed ${Date.now()}`,subject);
 const alert=page.getByTestId('incoming-mail-toast').filter({hasText:subject});await expect(alert).toBeVisible({timeout:80000});expect(refreshes).toBe(0);
 await alert.getByRole('button',{name:'Open',exact:true}).click();await expect(page.getByRole('heading',{name:subject,exact:true})).toBeVisible({timeout:20000});
 await test.info().attach('delivery-server-filed',{body:JSON.stringify({elapsedMs:Date.now()-began,manualRefreshes:refreshes}),contentType:'application/json'});
});
