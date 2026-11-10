import {test,expect,type Page} from '@playwright/test';
import {login,messageRow} from './helpers.ts';
test.use({serviceWorkers:'block'});test.setTimeout(90000);
// Drain fixture requests before Playwright disposes their response bodies.
test.afterEach(async({page})=>{await page.unrouteAll({behavior:'wait'});});
async function fixture(page:Page,locked=false){
 const values:Record<string,unknown>={'links.trustedSenders':[],'links.trustedSenderDomains':[],'mail.reader.unsafeHtmlMode':'sanitized','notifications.askPermission':false};
 const writes:{key:string;value:unknown}[]=[],grants:{scopeKind:string;scopeValue:string;grantedAt:string}[]=[];
 await page.route(/\/api\/settings(?:\/[^/?]+)?(?:\?.*)?$/,async route=>{
  const request=route.request();if(request.method()==='PUT'){
   const key=decodeURIComponent(new URL(request.url()).pathname.split('/').at(-1)!);if(locked&&key.startsWith('links.'))return route.fulfill({status:403,json:{code:'setting_locked'}});
   values[key]=request.postDataJSON().value;writes.push({key,value:values[key]});return route.fulfill({json:{value:values[key],locked:false,source:'user'}});
  }
  const response=await route.fetch(),body=await response.json() as {settings:Record<string,unknown>};
  for(const [key,value] of Object.entries(values))body.settings[key]={value,source:'global',locked:locked&&key.startsWith('links.')};await route.fulfill({response,json:body});
 });
 await page.route(/\/api\/remote-images\/(grants|grant|revoke)(?:\?.*)?$/,async route=>{
  if(route.request().method()==='GET')return route.fulfill({json:{list:grants}});
  const grant=route.request().postDataJSON();grants.push({...grant,grantedAt:new Date().toISOString()});await route.fulfill({json:{}});
 });
 await page.route(/\/api\/rc\/policy(?:\?.*)?$/,route=>route.fulfill({json:{mode:'ask',trackers:true,cleanLinks:true,allowDirect:false,inlineParts:true,locked:{mode:false,trackers:false,cleanLinks:false}}}));
 await page.route(/\/api\/rc\/sign$/,async route=>{
  const request=route.request().postDataJSON();await route.fulfill({json:{images:(request.urls??[]).map((url:string)=>({url,src:'/api/rc/i/controlled.signature'})),parts:[],files:[],refused:[]}});
 });
 await page.route(/\/api\/rc\/i\/controlled.signature$/,route=>route.fulfill({contentType:'image/svg+xml',headers:{'cross-origin-resource-policy':'cross-origin'},body:'<svg xmlns="http://www.w3.org/2000/svg" width="200" height="50"><rect width="200" height="50" fill="blue"/></svg>'}));
 await page.route(/\/jmap\/api$/,async route=>{
  if(!route.request().postDataJSON().methodCalls.some(([method]:[string])=>method==='Email/get'))return route.continue();
  const response=await route.fetch(),body=await response.json();
  for(const [method,args] of body.methodResponses??[])if(method==='Email/get')for(const email of args.list??[]){
   if(!['Lunch tomorrow?','Welcome to Mailwoman'].includes(email.subject))continue;
   email.from=[{name:'Sender',email:email.subject==='Lunch tomorrow?'?'sender@example.test':'other@example.test'}];
   email.htmlBody=[{partId:'trust-html',blobId:null,type:'text/html',size:200}];email.textBody=[];
   email.bodyValues={'trust-html':{value:'<p>Trust controls</p><a href="https://destination.example.test/page">Controlled link</a><img src="https://pictures.example.test/banner.png" width="200" height="50">',isEncodingProblem:false,isTruncated:false}};
  }await route.fulfill({response,json:body});
 });return {writes,grants};
}
const anchor=(page:Page)=>page.locator('section.reader:visible').frameLocator('iframe.reader__frame').frameLocator('iframe:visible').getByText('Controlled link',{exact:true});
test('sender and domain permissions stay independent and image grants can be promoted after Show images',async({page})=>{
 await page.setViewportSize({width:1440,height:1000});const {writes,grants}=await fixture(page);await login(page);await messageRow(page,'Lunch tomorrow?').click();await expect(anchor(page)).not.toHaveAttribute('href');
 await page.getByRole('button',{name:'Link options',exact:true}).click();await page.getByRole('menuitem',{name:'Always allow links from this sender',exact:true}).click();
 await expect.poll(()=>writes.at(-1)).toEqual({key:'links.trustedSenders',value:['sender@example.test']});
 await expect(page.getByText(/Links will be allowed in messages from .*sender@example\.test/)).toBeVisible();
 await page.getByRole('button',{name:'Link options',exact:true}).click();await page.getByRole('menuitem',{name:'Stop always allowing links from this sender',exact:true}).click();
 await expect.poll(()=>writes.at(-1)).toEqual({key:'links.trustedSenders',value:[]});
 await page.getByRole('button',{name:'Link options',exact:true}).click();await page.getByRole('menuitem',{name:'Always allow links from this sender domain',exact:true}).click();
 await expect.poll(()=>writes.at(-1)).toEqual({key:'links.trustedSenderDomains',value:['example.test']});expect(grants).toEqual([]);
 await messageRow(page,'Welcome to Mailwoman').click();await expect(anchor(page)).toHaveAttribute('href','https://destination.example.test/page');
 await page.getByRole('button',{name:'Show images',exact:true}).click();await expect.poll(()=>grants.at(-1)?.scopeKind).toBe('single');await expect(page.getByText('Image permission saved. Allowed images can now load.',{exact:true})).toBeVisible();
 await page.getByRole('button',{name:'More image options',exact:true}).click();await page.getByRole('menuitem',{name:/Always show images from .other@example\.test./}).click();
 await expect.poll(()=>grants.at(-1)?.scopeKind).toBe('per-sender');
 await page.getByRole('button',{name:'More image options',exact:true}).click();await page.getByRole('menuitem',{name:/Always show images from .example\.test.$/}).click();
 await expect.poll(()=>grants.at(-1)?.scopeKind).toBe('per-domain');expect(grants.at(-1)?.scopeValue).toBe('example.test');
});
test('administrator locks keep sender and domain link permissions read-only',async({page})=>{
 const {writes}=await fixture(page,true);await login(page);await messageRow(page,'Lunch tomorrow?').click();await page.getByRole('button',{name:'Link options',exact:true}).click();
 await expect(page.getByRole('menuitem',{name:'Always allow links from this sender',exact:true})).toHaveAttribute('aria-disabled','true');
 await expect(page.getByRole('menuitem',{name:'Always allow links from this sender domain',exact:true})).toHaveAttribute('aria-disabled','true');expect(writes).toEqual([]);
});
