import {test,expect} from '@playwright/test';
import {login,messageRow} from './helpers.ts';
test.use({serviceWorkers:'block'});test.setTimeout(90_000);

test('Microsoft tenant and MEO Empresas logos render locally in list, reader and linked mailbox provider cells',async({page},info)=>{
 const outside:string[]=[];
 page.on('request',request=>{if(request.resourceType()==='image'&&/^https?:/.test(request.url())&&new URL(request.url()).origin!=='http://localhost:8080')outside.push(request.url());});
 await page.addLocatorHandler(page.getByRole('dialog',{name:'Allow notifications?',exact:true}),async dialog=>{await dialog.getByRole('button',{name:'Not now',exact:true}).click();});
 const mapped=new Map<string,number>();
 const fixtures=[{address:'notice@tenant.onmicrosoft.com',id:'microsoft',subject:'Provider logo Microsoft'},{address:'billing@meoempresas.pt',id:'meo',subject:'Provider logo MEO'}];
 await page.route(/\/jmap(?:\/api)?$/,async route=>{
  if(route.request().method()!=='POST')return route.continue();
  const response=await route.fetch();const body=await response.json() as {methodResponses?:[string,{list?:Record<string,unknown>[]},string][]};
  for(const [method,args] of body.methodResponses??[])if(method==='Email/get')for(const email of args.list??[]){
   const id=String(email['id']);if(!mapped.has(id)&&mapped.size<fixtures.length)mapped.set(id,mapped.size);
   const index=mapped.get(id);if(index===undefined)continue;const f=fixtures[index]!;
   email['from']=[{name:f.id==='meo'?'MEO Empresas':'Microsoft tenant',email:f.address}];email['subject']=f.subject;email['mailwomanAuth']={dkim:{result:'pass',domain:f.address.split('@')[1]},dmarc:{result:'pass',aligned:true}};
  }
  await route.fulfill({response,json:body});
 });
 await page.route(/\/api\/accounts$/,async route=>{
  if(route.request().method()!=='GET')return route.continue();
  const response=await route.fetch();const body=await response.json() as {accounts:unknown[]};
  body.accounts=[...body.accounts,...fixtures.map(f=>({id:`logo-${f.id}`,accountId:`logo-${f.id}`,address:f.address,displayName:f.id==='meo'?'MEO Empresas':'Microsoft tenant',status:'disabled',revision:'1'}))];
  await route.fulfill({response,json:body});
 });
 await login(page);
 for(const f of fixtures){
  const row=messageRow(page,f.subject);const avatar=row.locator(`[data-sender-icon="${f.id}"]`);await expect(avatar).toBeVisible();
  if(f.id==='microsoft')await expect(avatar.locator('svg path')).toBeAttached();else await expect.poll(()=>avatar.locator('img').evaluate((img:HTMLImageElement)=>img.complete&&img.naturalWidth>0)).toBe(true);
  await row.click();const logo=page.locator(`.reader__address-line [data-brand="${f.id}"]`);await expect(logo).toBeVisible();
  if(f.id==='microsoft')await expect(logo.locator('path')).toBeAttached();else await expect(logo.locator('image')).toHaveAttribute('href',/^(?:data:image\/png;|.*meo-site)/);
 }
 await page.locator('#shell-nav').getByRole('button',{name:'Settings',exact:true}).click();
 await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Linked mailboxes',exact:true}).click();
 for(const f of fixtures){
  const row=page.locator('[data-linked-row]').filter({hasText:f.address});const logo=row.locator(`[data-brand="${f.id}"]`);await expect(logo).toBeVisible();
  if(f.id==='microsoft')await expect(logo.locator('path')).toBeAttached();else await expect(logo.locator('image')).toHaveAttribute('href',/^(?:data:image\/png;|.*meo-site)/);
 }
 await page.screenshot({path:info.outputPath('provider-logos-linked-mailboxes.png')});expect(outside).toEqual([]);
});
