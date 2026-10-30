import { test, expect } from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import {adminLogin} from './admin-login.ts';

test('SMTP transport policy controls persist DANE and MTA-STS without applying them to IMAP @phone',async({page},info)=>{
 await adminLogin(page);
 const name=`smtp-policy-${info.project.name}-${Date.now()}.example.test`;
 const endpoint=`${ADMIN}/domains/${name}`;
 expect((await page.request.put(endpoint,{data:{enabled:false,upstream:{version:2,backends:[]}}})).ok()).toBe(true);
 try{
  await page.goto(`${ADMIN}#/domains/${name}/backends/new`);
  await page.getByLabel(/^Backend ID/).fill('submission');
  await page.getByLabel(/^Protocol and service/).selectOption('imap');
  await page.getByLabel('Customize transport security',{exact:true}).check();
  await expect(page.getByLabel(/^DANE verification/)).toHaveCount(0);
  await expect(page.getByLabel(/^MTA-STS verification/)).toHaveCount(0);
  await page.getByLabel(/^Protocol and service/).selectOption('smtp');
  await page.getByLabel(/^Hostname or IP address/).fill('smtp.example.test');
  await page.getByLabel('Customize transport security',{exact:true}).check();
  await expect(page.getByLabel(/^DANE verification/)).toHaveValue('off');
  await expect(page.getByLabel(/^MTA-STS verification/)).toHaveValue('off');
  await page.getByLabel(/^DANE verification/).selectOption('required');
  await page.getByLabel(/^MTA-STS verification/).selectOption('auto');
  await page.getByLabel('SMTP policy domain',{exact:false}).fill('example.test');
  await page.getByRole('button',{name:'Save backend',exact:true}).click();
  await expect(page).toHaveURL(new RegExp(`#/domains/${name.replaceAll('.','\\.')}$`));
  const saved=await (await page.request.get(endpoint)).json();
  expect(saved.upstream.backends[0].endpoint.security.smtpPolicy).toEqual({dane:'required',mtaSts:'auto',policyDomain:'example.test'});
  await page.goto(`${ADMIN}#/domains/${name}/backends/submission`);
  await expect(page.getByLabel(/^DANE verification/)).toHaveValue('required');
  await expect(page.getByLabel(/^MTA-STS verification/)).toHaveValue('auto');
  await expect(page.getByLabel('SMTP policy domain',{exact:false})).toHaveValue('example.test');
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.screenshot({path:info.outputPath('smtp-transport-policy-persisted.png'),fullPage:true});
 }finally{
  const current=await page.request.get(endpoint);
  if(current.ok())expect((await page.request.delete(endpoint,{headers:{'if-match':current.headers()['etag']!}})).ok()).toBe(true);
 }
});

test('dedicated backend page saves ordered security and previews the wire username @phone', async ({ page }) => {
  await page.goto(ADMIN);
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  const authenticated = page.waitForResponse(response => response.url().endsWith(`${ADMIN}/login`) && response.request().method() === 'POST');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  expect((await authenticated).status()).toBe(200);
  await expect(page.locator('#admin-main')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toHaveCount(0);
  const status = await page.request.get(`${ADMIN}/domains/status`);
  expect(status.ok(), `domain status ${status.status()}: ${await status.text()}`).toBe(true);
  const capability = await status.json();
  expect(capability.mode, 'run this runtime test on the mail-engine fixture').toBe('engine');
  expect(capability.transportSecurity?.runtimeReady, 'use a fresh binary with verified transport capabilities').toBe(true);
  const name = `backend-${Date.now()}-${test.info().workerIndex}.example.test`;
  const path = `${ADMIN}/domains/${name}`;
  const create = await page.request.put(path, { data: { enabled: false, upstream: { version: 2, backends: [] } } });
  expect(create.ok(), `create fixture: ${create.status()}`).toBe(true);
  try {
    await page.goto(`${ADMIN}#/domains/${name}/backends/new`);
    await page.getByLabel(/^Backend ID/).fill('mail-primary');
    await page.getByLabel(/^Protocol and service/).selectOption('imap');
    await page.getByLabel(/^Hostname or IP address/).fill('mail.example.test');
    await page.getByLabel(/^Priority/).fill('7');
    await page.getByLabel('Preferred for this service', { exact: true }).check();
    await page.getByLabel('Customize transport security', { exact: true }).check();
    await page.getByLabel(/^Security selection/).selectOption('auto');
    await page.getByLabel(/^Method priority 1/).selectOption('implicit');
    await page.getByLabel(/^Method priority 2/).selectOption('starttls');
    await page.getByLabel(/^TLS server name override/).fill('mail.example.test');
    await page.getByLabel(/^Authentication username format/).selectOption('domainUser');
    await page.getByLabel(/^Authentication domain/).fill('CORP');
    await page.getByLabel(/^Example email identity/).fill(`Alice@${name}`);
    await page.getByRole('button', { name: 'Preview authentication username', exact: true }).click();
    await expect(page.getByText('CORP\\Alice', { exact: true })).toBeVisible();
    const save = page.getByRole('button', { name: 'Save backend', exact: true });
    expect((await save.boundingBox())!.width).toBeLessThan(250);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: test.info().outputPath('domain-backend-editor.png'), fullPage: true });
    await save.click();
    await expect(page).toHaveURL(new RegExp(`#/domains/${name.replaceAll('.', '\\.')}$`));
    const saved = await (await page.request.get(path)).json();
    expect(saved.upstream.backends[0]).toMatchObject({ id: 'mail-primary', priority: 7, preferred: true, username: { format: 'domainUser', domain: 'CORP' }, endpoint: { security: { mode: 'auto', allowed: ['implicit', 'starttls'], allowPlaintext: false, serverName: 'mail.example.test' } } });
    await page.goto(`${ADMIN}#/domains/${name}/backends/mail-primary`);
    await expect(page.getByLabel(/^Backend ID/)).toBeDisabled();
    await expect(page.getByLabel(/^Method priority 2/)).toHaveValue('starttls');
  } finally {
    const current = await page.request.get(path);
    if (current.ok()) {
      const removed = await page.request.delete(path, { headers: { 'if-match': current.headers()['etag']! } });
      expect(removed.ok(), `remove isolated fixture: ${removed.status()}`).toBe(true);
    }
  }
});
