import {test,expect} from '@playwright/test';
import {build,preview,loadConfigFromFile,type PreviewServer} from 'vite';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import type {JmapRequest} from '../src/api/jmap-types.ts';

let server:PreviewServer,origin:string;
test.beforeAll(async({browserName})=>{
 test.setTimeout(180000);
 const options={outDir:`.orchestration/body-prefetch-${browserName}`,emptyOutDir:true};
 const config=(await loadConfigFromFile({command:'build',mode:'production'}))!.config;
 const plugins=(config.plugins??[]).flat().filter(plugin=>!(plugin&&typeof plugin==='object'&&'name' in plugin&&['mw-boot-script','mw-asset-integrity','mw-bundle-graph'].includes(String(plugin.name))));
 if(!process.env['MW_BODY_PREFETCH_SKIP_BUILD'])await build({...config,configFile:false,logLevel:'warn',build:{...config.build,...options,rollupOptions:{}},plugins:[...plugins,{name:'body-prefetch-fixture',transformIndexHtml:{order:'pre',handler:()=>'<html data-theme="dark"><head><meta name="viewport" content="width=device-width,initial-scale=1"></head><body><div id="root"></div><script type="module" src="/e2e/body-prefetchHarness.tsx"></script></body></html>'}}]});
 server=await preview({build:options,preview:{host:'127.0.0.1',port:0}});origin=server.resolvedUrls!.local[0]!;
});
test.afterAll(async()=>{if(!server)return;if('closeAllConnections' in server.httpServer)server.httpServer.closeAllConnections();await new Promise<void>(resolve=>server.httpServer.close(()=>resolve()));});

test('idle bodies paint immediately on click while refresh is held, and user disable stops preloading',async({page})=>{
 let disabled=false,hold=false,release:(()=>void)|undefined,foreground=0;
 const bodies:string[]=[],methods:string[]=[];
 const emails=Array.from({length:20},(_,index)=>({id:`message-${index}`,subject:`Message ${index}`,mailboxIds:{inbox:true},from:[{name:'Sender',email:'sender@example.test'}],to:[],receivedAt:new Date(Date.UTC(2026,10,25)-index*60000).toISOString(),preview:'Body prefetch fixture',keywords:{},hasAttachment:false}));
 await page.route('**/api/**',route=>route.fulfill({json:route.request().url().includes('/settings')?{settings:{'mail.cache.idleDelayMs':{value:100,source:'user',locked:false},'mail.cache.maxQueued':{value:3,source:'user',locked:false},...(disabled?{'mail.cache.preloadMessages':{value:false,source:'user',locked:false}}:{})}}:{}}));
 const handler=readFileSync('../../crates/mw-server/src/reader-address-handler.js','utf8').replace(/\r\n?/g,'\n');
 const bridge=readFileSync('../../crates/mw-server/src/remote-frame.js','utf8').replace(/\r\n?/g,'\n')
  .replace('__MW_PARENT_ORIGIN__','null').replace('__MW_ADDRESS_HANDLER_SOURCE__',JSON.stringify(handler))
  .replace('__MW_ADDRESS_HANDLER_HASH__',JSON.stringify(createHash('sha256').update(handler).digest('base64')));
 await page.route('**/api/rc/frame*',route=>route.fulfill({contentType:'text/html',body:`<!doctype html><html><body style="margin:0"><script>${bridge}</script></body></html>`}));
 await page.route('**/fixture-jmap',async route=>{
  const request=route.request().postDataJSON() as JmapRequest;
  const responses=[];
  for(const [method,args,tag] of request.methodCalls){
   methods.push(method);
   if(method==='Email/get'&&args['fetchHTMLBodyValues']){
    if(args['mailwomanBodyUse']===false)bodies.push(String((args['ids'] as string[])[0]));
    else if(hold){foreground++;await new Promise<void>(resolve=>{release=resolve;});}
   }
   const ids=Array.isArray(args['ids'])?args['ids']:emails.map(email=>email.id);
   const list=emails.filter(email=>ids.includes(email.id)).map(email=>args['fetchHTMLBodyValues']?{...email,htmlBody:[{partId:'body',blobId:'body-blob',size:40,type:'text/html'}],bodyValues:{body:{value:`<p>Instant cached body for ${email.id}</p>`}}}:email);
   responses.push([method,method==='Mailbox/get'?{accountId:'acct1',state:'s',notFound:[],list:[{id:'inbox',name:'Inbox',role:'inbox',parentId:null,sortOrder:0,totalEmails:20,unreadEmails:20}]}:method==='Email/query'?{accountId:'acct1',queryState:'q',position:0,total:20,ids:emails.map(email=>email.id)}:method==='Email/get'?{accountId:'acct1',state:'s',list,notFound:[]}:{list:[]},tag]);
  }
  await route.fulfill({json:{sessionState:'s',methodResponses:responses}});
 });
 try{
  await page.goto(origin);
  const first=page.locator('.list__row[data-email-id="message-0"]');await expect(first).toBeVisible();
  await expect.poll(()=>new Set(bodies).size).toBe(3);
  const scroller=page.locator('.list__scroll');
  await scroller.evaluate(node=>{node.scrollTop=500;node.dispatchEvent(new Event('scroll'));});
  const visibleId=await scroller.evaluate(node=>{
   const top=node.getBoundingClientRect().top;
   return [...node.querySelectorAll<HTMLElement>('.list__row')].find(row=>row.getBoundingClientRect().bottom>top)?.dataset['emailId'];
  });
  expect(visibleId).toBeTruthy();await expect.poll(()=>bodies.includes(visibleId!)).toBe(true);
  await scroller.evaluate(node=>{node.scrollTop=0;node.dispatchEvent(new Event('scroll'));});
  expect(methods.some(method=>method.endsWith('/set'))).toBe(false);
  hold=true;await first.click();await expect.poll(()=>foreground).toBe(1);
  await expect(page.frameLocator('iframe.reader__frame').frameLocator('iframe').locator('body')).toContainText('Instant cached body for message-0');
  release?.();hold=false;
  disabled=true;bodies.length=0;await page.reload();await expect(first).toBeVisible();
  await first.hover();await page.waitForTimeout(600);expect(bodies).toHaveLength(0);
 }finally{release?.();await page.unrouteAll({behavior:'wait'});}
});
