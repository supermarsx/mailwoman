// Browser placement behavior; new setting reads/writes are isolated per test,
// so a standing backend predating the registry keys can serve the real mailbox.
import {test,expect,type Page} from '@playwright/test';
import {login,messageRow} from './helpers.ts';

test.use({serviceWorkers:'block'});test.setTimeout(75_000);
type Placement='toolbar'|'menu'|'hidden';
interface Entry {value:Placement;locked:boolean;source:string;userValue?:Placement;lockedBy?:string}
interface Change {id:string;method:string;value:Placement}
const defaults:Record<string,Placement>={reply:'toolbar','reply-all':'toolbar',forward:'toolbar',delete:'toolbar','save-contacts':'menu','create-filter':'menu'};
async function fixture(page:Page,options:{reply?:Placement;replyLocked?:boolean;replyFeature?:boolean;contacts?:Placement}={}){
 page.setDefaultTimeout(15000);await page.setViewportSize({width:1920,height:1080});
 await page.addLocatorHandler(page.getByRole('dialog',{name:'Allow notifications?',exact:true}),async dialog=>{await dialog.getByRole('button',{name:'Not now',exact:true}).click();});
 const entries:Record<string,Entry>=Object.fromEntries(Object.entries(defaults).map(([id,value])=>[id,{value,source:'default',locked:false}]));
 if(options.reply)entries['reply']={value:options.reply,source:options.replyLocked?'global':'user',locked:options.replyLocked??false,...(options.replyLocked?{userValue:'toolbar' as const,lockedBy:'global'}:{})};
 if(options.contacts)entries['save-contacts']={value:options.contacts,source:'user',locked:false};
 const writes:Change[]=[];
 await page.route(/\/api\/settings(?:\/reader\.actions\.[a-z-]+)?(?:\?.*)?$/,async route=>{
  const request=route.request(),path=new URL(request.url()).pathname;
  if(path.includes('/reader.actions.')){
   const id=decodeURIComponent(path.split('/').at(-1)!).replace('reader.actions.','');
   if(entries[id]?.locked){await route.fulfill({status:403,json:{code:'setting_locked',lockedBy:'global'}});return;}
   const value=request.method()==='DELETE'?defaults[id]!:(request.postDataJSON() as {value:Placement}).value;
   entries[id]={value,source:request.method()==='DELETE'?'default':'user',locked:false};writes.push({id,method:request.method(),value});
   await route.fulfill({status:200,json:entries[id]});return;
  }
  const response=await route.fetch();const body=await response.json() as {settings:Record<string,unknown>};
  for(const [id,entry] of Object.entries(entries))body.settings[`reader.actions.${id}`]=entry;
  body.settings['mail.allowReply']={value:options.replyFeature??true,source:'global',locked:options.replyFeature===false};
  body.settings['mail.allowForward']={value:true,source:'global',locked:false};
  body.settings['contacts.saveFromMessage.enabled']={value:true,source:'global',locked:false};
  await route.fulfill({response,json:body});
 });
 await login(page);await messageRow(page,'Lunch tomorrow?').click();
 await expect(page.locator('.reader__subject')).toHaveText('Lunch tomorrow?');
 return {entries,writes};
}
const inline=(page:Page,id:string)=>page.locator(`.reader__toolbar:visible [data-reader-item="${id}"]`);
const menuAction=(page:Page,id:string)=>page.locator(`[role="menu"] [data-action-id="${id}"]`);
async function more(page:Page){
 const button=page.locator('.reader__toolbar:visible').getByTestId('reader-more');await button.scrollIntoViewIfNeeded();
 await page.evaluate(()=>new Promise<void>(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolve()))));
 await button.click();await expect(page.getByRole('menu',{name:'More actions',exact:true})).toBeVisible();
}
async function preferences(page:Page){
 await page.locator('#shell-nav').getByRole('button',{name:'Settings',exact:true}).click();
 await page.getByRole('navigation',{name:'Settings sections'}).locator('[data-settings-entry="reader"]').click();
 const card=page.getByTestId('reader-action-preferences');await expect(card).toBeVisible();return card;
}
async function setPlacement(page:Page,id:string,value:Placement){
 const card=await preferences(page),select=card.locator(`#reader-action-${id}`);
 await select.selectOption(value);await expect(select).toHaveValue(value);
 await page.getByRole('button',{name:'Close Settings',exact:true}).click();
 await expect(page.locator('.reader__subject')).toBeVisible();
}

test('user moves Reply to More, hides it, and resets without reloading the reader',async({page})=>{
 const state=await fixture(page);await expect(inline(page,'reply')).toBeVisible();
 const toolbar=await page.locator('.reader__toolbar').elementHandle();
 await page.evaluate(()=>{document.documentElement.dataset['placementDocument']='retained';});
 await setPlacement(page,'reply','menu');await expect(inline(page,'reply')).toBeHidden();
 await more(page);await expect(menuAction(page,'reply')).toBeVisible();await page.keyboard.press('Escape');
 await setPlacement(page,'reply','hidden');await expect(inline(page,'reply')).toBeHidden();
 await more(page);await expect(menuAction(page,'reply')).toHaveCount(0);await page.keyboard.press('Escape');
 const card=await preferences(page);const select=card.locator('#reader-action-reply');
 await select.locator('..').getByRole('button').click();await expect(select).toHaveValue('toolbar');
 await page.getByRole('button',{name:'Close Settings',exact:true}).click();await expect(inline(page,'reply')).toBeVisible();
 expect(await toolbar!.evaluate(node=>node.isConnected)).toBe(true);
 expect(await page.locator('html').getAttribute('data-placement-document')).toBe('retained');
 expect(state.writes).toEqual([{id:'reply',method:'PUT',value:'menu'},{id:'reply',method:'PUT',value:'hidden'},{id:'reply',method:'DELETE',value:'toolbar'}]);
});

test('promoting Save contacts preserves its submenu and opens the real chooser',async({page})=>{
 const state=await fixture(page);await more(page);await expect(menuAction(page,'save-contacts')).toBeVisible();await page.keyboard.press('Escape');
 await setPlacement(page,'save-contacts','toolbar');await expect(inline(page,'save-contacts')).toBeVisible();
 await more(page);await expect(menuAction(page,'save-contacts')).toHaveCount(0);await page.keyboard.press('Escape');
 await inline(page,'save-contacts').getByRole('button').click();
 await expect(menuAction(page,'save-contacts:choose')).toBeVisible();await menuAction(page,'save-contacts:choose').click();
 const dialog=page.getByRole('dialog',{name:'Save contacts from this message',exact:true});await expect(dialog).toBeVisible();
 await expect(dialog.getByRole('checkbox').first()).toBeVisible();await dialog.getByRole('button',{name:'Cancel',exact:true}).click();
 expect(state.writes).toEqual([{id:'save-contacts',method:'PUT',value:'toolbar'}]);
});

test('administrator placement lock hides Reply and disables preference and reset',async({page})=>{
 const state=await fixture(page,{reply:'hidden',replyLocked:true});await expect(inline(page,'reply')).toBeHidden();
 await more(page);await expect(menuAction(page,'reply')).toHaveCount(0);await page.keyboard.press('Escape');
 const card=await preferences(page),select=card.locator('#reader-action-reply');await expect(select).toBeDisabled();await expect(select).toHaveValue('hidden');
 await expect(select.locator('..').getByRole('button')).toBeDisabled();expect(state.writes).toEqual([]);
});

test('placement cannot restore a disabled Reply feature and the toolbar fits a phone',async({page},info)=>{
 await fixture(page,{replyFeature:false,reply:'menu',contacts:'toolbar'});
 await setPlacement(page,'reply','toolbar');await expect(inline(page,'reply')).toBeHidden();await expect(inline(page,'reply-all')).toBeHidden();
 await more(page);await expect(menuAction(page,'reply')).toHaveCount(0);await expect(menuAction(page,'reply-all')).toHaveCount(0);await page.keyboard.press('Escape');
 await page.setViewportSize({width:390,height:844});await messageRow(page,'Lunch tomorrow?').click();const toolbar=page.locator('.reader__toolbar:visible');await expect(toolbar).toBeVisible();
 await expect.poll(()=>toolbar.evaluate(node=>node.scrollWidth<=node.clientWidth+1&&document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
 const geometry=await toolbar.evaluate(node=>({row:node.clientWidth,scroll:node.scrollWidth}));expect(geometry.scroll).toBeLessThanOrEqual(geometry.row+1);
 await more(page);await expect(menuAction(page,'reply')).toHaveCount(0);await expect(menuAction(page,'reply-all')).toHaveCount(0);
 await page.screenshot({path:info.outputPath('reader-actions-phone.png')});
});
