import {test,expect,type Page} from '@playwright/test';
import {build,preview,loadConfigFromFile,type PreviewServer} from 'vite';
import type {JmapRequest} from '../src/api/jmap-types.ts';
let server:PreviewServer,origin:string;
test.beforeAll(async({browserName})=>{test.setTimeout(180000);const options={outDir:`.orchestration/smart-archive-${browserName}`,emptyOutDir:true},config=(await loadConfigFromFile({command:'build',mode:'production'}))!.config;const plugins=(config.plugins??[]).flat().filter(plugin=>!(plugin&&typeof plugin==='object'&&'name' in plugin&&['mw-boot-script','mw-asset-integrity','mw-bundle-graph'].includes(String(plugin.name))));if(!process.env['MW_SMART_ARCHIVE_SKIP_BUILD'])await build({...config,configFile:false,logLevel:'warn',build:{...config.build,...options,rollupOptions:{}},plugins:[...plugins,{name:'smart-archive-fixture',transformIndexHtml:{order:'pre',handler:()=>'<html data-theme="dark"><head><meta name="viewport" content="width=device-width,initial-scale=1"></head><body><div id="root"></div><script type="module" src="/e2e/smartArchiveHarness.tsx"></script></body></html>'}}]});server=await preview({build:options,preview:{host:'127.0.0.1',port:0}});origin=server.resolvedUrls!.local[0]!;});
test.afterAll(async()=>{if(!server)return;if('closeAllConnections' in server.httpServer)server.httpServer.closeAllConnections();await new Promise<void>(resolve=>server.httpServer.close(()=>resolve()));});
async function fixture(page:Page,locked=false){
 const policy:Record<string,unknown>={enabled:locked,replaceArchive:false,strategy:'sender',multipleFolders:'ask',confirmation:'when-needed',fallback:'archive',historyLimit:100,includeSubdomains:false,excludedFolders:['role:inbox','role:sent','role:drafts','role:junk','role:trash']};
 const emails=['a','b'].map(id=>({id,subject:`Message ${id}`,mailboxIds:{inbox:true} as Record<string,boolean>,from:[{name:'Alice',email:'alice@example.test'}],to:[],receivedAt:'2026-11-25T12:00:00Z',preview:'Smart Archive browser fixture',keywords:{}}));
 const boxes=[{id:'inbox',name:'Inbox',role:'inbox'},{id:'archive',name:'Archive',role:'archive'},{id:'projects',name:'Projects',role:null},{id:'work',name:'Work',role:null,parentId:'projects'},{id:'personal',name:'Personal',role:null}].map(box=>({parentId:null,sortOrder:0,totalEmails:2,unreadEmails:2,myRights:{mayAddItems:true,mayRemoveItems:true},...box}));
 const history=[{...emails[0],id:'older',mailboxIds:{work:true},receivedAt:'2026-11-20T12:00:00Z'},{...emails[0],id:'latest',mailboxIds:{personal:true},receivedAt:'2026-11-22T12:00:00Z'}];
 const writes:Record<string,Record<string,unknown>>[]=[];
 const entry=(value:unknown)=>({value,source:locked?'global':'user',locked});
 await page.route('**/api/**',async route=>{
  const url=new URL(route.request().url()),key=decodeURIComponent(url.pathname.split('/').at(-1)!);
  if(url.pathname==='/api/settings')return route.fulfill({json:{settings:{...Object.fromEntries(Object.entries(policy).map(([key,value])=>[`mail.smartArchive.${key}`,entry(value)])),'mail.rowActions':{value:['quick:archive','quick:delete'],source:'default',locked:false}}}});
  if(url.pathname.startsWith('/api/settings/')&&route.request().method()==='PUT'){
   if(locked)return route.fulfill({status:403,json:{error:'locked'}});
   const value=(route.request().postDataJSON() as {value:unknown}).value;policy[key.replace('mail.smartArchive.','')]=value;return route.fulfill({json:entry(value)});
  }
  return route.fulfill({json:{}});
 });
 await page.route('**/fixture-jmap',async route=>{
  const request=route.request().postDataJSON() as JmapRequest;
  const responses=request.methodCalls.map(([method,args,tag])=>{
   if(method==='Email/set'){const update=args['update'] as Record<string,Record<string,unknown>>;writes.push(update);for(const [id,patch] of Object.entries(update)){const email=emails.find(email=>email.id===id);if(email&&patch['mailboxIds'])email.mailboxIds=patch['mailboxIds'] as Record<string,boolean>;}return [method,{accountId:'acct1',updated:Object.fromEntries(Object.keys(update).map(id=>[id,null]))},tag];}
   const current=emails.filter(email=>email.mailboxIds['inbox']);
   const ids=Array.isArray(args['ids'])?args['ids']:current.map(email=>email.id);
   return [method,method==='Mailbox/get'?{accountId:'acct1',state:'s',list:boxes,notFound:[]}:method==='Email/query'?{accountId:'acct1',queryState:'q',position:0,total:current.length,ids:tag==='sq'?history.map(email=>email.id):current.map(email=>email.id)}:method==='Email/get'?{accountId:'acct1',state:'s',list:tag==='sg'?history:emails.filter(email=>ids.includes(email.id)),notFound:[]}:{list:[]},tag];
  });await route.fulfill({json:{sessionState:'s',methodResponses:responses}});
 });
 await page.goto(origin);if(locked)await expect(page.getByLabel('Enable Smart Archive',{exact:true})).toBeDisabled();else await expect(page.getByLabel('Enable Smart Archive',{exact:true})).toBeEnabled();
 const row=page.locator('.list__row[data-email-id="a"]');await expect(row).toBeVisible();
 return {row,slot:row.locator('..'),writes,policy};
}
test('defaults stay ordinary, enabling adds a distinct action, ambiguity asks, cancellation is safe, Undo works and replacement preserves ordinary context Archive',async({page})=>{
 const {row,slot,writes}=await fixture(page);
 await expect(slot.getByRole('button',{name:'Archive',exact:true})).toHaveCount(1);await expect(slot.getByRole('button',{name:'Smart Archive',exact:true})).toHaveCount(0);
 await row.click({button:'right'});await page.getByRole('menuitem',{name:'Smart Archive',exact:true}).click();await expect(page.getByRole('menuitem',{name:'Folder of the latest email from this sender',exact:true})).toBeDisabled();await page.keyboard.press('Escape');await page.keyboard.press('Escape');
 await page.getByLabel('Enable Smart Archive',{exact:true}).check();await expect(slot.getByRole('button',{name:'Smart Archive',exact:true})).toHaveCount(1);await expect(slot.getByRole('button',{name:'Archive',exact:true})).toHaveCount(1);
 await slot.hover();await slot.getByRole('button',{name:'Smart Archive',exact:true}).click();const dialog=page.getByRole('dialog',{name:'Smart Archive',exact:true});await expect(dialog).toBeVisible();await expect(dialog.getByRole('button',{name:'Smart Archive',exact:true})).toBeDisabled();await expect(dialog.getByRole('combobox').locator('option')).toContainText(['Choose a destination folder','Personal','Projects / Work']);await dialog.getByRole('button',{name:'Cancel',exact:true}).click();expect(writes).toHaveLength(0);
 await slot.hover();await slot.getByRole('button',{name:'Smart Archive',exact:true}).click();await dialog.getByRole('combobox').selectOption('work');await dialog.getByRole('button',{name:'Smart Archive',exact:true}).click();await expect.poll(()=>writes.length).toBe(1);expect(writes[0]?.['a']?.['mailboxIds']).toEqual({work:true});
 await page.getByTestId('toast-region').getByRole('button',{name:'Undo',exact:true}).click();await expect(row).toBeVisible();expect(writes[1]?.['a']?.['mailboxIds']).toEqual({inbox:true});
 await page.getByLabel('Replace the main Archive action',{exact:true}).check();await expect(slot.getByRole('button',{name:'Archive',exact:true})).toHaveCount(0);await expect(slot.getByRole('button',{name:'Smart Archive',exact:true})).toHaveCount(1);
 await row.click({button:'right'});await page.getByRole('menuitem',{name:'Archive',exact:true}).click();await expect.poll(()=>writes.length).toBe(3);expect(writes[2]?.['a']?.['mailboxIds']).toEqual({archive:true});
});
test('administrator locks disable preferences while permitting the enabled Smart Archive action',async({page})=>{const {slot}=await fixture(page,true);await expect(page.getByLabel('Enable Smart Archive',{exact:true})).toBeChecked();for(const control of await page.locator('aside input,aside select,aside textarea').all())await expect(control).toBeDisabled();await expect(slot.getByRole('button',{name:'Smart Archive',exact:true})).toHaveCount(1);});
