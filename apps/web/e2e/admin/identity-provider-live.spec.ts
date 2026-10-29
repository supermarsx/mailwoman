import { createServer,type Server } from 'node:http';
import { ADMIN } from './admin-path.ts';
import { readFile } from 'node:fs/promises';
import { test,expect } from '../csrf.ts';
// The issuer is a public HTTPS origin in front of a Mailwoman server whose
// MW_PUBLIC_URL is that origin (strict CSRF compares Origin with it). The CI
// stack serves plain HTTP on :8090 with no such front, so this runs only where
// one is provided; it used to hard-code https://localhost:42215 and, locally,
// silently drove whatever happened to listen there.
const issuer=process.env['MW_E2E_OIDC_ISSUER']??'';const rp='http://127.0.0.1:42217';let server:Server|undefined;
test.use({ignoreHTTPSErrors:true});
test.skip(issuer==='','needs an HTTPS Mailwoman origin whose MW_PUBLIC_URL is that origin, e.g. a TLS front at https://localhost:42215 (set MW_E2E_OIDC_ISSUER)');
test.beforeAll(async()=>{if(issuer==='')return;const html=await readFile(new URL('../fixtures/oidc-rp.html',import.meta.url));const fixture=createServer((_request,response)=>{response.writeHead(200,{'content-type':'text/html; charset=utf-8','cache-control':'no-store'});response.end(html);});server=fixture;await new Promise<void>(resolve=>fixture.listen(42217,'127.0.0.1',resolve));});
test.afterAll(async()=>{if(server===undefined)return;const open=server;await new Promise<void>((resolve,reject)=>open.close(error=>error?reject(error):resolve()));});
test('real relying party verifies identity, refreshes and confirms logout @phone',async({page,browserName})=>{
 test.setTimeout(120000);await page.goto(`${issuer}${ADMIN}`);await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect.poll(async()=> (await page.request.get(`${issuer}${ADMIN}/session`)).status()).toBe(200);
 const config=await page.request.get(`${issuer}${ADMIN}/oidc`);expect(config.status()).toBe(200);const existing=await config.json();
 if(!existing.enabled){await page.goto(`${issuer}${ADMIN}#/identity-provider/configure`);await page.getByLabel('Public HTTPS issuer URL').fill(`${issuer}/oidc`);await page.getByLabel('Enabled',{exact:true}).check();await page.getByRole('button',{name:'Save',exact:true}).click();const dialog=page.getByRole('dialog',{name:'Confirm it is you'});await expect(dialog).toBeVisible();await dialog.getByLabel('Password',{exact:true}).fill('hunter2');await dialog.getByRole('button',{name:'Confirm and continue'}).click();await expect(page).toHaveURL(/#\/identity-provider$/);}
 const policy=await page.request.get(`${issuer}${ADMIN}/settings/signin.retryCooldownSecs`);expect(policy.ok()).toBe(true);expect((await page.request.put(`${issuer}${ADMIN}/settings/signin.retryCooldownSecs`,{headers:{'if-match':policy.headers()['etag']!},data:{globalValue:0}})).ok()).toBe(true);
 const name=`Browser relying party ${browserName} ${Date.now()}`;
 await page.goto(`${issuer}${ADMIN}#/identity-provider/clients/new`);await page.getByLabel('Name',{exact:true}).fill(name);await page.getByLabel('Authorization callback URIs').fill(`${rp}/callback`);await page.getByLabel('Post-logout redirect URIs').fill(`${rp}/logged-out`);await page.getByLabel('Enabled',{exact:true}).check();await page.getByLabel('email',{exact:true}).check();await page.getByLabel('offline_access',{exact:true}).check();const saved=page.waitForResponse(response=>response.url().endsWith(`${ADMIN}/oidc/clients`)&&response.request().method()==='POST'&&response.status()===201);await page.getByRole('button',{name:'Save',exact:true}).click();
 const stepup=page.getByRole('dialog',{name:'Confirm it is you'});if(await Promise.race([saved.then(()=>false),stepup.waitFor({state:'visible'}).then(()=>true)])){await stepup.getByLabel('Password',{exact:true}).fill('hunter2');await stepup.getByRole('button',{name:'Confirm and continue'}).click();}
 await saved;await expect(page).toHaveURL(/#\/identity-provider$/);const clients=await(await page.request.get(`${issuer}${ADMIN}/oidc/clients`)).json();const client=clients.clients.find((row:{name:string})=>row.name===name);expect(client).toBeTruthy();
 await page.screenshot({path:test.info().outputPath('identity-clients.png'),fullPage:true});
 await page.goto(`${rp}/?client=${encodeURIComponent(client.clientId)}`);await page.getByRole('button',{name:'Sign in with Mailwoman'}).click();await expect(page).toHaveURL(/\/oidc\/consent\?flow=/);
 await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('heading',{name:'Share your identity'})).toBeVisible();await expect(page.getByText(name,{exact:true})).toBeVisible();expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);await page.screenshot({path:test.info().outputPath('identity-consent.png'),fullPage:true});
 await page.getByRole('button',{name:'Allow identity access'}).click();await expect(page.getByRole('status')).toContainText('Verified signature, issuer, audience, nonce and access-token hash.');await page.getByRole('button',{name:'Refresh identity'}).click();await expect(page.getByRole('status')).toContainText('Refresh verified');await page.screenshot({path:test.info().outputPath('independent-rp-verified.png'),fullPage:true});
 await page.getByRole('button',{name:'Sign out',exact:true}).click();await expect(page.getByRole('heading',{name:'Sign out of Mailwoman'})).toBeVisible();await page.getByRole('button',{name:'Sign out of Mailwoman'}).click();await expect(page.getByRole('status')).toHaveText('Signed out of Mailwoman');expect((await page.request.get(`${issuer}/api/me`)).status()).toBe(401);
});
