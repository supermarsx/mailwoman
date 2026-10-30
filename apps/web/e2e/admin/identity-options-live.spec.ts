import {test,expect} from '../csrf.ts';
import type {Identity} from '../../src/screens/Settings/types.ts';

test('identity options preserve signature, Reply-To, organization, Bcc and an owned OpenPGP key after reload @phone',async({page},info)=>{
 test.setTimeout(180000);
 const tag=`identity-${info.project.name}-${Date.now()}`,email=`${tag}@example.org`,signature=`Signature ${tag}`;
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill(email);await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('region',{name:'Messages',exact:true})).toBeVisible();
 const me=await(await page.request.get('/api/me')).json() as {accountId:string;storageAuthority:string};const headers={'x-mailwoman-storage-authority':me.storageAuthority};
 let identityId='',keyId='';
 const drawer=async()=>{const open=page.getByRole('button',{name:'Open folders and apps',exact:true});if(await open.isVisible()&&await open.getAttribute('aria-expanded')!=='true')await open.click();};
 const jmap=async(method:string,args:Record<string,unknown>)=>{const response=await page.request.post('/jmap/api',{data:{using:['urn:ietf:params:jmap:core','urn:mailwoman:crypto','urn:mailwoman:security'],methodCalls:[[method,{accountId:me.accountId,...args},'key']]}});expect(response.ok()).toBe(true);const body=await response.json() as {methodResponses:[string,Record<string,unknown>,string][]};expect(body.methodResponses[0]?.[0]).toBe(method);return body.methodResponses[0]![1];};
 try{
  expect((await page.request.post('/api/account/signatures',{headers,data:{name:signature,body:'Identity signature',isDefault:false}})).ok()).toBe(true);
  await drawer();await page.getByTestId('nav-keys').click();await page.getByRole('button',{name:'Generate key',exact:true}).click();
  const dialog=page.getByRole('dialog',{name:'Generate a key'});await dialog.getByLabel('Key type').selectOption('pgp');await dialog.getByLabel('Email',{exact:true}).fill(email);await dialog.getByLabel('Key passphrase',{exact:true}).fill('identity-fixture-passphrase');await dialog.getByRole('button',{name:'Generate',exact:true}).click();await expect(dialog).toBeHidden({timeout:60000});
  const ids=(await jmap('CryptoKey/query',{}))['ids'];const keys=(await jmap('CryptoKey/get',{ids,properties:['id','addresses','isOwn','kind']}))['list'] as {id:string;addresses:string[];isOwn:boolean;kind:string}[];keyId=keys.find(key=>key.isOwn&&key.kind==='pgp'&&key.addresses.includes(email))?.id??'';expect(keyId).not.toBe('');
  const openIdentities=async()=>{await drawer();await page.locator('.sidebar, #shell-nav').getByRole('button',{name:'Settings',exact:true}).first().click();await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Identities',exact:true}).click();};
  await openIdentities();await page.getByRole('button',{name:'New identity',exact:true}).click();
  await page.getByLabel('Display name',{exact:true}).fill(tag);await page.getByLabel('Email address',{exact:true}).fill(email);await page.getByLabel('Reply-to address (optional)',{exact:true}).fill('reply@example.org');await page.getByLabel('Organization',{exact:true}).fill('Example organization');await page.getByLabel('Automatically Bcc',{exact:true}).fill('"Archive, Team" <archive@example.org>, audit@example.org');await page.getByLabel('Default signature',{exact:true}).selectOption(signature);await expect(page.getByLabel('OpenPGP key',{exact:true}).locator(`option[value="${keyId}"]`)).toBeAttached();await page.getByLabel('OpenPGP key',{exact:true}).selectOption(keyId);await page.getByTestId('identity-save').click();
  await expect(page.getByTestId('identity-list')).toContainText(tag);
  const records=await(await page.request.get('/api/account/identities')).json() as {identities:Identity[]};const stored=records.identities.find(value=>value.name===tag)!;identityId=stored.id;expect(stored).toMatchObject({email,replyTo:'reply@example.org',organization:'Example organization',autoBcc:'"Archive, Team" <archive@example.org>, audit@example.org',signatureName:signature,pgpKeyId:keyId});
  await page.reload();await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();await openIdentities();await page.getByTestId('identity-list').locator('li').filter({hasText:tag}).getByRole('button',{name:'Edit',exact:true}).click();
  await expect(page.getByLabel('Organization',{exact:true})).toHaveValue('Example organization');await expect(page.getByLabel('Automatically Bcc',{exact:true})).toHaveValue('"Archive, Team" <archive@example.org>, audit@example.org');await expect(page.getByLabel('Reply-to address (optional)',{exact:true})).toHaveValue('reply@example.org');await expect(page.getByLabel('Default signature',{exact:true})).toHaveValue(signature);await expect(page.getByLabel('OpenPGP key',{exact:true})).toHaveValue(keyId);
  await expect(page.getByLabel('OpenPGP key',{exact:true}).locator('option:checked')).toContainText(email);
  await page.screenshot({path:info.outputPath('identity-options.png'),fullPage:true});expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
 }finally{
  if(identityId)expect((await page.request.delete(`/api/account/identities/${encodeURIComponent(identityId)}`,{headers})).ok()).toBe(true);
  expect((await page.request.delete(`/api/account/signatures/${encodeURIComponent(signature)}`,{headers})).ok()).toBe(true);
  if(keyId){const result=await jmap('CryptoKey/set',{destroy:[keyId]});expect(result['destroyed']).toContain(keyId);}
 }
});
