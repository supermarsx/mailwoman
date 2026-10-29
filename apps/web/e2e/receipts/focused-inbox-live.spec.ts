// t73: the Focused inbox against a live server built from the tree.
// `mail.focusedInbox` is off by default and has no switch in the Inbox header;
// the user turns it on in Settings > Folders and sidebar, and the Inbox then
// shows one slim Focused / Other pair with counts. It persists across a
// reload. In the admin console an administrator makes it the deployment
// default (a fresh user then sees the tabs) and locks it off (the user's
// Settings control is disabled and the tabs go).
// Runs at 1440 and 390 wide in the chromium and firefox projects (the phone
// project is the 390 Chromium leg already). Screenshots go to MW_T73_SHOTS.
import {test,expect} from '../csrf.ts';
import type {Browser,Page,TestInfo} from '@playwright/test';
import net from 'node:net';
import path from 'node:path';

const smtpPort=Number(process.env['MW_E2E_SMTP_PORT']??42225);
const KEY='mail.focusedInbox';
const run=Date.now().toString(36);
const fixtures=[
 {from:'Ana Person <ana@example.net>',subject:`Focused person ${run}`},
 {from:'Shop News <newsletter@shop.example>',subject:`Focused newsletter ${run}`},
];
function deliver(to:string,m:{from:string;subject:string},n:number):Promise<void>{
 return new Promise((resolve,reject)=>{
  const socket=net.connect(smtpPort,'127.0.0.1');
  const addr=/<(.+)>/.exec(m.from)![1]!;
  const data=`From: ${m.from}\r\nTo: ${to}\r\nSubject: ${m.subject}\r\nDate: ${new Date(Date.now()+n*1000).toUTCString().replace('GMT','+0000')}\r\nMessage-ID: <t73-${run}-${n}-${to}>\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nFocused inbox fixture.\r\n.\r\n`;
  const steps=['EHLO t73\r\n',`MAIL FROM:<${addr}>\r\n`,`RCPT TO:<${to}>\r\n`,'DATA\r\n',data,'QUIT\r\n'];
  let i=0,buffer='';
  socket.on('data',chunk=>{buffer+=chunk.toString();if(!buffer.endsWith('\r\n'))return;const last=buffer.trim().split('\r\n').at(-1)!;if(/^\d{3}-/.test(last))return;buffer='';if(/^[45]/.test(last)){socket.destroy();reject(new Error(last));return;}if(i<steps.length)socket.write(steps[i++]!);});
  socket.on('close',()=>resolve());socket.on('error',reject);
 });
}
test.describe.configure({timeout:300000});
let seeded=false;
test.beforeEach(async({},info)=>{
 test.skip(info.project.name==='phone','the 390 legs run in the chromium and firefox projects');
 if(!seeded){for(const [n,m] of fixtures.entries())await deliver('testuser@example.org',m,n);seeded=true;await new Promise(r=>setTimeout(r,3100));}
});

const shot=(info:TestInfo,name:string)=>{const dir=process.env['MW_T73_SHOTS'];return dir?path.join(dir,`${info.project.name}-${name}.png`):info.outputPath(`${name}.png`);};
async function signIn(page:Page,user='testuser',password='testpass'):Promise<void>{
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill(user);await page.getByLabel('Password',{exact:true}).fill(password);await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByLabel('Email or username',{exact:true})).toHaveCount(0,{timeout:60000});
 await expect(page.locator('.mail-pane')).toBeVisible({timeout:60000});
}
async function headers(page:Page):Promise<Record<string,string>>{const me=await (await page.request.get('/api/me')).json() as {storageAuthority:string};return {'x-mailwoman-storage-authority':me.storageAuthority};}
async function clearOwn(page:Page):Promise<void>{await page.request.delete(`/api/settings/${KEY}`,{headers:await headers(page)});}
async function drawer(page:Page):Promise<void>{const menu=page.locator('header.shell__bar button[aria-controls="shell-nav"]').first();if(await menu.isVisible()&&(await menu.getAttribute('aria-expanded'))!=='true')await menu.click();}
async function openFolderSettings(page:Page):Promise<void>{
 const nav=page.getByRole('navigation',{name:'Settings sections'});
 if(await nav.isVisible()){await nav.getByRole('button',{name:'Folders and sidebar',exact:true}).click();await expect(page.getByTestId('focused-inbox-preference')).toBeVisible();return;}
 const settings=page.locator('.sidebar, #shell-nav').getByRole('button',{name:'Settings',exact:true}).first();
 if(!await settings.isVisible())await drawer(page);
 await settings.click();
 await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Folders and sidebar',exact:true}).click();
 await expect(page.getByTestId('focused-inbox-preference')).toBeVisible();
}
async function backToMail(page:Page):Promise<void>{
 const tab=page.getByRole('tab',{name:'Mail',exact:true});if(await tab.isVisible()){await tab.click();return;}
 const mail=page.locator('.sidebar, #shell-nav').getByRole('button',{name:'Mail',exact:true}).first();if(!await mail.isVisible())await drawer(page);await mail.click();
}
const tabs=(page:Page)=>page.getByTestId('inbox-tabs');
/** Nothing Focused-inbox related in the mail pane: no tabs, no enable or turn-off button. */
async function expectNoHeaderControl(page:Page):Promise<void>{
 const pane=page.locator('.mail-pane');
 await expect(pane).toBeVisible();
 await expect(tabs(page)).toHaveCount(0);
 await expect(pane.getByRole('button',{name:'Focused inbox',exact:true})).toHaveCount(0);
 await expect(pane.getByRole('button',{name:'Turn off',exact:true})).toHaveCount(0);
 await expect(pane.getByRole('tab',{name:/^(Focused|Other)/})).toHaveCount(0);
}
/** The compact pair: two tabs, one row no taller than the list toolbar, no buttons. */
async function expectCompactTabs(page:Page):Promise<void>{
 await expect(tabs(page)).toBeVisible({timeout:30000});
 await expect(tabs(page).getByRole('tab')).toHaveCount(2);
 await expect(tabs(page).getByRole('button')).toHaveCount(0);
 // Measured in one pass in the page, so a re-render between reads cannot null a box.
 const geometry=()=>tabs(page).evaluate(el=>{const [a,b]=[...el.querySelectorAll('[role="tab"]')].map(t=>t.getBoundingClientRect());return {height:el.getBoundingClientRect().height,offset:a&&b?Math.abs(a.top-b.top):99};});
 await expect.poll(async()=>(await geometry()).offset,{message:'the two tabs share a row'}).toBeLessThan(2);
 expect((await geometry()).height,'one slim row').toBeLessThanOrEqual(34);
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
}

/** The admin console, in its own browser context, on the setting's page. */
async function adminSetting(browser:Browser,info:TestInfo,width:number){
 const context=await browser.newContext({baseURL:String(info.project.use.baseURL),viewport:{width,height:900},serviceWorkers:'block'});
 const page=await context.newPage();
 await page.goto('/admin');
 await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');
 await page.waitForTimeout(3100);
 const login=page.waitForResponse(r=>new URL(r.url()).pathname==='/admin/login'&&r.request().method()==='POST');
 await page.getByRole('button',{name:'Sign in',exact:true}).click();
 const result=await login;test.skip(result.status()!==200,`no admin sign-in on this stack (${result.status()})`);
 await page.goto(`/admin#/settings/${KEY}`);
 const detail=page.getByTestId('setting-detail');
 await expect(detail).toBeVisible({timeout:30000});
 await expect(detail.getByText(`Key: ${KEY}`,{exact:true})).toBeVisible();
 return {context,page,detail};
}
/** Back to the shipped default (no stored value, unlocked), through the admin API. */
async function resetAdmin(playwright:{request:{newContext:(o:{baseURL:string})=>Promise<import('@playwright/test').APIRequestContext>}},info:TestInfo):Promise<void>{
 const api=await playwright.request.newContext({baseURL:String(info.project.use.baseURL)});
 try{
  expect((await api.post('/admin/login',{data:{username:'root',password:'hunter2'}})).status()).toBe(200);
  const etag=(await api.get(`/admin/settings/${KEY}`)).headers()['etag'];
  const res=await api.delete(`/admin/settings/${KEY}`,{headers:etag?{'if-match':etag}:{}});
  expect([200,204,404]).toContain(res.status());
 }finally{await api.dispose();}
}
async function adminSave(page:Page,value:boolean,lock:boolean):Promise<void>{
 const detail=page.getByTestId('setting-detail');
 const form=detail.locator('form[aria-labelledby="setting-global-heading"]');
 const box=form.locator('input[type="checkbox"]').first();
 if(await box.isChecked()!==value)await box.click();
 const lockSwitch=form.getByRole('switch');
 if((await lockSwitch.getAttribute('aria-checked')==='true')!==lock){
  await lockSwitch.click();
  const confirm=page.getByRole('dialog').getByRole('button',{name:'Lock',exact:true});
  if(await confirm.isVisible({timeout:1500}).catch(()=>false))await confirm.click();
 }
 const saved=page.waitForResponse(r=>new URL(r.url()).pathname===`/admin/settings/${KEY}`&&r.request().method()==='PUT');
 await form.getByRole('button',{name:'Save',exact:true}).click();
 const confirm=page.getByRole('dialog').getByRole('button',{name:/^(Lock|Save|Apply)/});
 if(await confirm.isVisible({timeout:1500}).catch(()=>false))await confirm.first().click();
 expect((await saved).status()).toBe(200);
 // The page reloads its list after a save; wait for the detail to come back with the stored value.
 const again=page.getByTestId('setting-detail').locator('form[aria-labelledby="setting-global-heading"]');
 await expect(again).toBeVisible({timeout:30000});
 if(value)await expect(again.locator('input[type="checkbox"]').first()).toBeChecked();else await expect(again.locator('input[type="checkbox"]').first()).not.toBeChecked();
 await expect(again.getByRole('switch')).toHaveAttribute('aria-checked',String(lock));
}

for(const width of [1440,390]){
 test.describe(`Focused inbox at ${width}px`,()=>{
  test('off by default with no header control; on in Settings shows the compact tabs, live and after a reload',async({page,playwright},info)=>{
   await page.setViewportSize({width,height:900});
   await resetAdmin(playwright,info);
   await signIn(page);await clearOwn(page);await page.reload();
   await expect(page.locator('.list__row').first()).toBeVisible({timeout:60000});
   await expectNoHeaderControl(page);
   expect((await (await page.request.get('/api/settings')).json() as {settings:Record<string,{value:unknown;source:string}>}).settings[KEY]).toMatchObject({value:false,source:'default'});
   await page.screenshot({path:shot(info,`off-default-${width}`)});
   try{
    await openFolderSettings(page);
    const box=page.getByLabel('Focused inbox',{exact:true});
    await expect(box).not.toBeChecked();await expect(box).toBeEnabled();
    const put=page.waitForResponse(r=>new URL(r.url()).pathname===`/api/settings/${KEY}`&&r.request().method()==='PUT');
    await box.check();expect((await put).status()).toBe(200);
    await expect(box).toBeChecked();
    await page.getByTestId('focused-inbox-preference').scrollIntoViewIfNeeded();
    await page.screenshot({path:shot(info,`settings-on-${width}`)});
    // Live: no reload between Settings and the Inbox.
    await backToMail(page);
    await expectCompactTabs(page);
    const focused=tabs(page).getByRole('tab',{name:/^Focused/});const other=tabs(page).getByRole('tab',{name:/^Other/});
    await expect(focused).toHaveAttribute('aria-selected','true');
    await expect(page.locator('.list__row').filter({hasText:fixtures[0]!.subject})).toBeVisible({timeout:30000});
    await expect(page.locator('.list__row').filter({hasText:fixtures[1]!.subject})).toHaveCount(0);
    await page.screenshot({path:shot(info,`tabs-focused-${width}`)});
    await other.click();await expect(other).toHaveAttribute('aria-selected','true');
    await expect(page.locator('.list__row').filter({hasText:fixtures[1]!.subject})).toBeVisible({timeout:30000});
    await tabs(page).screenshot({path:shot(info,`tabs-row-${width}`)});
    await page.screenshot({path:shot(info,`tabs-other-${width}`)});
    // Persisted on the server: a reload keeps it.
    await page.reload();
    await expect(page.locator('.mail-pane')).toBeVisible({timeout:60000});
    await expectCompactTabs(page);
    await page.screenshot({path:shot(info,`tabs-after-reload-${width}`)});
    // Turned off in Settings, the tabs go without a reload.
    await openFolderSettings(page);
    await page.getByLabel('Focused inbox',{exact:true}).uncheck();
    await expect(page.getByLabel('Focused inbox',{exact:true})).not.toBeChecked();
    await backToMail(page);await expectNoHeaderControl(page);
   }finally{await clearOwn(page);}
  });

  test('the admin console sets the default for a fresh user and a lock disables the user control',async({page,browser,playwright},info)=>{
   await page.setViewportSize({width,height:900});
   const admin=await adminSetting(browser,info,width);
   const fresh=`fresh-${run}-${info.project.name}-${width}@example.org`;
   try{
    await admin.page.screenshot({path:shot(info,`admin-setting-${width}`),fullPage:true});
    // Domain and per-user values live on the same page.
    await expect(admin.detail.getByRole('heading',{name:'Domain values',exact:true})).toBeVisible();
    await expect(admin.detail.getByRole('heading',{name:'Per-user values',exact:true})).toBeVisible();
    await adminSave(admin.page,true,false);
    await admin.page.screenshot({path:shot(info,`admin-default-on-${width}`),fullPage:true});
    await deliver(fresh,fixtures[1]!,9);
    await signIn(page,fresh,'fresh-pass');
    await expectCompactTabs(page);
    expect((await (await page.request.get('/api/settings')).json() as {settings:Record<string,{value:unknown;source:string}>}).settings[KEY]).toMatchObject({value:true,source:'global'});
    await page.screenshot({path:shot(info,`fresh-user-tabs-${width}`)});
    // The user may still turn it off while it is not locked.
    await openFolderSettings(page);
    await expect(page.getByLabel('Focused inbox',{exact:true})).toBeChecked();
    await expect(page.getByLabel('Focused inbox',{exact:true})).toBeEnabled();

    await adminSave(admin.page,false,true);
    await admin.page.screenshot({path:shot(info,`admin-locked-off-${width}`),fullPage:true});
    await page.reload();
    await expect(page.locator('.mail-pane, [aria-label="Settings sections"]').filter({visible:true}).first()).toBeVisible({timeout:60000});
    await openFolderSettings(page);
    const box=page.getByLabel('Focused inbox',{exact:true});
    await expect(box).toBeDisabled();await expect(box).not.toBeChecked();
    await expect(page.getByTestId('focused-inbox-preference').getByTestId('setting-lock-note')).toBeVisible();
    const refused=await page.request.put(`/api/settings/${KEY}`,{data:{value:true},headers:await headers(page)});
    expect(refused.status()).toBe(403);
    await page.getByTestId('focused-inbox-preference').scrollIntoViewIfNeeded();
    await page.screenshot({path:shot(info,`user-locked-${width}`)});
    await backToMail(page);await expectNoHeaderControl(page);
   }finally{
    await admin.context.close();
    await resetAdmin(playwright,info);
   }
  });
 });
}
