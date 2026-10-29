// t39 R7: trusted link domains against a live server built from the tree.
// One HTML message with five links is delivered over Greenmail SMTP:
//   https://docs.partner.test/report   trusted (subdomain of partner.test)
//   https://partner.test/start         trusted (exact)
//   https://partner.test.evil.test/    lookalike (suffix after the trusted name)
//   https://evilpartner.test/          lookalike (no label boundary)
//   http://partner.test/plain          trusted host over plain HTTP
// The user saves partner.test + subdomains in Settings > Privacy; Review links
// then opens the two trusted HTTPS links directly (no destination step) and
// keeps the other three under review. An administrator lock makes the
// Settings control read-only and removes "Trust <host>".
// Runs at 1440 and 390 wide in the chromium and firefox projects (the phone
// project is the 390 Chromium leg already). Screenshots go to MW_T39_SHOTS.
import {test,expect} from '../csrf.ts';
import type {Page,TestInfo,BrowserContext} from '@playwright/test';
import net from 'node:net';
import path from 'node:path';

const smtpPort=Number(process.env['MW_E2E_SMTP_PORT']??42225);
const ADMIN={username:process.env['MW_ADMIN_USER']??'root',password:process.env['MW_ADMIN_PASSWORD']??'hunter2'};
const run=Date.now().toString(36);
const subject=`Trusted links ${run}`;
const LINKS={sub:'https://docs.partner.test/report',exact:'https://partner.test/start',suffix:'https://partner.test.evil.test/',joined:'https://evilpartner.test/',plain:'http://partner.test/plain'};
const body=`<html><body><p>Links for review.</p><ul>${Object.entries(LINKS).map(([k,v])=>`<li><a href="${v}">link-${k}</a></li>`).join('')}</ul></body></html>`;

function deliver():Promise<void>{
 return new Promise((resolve,reject)=>{
  const socket=net.connect(smtpPort,'127.0.0.1');
  const data=`From: Link Fixture <links@example.net>\r\nTo: testuser@example.org\r\nSubject: ${subject}\r\nDate: ${new Date().toUTCString().replace('GMT','+0000')}\r\nMessage-ID: <t39-r7-${run}@example.net>\r\nMIME-Version: 1.0\r\nContent-Type: text/html; charset=utf-8\r\n\r\n${body}\r\n.\r\n`;
  const steps=['EHLO t39\r\n','MAIL FROM:<links@example.net>\r\n','RCPT TO:<testuser@example.org>\r\n','DATA\r\n',data,'QUIT\r\n'];
  let i=0,buffer='';
  socket.on('data',chunk=>{buffer+=chunk.toString();if(!buffer.endsWith('\r\n'))return;const last=buffer.trim().split('\r\n').at(-1)!;if(/^\d{3}-/.test(last))return;buffer='';if(/^[45]/.test(last)){socket.destroy();reject(new Error(last));return;}if(i<steps.length)socket.write(steps[i++]!);});
  socket.on('close',()=>resolve());socket.on('error',reject);
 });
}
test.describe.configure({timeout:240000});
let seeded=false;
test.beforeEach(async()=>{if(!seeded){await deliver();seeded=true;await new Promise(r=>setTimeout(r,3100));}});

const shot=(info:TestInfo,name:string)=>{const dir=process.env['MW_T39_SHOTS'];return dir?path.join(dir,`${info.project.name}-${name}.png`):info.outputPath(`${name}.png`);};
async function signIn(page:Page):Promise<void>{
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.locator('.list__row').first()).toBeVisible({timeout:60000});
}
async function headers(page:Page):Promise<Record<string,string>>{const me=await (await page.request.get('/api/me')).json() as {storageAuthority:string};return {'x-mailwoman-storage-authority':me.storageAuthority};}
async function clearOwn(page:Page):Promise<void>{const h=await headers(page);for(const key of ['links.trustedDomains','links.trustedSubdomains'])await page.request.delete(`/api/settings/${key}`,{headers:h});}
async function drawer(page:Page):Promise<void>{const menu=page.locator('header.shell__bar button[aria-controls="shell-nav"]').first();if(await menu.isVisible()&&(await menu.getAttribute('aria-expanded'))!=='true')await menu.click();}
async function openPrivacy(page:Page):Promise<void>{
 const settings=page.locator('.sidebar, #shell-nav').getByRole('button',{name:'Settings',exact:true}).first();
 if(!await settings.isVisible())await drawer(page);
 await settings.click();
 await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Privacy',exact:true}).click();
 await expect(page.getByTestId('trusted-link-domains')).toBeVisible();
}
async function backToMail(page:Page):Promise<void>{
 const tab=page.getByRole('tab',{name:'Mail',exact:true});if(await tab.isVisible()){await tab.click();return;}
 const mail=page.locator('.sidebar, #shell-nav').getByRole('button',{name:'Mail',exact:true}).first();if(!await mail.isVisible())await drawer(page);await mail.click();
}
async function openMessage(page:Page):Promise<void>{
 const row=page.locator('.list__row').filter({hasText:subject}).first();
 await expect(row).toBeVisible({timeout:60000});await row.click();
 await expect(page.locator('.reader__subject').filter({visible:true}).first()).toHaveText(subject);
}
async function reviewLinks(page:Page){
 const reader=page.locator('section.reader').filter({visible:true}).first();
 const inline=reader.getByRole('button',{name:/^Review links \(\d+\)$/});
 // The action appears once the body is parsed; at 390 it may sit in More.
 await expect(reader.locator('iframe.reader__frame').first()).toBeAttached();
 await expect.poll(async()=>await inline.isVisible()||await reader.getByTestId('reader-more').isVisible(),{timeout:30000}).toBe(true);
 await page.waitForTimeout(500);
 if(await inline.isVisible())await inline.click();
 else{await page.mouse.move(1,1);await reader.getByTestId('reader-more').click();await page.getByRole('menuitem',{name:/^Review links/}).click();}
 const dialog=page.getByRole('dialog',{name:'Message links'});await expect(dialog).toBeVisible();return dialog;
}
/** The destinations themselves are not under test: answer them locally so a popup has a URL to report. */
async function stubDestinations(context:BrowserContext):Promise<void>{
 for(const pattern of ['https://docs.partner.test/**','https://partner.test/**'])await context.route(pattern,route=>route.fulfill({status:200,contentType:'text/html',body:'<title>destination</title>ok'}));
}

for(const width of [1440,390]){
 test.describe(`at ${width}px`,()=>{
  test.beforeEach(async({page},info)=>{
   test.skip(info.project.name==='phone','the phone project is the 390 Chromium leg; these specs size the viewport themselves');
   await page.setViewportSize({width,height:width>600?900:844});
  });

  test('a trusted domain opens from Review links; untrusted, lookalike and plain-HTTP links stay under review',async({page,context},info)=>{
   await signIn(page);await clearOwn(page);await page.reload();await expect(page.locator('.list__row').first()).toBeVisible({timeout:60000});
   // Before: nothing is trusted, every link needs the destination step.
   await openMessage(page);
   let dialog=await reviewLinks(page);
   await expect(dialog.getByRole('link')).toHaveCount(0);
   await expect(dialog.getByRole('button',{name:'link-sub'})).toBeVisible();
   await page.keyboard.press('Escape');await expect(dialog).toBeHidden();

   await openPrivacy(page);
   const box=page.getByLabel('Domains',{exact:true});await expect(box).toBeEnabled();
   await box.fill('*.partner.test');await page.getByRole('button',{name:'Save trusted domains'}).click();
   await expect(page.getByTestId('trusted-link-domains').getByRole('alert')).toContainText('*.partner.test');
   await box.fill('Partner.Test');await page.getByRole('button',{name:'Save trusted domains'}).click();
   await expect.poll(async()=>(await (await page.request.get('/api/settings')).json()).settings['links.trustedDomains'].value).toEqual(['partner.test']);
   await page.getByLabel(/Also trust subdomains/).check();
   await expect.poll(async()=>(await (await page.request.get('/api/settings')).json()).settings['links.trustedSubdomains'].value).toBe(true);
   await page.getByTestId('trusted-link-domains').scrollIntoViewIfNeeded();
   await page.screenshot({path:shot(info,`r7-settings-${width}`)});
   await backToMail(page);

   await openMessage(page);
   dialog=await reviewLinks(page);
   const direct=dialog.getByRole('link');
   await expect(direct).toHaveCount(2);
   for(const [name,href] of [['link-sub',LINKS.sub],['link-exact',LINKS.exact]] as const){
    const link=dialog.getByRole('link',{name});
    await expect(link).toHaveAttribute('href',href);await expect(link).toHaveAttribute('rel','noopener noreferrer');await expect(link).toHaveAttribute('target','_blank');
   }
   await expect(dialog.getByText('Trusted domain: opens without review')).toHaveCount(2);
   for(const name of ['link-suffix','link-joined','link-plain'])await expect(dialog.getByRole('button',{name,exact:true})).toBeVisible();
   await dialog.screenshot({path:shot(info,`r7-review-${width}`)});
   // The trusted link opens straight away, with no destination step.
   await stubDestinations(context);
   const [popup]=await Promise.all([context.waitForEvent('page'),dialog.getByRole('link',{name:'link-sub'}).click()]);
   await expect.poll(()=>popup.url()).toBe(LINKS.sub);await popup.close();
   await expect(dialog.getByRole('region',{name:'Destination to open'})).toHaveCount(0);
   // The lookalikes and plain HTTP still go through the destination step.
   for(const name of ['link-suffix','link-joined','link-plain']){
    await dialog.getByRole('button',{name,exact:true}).click();
    const destination=dialog.getByRole('region',{name:'Destination to open'});
    await expect(destination).toBeVisible();
    await expect(destination.getByText('This destination comes from the message sender.',{exact:false})).toBeVisible();
    await expect(destination.getByRole('link',{name:'Open in a new tab'})).toBeVisible();
   }
   await expect(dialog.getByRole('region',{name:'Destination to open'}).getByText('This address uses unencrypted HTTP.')).toBeVisible();
   await dialog.screenshot({path:shot(info,`r7-lookalike-${width}`)});
   // "Trust <host>" from the destination step adds the exact host.
   await dialog.getByRole('button',{name:'link-joined',exact:true}).click();
   await dialog.getByRole('button',{name:'Trust evilpartner.test'}).click();
   await expect(dialog.getByRole('link',{name:'link-joined'})).toHaveAttribute('href',LINKS.joined);
   await expect.poll(async()=>(await (await page.request.get('/api/settings')).json()).settings['links.trustedDomains'].value).toEqual(['partner.test','evilpartner.test']);
   await page.keyboard.press('Escape');
   await clearOwn(page);
  });

  test('an administrator default applies and a lock makes the control read-only',async({page,playwright},info)=>{
   const admin=await playwright.request.newContext({baseURL:String(info.project.use.baseURL)});
   const login=await admin.post('/admin/login',{data:ADMIN});
   test.skip(login.status()!==200,`no admin sign-in on this stack (${login.status()})`);
   const put=async(key:string,data:Record<string,unknown>)=>{const etag=(await admin.get(`/admin/settings/${key}`)).headers()['etag'];return (await admin.put(`/admin/settings/${key}`,{data,headers:etag?{'if-match':etag}:{}})).status();};
   try{
    await signIn(page);await clearOwn(page);
    expect(await put('links.trustedDomains',{globalValue:['partner.test'],locked:true})).toBe(200);
    expect(await put('links.trustedSubdomains',{globalValue:false,locked:true})).toBe(200);
    await page.reload();await expect(page.locator('.list__row').first()).toBeVisible({timeout:60000});
    await openMessage(page);
    const dialog=await reviewLinks(page);
    // Exact only (subdomains locked off): partner.test opens, docs.partner.test does not.
    await expect(dialog.getByRole('link')).toHaveCount(1);
    await expect(dialog.getByRole('link',{name:'link-exact'})).toHaveAttribute('href',LINKS.exact);
    await dialog.getByRole('button',{name:'link-sub',exact:true}).click();
    await expect(dialog.getByRole('region',{name:'Destination to open'})).toBeVisible();
    await expect(dialog.getByRole('button',{name:/^Trust /})).toHaveCount(0);
    await page.keyboard.press('Escape');
    await openPrivacy(page);
    const section=page.getByTestId('trusted-link-domains');
    await expect(page.getByLabel('Domains',{exact:true})).toBeDisabled();
    await expect(page.getByLabel('Domains',{exact:true})).toHaveValue('partner.test');
    await expect(page.getByLabel(/Also trust subdomains/)).toBeDisabled();
    await expect(section.getByTestId('setting-lock-note')).toHaveCount(2);
    const refused=await page.request.put('/api/settings/links.trustedDomains',{data:{value:['evil.test']},headers:await headers(page)});
    expect(refused.status()).toBe(403);
    await section.scrollIntoViewIfNeeded();
    await page.screenshot({path:shot(info,`r7-locked-${width}`)});
   }finally{
    await admin.delete('/admin/settings/links.trustedDomains');await admin.delete('/admin/settings/links.trustedSubdomains');
    await admin.dispose();
   }
  });
 });
}
