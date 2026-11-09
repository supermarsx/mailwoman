// GET-only effective settings and Email/get fixtures against the standing mock
// stack. Destinations stay intercepted at the context before any page opens.
import {test,expect,type Page} from '@playwright/test';
import {login,messageRow} from './helpers.ts';
test.use({serviceWorkers:'block'});test.setTimeout(90_000);
async function fixture(page:Page,html:string){
 await page.addLocatorHandler(page.getByRole('dialog',{name:'Allow notifications?',exact:true}),async dialog=>{await dialog.getByRole('button',{name:'Not now',exact:true}).click();});
 await page.route(/\/api\/settings(?:\?.*)?$/,async route=>{
  const response=await route.fetch();const body=await response.json() as {settings:Record<string,unknown>};
  for(const [key,value]of Object.entries({'mail.reader.allowOriginalHtml':true,'cap.reading.original_html':true,'mail.reader.unsafeHtmlMode':'sanitized','links.trustedDomains':[],'links.trustedSubdomains':false,'reader.loading':'skeleton','reader.loadingDelayMs':0}))body.settings[key]={value,source:'global',locked:false};
  await route.fulfill({response,json:body});
 });
 await page.route(/\/api\/features(?:\?.*)?$/,async route=>{
  const response=await route.fetch();const body=await response.json() as {features:Record<string,unknown>};body.features['reading.original_html']={enabled:true,limits:{},settings:{}};await route.fulfill({response,json:body});
 });
 const ids=new Set<string>();
 await page.route(/\/jmap(?:\/api)?$/,async route=>{
  if(route.request().method()!=='POST')return route.continue();
  const request=route.request().postDataJSON() as {methodCalls?:[string,unknown,string][]};
  if(!request.methodCalls?.some(([method])=>method==='Email/get'))return route.continue();
  const response=await route.fetch();const body=await response.json() as {methodResponses?:[string,{list?:Record<string,unknown>[]},string][]};
  for(const[method,args]of body.methodResponses??[])if(method==='Email/get')for(const email of args.list??[]){
   if(email['subject']==='Lunch tomorrow?')ids.add(String(email['id']));if(!ids.has(String(email['id'])))continue;
   email['htmlBody']=[{partId:'browser-html',type:'text/html'}];email['textBody']=[];email['bodyValues']={'browser-html':{value:html,isEncodingProblem:false,isTruncated:false}};
  }
  await route.fulfill({response,json:body});
 });
}
async function body(page:Page){
 const reader=page.locator('section.reader').filter({visible:true}).first();const outer=reader.frameLocator('iframe.reader__frame');
 await expect(outer.locator('body')).toBeVisible();
 return(await reader.locator('iframe.reader__frame').getAttribute('src'))?.includes('/api/rc/frame')?outer.frameLocator('iframe:visible'):outer;
}
test('safe and Original HTML retain ordinary link destinations, reviewed opening and outside-dismiss',async({page,context,baseURL},info)=>{
 const rawDestination=new URL('/browser-link-destination',baseURL!).href+'?name=Jane Doe&query=a b&literal=a+b';
 const destination=new URL(rawDestination).href;
 let visited=0,headers:Record<string,string>|undefined;
 await context.route(destination,async route=>{visited++;headers=route.request().headers();await route.fulfill({contentType:'text/html',body:'<!doctype html><title>Controlled link destination</title><p>Opened through review</p>'});});
 const html=`<!doctype html><html><head><style>.original-grid{display:grid;grid-template-columns:1fr 1fr;gap:12px}</style></head><body><div class="original-grid"><p>Original layout marker</p><a href="${rawDestination.replaceAll('&','&amp;')}"><strong>Invoice</strong> details</a></div></body></html>`;
 await fixture(page,html);await login(page);await messageRow(page,'Lunch tomorrow?').click();
 for(const mode of ['safe','original']as const){
  if(mode==='original')await page.getByRole('button',{name:'Show Original HTML',exact:true}).click();
  const frame=await body(page);await expect(frame.getByText('Original layout marker',{exact:true})).toBeVisible();
  const anchor=frame.getByText('Invoice details',{exact:true});await expect(anchor).toBeVisible();await expect(anchor).not.toHaveAttribute('href');
  if(mode==='original')await expect(frame.locator('.original-grid')).toHaveCSS('display','grid');
  const trigger=page.getByTestId('reader-links');await expect(trigger).toBeVisible();await trigger.click();
  let dialog=page.getByRole('dialog',{name:'Message links',exact:true});await expect(dialog).toBeVisible();
  await dialog.getByRole('button',{name:'Invoice details',exact:true}).click();await expect(dialog).toBeVisible();
  const open=dialog.getByRole('link',{name:'Open in a new tab',exact:true});await expect(open).toHaveAttribute('href',destination);
  await dialog.getByRole('heading',{name:'Destination to open',exact:true}).click();await expect(dialog).toBeVisible();
  // A genuine pointer click in the scrim, outside the centered popup box.
  await dialog.click({position:{x:3,y:3}});await expect(dialog).toHaveCount(0);await expect(trigger).toBeFocused();expect(visited).toBe(mode==='safe'?0:1);
  await trigger.click();dialog=page.getByRole('dialog',{name:'Message links',exact:true});await dialog.getByRole('button',{name:'Invoice details',exact:true}).click();
  const opened=context.waitForEvent('page');await dialog.getByRole('link',{name:'Open in a new tab',exact:true}).click();const popup=await opened;await expect(popup).toHaveTitle('Controlled link destination');expect(await popup.evaluate(()=>window.opener)).toBeNull();expect(headers?.['referer']).toBeUndefined();await popup.close();
  await page.keyboard.press('Escape');await expect(dialog).toHaveCount(0);await expect(trigger).toBeFocused();
 }
 expect(visited).toBe(2);await page.screenshot({path:info.outputPath('original-html-links-review.png')});
});

for(const width of [1280,390])test(`real reader Loading label has inline margins at ${width}px`,async({page},info)=>{
 await fixture(page,'<p>Loader finished marker</p>');await login(page);
 if(width===390)await page.setViewportSize({width,height:844});
 let release!:()=>void;const held=new Promise<void>(resolve=>{release=resolve;});let reached=0;
 const policy=/\/api\/rc\/policy(?:\?.*)?$/;
 const handler=async(route:import('@playwright/test').Route)=>{reached++;await held;await route.continue();};
 await page.route(policy,handler);
 try{
  await messageRow(page,'Lunch tomorrow?').click();await expect.poll(()=>reached).toBeGreaterThan(0);
  const label=page.locator('section.reader:visible .inline-loading[data-visible="true"]').first();await expect(label).toBeVisible();await expect(label).toHaveText('Loading…');
  const geometry=await label.evaluate(element=>{const style=getComputedStyle(element),box=element.getBoundingClientRect();return {marginLeft:parseFloat(style.marginLeft),marginRight:parseFloat(style.marginRight),marginTop:parseFloat(style.marginTop),position:style.position,width:box.width,height:box.height,textWidth:element.lastElementChild!.getBoundingClientRect().width};});
  expect(geometry.marginLeft).toBe(16);expect(geometry.marginRight).toBe(16);expect(geometry.marginTop).toBe(8);expect(geometry.position).not.toBe('fixed');expect(geometry.textWidth).toBeLessThan(180);expect(geometry.height).toBeLessThan(70);
  await expect(page.locator('section.reader:visible .reader__toolbar')).toBeVisible();
  expect(await page.locator('section.reader:visible').evaluate(element=>element.scrollWidth<=element.clientWidth+1)).toBe(true);
  await page.screenshot({path:info.outputPath(`reader-loading-${width}.png`)});
 }finally{release();await page.unroute(policy,handler);}
 const frame=await body(page);await expect(frame.getByText('Loader finished marker',{exact:true})).toBeVisible();
 await expect(page.locator('section.reader:visible .inline-loading[data-visible="true"]')).toHaveCount(0);
});

for(const count of [3,60])test(`link review adapts to ${count} links without overflowing the viewport`,async({page})=>{
 await page.setViewportSize({width:1280,height:1000});
 const html=Array.from({length:count},(_,index)=>`<p><a href="https://example.org/invoice-${index}?details=${'a'.repeat(1600)}">Invoice link ${index}</a></p>`).join('');
 await fixture(page,html);await login(page);await messageRow(page,'Lunch tomorrow?').click();await page.getByTestId('reader-links').click();
 const dialog=page.getByRole('dialog',{name:'Message links',exact:true}),panel=dialog.locator('section.message-links'),close=dialog.getByRole('button',{name:'Close',exact:true});
 await expect(dialog).toBeVisible();await expect(panel.locator('.message-links__row')).toHaveCount(count);
 const large=await panel.boundingBox();expect(large).not.toBeNull();expect(large!.y).toBeGreaterThanOrEqual(0);expect(large!.y+large!.height).toBeLessThanOrEqual(1000);
 if(count===60)expect(large!.height).toBeGreaterThan(600);else expect(large!.height).toBeLessThan(500);
 await page.setViewportSize({width:1280,height:450});
 await expect.poll(async()=>{const box=await panel.boundingBox();return !!box&&box.x>=0&&box.y>=0&&box.x+box.width<=1280&&box.y+box.height<=450;}).toBe(true);
 await panel.evaluate(element=>{element.scrollTop=element.scrollHeight;});await expect(close).toBeInViewport();
 await panel.getByRole('button',{name:`Invoice link ${count-1}`,exact:true}).click();
 const open=dialog.getByRole('link',{name:'Open in a new tab',exact:true});await open.scrollIntoViewIfNeeded();await expect(open).toBeInViewport();await expect(close).toBeInViewport();
 const overflow=await panel.evaluate(element=>element.scrollWidth-element.clientWidth);expect(overflow).toBeLessThanOrEqual(1);
 await page.keyboard.press('Escape');await expect(dialog).toHaveCount(0);await expect(page.getByTestId('reader-links')).toBeFocused();
 // The compact layout remounts the reader; open its dialog afresh at phone width.
 await page.setViewportSize({width:390,height:450});await messageRow(page,'Lunch tomorrow?').click();await page.getByTestId('reader-links').click();
 await expect(dialog).toBeVisible();const phone=await panel.boundingBox();expect(phone).not.toBeNull();expect(phone!.x).toBeGreaterThanOrEqual(0);expect(phone!.y).toBeGreaterThanOrEqual(0);expect(phone!.x+phone!.width).toBeLessThanOrEqual(390);expect(phone!.y+phone!.height).toBeLessThanOrEqual(450);
 await page.keyboard.press('Escape');await expect(dialog).toHaveCount(0);await page.unrouteAll({behavior:'wait'});
});
