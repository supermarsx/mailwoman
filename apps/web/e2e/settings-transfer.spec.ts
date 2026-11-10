import {test,expect} from '@playwright/test';
import {login,settingsSurface} from './helpers.ts';
test.use({serviceWorkers:'block'});test.setTimeout(90000);
for(const width of [1280,390])test(`preferences export/import and ribbon configuration fit ${width}px`,async({page},info)=>{
 await page.setViewportSize({width:1280,height:900});
 const entries:Record<string,{value:unknown;userValue?:unknown;source:string;locked:boolean}>={
  'reader.fontSize':{value:14,userValue:14,source:'user',locked:false},'reader.zoom':{value:100,source:'global',locked:true},
  'reader.actions.reply':{value:'toolbar',userValue:'toolbar',source:'user',locked:false},
  'reader.actions.reply-all':{value:'toolbar',source:'global',locked:false},
  'notifications.askPermission':{value:false,source:'global',locked:true},
 };
 const writes:{key:string;value:unknown}[]=[];
 await page.route(/\/api\/settings(?:\/[^/?]+)?(?:\?.*)?$/,async route=>{
  const request=route.request();if(request.method()==='PUT'){
   const key=decodeURIComponent(new URL(request.url()).pathname.split('/').at(-1)!);const value=request.postDataJSON().value;
   if(entries[key]?.locked)return route.fulfill({status:403,json:{code:'setting_locked'}});
   writes.push({key,value});entries[key]={value,userValue:value,source:'user',locked:false};return route.fulfill({json:entries[key]});
  }
  await route.fulfill({json:{settings:entries}});
 });
 await page.addLocatorHandler(page.getByRole('dialog',{name:'Allow notifications?',exact:true}),async dialog=>{await dialog.getByRole('button',{name:'Not now',exact:true}).click();});
 await login(page);await page.getByRole('button',{name:'Settings',exact:true}).click();
 const screen=settingsSurface(page);await expect(screen).toBeVisible();
 const sections=screen.getByRole('navigation',{name:'Settings sections'});
 await sections.getByRole('button',{name:'Appearance',exact:true}).click();
 if(width===390)await page.setViewportSize({width,height:844});
 const transfer=screen.getByTestId('settings-transfer');await transfer.scrollIntoViewIfNeeded();
 await transfer.getByRole('button',{name:'Export preferences',exact:true}).click();
 await transfer.getByRole('button',{name:'Review export',exact:true}).click();
 const [download]=await Promise.all([page.waitForEvent('download'),transfer.getByRole('button',{name:/Download preferences/}).click()]);
 expect(download.suggestedFilename()).toMatch(/^mailwoman-settings-.*\.json$/);
 await transfer.getByRole('button',{name:'Import preferences',exact:true}).click();
 const backup={format:'mailwoman-user-settings',version:1,createdAt:new Date().toISOString(),source:{accountId:'foreign-account',origin:'https://other.invalid'},items:[
  {kind:'setting',key:'reader.fontSize',value:18},{kind:'setting',key:'reader.zoom',value:125},{kind:'interface',key:'folderHidden',value:['private-folder']},
 ]};
 await transfer.getByLabel('Choose a Mailwoman preferences file').setInputFiles({name:'settings.json',mimeType:'application/json',buffer:Buffer.from(JSON.stringify(backup))});
 await expect(transfer.getByText('Locked by your administrator',{exact:true})).toBeVisible();
 await expect(transfer.getByText('Requires the original account and server',{exact:true})).toBeVisible();
 expect(writes).toEqual([]);await transfer.getByRole('button',{name:'Apply selected changes (1)',exact:true}).click();
 await expect(transfer.getByRole('status')).toContainText('Saved: 1');expect(writes).toEqual([{key:'reader.fontSize',value:18}]);
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 await transfer.screenshot({path:info.outputPath(`settings-transfer-${width}.png`)});
 if(width===390)await page.setViewportSize({width:1280,height:900});
 await sections.getByRole('button',{name:'Message display',exact:true}).click();
 const ribbon=screen.getByTestId('reader-action-preferences');await ribbon.getByRole('searchbox',{name:'Find a ribbon action'}).fill('Reply');
 const reply=ribbon.getByRole('combobox',{name:'Reply',exact:true});await reply.selectOption('menu');
 await expect.poll(()=>writes.at(-1)).toEqual({key:'reader.actions.reply',value:'menu'});
 await ribbon.getByText('Preview your actions ribbon',{exact:true}).click();
 const preview=ribbon.getByRole('list',{name:'Preview your actions ribbon'});
 await expect(preview.getByText('Reply',{exact:true})).toHaveCount(0);
 await expect(preview.getByText('More menu',{exact:true})).toBeVisible();
 await page.unrouteAll({behavior:'wait'});
});
