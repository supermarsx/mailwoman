import {test,expect,type Page} from '@playwright/test';
import {login,messageRow,sidebarInbox} from './helpers.ts';
test.use({serviceWorkers:'block'});test.setTimeout(90000);
const name='wide-image.svg',blob='preview-wide-image';
async function fixture(page:Page,thumbnails=false){
 let downloads=0;
 await page.route(/\/api\/settings(?:\?.*)?$/,async route=>{
  const response=await route.fetch(),body=await response.json();
  for(const [key,value] of Object.entries({'reader.attachmentThumbnails':thumbnails,'reader.attachmentActionStyle':'icons','reader.attachmentsCollapsed':false,'notifications.askPermission':false}))body.settings[key]={value,source:'global',locked:false};
  await route.fulfill({response,json:body});
 });
 await page.route(/\/jmap\/download\//,async route=>{
  if(!new URL(route.request().url()).pathname.includes(`/${blob}/`))return route.continue();
  downloads++;await route.fulfill({contentType:'image/svg+xml',body:'<svg xmlns="http://www.w3.org/2000/svg" width="2000" height="1000"><rect width="2000" height="1000" fill="royalblue"/></svg>'});
 });
 await page.route(/\/jmap(?:\/api)?$/,async route=>{
  if(route.request().method()!=='POST')return route.continue();
  const response=await route.fetch(),body=await response.json();
  for(const [method,args] of body.methodResponses??[])if(method==='Email/get')for(const email of args.list??[])if(email.subject==='Lunch tomorrow?')email.attachments=[{partId:'preview',blobId:blob,type:'image/svg+xml',name,size:200,disposition:'attachment'}];
  await route.fulfill({response,json:body});
 });
 return ()=>downloads;
}
test('image preview fits, zooms, rotates and keeps compact actions beside the title',async({page},info)=>{
 await page.setViewportSize({width:1200,height:800});const downloads=await fixture(page);await login(page);await messageRow(page,'Lunch tomorrow?').click();
 const option=page.getByRole('option').filter({hasText:name});await expect(option).toBeVisible();expect(downloads()).toBe(0);await expect(option.locator('img')).toHaveCount(0);await option.click();
 const pane=page.getByRole('tabpanel',{name,exact:true});await expect(pane.getByRole('button',{name:'Zoom in',exact:true})).toBeEnabled();
 const header=pane.locator('.attachment-tab__header'),download=header.getByRole('link',{name:'Download file',exact:true}),details=header.getByRole('button',{name:'File details',exact:true});
 await expect(download).toBeVisible();await expect(details).toBeVisible();await expect(download).toHaveText('');await expect(details).toHaveText('');
 const titleBox=await header.locator('h2').boundingBox(),actionsBox=await download.boundingBox();expect(Math.abs(titleBox!.y-actionsBox!.y)).toBeLessThan(15);
 const viewport=pane.locator('.image-preview__viewport'),frame=pane.locator('iframe.mw-viewer__frame--image');
 await expect.poll(()=>frame.contentFrame().locator('img').evaluate(img=>{const box=img.getBoundingClientRect();return box.x===0&&box.y===0&&Math.abs(box.width-innerWidth)<=1&&Math.abs(box.height-innerHeight)<=1;})).toBe(true);
 await expect.poll(()=>viewport.evaluate(el=>el.scrollWidth<=el.clientWidth+1&&el.scrollHeight<=el.clientHeight+1)).toBe(true);
 await pane.getByRole('button',{name:'Actual size (100%)',exact:true}).click();await expect(pane.getByLabel('Zoom level')).toHaveText('100%');
 expect(await viewport.evaluate(el=>el.scrollWidth>el.clientWidth)).toBe(true);
 await pane.getByRole('button',{name:'Zoom in',exact:true}).click();await expect(pane.getByLabel('Zoom level')).toHaveText('125%');
 await pane.getByRole('button',{name:'Rotate right',exact:true}).click();await expect(frame).toHaveCSS('transform',/matrix/);
 await pane.getByRole('button',{name:'Fit image to view',exact:true}).click();await expect.poll(()=>viewport.evaluate(el=>el.scrollWidth<=el.clientWidth+1&&el.scrollHeight<=el.clientHeight+1)).toBe(true);
 await page.setViewportSize({width:800,height:650});await expect.poll(()=>viewport.evaluate(el=>el.scrollWidth<=el.clientWidth+1&&el.scrollHeight<=el.clientHeight+1)).toBe(true);
 await expect(frame).toHaveAttribute('sandbox','');expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
 await page.screenshot({path:info.outputPath('image-preview-fit.png')});
});
test('thumbnail opt-in loads previews and folder tabs use the shared message row layout',async({page})=>{
 await page.setViewportSize({width:1200,height:900});const downloads=await fixture(page,true);await login(page);await messageRow(page,'Lunch tomorrow?').click();
 const option=page.getByRole('option').filter({hasText:name});await expect(option.locator('img')).toBeVisible();expect(downloads()).toBeGreaterThan(0);
 await sidebarInbox(page).click({button:'right'});await page.getByRole('menuitem',{name:'Open in new tab',exact:true}).click();
 const pane=page.getByRole('tabpanel',{name:'Inbox',exact:true}),row=pane.locator('[data-folder-message]').filter({hasText:'Lunch tomorrow?'});
 await expect(row.locator('.list__sender')).toBeVisible();await expect(row.locator('.list__date')).toBeVisible();await expect(row.locator('.list__subject')).toBeVisible();
 await expect(pane.getByRole('checkbox').first()).toBeVisible();await pane.getByRole('checkbox').first().check();await expect(pane.getByRole('checkbox').first()).toBeChecked();
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
});
