import {test,expect} from '../csrf.ts';
import {pinComposerRibbonOnPhone} from '../helpers.ts';

// 9dab0fe7 (D4): below 600 px the ribbon starts in auto-hide; these legs are about
// what its controls do, so the phone project pins it for the test (helpers.ts).
let unpinRibbon:()=>Promise<void>=async()=>{};
test.beforeEach(async({playwright,baseURL},info)=>{unpinRibbon=await pinComposerRibbonOnPhone(playwright.request,baseURL,info.project.name);});
test.afterEach(async()=>{await unpinRibbon();});
test('account item picker is a bounded modal and cancellation preserves the draft',async({page},info)=>{
 await page.route('**/*',async route=>{const url=new URL(route.request().url());
  // The shared native fixture predates account-wide Email/query. Control only
  // this picker's synthetic list/export; this is UI proof, not upstream export.
  if(url.pathname==='/jmap/api'&&route.request().method()==='POST'){
   const request=route.request().postDataJSON() as {methodCalls:Array<[string,{accountId:string},string]>};const call=request.methodCalls[0];
   if(call?.[2]==='item'&&(call[0]==='Email/query'||call[0]==='Email/get'))return route.fulfill({json:{methodResponses:[[call[0],call[0]==='Email/query'?{accountId:call[1].accountId,ids:['picker-fixture-message'],total:1}:{accountId:call[1].accountId,list:[{id:'picker-fixture-message',subject:'Picker fixture message'}]},'item']]}});
  }
  if(url.pathname==='/api/export/picker-fixture-message')return route.fulfill({contentType:'message/rfc822',body:'From: fixture@example.test\r\nSubject: Picker fixture message\r\n\r\nFixture attachment.\r\n'});
  return route.continue();});
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();await page.locator('#shell-nav').getByRole('button',{name:'Compose',exact:true}).click();
 const compose=page.locator('.workspace-compose').last();await compose.getByLabel('Subject',{exact:true}).fill('Picker cancellation proof');await compose.getByRole('tab',{name:'Insert',exact:true}).click();
 // A narrow ribbon folds its Attachments group into "More Attachments commands";
 // the folded copy of the button is the one on screen then.
 const more=compose.getByRole('button',{name:'More Attachments commands',exact:true});
 // Folding is measured after the tab is shown. Do not sample isVisible before
 // that layout pass and then wait forever for a control hidden by the fold.
 if(info.project.name==='phone'){await expect(more).toBeVisible();await more.click();await expect(more).toHaveAttribute('aria-expanded','true');}
 else if(await more.isVisible())await more.click();
 const trigger=page.getByRole('button',{name:'Insert account item',exact:true}).filter({visible:true}).last();await trigger.click();
 const dialog=page.getByRole('dialog',{name:'Insert account item',exact:true});await expect(dialog).toBeVisible();await expect(dialog).toHaveAttribute('aria-modal','true');
 await expect(dialog.getByLabel('Item type')).toBeEnabled();await expect(dialog.getByRole('alert')).toHaveCount(0);await expect.poll(()=>dialog.getByLabel('Select an item').locator('option').count()).toBeGreaterThan(0);
 const geometry=await dialog.evaluate(node=>{const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:innerWidth,height:innerHeight,overflow:document.documentElement.scrollWidth-innerWidth};});
 expect(geometry.left).toBeGreaterThanOrEqual(0);expect(geometry.right).toBeLessThanOrEqual(geometry.width);expect(geometry.top).toBeGreaterThanOrEqual(0);expect(geometry.bottom).toBeLessThanOrEqual(geometry.height);expect(geometry.overflow).toBeLessThanOrEqual(1);
 const returned=info.project.name==='phone'?more:trigger;
 const reopen=async()=>{if(info.project.name==='phone')await more.click();await trigger.click();await expect(dialog).toBeVisible();};
 await page.screenshot({path:info.outputPath('account-item-modal.png'),fullPage:true});await dialog.getByRole('button',{name:'Cancel',exact:true}).click();await expect(dialog).toHaveCount(0);await expect(returned).toBeFocused();await expect(compose.getByLabel('Subject',{exact:true})).toHaveValue('Picker cancellation proof');
 await reopen();await page.keyboard.press('Escape');await expect(dialog).toHaveCount(0);await expect(returned).toBeFocused();
 await reopen();await dialog.getByLabel('Select an item').selectOption('picker-fixture-message');await dialog.getByRole('button',{name:'Attach selected item',exact:true}).click();await expect(dialog).toHaveCount(0);await expect(compose.getByText('Picker fixture message.eml',{exact:true})).toBeVisible();
});
