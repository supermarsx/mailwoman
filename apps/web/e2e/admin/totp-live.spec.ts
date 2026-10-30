import {createHmac} from 'node:crypto';
import { ADMIN } from './admin-path.ts';
import {test,expect} from '../csrf.ts';
function totp(secret:string):string {const alphabet='ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';const bits=[...secret.replace(/=+$/,'').toUpperCase()].map(char=>alphabet.indexOf(char).toString(2).padStart(5,'0')).join('');const key=Buffer.from(bits.match(/.{8}/g)!.map(byte=>Number.parseInt(byte,2)));const counter=Buffer.alloc(8);counter.writeBigUInt64BE(BigInt(Math.floor(Date.now()/30000)));const hash=createHmac('sha1',key).update(counter).digest();const offset=hash[19]!&15;return String((hash.readUInt32BE(offset)&0x7fffffff)%1000000).padStart(6,'0');}
// Enrollment runs on a throwaway administrator that root creates and deletes, never on
// root: recovery codes deliberately outlive the authenticator they were issued with
// (mw-store admin_totp.rs: re-enrollment never replaces an issued set), so on root a
// second project would find no new codes, and a failure midway would leave root with a
// second factor every later spec cannot pass.
test('admin authenticator enrollment and lost-factor recovery removal use actual server proofs @phone',async({page,context,request},info)=>{
 test.setTimeout(120_000);
 const username=`totp-${info.project.name}-${Date.now()}`;const password='totp-fixture-password';
 // The shared fixture source keeps its sign-in pacing from earlier specs.
 await page.waitForTimeout(3100);
 const root=await request.post(`${ADMIN}/login`,{data:{username:'root',password:'hunter2'}});expect(root.status(),await root.text()).toBe(200);
 expect((await request.post(`${ADMIN}/step-up`,{data:{password:'hunter2'}})).ok()).toBe(true);
 const created=await request.post(`${ADMIN}/admins`,{data:{username,password,roles:[{roleId:'auditor'}]}});expect(created.status(),await created.text()).toBeLessThan(300);
 const id=String((await created.json()).id);
 const cooldown=await request.get(`${ADMIN}/settings/signin.retryCooldownSecs`);expect(cooldown.ok()).toBe(true);const previousCooldown=(await cooldown.json()).globalValue??null;
 expect((await request.put(`${ADMIN}/settings/signin.retryCooldownSecs`,{headers:{'if-match':cooldown.headers()['etag']!},data:{globalValue:0}})).ok()).toBe(true);
 async function login(){await page.goto(ADMIN);await page.getByLabel('Admin username').fill(username);await page.getByLabel('Password',{exact:true}).fill(password);await page.getByRole('button',{name:'Sign in',exact:true}).click();}
 try{
 await login();await expect.poll(async()=> (await page.request.get(`${ADMIN}/session`)).status()).toBe(200);
 await page.goto(`${ADMIN}#/preferences/security`);await page.getByRole('button',{name:'Manage authenticator and recovery codes'}).click();await expect(page).toHaveURL(/preferences\/totp$/);await page.getByRole('button',{name:'Set up authenticator'}).click();const stepup=page.getByRole('dialog',{name:'Confirm it is you'});await stepup.getByRole('textbox',{name:'Password',exact:true}).fill(password);await stepup.getByRole('button',{name:'Confirm and continue'}).click();const key=page.getByLabel('Manual setup key');await expect(key).toBeVisible();const secret=await key.inputValue();const qr=page.getByAltText('Authenticator setup QR code, generated on this device');await expect(qr).toBeVisible();expect(await qr.getAttribute('src')).toMatch(/^data:image\/svg\+xml,/);await expect.poll(()=>qr.evaluate(node=>(node as HTMLImageElement).naturalWidth)).toBeGreaterThan(0);await page.screenshot({path:info.outputPath('admin-totp-enrollment.png'),fullPage:true,animations:'disabled'});await page.getByLabel('Authenticator code',{exact:true}).fill(totp(secret));await page.getByRole('button',{name:'Confirm authenticator'}).click();const recovery=page.getByRole('region',{name:'Save your recovery codes'});await expect(recovery).toBeVisible();const codes=await recovery.locator('code').allTextContents();expect(codes.length).toBeGreaterThanOrEqual(2);await expect(key).toHaveCount(0);await recovery.getByRole('button',{name:'I have saved my recovery codes'}).click();await expect(recovery).toHaveCount(0);
 await context.clearCookies();await login();await page.getByRole('checkbox',{name:'Use a recovery code'}).check();await page.getByLabel('Recovery code',{exact:true}).fill(codes[0]!);await page.getByRole('button',{name:'Verify',exact:true}).click();await expect(page.getByRole('button',{name:'Verify',exact:true})).toHaveCount(0);await page.goto(`${ADMIN}#/preferences/totp`);await page.getByRole('button',{name:'Remove authenticator',exact:true}).click();await page.getByRole('alertdialog').getByRole('button',{name:'Remove authenticator',exact:true}).click();await stepup.getByRole('textbox',{name:'Password',exact:true}).fill(password);await stepup.getByLabel('Recovery code (with your password, instead of an authenticator code)').fill(codes[1]!);await stepup.getByRole('button',{name:'Confirm and continue'}).click();await expect(page.getByRole('button',{name:'Set up authenticator'})).toBeVisible();await expect(page.getByLabel('Authenticator code',{exact:true})).toHaveCount(0);expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);await page.screenshot({path:info.outputPath('admin-totp-removed-with-recovery.png'),fullPage:true,animations:'disabled'});
 }finally{
  // Root's API session puts pacing back and deletes the fixture administrator with its factors.
  const current=await request.get(`${ADMIN}/settings/signin.retryCooldownSecs`);
  expect((await request.put(`${ADMIN}/settings/signin.retryCooldownSecs`,{headers:{'if-match':current.headers()['etag']!},data:{globalValue:previousCooldown}})).ok()).toBe(true);
  expect((await request.post(`${ADMIN}/step-up`,{data:{password:'hunter2'}})).ok()).toBe(true);
  const row=await request.get(`${ADMIN}/admins/${id}`);expect(row.ok()).toBe(true);
  const removed=await request.delete(`${ADMIN}/admins/${id}`,{headers:{'if-match':row.headers()['etag']!}});expect(removed.status(),await removed.text()).toBeLessThan(300);
 }
});
