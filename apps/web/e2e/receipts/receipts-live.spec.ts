import {test,expect} from '../csrf.ts';
import {messageRow,postJmap} from '../helpers.ts';
// Email/import (the seeded requests) needs UIDPLUS or IMAP4rev2: on Greenmail the
// imported message never reaches the list, so this runs on the receipts stack's
// Dovecot leg (MW_STACK_IMAP_SERVER=dovecot) and skips on the Greenmail one.
test.skip(process.env['MW_RECEIPTS_IMAP_SERVER']!=='dovecot','needs UIDPLUS for Email/import: runs on the Dovecot leg of scripts/e2e-receipts-stack.mjs');
// The identity is the account's own authorized address: since sending-identities
// enforcement an identity for any other address is refused 403 forbiddenFrom.
test('saves receipt policy and responds only to displayed requested messages once',async({page},info)=>{
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');
 const login=page.waitForResponse(response=>response.url().endsWith('/api/login')&&response.request().method()==='POST');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 const identity=await (await login).json() as {accountId:string;storageAuthority:string};expect(identity.accountId).toBeTruthy();expect(identity.storageAuthority).toBeTruthy();
 const saveIdentity=await page.request.post('/api/account/identities',{headers:{'x-mailwoman-account':identity.accountId,'x-mailwoman-storage-authority':identity.storageAuthority},data:{name:'Receipt fixture',email:'testuser@example.org'}});expect(saveIdentity.ok()).toBe(true);
 const call=async(name:string,args:Record<string,unknown>,expectSuccess=true)=>{
  const response=await postJmap(page.request,{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[[name,{accountId:identity.accountId,...args},'test']]}});expect(response.ok()).toBe(true);
  const value=await response.json() as {methodResponses:[string,Record<string,unknown>,string][]};if(expectSuccess)expect(value.methodResponses[0]?.[0]).toBe(name);return value.methodResponses[0]![1];
 };
 const boxes=await call('Mailbox/get',{});const inbox=(boxes['list'] as {id:string;role:string}[]).find(box=>box.role==='inbox')!;expect(inbox).toBeTruthy();
 const seed=async(label:string)=>{
  const subject=`Receipt ${label} ${info.project.name} ${Date.now()}`;
  const raw=`Return-Path: <receipt@example.org>\r\nFrom: receipt@example.org\r\nTo: testuser@example.org\r\nSubject: ${subject}\r\nMessage-ID: <${Date.now()}-${label}@receipt.invalid>\r\nDisposition-Notification-To: receipt@example.org\r\nContent-Type: text/plain\r\n\r\nDisplayed receipt browser fixture.\r\n`;
  const upload=await page.request.post(`/jmap/upload/${identity.accountId}`,{data:raw,headers:{'Content-Type':'message/rfc822'}});expect(upload.ok()).toBe(true);const blob=await upload.json() as {blobId:string};
  const imported=await call('Email/import',{emails:{message:{blobId:blob.blobId,mailboxIds:{[inbox.id]:true}}}});const id=(imported['created'] as Record<string,{id:string}>)['message']!.id;
  return {subject,id};
 };
 const manual=await seed('manual');const automatic=await seed('automatic');
 const initial=await page.request.put('/api/account/mdn-policy',{data:{policy:'ask'}});expect(initial.ok()).toBe(true);
 await page.reload();await expect(page.getByRole('region',{name:'Messages',exact:true})).toHaveAttribute('aria-busy','false');await messageRow(page,manual.subject).click();
 const send=page.getByRole('button',{name:'Send read receipt',exact:true});await expect(send).toBeVisible();
 await page.waitForTimeout(1800);const before=await call('Email/get',{ids:[manual.id],properties:['mailwomanMdn']});expect(((before['list'] as Record<string,unknown>[])[0]!['mailwomanMdn'] as {sent:boolean}).sent).toBe(false);
 const sent=page.waitForResponse(response=>response.url().endsWith('/jmap/api')&&(response.request().postData()??'').includes('MDN/send'));await send.click();const accepted=await (await sent).json();expect(accepted.methodResponses[0][1].sent).toBe(true);await expect(send).not.toBeVisible();
 await page.screenshot({path:info.outputPath('manual-receipt-displayed.png'),fullPage:true});
 if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
 await page.getByRole('button',{name:'Settings',exact:true}).click();await page.getByRole('button',{name:'Read receipts',exact:true}).click();await page.getByRole('button',{name:'Edit receipt responses'}).click();
 await page.getByLabel('Response policy').selectOption('always');await page.getByRole('button',{name:'Save receipt policy'}).click();await expect(page.getByRole('heading',{name:'Read receipts',exact:true})).toBeVisible();
 await page.screenshot({path:info.outputPath('receipt-settings.png'),fullPage:true});
 const autoSent=page.waitForResponse(response=>response.url().endsWith('/jmap/api')&&(response.request().postData()??'').includes('MDN/send'));
 // Settings is a retained workspace tab (f1a656bb): the reload restores it, so go back to Mail.
 await page.reload();await page.getByRole('tab',{name:'Mail',exact:true}).click();await expect(page.getByRole('region',{name:'Messages',exact:true})).toHaveAttribute('aria-busy','false');await messageRow(page,automatic.subject).click();const autoResult=await (await autoSent).json();expect(autoResult.methodResponses[0][1].sent).toBe(true);expect(autoResult.methodResponses[0][1].automatic).toBe(true);
 const again=await call('MDN/send',{emailId:automatic.id,automatic:true},false);expect(again['type']).toBe('mdnAlreadySent');
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);await page.screenshot({path:info.outputPath('automatic-receipt-once.png'),fullPage:true});
});
