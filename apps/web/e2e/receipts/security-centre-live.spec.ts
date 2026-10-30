// t97: the Security centre against a live server built from the tree.
//  * Overview is a mini dashboard with real data (failed sign-ins seeded here),
//    loaded in place with skeletons;
//  * the Requirements tab holds the require-2FA and per-domain controls, and
//    they still save; old deep links land on it; its breadcrumb is friendly;
//  * a restricted administrator (bans.read only) sees fewer tiles;
//  * Adaptive protection: plain-language summary, preset diff, an inline rule
//    edit with step-up and a reset, and the "what would happen" simulator.
// Runs at 1440 and 390 wide in the chromium and firefox projects. Screenshots go
// to MW_T97_SHOTS.
import {test,expect} from '../csrf.ts';
import { ADMIN } from '../admin/admin-path.ts';
import { adminLogin } from '../admin/admin-login.ts';
import type {Page,TestInfo,APIRequestContext} from '@playwright/test';
import path from 'node:path';

test.describe.configure({timeout:300000});
const skipPhone=(info:TestInfo)=>test.skip(info.project.name==='phone','the 390 legs run in the chromium and firefox projects');
const shot=(info:TestInfo,name:string)=>{const dir=process.env['MW_T97_SHOTS'];return dir?path.join(dir,`${info.project.name}-${name}.png`):info.outputPath(`${name}.png`);};
const WIDTHS=[1440,390] as const;

async function noSideScroll(page:Page):Promise<void>{expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);}
/** Failed sign-ins to record: wrong admin passwords (the stack's Greenmail accepts
 *  any mailbox password). Three stay under the admin lock limit of five, and the
 *  fixture's 3 s retry cooldown is respected. */
async function seedFailures(request:APIRequestContext,count:number):Promise<void>{
 for(let i=0;i<count;i++){await new Promise(r=>setTimeout(r,3200));const r=await request.post(`${ADMIN}/login`,{data:{username:`t97-nobody-${i}`,password:'wrong password'}});expect(r.status()).toBeGreaterThanOrEqual(400);}
 await new Promise(r=>setTimeout(r,3200));
}
async function unlockAll(request:APIRequestContext):Promise<void>{
 const list=await request.get(`${ADMIN}/bans/automatic`);if(!list.ok())return;
 for(const lock of (await list.json()).locks as {surface:string;ip:string;revision:string}[]){await request.delete(`${ADMIN}/bans/automatic/${lock.surface}/${encodeURIComponent(lock.ip)}`,{headers:{'if-match':`"${lock.revision}"`}});}
}
/** Answer the step-up dialog if the server asks for it before `done` arrives. */
async function stepUp(page:Page,done:Promise<unknown>):Promise<void>{
 const proof=page.getByRole('dialog',{name:'Confirm it is you'});
 const asked=await Promise.race([proof.waitFor({state:'visible',timeout:30000}).then(()=>true),done.then(()=>false)]);
 if(asked){await proof.getByRole('textbox',{name:'Password',exact:true}).fill('hunter2');await proof.getByRole('button',{name:'Confirm and continue'}).click();}
 await done;
}

test('Overview dashboard, Requirements tab and deep links, with real data',async({page},info)=>{
 skipPhone(info);
 await seedFailures(page.request,3);
 await adminLogin(page);
 try{
  for(const width of WIDTHS){
   await page.setViewportSize({width,height:width>800?900:844});
   await page.goto(`${ADMIN}#/security`);
   const dash=page.getByTestId('security-dashboard');
   await expect(dash).toBeVisible();
   const failed=page.getByTestId('secdash-failed');
   await expect(failed).toBeVisible();
   expect(Number((await failed.locator('p').first().innerText()).replace(/\D/g,''))).toBeGreaterThanOrEqual(3);
   for(const id of ['secdash-factors','secdash-protection','secdash-locks','secdash-admins','secdash-posture','secdash-signins-panel','secdash-audit-panel','secdash-posture-panel'])await expect(page.getByTestId(id)).toBeVisible();
   await expect(page.getByTestId('secdash-spark-24h')).toBeVisible();
   await expect(page.getByText('Require two-factor for the whole deployment')).toHaveCount(0);
   await expect(page.getByTestId('secdash-admins-panel').getByText('root').first()).toBeVisible();
   await noSideScroll(page);
   await page.screenshot({path:shot(info,`overview-${width}`),fullPage:true});
   // A tile opens its detail page.
   await page.getByTestId('secdash-factors').getByRole('link',{name:'Open requirements'}).click();
   await expect(page.getByTestId('security-requirements')).toBeVisible();
   await expect(page.getByRole('checkbox',{name:'Require two-factor for the whole deployment'})).toBeVisible();
   await expect(page.getByRole('heading',{name:'Per-domain requirements'})).toBeVisible();
   await expect(page.getByRole('link',{name:/Passkey|passkey/}).first()).toBeVisible();
   const crumbs=page.getByRole('navigation',{name:/Breadcrumb/i});
   await expect(crumbs).toContainText('Two-factor & sign-in requirements');
   await noSideScroll(page);
   await page.screenshot({path:shot(info,`requirements-${width}`),fullPage:true});
  }
  // The per-domain control still saves, unchanged.
  const domain=`t97-${info.project.name}.invalid`;
  await page.getByPlaceholder('example.com').fill(domain);
  const added=page.waitForResponse(r=>r.url().endsWith(`${ADMIN}/2fa/policy`)&&r.request().method()==='POST');
  await page.getByTestId('admin-2fa-add-submit').click();expect((await added).ok()).toBe(true);
  const row=page.getByRole('checkbox',{name:`Require two-factor for ${domain}`});
  await expect(row).toBeChecked();
  await page.reload();await expect(page.getByRole('checkbox',{name:`Require two-factor for ${domain}`})).toBeChecked();
  const off=page.waitForResponse(r=>r.url().endsWith(`${ADMIN}/2fa/policy`)&&r.request().method()==='POST');
  await page.getByRole('checkbox',{name:`Require two-factor for ${domain}`}).uncheck();expect((await off).ok()).toBe(true);
  await page.reload();await expect(page.getByRole('checkbox',{name:`Require two-factor for ${domain}`})).not.toBeChecked();
  // Old deep links land on the Requirements tab.
  for(const legacy of ['#/two-factor','#/security/two-factor']){
   await page.goto(`${ADMIN}${legacy}`);
   await expect(page.getByTestId('security-requirements')).toBeVisible();
   await expect(page.getByRole('navigation',{name:'Security centre'}).getByRole('link',{name:'Two-factor & sign-in requirements'})).toHaveAttribute('aria-current','page');
  }
 }finally{await unlockAll(page.request);}
});

test('a restricted administrator sees only the tiles they have rights for',async({page,browser},info)=>{
 skipPhone(info);
 await adminLogin(page);
 const name=`t97-bans-${info.project.name}`;
 expect((await page.request.post(`${ADMIN}/step-up`,{data:{password:'hunter2'}})).status()).toBe(200);
 const role=await (await page.request.post(`${ADMIN}/roles`,{data:{name:`${name} role`,capabilities:['bans.read']}})).json();
 const created=await page.request.post(`${ADMIN}/admins`,{data:{username:name,password:'long test password',roles:[{roleId:role.id}]}});
 expect(created.status(),await created.text()).toBe(201);
 const admin=await created.json();
 const context=await browser.newContext({baseURL:info.project.use.baseURL!});
 const other=await context.newPage();
 try{
  for(const width of WIDTHS){
   await other.setViewportSize({width,height:width>800?900:844});
   if(width===1440){
    await other.goto(ADMIN);await other.getByLabel('Admin username').fill(name);await other.getByLabel('Password',{exact:true}).fill('long test password');await other.waitForTimeout(3100);
    await other.getByRole('button',{name:'Sign in',exact:true}).click();await expect(other.locator('#admin-main')).toBeVisible();
   }
   await other.goto(`${ADMIN}#/security`);
   await expect(other.getByTestId('secdash-failed')).toBeVisible();
   await expect(other.getByTestId('secdash-protection')).toBeVisible();
   for(const id of ['secdash-factors','secdash-admins','secdash-posture','secdash-audit-panel'])await expect(other.getByTestId(id)).toHaveCount(0);
   await expect(other.getByText(/Addresses are shortened to their network/)).toBeVisible();
   await expect(other.getByRole('navigation',{name:'Security centre'}).getByRole('link',{name:'Two-factor & sign-in requirements'})).toHaveCount(0);
   await noSideScroll(other);
   await other.screenshot({path:shot(info,`restricted-${width}`),fullPage:true});
  }
 }finally{
  await context.close();
  await page.request.delete(`${ADMIN}/admins/${admin.id}`);
  await page.request.delete(`${ADMIN}/roles/${role.id}`);
 }
});

test('Adaptive protection: summary, preset diff, inline edit with step-up, reset and simulator',async({page},info)=>{
 skipPhone(info);
 await adminLogin(page);
 for(const width of WIDTHS){
  await page.setViewportSize({width,height:width>800?900:844});
  await page.goto(`${ADMIN}#/security/protection`);
  const summary=page.getByTestId('protection-summary');
  await expect(summary).toContainText('Request budget:');
  await expect(summary).toContainText('Unusual requests:');
  await page.getByRole('radio',{name:/Strict/}).check();
  await expect(page.getByTestId('preset-diff')).toBeVisible();
  await expect(page.getByTestId('preset-diff')).toContainText('Enforce');
  await noSideScroll(page);
  await page.screenshot({path:shot(info,`protection-${width}`),fullPage:true});
  await page.getByRole('button',{name:'Cancel',exact:true}).first().click();
 }
 // Inline edit: the slow-down becomes 5 s, with step-up, then back to default.
 const row=page.getByTestId('protect-rule-throttle-secs');
 await row.getByRole('spinbutton',{name:'Slow-down'}).fill('5');
 const saved=page.waitForResponse(r=>r.url().endsWith(`${ADMIN}/bans/protection/policy`)&&r.request().method()==='PUT'&&r.ok());
 await row.getByRole('button',{name:'Save Slow-down'}).click();
 await stepUp(page,saved);
 await expect(row.getByText('Default: 3 s')).toBeVisible();
 await page.reload();
 await expect(page.getByTestId('protect-rule-throttle-secs').getByRole('spinbutton',{name:'Slow-down'})).toHaveValue('5');
 const reset=page.waitForResponse(r=>r.url().endsWith(`${ADMIN}/bans/protection/policy`)&&r.request().method()==='PUT'&&r.ok());
 await page.getByTestId('protect-rule-throttle-secs').getByRole('button',{name:'Reset Slow-down to its default'}).click();
 await stepUp(page,reset);
 await expect(page.getByTestId('protect-rule-throttle-secs').getByText('This is the default.')).toBeVisible();
 // Out of range: refused on the row, nothing sent.
 await page.getByTestId('protect-rule-throttle-secs').getByRole('spinbutton',{name:'Slow-down'}).fill('9999');
 await page.getByTestId('protect-rule-throttle-secs').getByRole('button',{name:'Save Slow-down'}).click();
 await expect(page.getByTestId('protect-rule-throttle-secs').getByRole('alert')).toContainText('The largest allowed value is 300.');
 await page.getByTestId('protect-rule-throttle-secs').getByRole('button',{name:'Cancel'}).click();
 // Simulator with the Strict preset as a draft: 15 unusual requests reach it.
 await page.getByRole('radio',{name:/Strict/}).check();
 const sim=page.getByTestId('protection-simulator');
 await sim.getByLabel('Unusual requests already in the short window').fill('15');
 await sim.getByLabel('Use my unsaved changes and the selected preset').check();
 await sim.getByRole('button',{name:'Check'}).click();
 const result=page.getByTestId('protection-sim-result');
 await expect(result).toHaveAttribute('data-action','throttle');
 await expect(result).toContainText('Short-window anomaly count');
 await result.scrollIntoViewIfNeeded();
 await page.screenshot({path:shot(info,'protection-simulator'),fullPage:true});
 // Without the draft, the stored policy (off by default) allows it.
 await sim.getByLabel('Use my unsaved changes and the selected preset').uncheck();
 await sim.getByRole('button',{name:'Check'}).click();
 await expect(result).toHaveAttribute('data-action','allow');
});
