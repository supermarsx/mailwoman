// Preference changes are isolated in each browser context: no shared backend
// settings are mutated. JMAP reads still use the standing mailbox server.
import {test,expect,type Page} from '@playwright/test';
import {login} from './helpers.ts';
test.use({serviceWorkers:'block'});test.setTimeout(75000);
type Order='newest'|'oldest'|'sender'|'subject';
async function fixture(page:Page,options:{style?:'cards'|'table';empty?:boolean;locked?:boolean}={}){
 page.setDefaultTimeout(15000);await page.setViewportSize({width:1280,height:900});
 await page.addLocatorHandler(page.getByRole('dialog',{name:'Allow notifications?',exact:true}),async dialog=>{await dialog.getByRole('button',{name:'Not now',exact:true}).click();});
 let entry={value:'newest' as Order,source:options.locked?'global':'user',locked:options.locked??false};const writes:Order[]=[];
 await page.route(/\/api\/settings(?:\/mail\.messageListOrder)?(?:\?.*)?$/,async route=>{
  const request=route.request();if(new URL(request.url()).pathname.endsWith('/mail.messageListOrder')){
   if(entry.locked){await route.fulfill({status:403,json:{code:'setting_locked'}});return;}
   const value=(request.postDataJSON()as{value:Order}).value;entry={value,source:'user',locked:false};writes.push(value);await route.fulfill({status:200,json:entry});return;
  }
  const response=await route.fetch(),body=await response.json()as{settings:Record<string,unknown>};
  body.settings['notifications.askPermission']={value:false,source:'global',locked:true};
  body.settings['mail.messageListOrder']=entry;body.settings['mail.messageListStyle']={value:options.style??'cards',source:'user',locked:false};
  await route.fulfill({response,json:body});
 });
 if(options.empty)await page.route(/\/jmap(?:\/api)?(?:\?.*)?$/,async route=>{
  const response=await route.fetch(),body=await response.json()as{methodResponses:[string,Record<string,unknown>,string][]};
  for(const [name,args]of body.methodResponses??[]){if(name==='Mailbox/get'){args.mailwomanInitialSync='ready';if(Array.isArray(args.list))args.list=args.list.map(box=>({...box,totalEmails:0,unreadEmails:0,mailwomanSynced:true}));}if(name==='Email/query'){args.ids=[];args.total=0;args.position=0;}if(name==='Email/get'){args.list=[];args.notFound=[];}}
  await route.fulfill({response,json:body});
 });
 await login(page);const list=page.locator('section.list:visible');await expect(list).toBeVisible();
 if(options.empty)await expect(list.getByText('No messages',{exact:true})).toBeVisible();else await expect(list.locator('.list__row').first()).toBeVisible();
 return {list,writes};
}
async function iconMenu(page:Page){await page.getByTestId('list-order-menu').click();const menu=page.getByRole('menu',{name:'Order by',exact:true});await expect(menu).toBeVisible();return menu;}
async function dismissNotice(page:Page){const dialog=page.getByRole('dialog',{name:'Allow notifications?',exact:true});if(await dialog.isVisible())await dialog.getByRole('button',{name:'Not now',exact:true}).click();}
async function rowOrder(page:Page){await dismissNotice(page);await page.locator('section.list:visible .list__row').first().click({button:'right',position:{x:10,y:10}});await page.getByRole('menuitem',{name:'Order by',exact:true}).click();return page.getByRole('menu',{name:'Order by',exact:true});}
async function backgroundOrder(page:Page){await page.locator('section.list:visible').click({button:'right',position:{x:3,y:3}});await page.getByRole('menuitem',{name:'Order by',exact:true}).click();return page.getByRole('menu',{name:'Order by',exact:true});}

test('cards icon, row and background menus share checked order and persist the isolated preference',async({page})=>{
 const state=await fixture(page);const button=page.getByTestId('list-order-menu');await expect(button).toBeVisible();await expect(button.locator('[data-icon="sort"]')).toBeVisible();
 let menu=await iconMenu(page);await expect(menu.getByRole('menuitemradio',{name:'Newest first',exact:true})).toHaveAttribute('aria-checked','true');
 await menu.getByRole('menuitemradio',{name:'Oldest first',exact:true}).click();await expect.poll(()=>state.writes).toEqual(['oldest']);
 menu=await rowOrder(page);await expect(menu.getByRole('menuitemradio',{name:'Oldest first',exact:true})).toHaveAttribute('aria-checked','true');await menu.getByRole('menuitemradio',{name:'Sender',exact:true}).click();
 await expect.poll(()=>state.writes).toEqual(['oldest','sender']);menu=await backgroundOrder(page);await expect(menu.getByRole('menuitemradio',{name:'Sender',exact:true})).toHaveAttribute('aria-checked','true');await menu.getByRole('menuitemradio',{name:'Subject',exact:true}).click();
 await expect.poll(()=>state.writes).toEqual(['oldest','sender','subject']);await page.reload();await expect(page.getByTestId('list-order-menu')).toBeVisible();menu=await iconMenu(page);await expect(menu.getByRole('menuitemradio',{name:'Subject',exact:true})).toHaveAttribute('aria-checked','true');
});
test('table keeps the icon and existing select synchronized',async({page})=>{
 const state=await fixture(page,{style:'table'});const select=state.list.getByRole('combobox',{name:'Message order',exact:true});await expect(select).toHaveValue('newest');
 await (await iconMenu(page)).getByRole('menuitemradio',{name:'Oldest first',exact:true}).click();await expect(select).toHaveValue('oldest');
 await select.selectOption('sender');await expect.poll(()=>state.writes).toEqual(['oldest','sender']);await expect((await rowOrder(page)).getByRole('menuitemradio',{name:'Sender',exact:true})).toHaveAttribute('aria-checked','true');
});
test('empty list retains the sort icon and background submenu',async({page})=>{
 const state=await fixture(page,{empty:true});await expect(page.getByTestId('list-order-menu')).toBeVisible();await expect((await iconMenu(page)).getByRole('menuitemradio',{name:'Newest first',exact:true})).toHaveAttribute('aria-checked','true');await page.keyboard.press('Escape');
 await expect((await backgroundOrder(page)).getByRole('menuitemradio',{name:'Subject',exact:true})).toBeVisible();expect(state.writes).toEqual([]);
});
test('administrator lock prevents changes and the sort icon fits a visible phone list',async({page},info)=>{
 const state=await fixture(page,{locked:true});await dismissNotice(page);await expect(page.getByTestId('list-order-menu')).toBeDisabled();await state.list.locator('.list__row').first().click({button:'right',position:{x:10,y:10}});await expect(page.getByRole('menuitem',{name:'Order by',exact:true})).toHaveAttribute('aria-disabled','true');await page.keyboard.press('Escape');
 await page.setViewportSize({width:390,height:844});const button=page.getByTestId('list-order-menu');await expect(button).toBeVisible();await expect(button).toBeDisabled();
 const geometry=await button.evaluate(node=>{const rect=node.getBoundingClientRect();return{left:rect.left,right:rect.right,viewport:innerWidth};});expect(geometry.left).toBeGreaterThanOrEqual(0);expect(geometry.right).toBeLessThanOrEqual(geometry.viewport);expect(state.writes).toEqual([]);
 await page.screenshot({path:info.outputPath('message-order-phone.png')});
});
