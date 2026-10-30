import {test,expect,ANONYMOUS_PAIR,type Page} from '../csrf.ts';
import {adminLogin} from '../admin/admin-login.ts';
import {ADMIN} from '../admin/admin-path.ts';
import {readFileSync} from 'node:fs';
import {injectViaSmtp,messageRow} from '../helpers.ts';

async function savePolicy(page:Page):Promise<void>{
 const saved=page.waitForResponse(response=>response.url().endsWith(`${ADMIN}/client-access`)&&response.request().method()==='PUT'&&response.status()===200);
 await page.getByRole('button',{name:'Save client access',exact:true}).click();
 const proof=page.getByRole('dialog',{name:'Confirm it is you'});
 const asked=await Promise.race([proof.waitFor({state:'visible',timeout:30000}).then(()=>true).catch(()=>false),saved.then(()=>false)]);
 if(asked){await proof.getByLabel('Password',{exact:true}).fill(process.env['MW_FIXTURE_ADMIN_PASSWORD']??'hunter2');await proof.getByRole('button',{name:'Confirm and continue'}).click();}
 await saved;await expect(page.getByRole('button',{name:'Save client access',exact:true})).toBeDisabled();
}
async function unlock(page:Page,key:string,signedIn:boolean,onUnlocked:()=>void):Promise<void>{
 for(let attempt=0;attempt<2;attempt++){
  await expect(page.getByRole('heading',{name:'Client access',exact:true})).toBeVisible();await page.getByLabel('Access key',{exact:true}).fill(key);
  const response=page.waitForResponse(value=>new URL(value.url()).pathname.endsWith('/api/client-access/unlock')&&value.request().method()==='POST');await page.getByRole('button',{name:'Continue',exact:true}).click();const result=await response;
  if(result.status()===429&&attempt===0){const seconds=Number(result.headers()['retry-after']);expect(Number.isFinite(seconds)).toBe(true);expect(seconds).toBeGreaterThan(0);expect(seconds).toBeLessThanOrEqual(60);await expect(page.getByLabel('Access key',{exact:true})).toBeVisible();await page.waitForTimeout(seconds*1000+150);continue;}
  expect(result.status()).toBe(303);onUnlocked();break;
 }
 if(signedIn){await expect(page.getByRole('tab',{name:'Mail',exact:true})).toBeVisible();await page.getByRole('tab',{name:'Mail',exact:true}).click();await expect(page.getByRole('searchbox',{name:'Search mail'})).toBeVisible();}else await expect(page.getByLabel('Email or username',{exact:true})).toBeVisible();
}
async function expectHiddenEntry(page:Page,slug:string):Promise<void>{
 const hidden=await page.goto('/');expect(hidden?.status()).toBe(404);
 await expect(page.locator('body')).toContainText('Not found');expect(await page.locator('body').innerText()).not.toContain(slug);
 const direct=await page.request.get('/');expect(direct.status()).toBe(404);expect(await direct.text()).not.toContain(slug);
}
test('client entry key and private address work independently and together without blocking admin recovery',async({page,browser,baseURL},info)=>{
 test.setTimeout(180000);await adminLogin(page);await page.goto(`${ADMIN}#/security/client-access`);await expect(page.getByTestId('client-entry-path')).toBeVisible();
 const initial=await page.request.get(`${ADMIN}/client-access`);expect(initial.ok()).toBe(true);const original=await initial.json();expect(original.pskConfigured).toBe(false);expect(original.pskEnabled).toBe(false);expect(original.slugEnabled).toBe(false);
 const key=`client-entry-${info.project.name}-secret-2026`;const slug=`private-${info.project.name}-${Date.now()}`;
 const context=await browser.newContext({baseURL:baseURL!,serviceWorkers:'block',viewport:info.project.name==='phone'?{width:390,height:844}:{width:1440,height:900}});const client=await context.newPage();
 let nativePrompt=true;const cspErrors:string[]=[];const allCspErrors:{text:string;url:string;nativePrompt:boolean}[]=[];
 const onUnlocked=():void=>{expect(cspErrors).toEqual([]);nativePrompt=false;};
 client.on('console',message=>{if(message.type()==='error'&&/Content Security Policy|Content-Security-Policy|violates.*directive|Refused to (?:apply|load).*style/i.test(message.text())){allCspErrors.push({text:message.text(),url:message.location().url,nativePrompt});if(nativePrompt)cspErrors.push(message.text());}});
 try{
  await page.getByLabel('Require a pre-shared key',{exact:true}).check();await page.getByLabel('New pre-shared key',{exact:true}).fill(key);await savePolicy(page);await expect(page.getByLabel('New pre-shared key',{exact:true})).toHaveValue('');
  const stored=await page.request.get(`${ADMIN}/client-access`);const storedText=await stored.text();expect(storedText).not.toContain(key);expect(JSON.parse(storedText)).not.toHaveProperty('psk');expect(JSON.parse(storedText)).not.toHaveProperty('pskHash');
  const denied=await context.request.post('/api/login',{headers:{...ANONYMOUS_PAIR,origin:new URL(baseURL!).origin},data:{username:'testuser',password:'testpass'}});expect(denied.status()).toBe(403);expect((await denied.json()).code).toBe('clientAccessRequired');
  await client.goto('/');await expect(client.getByLabel('Access key',{exact:true})).toBeVisible();
  expect(await client.evaluate(()=>document.styleSheets.length)).toBeGreaterThan(0);expect(await client.getByRole('button',{name:'Continue',exact:true}).evaluate(node=>node.getBoundingClientRect().height)).toBeGreaterThanOrEqual(32);expect(await client.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);expect(cspErrors).toEqual([]);await client.screenshot({path:info.outputPath('client-gate-prompt.png'),fullPage:true});
  await client.getByLabel('Access key',{exact:true}).fill('wrong-access-key');
  const wrong=client.waitForResponse(response=>new URL(response.url()).pathname.endsWith('/api/client-access/unlock')&&response.request().method()==='POST');await client.getByRole('button',{name:'Continue',exact:true}).click();expect((await wrong).status()).toBe(403);await expect(client.getByLabel('Email or username',{exact:true})).toHaveCount(0);
  await unlock(client,key,false,onUnlocked);expect((await context.request.get('/jmap/session')).status()).toBe(401);
  // The opaque message sandbox cannot rely on the gate cookie. Only validated
  // signed resources remain usable; minting new capabilities stays protected.
  const subject=`Gated signed image ${Date.now()}`;await injectViaSmtp({from:'gate-image@example.org',subject,text:'Gate image fixture',html:'<img alt="Gate image" width="192" height="192" src="cid:gate@fixture">',attachments:[{filename:'gate.png',contentType:'image/png',base64:readFileSync(new URL('../../public/icons/icon-192.png',import.meta.url)).toString('base64'),cid:'gate@fixture'}]});
  await client.getByLabel('Email or username',{exact:true}).fill('testuser');await client.getByLabel('Password',{exact:true}).fill('testpass');await client.getByRole('button',{name:'Sign in',exact:true}).click();await expect(messageRow(client,subject)).toBeVisible({timeout:60000});
  await messageRow(client,subject).click();
  const frame=client.locator('section.reader').filter({visible:true}).first().locator('iframe.reader__frame');await expect(frame).toBeVisible();const outer=frame.contentFrame();const body=(await frame.getAttribute('src'))?.includes('/api/rc/frame')?outer.frameLocator('iframe'):outer;
  const image=body.getByAltText('Gate image');await expect(image).toHaveJSProperty('naturalWidth',192,{timeout:60000});const signed=await image.getAttribute('src');expect(signed).toContain('/api/rc/p/');
  const anonymous=await browser.newContext({baseURL:baseURL!,serviceWorkers:'block'});try{expect((await anonymous.request.get(signed!)).status()).toBe(200);const mint=await anonymous.request.post('/api/rc/sign',{headers:{...ANONYMOUS_PAIR,origin:new URL(baseURL!).origin},data:{emailId:'untrusted',urls:['http://1.2.3.4/test.png']}});expect(mint.status()).toBe(403);expect((await mint.json()).code).toBe('clientAccessRequired');}finally{await anonymous.close();}
  await client.screenshot({path:info.outputPath('client-gate-signed-image.png'),fullPage:true});
  await page.getByLabel('Require a private client address',{exact:true}).check();await page.getByLabel('Client address segment',{exact:true}).fill(slug);await savePolicy(page);
  await expectHiddenEntry(client,slug);
  nativePrompt=true;await client.goto(`/${slug}`);await unlock(client,key,true,onUnlocked);await client.screenshot({path:info.outputPath('client-private-key-unlocked.png'),fullPage:true});
  await client.goto(`/${slug}`);await expect(client.getByRole('searchbox',{name:'Search mail'})).toBeVisible();await expect(client.getByLabel('Access key',{exact:true})).toHaveCount(0);
  const recovery=await browser.newContext({baseURL:baseURL!,serviceWorkers:'block'});try{const admin=await recovery.newPage();await admin.goto(ADMIN);await expect(admin.getByLabel('Admin username')).toBeVisible();}finally{await recovery.close();}
  await page.getByLabel('Require a pre-shared key',{exact:true}).uncheck();await savePolicy(page);
  const freshVisitor=await browser.newContext({baseURL:baseURL!,serviceWorkers:'block'});try{const visitor=await freshVisitor.newPage();await expectHiddenEntry(visitor,slug);await visitor.goto(`/${slug}`);await expect(visitor.getByLabel('Email or username',{exact:true})).toBeVisible();await expect(visitor.getByLabel('Access key',{exact:true})).toHaveCount(0);}finally{await freshVisitor.close();}
  await page.screenshot({path:info.outputPath('client-access-admin-controls.png'),fullPage:true});
  expect(cspErrors).toEqual([]);
 }finally{
  try{
  expect((await page.request.post(`${ADMIN}/step-up`,{data:{password:process.env['MW_FIXTURE_ADMIN_PASSWORD']??'hunter2'}})).ok()).toBe(true);
  const current=await page.request.get(`${ADMIN}/client-access`);expect(current.ok()).toBe(true);
  const restored=await page.request.put(`${ADMIN}/client-access`,{headers:{'if-match':current.headers()['etag']!},data:{pskEnabled:false,slugEnabled:false,slug:original.slug,grantHours:original.grantHours,clearPsk:true}});expect(restored.ok(),await restored.text()).toBe(true);
  }finally{await info.attach('client-access-csp-events',{body:JSON.stringify(allCspErrors,null,2),contentType:'application/json'});await context.close();}
 }
});
