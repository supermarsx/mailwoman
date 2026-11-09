import {test,expect} from '@playwright/test';
import {composeSurface,login,selectComposeFormat} from './helpers.ts';
test.use({serviceWorkers:'block'});test.setTimeout(90_000);
const card={id:'browser-mention-jane',addressBookId:'browser-contacts',uid:'browser-mention-jane',kind:'individual',name:{full:'Jane Doe',given:'Jane',surname:'Doe',prefix:'',suffix:''},nicknames:[],organizations:[],titles:[],emails:[{context:'work',value:'jane@example.org',pref:1}],phones:[],onlineServices:[],addresses:[],anniversaries:[],notes:'',photoBlobId:null,isFavorite:true,groupIds:[],pgpKey:null,smimeCert:null,etag:null};
for(const format of ['html','plain','rich']as const)test(`explicit @mention selection inserts portable content and one recipient in ${format} compose`,async({page},info)=>{
 const pageErrors:string[]=[];page.on('pageerror',error=>pageErrors.push(error.message));
 await page.addLocatorHandler(page.getByRole('dialog',{name:'Allow notifications?',exact:true}),async dialog=>{await dialog.getByRole('button',{name:'Not now',exact:true}).click();});
 await page.route(/\/api\/settings(?:\?.*)?$/,async route=>{const response=await route.fetch();const body=await response.json()as{settings:Record<string,unknown>};body.settings['mail.draftSaveMode']={value:'manual',source:'global',locked:true};await route.fulfill({response,json:body});});
 // Response-only contacts. Saving/sending is outside this browser regression.
 await page.route(/\/jmap(?:\/api)?$/,async route=>{
  if(route.request().method()!=='POST')return route.continue();
  const calls=(route.request().postDataJSON()as{methodCalls:[string,{accountId?:string},string][]}).methodCalls;
  const response=await route.fetch();const body=await response.json()as{methodResponses:[string,Record<string,unknown>,string][]};
  body.methodResponses=body.methodResponses.map(tuple=>{const call=calls.find(item=>item[2]===tuple[2]);if(call?.[0]==='ContactCard/get')return[call[0],{accountId:call[1].accountId,list:[card],state:'browser-mentions',notFound:[]},tuple[2]];
   if(call?.[0]==='AddressBook/get')return[call[0],{accountId:call[1].accountId,list:[{id:'browser-contacts',name:'Contacts',isDefault:true}],state:'browser-mentions',notFound:[]},tuple[2]];
   if(call?.[0]==='ContactGroup/get')return[call[0],{accountId:call[1].accountId,list:[],state:'browser-mentions',notFound:[]},tuple[2]];
   if(call?.[0]==='ContactCard/query')return[call[0],{accountId:call[1].accountId,ids:[card.id],queryState:'browser-mentions',position:0,canCalculateChanges:false,total:1},tuple[2]];
   if(call?.[0]==='ContactCard/autocomplete')return['ContactCard/autocomplete',{accountId:call[1].accountId,list:[{id:card.id,name:card.name.full,email:'jane@example.org',isFavorite:true}]},tuple[2]];return tuple;});
  await route.fulfill({response,json:body});
 });
 await login(page);await page.getByRole('button',{name:'Compose',exact:true}).click();const compose=composeSurface(page);await expect(compose).toBeVisible();
 if(format!=='html')await selectComposeFormat(page,compose,format);
 const native=compose.frameLocator('iframe[title="Visual editor"]');const editor=format==='html'?native.getByRole('textbox',{name:'Body',exact:true}):compose.getByRole('textbox',{name:'Body',exact:true});
 const popup=format==='html'?native.getByRole('listbox'):page.getByRole('listbox',{name:'Mention suggestions',exact:true});
 const recipients=async()=>compose.getByTestId('compose-rf-to').evaluate(root=>[...root.querySelectorAll('[data-recipient]')].map(node=>node.getAttribute('data-recipient')??'').concat([...root.querySelectorAll('input')].map(input=>input.value)).join(','));
 for(let attempt=0;attempt<2;attempt++){
  await editor.fill('');await editor.pressSequentially('@Ja');await expect(popup).toBeVisible();if(attempt===0)expect(await recipients()).not.toContain('jane@example.org');
  if(format==='html')await popup.getByRole('option',{name:'Jane Doe <jane@example.org>',exact:true}).click();else await editor.press('Enter');
  await expect(popup).toHaveCount(0);await expect.poll(recipients).toContain('jane@example.org');expect((await recipients()).match(/jane@example\.org/g)).toHaveLength(1);
  if(format==='plain')await expect(editor).toHaveValue('@jane@example.org ');else await expect(editor.locator('a')).toHaveAttribute('href','mailto:jane%40example.org');
 }
 expect(pageErrors).toEqual([]);await page.screenshot({path:info.outputPath(`compose-mention-${format}.png`)});
});
