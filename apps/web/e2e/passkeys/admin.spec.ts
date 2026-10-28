import { test,expect } from '../csrf.ts';
import { ceremony } from './ceremonies.ts';
import { disableSigninCooldown } from '../helpers.ts';

test('administrator passkeys use real browser UI, require factors and support one-use recovery',async({page,context,playwright,baseURL})=>{
  // Earlier specs on the same stack sign in wrongly on purpose, which starts the
  // per-IP sign-in cooldown (failed attempts only, 63d344c0); the UI sign-in below
  // would then get 429. Set the policy to 0 first (this spec did so after it).
  await disableSigninCooldown(playwright.request, baseURL!);
  const cdp=await context.newCDPSession(page);await cdp.send('WebAuthn.enable');
  await cdp.send('WebAuthn.addVirtualAuthenticator',{options:{protocol:'ctap2',ctap2Version:'ctap2_1',transport:'internal',hasResidentKey:true,hasUserVerification:true,isUserVerified:true,automaticPresenceSimulation:true,defaultBackupEligibility:true,defaultBackupState:true}});
  await page.goto('/admin');
  await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');await page.getByRole('button',{name:'Sign in',exact:true}).click();
  await expect.poll(async()=> (await page.request.get('/admin/session')).status()).toBe(200);
  const cooldown=await page.request.get('/admin/settings/signin.retryCooldownSecs');expect(cooldown.ok()).toBe(true);
  expect((await page.request.put('/admin/settings/signin.retryCooldownSecs',{headers:{'if-match':cooldown.headers()['etag']!},data:{globalValue:0}})).ok()).toBe(true);
  await page.goto('/admin#/preferences');
  const section=page.getByRole('region',{name:'Administrator passkeys',exact:true});
  await section.getByLabel('Passkey name').fill('Real browser admin key');
  await section.getByRole('button',{name:'Add administrator passkey'}).click();
  const prompt=page.getByRole('dialog',{name:'Confirm it is you'});
  await prompt.getByLabel('Password',{exact:true}).fill('hunter2');
  const saved=page.waitForResponse(r=>r.url().endsWith('/me/passkeys/finish'));
  await prompt.getByRole('button',{name:'Confirm and continue',exact:true}).click();
  const enrollment=await saved;expect(enrollment.status()).toBe(200);
  const codes=(await enrollment.json()).recoveryCodes as string[];expect(codes.length).toBeGreaterThan(1);
  await expect(section.getByText('Real browser admin key',{exact:true})).toBeVisible();
  await section.getByRole('button',{name:'I have saved the codes'}).click();
  await section.screenshot({path:test.info().outputPath('administrator-passkeys.png')});
  expect((await page.request.post('/admin/step-up',{data:{password:'hunter2'}})).status()).toBe(422);
  const key=(await (await page.request.get('/admin/me/passkeys')).json()).passkeys[0];
  // Mailbox passkey lookup must never accept administrator credentials.
  const mailBegin=await (await page.request.post('/api/login/passkey/begin',{data:{}})).json();
  const mailProof=await ceremony(page,{...mailBegin,credentialIds:[key.credentialId]},false);
  expect((await page.request.post('/api/login/passkey/finish',{data:{...mailProof,pendingToken:mailBegin.pendingToken}})).status()).toBe(401);
  await page.request.post('/admin/logout',{data:{}});
  const password=await page.request.post('/admin/login',{data:{username:'root',password:'hunter2'}});
  const challenge=await password.json();expect(challenge.twofaRequired).toBe(true);expect(challenge.factors).toContain('webauthn');
  expect((await page.request.get('/admin/session')).status()).toBe(401);
  expect((await page.request.post('/admin/login/recovery',{data:{pendingToken:challenge.pendingToken,code:codes[0]}})).status()).toBe(200);
  await page.request.post('/admin/logout',{data:{}});
  const again=await (await page.request.post('/admin/login',{data:{username:'root',password:'hunter2'}})).json();
  expect((await page.request.post('/admin/login/recovery',{data:{pendingToken:again.pendingToken,code:codes[0]}})).status()).toBe(401);
  // Missing userHandle and reused discovery challenges are both refused.
  const begin=await (await page.request.post('/admin/login/passkey/begin',{data:{}})).json();
  const proof=await ceremony(page,begin,false);
  expect((await page.request.post('/admin/login/passkey/finish',{data:{...proof,userHandle:null,pendingToken:begin.pendingToken}})).status()).toBe(401);
  expect((await page.request.post('/admin/login/passkey/finish',{data:{...proof,pendingToken:begin.pendingToken}})).status()).toBe(401);
  await context.clearCookies(); // A fresh browser has neither an auth nor a CSRF cookie.
  expect((await page.request.post('/admin/login/passkey/begin',{data:{},headers:{Origin:'https://attacker.invalid'}})).status()).toBe(403);
  await page.goto('/admin');
  const passwordlessBegin=page.waitForResponse(r=>r.url().endsWith('/login/passkey/begin'));
  await page.getByRole('button',{name:'Sign in with a passkey',exact:true}).click();
  expect((await passwordlessBegin).status()).toBe(200);
  await expect(page.getByRole('button',{name:'Sign in with a passkey',exact:true})).toHaveCount(0);
  await page.goto('/admin#/preferences');
  await section.getByRole('button',{name:'Replace recovery codes'}).click();
  await prompt.getByRole('button',{name:'Use a passkey'}).click();
  await expect(section.getByRole('button',{name:'I have saved the codes'})).toBeVisible();
  // Rotation invalidates earlier unused codes even with the correct password.
  expect((await page.request.post('/admin/step-up',{data:{password:'hunter2',recoveryCode:codes[1]}})).status()).toBe(403);
  await section.getByRole('button',{name:'I have saved the codes'}).click();
  const removed=page.waitForResponse(r=>r.url().endsWith('/me/passkeys/remove'));
  await section.getByRole('button',{name:'Remove passkey'}).click();expect((await removed).status()).toBe(200);
  await expect(section.getByText('Real browser admin key',{exact:true})).toHaveCount(0);
  // The administrator-management reset revokes sessions as well as credentials.
  await section.getByLabel('Passkey name').fill('Managed reset key');
  await section.getByRole('button',{name:'Add administrator passkey'}).click();
  await prompt.getByLabel('Password',{exact:true}).fill('hunter2');
  await prompt.getByRole('button',{name:'Confirm and continue',exact:true}).click();
  await expect(section.getByText('Managed reset key',{exact:true})).toBeVisible();
  const accounts=(await (await page.request.get('/admin/admins')).json()).admins;
  const root=accounts.find((a:{username:string})=>a.username==='root');expect(root.passkeyCount).toBe(1);
  await page.goto(`/admin#/admins/${root.id}/edit`);
  await page.getByRole('button',{name:'Revoke all passkeys'}).click();
  const confirmation=page.getByRole('alertdialog');
  await confirmation.getByRole('textbox').fill('root');
  await confirmation.getByRole('button',{name:'Confirm change'}).click();
  const reset=page.waitForResponse(r=>r.url().endsWith(`/admins/${root.id}/passkeys`)&&r.status()===204);
  await prompt.getByRole('button',{name:'Use a passkey'}).click();
  await reset;
  expect((await page.request.get('/admin/session')).status()).toBe(401);
  await page.request.post('/admin/logout',{data:{}});
  const revokedBegin=await (await page.request.post('/admin/login/passkey/begin',{data:{}})).json();
  const revoked=await ceremony(page,revokedBegin,false);
  expect((await page.request.post('/admin/login/passkey/finish',{data:{...revoked,pendingToken:revokedBegin.pendingToken}})).status()).toBe(401);
});
