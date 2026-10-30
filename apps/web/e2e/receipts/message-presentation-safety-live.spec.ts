import {test,expect} from '../csrf.ts';
import type {Page} from '@playwright/test';
import {injectViaSmtp,messageRow,postJmap} from '../helpers.ts';

async function signIn(page:Page):Promise<void>{
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('searchbox',{name:'Search mail'})).toBeVisible();
}
async function messageInfo(page:Page,subject:string):Promise<{downloadUrl:string;accountId:string;email:{id:string;blobId:string}}>{
 const session=await(await page.request.get('/jmap/session')).json();const accountId=session.primaryAccounts['urn:ietf:params:jmap:mail'];
 const response=await postJmap(page.request,{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Email/query',{accountId,filter:{text:subject}},'q'],['Email/get',{accountId,'#ids':{resultOf:'q',name:'Email/query',path:'/ids'},properties:['id','subject','blobId']},'g']]}});
 const email=(await response.json()).methodResponses[1][1].list.find((value:{subject:string})=>value.subject===subject);expect(email).toBeTruthy();
 return {downloadUrl:session.downloadUrl,accountId,email};
}
async function rawMessage(page:Page,subject:string):Promise<string>{
 const {downloadUrl,accountId,email}=await messageInfo(page,subject);
 const url=downloadUrl.replace('{accountId}',encodeURIComponent(accountId)).replace('{blobId}',encodeURIComponent(email.blobId)).replace('{name}','message.eml').replace('{type}',encodeURIComponent('message/rfc822'));
 const original=await page.request.get(url);expect(original.ok()).toBe(true);return original.text();
}
async function readerBody(page:Page){
 const reader=page.locator('section.reader').filter({visible:true}).first();const iframe=reader.locator('iframe.reader__frame');await expect(iframe).toBeVisible();
 const outer=iframe.contentFrame();return (await iframe.getAttribute('src'))?.includes('/api/rc/frame')?outer.frameLocator('iframe'):outer;
}

test('hostile message HTML is nullified while safe text and original bytes remain available',async({page},info)=>{
 const subject=`Hostile body ${info.project.name} ${Date.now()}`;const benign=`Benign body ${info.project.name} ${Date.now()}`;
 const hostile='<p><strong>Safe fallback text</strong></p><script>window.hostileExecuted=true</script><img src="https://hostile.invalid/pixel" onerror="window.hostileExecuted=true"><a href="javascript:alert(1)">Unsafe destination</a>';
 await injectViaSmtp({from:'safety@example.org',subject,text:'Safe fallback text',html:hostile,requestReceipt:true});await injectViaSmtp({from:'safety@example.org',subject:benign,text:'Benign message works',html:'<p>Benign message works: data-mw-body-blocked="true"</p>'});
 await signIn(page);await expect(messageRow(page,subject)).toBeVisible({timeout:60000});
 const policy=(await(await page.request.get('/api/account/mdn-policy')).json()).policy;
 const requested=await messageInfo(page,subject);const authority=(await(await page.request.get('/api/me')).json()).storageAuthority;const identityHeaders={'x-mailwoman-account':requested.accountId,'x-mailwoman-storage-authority':authority};let createdIdentity:string|undefined;
 const identities=await page.request.get('/api/account/identities',{headers:identityHeaders});expect(identities.ok()).toBe(true);
 if(!(await identities.json()).identities.some((identity:{email:string})=>identity.email==='testuser@example.org')){const name=`Body safety ${Date.now()}`;expect((await page.request.post('/api/account/identities',{headers:identityHeaders,data:{name,email:'testuser@example.org'}})).ok()).toBe(true);const list=(await(await page.request.get('/api/account/identities',{headers:identityHeaders})).json()).identities;createdIdentity=list.find((identity:{name:string})=>identity.name===name)?.id;expect(createdIdentity).toBeTruthy();}
 expect((await page.request.put('/api/account/mdn-policy',{data:{policy:'always'}})).ok()).toBe(true);
 try{
 const decision=await page.request.get(`/api/account/mdn-decision/${encodeURIComponent(requested.email.id)}`);expect(decision.ok()).toBe(true);expect((await decision.json()).action).toBe('automatic');
 const forbidden:string[]=[];page.on('request',request=>{const url=new URL(request.url());if(url.hostname==='hostile.invalid'||url.pathname==='/api/rc/sign')forbidden.push(url.pathname);if(request.method()==='POST'&&url.pathname==='/jmap/api'&&request.postData()?.includes('MDN/send'))forbidden.push('MDN/send');});
 await messageRow(page,subject).click();await expect(page.getByText('Message HTML blocked for safety',{exact:true})).toBeVisible();
 await expect(page.getByRole('button',{name:'Enable links',exact:true})).toHaveCount(0);
 const body=await readerBody(page);await expect(body.locator('script,img,a[href],object,embed,form')).toHaveCount(0);await expect(body.locator('body')).toContainText('Safe fallback text');expect(forbidden).toEqual([]);
 const raw=await rawMessage(page,subject);expect(raw).toContain(hostile);expect(raw).toContain('Disposition-Notification-To: safety@example.org');
 await page.waitForTimeout(1000);expect(forbidden).toEqual([]);
 await page.screenshot({path:info.outputPath('hostile-body-nullified.png'),fullPage:true});
 await page.getByRole('button',{name:'Show sanitized HTML',exact:true}).click();
 const safeBody=await readerBody(page);await expect(safeBody.locator('strong')).toHaveText('Safe fallback text');
 await expect(safeBody.locator('script,img[src],a[href],object,embed,form')).toHaveCount(0);expect(forbidden).toEqual([]);
 await expect(page.getByRole('button',{name:'Enable links',exact:true})).toHaveCount(0);
 await page.screenshot({path:info.outputPath('hostile-body-sanitized-consent.png'),fullPage:true});
 await page.getByRole('button',{name:'Show safe text',exact:true}).click();
 await expect((await readerBody(page)).locator('strong')).toHaveCount(0);expect(forbidden).toEqual([]);

 await page.getByRole('tab',{name:'Mail',exact:true}).click();await messageRow(page,benign).click();await expect((await readerBody(page)).locator('body')).toContainText('Benign message works: data-mw-body-blocked="true"');await expect(page.locator('section.reader').filter({visible:true}).first().getByTestId('reader-body-blocked')).toHaveCount(0);
 }finally{expect((await page.request.put('/api/account/mdn-policy',{data:{policy}})).ok()).toBe(true);if(createdIdentity)expect((await page.request.delete(`/api/account/identities/${encodeURIComponent(createdIdentity)}`,{headers:identityHeaders})).ok()).toBe(true);}
});

test('leetspeak changes interface and reading display while reply text and stored source stay original',async({page},info)=>{
 const subject=`Readable message ${info.project.name} ${Date.now()}`;const originalText='Readable message body';
 await injectViaSmtp({from:'display@example.org',subject,text:originalText,html:`<p>${originalText}</p>`});await signIn(page);await expect(messageRow(page,subject)).toBeVisible({timeout:60000});const original=await rawMessage(page,subject);
 const authority=(await(await page.request.get('/api/me')).json()).storageAuthority;
 try{
  if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps',exact:true}).click();
  await page.locator('.sidebar, #shell-nav').getByRole('button',{name:'Settings',exact:true}).first().click();await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Appearance',exact:true}).click();
  const mode=page.getByTestId('text-mode-preference');await expect(mode).toBeVisible();await mode.getByRole('button',{name:'Leetspeak (1337)',exact:true}).click();await expect(mode.getByRole('button',{name:'Leetspeak (1337)',exact:true})).toHaveAttribute('aria-pressed','true');await expect(mode.getByRole('button',{name:'Leetspeak (1337)',exact:true})).toBeEnabled();await page.getByRole('region',{name:'53771n95',exact:true}).getByRole('button',{name:'Cl053 53771n95',exact:true}).click();
  await page.getByRole('tab',{name:'M41l',exact:true}).click();const row=messageRow(page,subject.slice(subject.lastIndexOf(' ')+1));await expect(row).toBeVisible();await expect(row).toContainText('R34d48l3 m355493');await row.click();
  await expect((await readerBody(page)).locator('body')).toContainText('R34d48l3 m355493 80dy');expect(await rawMessage(page,subject)).toBe(original);
  await page.getByTestId('reader-reply').click();const editor=page.getByRole('textbox',{name:'80dy',exact:true});await expect(editor).toBeVisible();await expect.poll(()=>editor.evaluate(node=>node instanceof HTMLTextAreaElement?node.value:node.textContent??'')).toContain(originalText);
  await page.screenshot({path:info.outputPath('leetspeak-original-reply.png'),fullPage:true});
 }finally{const reset=await page.request.put('/api/settings/appearance.leetspeak',{headers:{'x-mailwoman-storage-authority':authority},data:{value:false}});expect(reset.ok()).toBe(true);}
});
