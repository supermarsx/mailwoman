import {test,expect,type Page} from '@playwright/test';
import {login,messageRow,settingsSurface} from './helpers.ts';
test.use({serviceWorkers:'block'});test.setTimeout(90000);
async function policy(page:Page,permanent=false){
 const entries:Record<string,{value:unknown;source:string;locked:boolean}>={
  'mail.cache.keepIndefinitely':{value:permanent,source:'user',locked:false},
  'mail.conversations.enabled':{value:false,source:'default',locked:false},
  'mail.conversations.allowed':{value:true,source:'global',locked:false},
  'notifications.askPermission':{value:false,source:'global',locked:true},
 };
 await page.route(/\/api\/settings(?:\/[^/?]+)?(?:\?.*)?$/,async route=>{
  const request=route.request(),key=decodeURIComponent(new URL(request.url()).pathname.split('/').at(-1)!);
  if(request.method()==='PUT'){entries[key]={value:request.postDataJSON().value,source:'user',locked:false};return route.fulfill({json:entries[key]});}
  await route.fulfill({json:{settings:entries}});
 });return entries;
}
test('conversation opt-in is separate from category grouping in settings and the order menu',async({page})=>{
 await page.setViewportSize({width:1280,height:900});const entries=await policy(page);await login(page);
 await page.getByTestId('list-order-menu').filter({visible:true}).click();
 const toggle=page.getByRole('menuitemcheckbox',{name:'Group messages into conversations'});await expect(toggle).toHaveAttribute('aria-checked','false');await toggle.click();
 await expect.poll(()=>entries['mail.conversations.enabled']?.value).toBe(true);
 expect(entries['mail.messageListGroup']).toBeUndefined();
 await page.getByRole('button',{name:'Settings',exact:true}).click();const screen=settingsSurface(page);
 const input=screen.getByRole('checkbox',{name:'Group messages into conversations'});await expect(input).toBeChecked();await input.uncheck();
 await expect.poll(()=>entries['mail.conversations.enabled']?.value).toBe(false);
 expect(entries['mail.messageListGroup']).toBeUndefined();
});
test('encrypted mail survives reload and supplies the body when the browser goes offline',async({page,context})=>{
 await page.setViewportSize({width:1280,height:900});await policy(page,true);await login(page);
 await messageRow(page,'Lunch tomorrow?').click();await expect(page.locator('.reader__subject').filter({visible:true})).toHaveText('Lunch tomorrow?');
 const count=()=>page.evaluate(()=>new Promise<number>((resolve,reject)=>{
  const request=indexedDB.open('mailwoman-mail-cache');request.onerror=()=>reject(request.error);request.onsuccess=()=>{const db=request.result;if(!db.objectStoreNames.contains('entries')){db.close();resolve(0);return;}const read=db.transaction('entries').objectStore('entries').getAll();read.onsuccess=()=>{if(read.result.some(row=>!(row.encrypted instanceof Uint8Array)||Object.hasOwn(row,'value'))){db.close();reject(new Error('Mail cache must contain encrypted bytes'));return;}resolve(read.result.filter(row=>row.kind==='message').length);db.close();};read.onerror=()=>{db.close();reject(read.error);};};
 }));
 await expect.poll(count).toBeGreaterThan(0);
 let blockedBodies=0;await page.route(/\/jmap\/api$/,async route=>{const calls=route.request().postDataJSON().methodCalls as [string,{properties?:string[]}][];if(calls.some(([method,args])=>method==='Email/get'&&args.properties?.includes('bodyValues'))){blockedBodies++;await route.abort('failed');return;}await route.continue();});
 await page.reload();await page.getByRole('button',{name:'Settings',exact:true}).click();const settings=settingsSurface(page);await settings.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Message display',exact:true}).click();await expect(settings.getByRole('checkbox',{name:'Keep cached mail indefinitely'})).toBeChecked();await page.getByRole('tab',{name:'Mail',exact:true}).click();await expect(messageRow(page,'Lunch tomorrow?')).toBeVisible();
 await context.setOffline(true);await messageRow(page,'Lunch tomorrow?').click();
 await expect(page.locator('.reader__subject').filter({visible:true})).toHaveText('Lunch tomorrow?');
 const body=page.getByTestId('reader-offline-frame').contentFrame().locator('body');
 await expect(body).toContainText('Are you free at noon?');expect(blockedBodies).toBeGreaterThan(0);
 await context.setOffline(false);
 await page.getByRole('button',{name:'Settings',exact:true}).click();const screen=settingsSurface(page);
 await screen.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Message display',exact:true}).click();
 await screen.getByRole('button',{name:'Clear cached mail now',exact:true}).click();await expect.poll(count).toBe(0);
});
