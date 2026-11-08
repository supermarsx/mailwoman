// Browser behavior and HTTP contracts against the standing mock stack.
// The new effective setting and preference writes stay in per-test route state;
// no shared account preferences are mutated. Native persistence is outside this spec.
import {test,expect,type Page} from '@playwright/test';
import type {Mailbox} from '../src/api/jmap-types.ts';
import type {AccountUiPreferences} from '../src/api/accountPreferences.ts';
import {login} from './helpers.ts';

test.use({serviceWorkers:'block'});test.setTimeout(90_000);
const SPAM='browser-empty-spam',DRAFTS='browser-empty-drafts',READ='browser-read',PARENT='browser-parent',CHILD='browser-child',EMPTY_CHILD='browser-empty-child';
interface State {hide:boolean;locked:boolean;spamCount:number;prefs:AccountUiPreferences;settingsWrites:unknown[];preferenceWrites:AccountUiPreferences[]}
async function fixture(page:Page,options:{hide?:boolean;locked?:boolean;override?:boolean}={}):Promise<State>{
 page.setDefaultTimeout(15000);
 const state:State={hide:options.hide??false,locked:options.locked??false,spamCount:0,prefs:{folderCollapsed:[],folderHidden:[],userFoldersCollapsed:false,folderProps:options.override===undefined?{}:{[SPAM]:{hideWhenEmpty:options.override}}},settingsWrites:[],preferenceWrites:[]};
 await page.addLocatorHandler(page.getByRole('dialog',{name:'Allow notifications?',exact:true}),async dialog=>{await dialog.getByRole('button',{name:'Not now',exact:true}).click();});
 const entry=()=>({value:state.hide,source:state.locked?'global':'user',locked:state.locked,...(state.locked?{lockedBy:'global'}:{}),userValue:state.hide});
 await page.route(/\/api\/settings(?:\/folders\.hideEmpty)?(?:\?.*)?$/,async route=>{
  const request=route.request(),url=new URL(request.url());
  if(url.pathname.endsWith('/folders.hideEmpty')){
   if(state.locked){await route.fulfill({status:403,json:{code:'setting_locked',lockedBy:'global'}});return;}
   if(request.method()==='PUT'){state.hide=(request.postDataJSON() as {value:boolean}).value;state.settingsWrites.push(state.hide);}
   else if(request.method()==='DELETE'){state.hide=false;state.settingsWrites.push('reset');}
   await route.fulfill({status:200,json:entry()});return;
  }
  const response=await route.fetch();const body=await response.json();
  const settings=body.settings as Record<string,{value:unknown}>;
  settings['folders.hideEmpty']=entry();
  const allowed=settings['properties.folder.overrides.allowed'];
  if(allowed&&Array.isArray(allowed.value))allowed.value=[...new Set([...allowed.value,'hideWhenEmpty'])];
  await route.fulfill({response,json:body});
 });
 let base:Record<string,unknown>|undefined,version=0;
 await page.route(/\/api\/account\/preferences(?:\?.*)?$/,async route=>{
  if(route.request().method()==='PUT'){
   state.prefs=(route.request().postDataJSON() as {preferences:AccountUiPreferences}).preferences;
   state.preferenceWrites.push(structuredClone(state.prefs));version++;
  }else if(!base){
   const response=await route.fetch();base=await response.json() as Record<string,unknown>;
   state.prefs={...base['preferences'] as AccountUiPreferences,...state.prefs};
  }
  await route.fulfill({status:200,headers:{'content-type':'application/json',etag:`"browser-prefs-${version}"`},json:{...base,version:1,preferences:state.prefs}});
 });
 const box=(id:string,name:string,totalEmails:number,parentId:string|null=null,role:string|null=null):Mailbox=>({id,name,parentId,role,sortOrder:10,totalEmails,unreadEmails:0,isSubscribed:true,myRights:{mayReadItems:true,mayAddItems:false,mayRemoveItems:false,maySetSeen:false,maySetKeywords:false,mayCreateChild:false,mayRename:false,mayDelete:false}});
 await page.route(/\/jmap(?:\/api)?$/,async route=>{
  if(route.request().method()!=='POST')return route.continue();
  const request=route.request().postDataJSON() as {methodCalls:[string,{ids?:string[]|null},string][]};
  const response=await route.fetch();const body=await response.json() as {methodResponses:[string,{list?:Mailbox[];notFound?:string[]},string][]};
  const folders=[box(SPAM,'Browser Spam',state.spamCount,null,'junk'),box(DRAFTS,'Browser Drafts',0,null,'drafts'),box(READ,'Browser Read messages',3),box(PARENT,'Browser Parent',0),box(CHILD,'Browser Child',1,PARENT),box(EMPTY_CHILD,'Browser Empty child',0,PARENT)];
  for(const [method,args,tag] of body.methodResponses){
   if(method!=='Mailbox/get')continue;
   const ids=request.methodCalls.find(call=>call[2]===tag)?.[1].ids;
   const injected=folders.filter(folder=>!ids||ids.includes(folder.id));
   args.list=[...(args.list??[]).filter(folder=>!folders.some(seed=>seed.id===folder.id)),...injected];
   args.notFound=(args.notFound??[]).filter(id=>!injected.some(folder=>folder.id===id));
  }
  await route.fulfill({response,json:body});
 });
 return state;
}
const sidebar=(page:Page,id:string)=>page.locator(`.sidebar [data-folder-id="${id}"]`);
async function foldersSettings(page:Page){
 await page.locator('#shell-nav').getByRole('button',{name:'Settings',exact:true}).click();
 await page.getByRole('navigation',{name:'Settings sections'}).locator('[data-settings-entry="folders"]').click();
 const panel=page.locator('[data-settings-section="folders"]');await expect(panel).toBeVisible();return panel;
}
async function propertiesFromManager(page:Page,id:string){
 const panel=await foldersSettings(page);const trigger=panel.locator(`tr[data-folder-id="${id}"]`).getByRole('button',{name:'More actions for Browser Spam',exact:true});
 await trigger.scrollIntoViewIfNeeded();
 await page.evaluate(()=>new Promise<void>(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolve()))));
 await trigger.click();
 await page.getByRole('menuitem',{name:'Properties…',exact:true}).click();
 const dialog=page.getByTestId('folder-properties');await expect(dialog).toBeVisible();
 await dialog.getByRole('tab',{name:'Display',exact:true}).click();
 return dialog.getByTestId('ovr-hideWhenEmpty');
}

test('user hides only empty folders, refresh restores nonempty folders, and management keeps all folders',async({page},info)=>{
 const state=await fixture(page);await login(page);
 await expect(sidebar(page,SPAM)).toBeVisible();await expect(sidebar(page,DRAFTS)).toBeVisible();
 const panel=await foldersSettings(page);const hide=panel.locator('#folders-hide-empty');await expect(hide).not.toBeChecked();await hide.check();
 await expect.poll(()=>state.settingsWrites).toEqual([true]);
 await expect(sidebar(page,SPAM)).toHaveCount(0);await expect(sidebar(page,DRAFTS)).toHaveCount(0);
 for(const id of [SPAM,DRAFTS,EMPTY_CHILD])await expect(panel.locator(`tr[data-folder-id="${id}"]`)).toBeVisible();
 await expect(sidebar(page,READ)).toBeVisible();await expect(sidebar(page,PARENT)).toBeVisible();await expect(sidebar(page,CHILD)).toBeVisible();await expect(sidebar(page,EMPTY_CHILD)).toHaveCount(0);
 await page.getByRole('button',{name:'Close Settings',exact:true}).click();
 state.spamCount=2;await page.getByRole('button',{name:'Refresh folder',exact:true}).click();await expect(sidebar(page,SPAM)).toBeVisible();
 state.spamCount=0;await page.getByRole('button',{name:'Refresh folder',exact:true}).click();await expect(sidebar(page,SPAM)).toHaveCount(0);
 await page.screenshot({path:info.outputPath('hide-empty-sidebar.png')});
});

test('per-folder Off keeps an empty folder visible and resetting follows the user default',async({page})=>{
 const state=await fixture(page,{hide:true,override:false});await login(page);await expect(sidebar(page,SPAM)).toBeVisible();
 const row=await propertiesFromManager(page,SPAM);await expect(row.getByRole('radio',{name:'Off',exact:true})).toHaveAttribute('aria-checked','true');
 await expect(row.getByTestId('ovr-hideWhenEmpty-source')).toContainText('Overridden for this folder');
 await row.getByTestId('ovr-hideWhenEmpty-reset').click();await expect(sidebar(page,SPAM)).toHaveCount(0);
 await expect(row.getByRole('radio',{name:'Same as default',exact:true})).toHaveAttribute('aria-checked','true');
 await expect.poll(()=>state.prefs.folderProps?.[SPAM]?.hideWhenEmpty).toBeUndefined();
 await row.getByRole('radio',{name:'Off',exact:true}).click();await expect(sidebar(page,SPAM)).toBeVisible();
 await expect.poll(()=>state.prefs.folderProps?.[SPAM]?.hideWhenEmpty).toBe(false);
});

for(const value of [true,false])test(`administrator lock ${value} overrides stored opposite folder setting`,async({page})=>{
 const state=await fixture(page,{hide:value,locked:true,override:!value});await login(page);
 if(value)await expect(sidebar(page,SPAM)).toHaveCount(0);else await expect(sidebar(page,SPAM)).toBeVisible();
 const row=await propertiesFromManager(page,SPAM);await expect(row.getByTestId('ovr-hideWhenEmpty-source')).toContainText('Locked by your administrator');
 await expect(row.getByTestId('ovr-hideWhenEmpty-locked')).toContainText(value?'On':'Off');
 await expect(row.getByRole('radio')).toHaveCount(0);await expect(row.getByTestId('ovr-hideWhenEmpty-reset')).toHaveCount(0);
 await page.getByTestId('props-done').click();
 const hide=page.locator('#folders-hide-empty');await expect(hide).toBeDisabled();if(value)await expect(hide).toBeChecked();else await expect(hide).not.toBeChecked();
 expect(state.settingsWrites).toEqual([]);expect(state.preferenceWrites).toEqual([]);
});
