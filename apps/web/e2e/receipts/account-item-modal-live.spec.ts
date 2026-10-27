import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {test,expect} from '../csrf.ts';
test('account item picker is a bounded modal and cancellation preserves the draft',async({page},info)=>{
 const dist=path.resolve(process.env['MW_PICKER_DIST']??'../../.orchestration/reports/codex-account-picker-web');
 await page.route('**/*',async route=>{const url=new URL(route.request().url());
  // The shared native fixture predates account-wide Email/query. Control only
  // this picker's synthetic list/export; this is UI proof, not upstream export.
  if(url.pathname==='/jmap/api'&&route.request().method()==='POST'){
   const request=route.request().postDataJSON() as {methodCalls:Array<[string,{accountId:string},string]>};const call=request.methodCalls[0];
   if(call?.[2]==='item'&&(call[0]==='Email/query'||call[0]==='Email/get'))return route.fulfill({json:{methodResponses:[[call[0],call[0]==='Email/query'?{accountId:call[1].accountId,ids:['picker-fixture-message'],total:1}:{accountId:call[1].accountId,list:[{id:'picker-fixture-message',subject:'Picker fixture message'}]},'item']]}});
  }
  if(url.pathname==='/api/export/picker-fixture-message')return route.fulfill({contentType:'message/rfc822',body:'From: fixture@example.test\r\nSubject: Picker fixture message\r\n\r\nFixture attachment.\r\n'});
  if(url.pathname!=='/'&&!url.pathname.startsWith('/assets/'))return route.continue();const file=path.resolve(dist,url.pathname==='/'?'index.html':url.pathname.slice(1));if(!file.startsWith(dist+path.sep))return route.abort();const body=await readFile(file);const response=await route.fetch();await route.fulfill({response,status:200,body,contentType:file.endsWith('.html')?'text/html':file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':'application/octet-stream'});});
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();await page.locator('#shell-nav').getByRole('button',{name:'Compose',exact:true}).click();
 const compose=page.locator('.workspace-compose').last();await compose.getByLabel('Subject',{exact:true}).fill('Picker cancellation proof');await compose.getByRole('tab',{name:'Insert',exact:true}).click();
 const trigger=compose.getByRole('button',{name:'Insert account item',exact:true});await trigger.click();
 const dialog=page.getByRole('dialog',{name:'Insert account item',exact:true});await expect(dialog).toBeVisible();await expect(dialog).toHaveAttribute('aria-modal','true');
 await expect(dialog.getByLabel('Item type')).toBeEnabled();await expect(dialog.getByRole('alert')).toHaveCount(0);await expect.poll(()=>dialog.getByLabel('Select an item').locator('option').count()).toBeGreaterThan(0);
 const geometry=await dialog.evaluate(node=>{const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:innerWidth,height:innerHeight,overflow:document.documentElement.scrollWidth-innerWidth};});
 expect(geometry.left).toBeGreaterThanOrEqual(0);expect(geometry.right).toBeLessThanOrEqual(geometry.width);expect(geometry.top).toBeGreaterThanOrEqual(0);expect(geometry.bottom).toBeLessThanOrEqual(geometry.height);expect(geometry.overflow).toBeLessThanOrEqual(1);
 await page.screenshot({path:info.outputPath('account-item-modal.png'),fullPage:true});await dialog.getByRole('button',{name:'Cancel',exact:true}).click();await expect(dialog).toHaveCount(0);await expect(trigger).toBeFocused();await expect(compose.getByLabel('Subject',{exact:true})).toHaveValue('Picker cancellation proof');
 await trigger.click();await expect(dialog).toBeVisible();await page.keyboard.press('Escape');await expect(dialog).toHaveCount(0);await expect(trigger).toBeFocused();
 await trigger.click();await dialog.getByLabel('Select an item').selectOption('picker-fixture-message');await dialog.getByRole('button',{name:'Attach selected item',exact:true}).click();await expect(dialog).toHaveCount(0);await expect(compose.getByText('Picker fixture message.eml',{exact:true})).toBeVisible();
});
