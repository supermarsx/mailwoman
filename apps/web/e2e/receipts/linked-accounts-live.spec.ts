import {test,expect} from '../csrf.ts';
import {connect} from 'node:net';
import type {Page} from '@playwright/test';

// Stack: node apps/web/e2e/receipts/linked-live-fixture.mjs (a server built from
// the tree over a private Greenmail; see that file). MW_RECEIPTS_SMTP is the
// Greenmail SMTP port the fixture prints.
const SMTP=process.env['MW_RECEIPTS_SMTP']??'127.0.0.1:46625';

/** Deliver one message to `to` over plain SMTP (Greenmail accepts any sender). */
async function deliver(to:string,subject:string):Promise<void>{
  const [host,port]=SMTP.split(':');
  await new Promise<void>((resolve,reject)=>{
    const socket=connect(Number(port),host!);
    const steps=['EHLO linked.test','MAIL FROM:<sender@linked.test>',`RCPT TO:<${to}>`,'DATA',
      `From: sender@linked.test\r\nTo: ${to}\r\nSubject: ${subject}\r\nMessage-ID: <${subject}@linked.test>\r\n\r\nLive update.\r\n.`,'QUIT'];
    let buffer='';
    socket.setEncoding('utf8');
    socket.on('data',chunk=>{
      buffer+=chunk;
      const lines=buffer.split('\r\n');buffer=lines.pop()!;
      for(const line of lines){
        if(/^\d{3}-/.test(line))continue;
        if(!/^[23]/.test(line)){socket.destroy();reject(new Error(`smtp: ${line}`));return;}
        const next=steps.shift();
        if(next===undefined){socket.end();resolve();return;}
        socket.write(`${next}\r\n`);
      }
    });
    socket.on('error',reject);
  });
}

/** At phone width the sidebar (picker, Settings) is in the drawer. */
async function drawer(page:Page){
  const open=page.getByRole('button',{name:'Open folders and apps',exact:true});
  if(await open.isVisible()&&await open.getAttribute('aria-expanded')!=='true')await open.click();
}
async function picker(page:Page){
  const trigger=page.locator('button[data-account-indicator="picker"]');
  if(!await trigger.isVisible())await drawer(page);
  await expect(trigger).toBeVisible();
  return trigger;
}
async function openMenu(page:Page){
  const trigger=await picker(page);
  if(await trigger.getAttribute('aria-expanded')!=='true')await trigger.click();
  const menu=page.getByRole('menu',{name:'Mailbox account',exact:true});
  await expect(menu).toBeVisible();
  return menu;
}
async function closeMenu(page:Page){
  const trigger=page.locator('button[data-account-indicator="picker"]');
  if(await trigger.count()>0&&await trigger.getAttribute('aria-expanded')==='true')await page.keyboard.press('Escape');
}
async function badge(page:Page,link:string):Promise<number>{
  const menu=await openMenu(page);
  const count=menu.locator(`[data-mailbox-item][data-link="${link}"] [data-mailbox-unread]`);
  return await count.count()===0?0:Number((await count.first().innerText()).match(/\d+/)?.[0]??0);
}
/** The child's Inbox unread count as the server holds it (what the badge must reach). */
async function serverUnread(page:Page,link:string):Promise<number>{
  const me=await (await page.request.get('/api/me',{headers:{'x-mailwoman-link':link}})).json() as {accountId:string;storageAuthority:string};
  const boxes=await (await page.request.post('/jmap/api',{headers:{'x-mailwoman-link':link,'x-mailwoman-storage-authority':me.storageAuthority},
    data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Mailbox/get',{accountId:me.accountId,properties:['role','unreadEmails']},'b']]}})).json();
  return (boxes.methodResponses[0][1].list as {role?:string;unreadEmails:number}[]).find(box=>box.role==='inbox')!.unreadEmails;
}
async function openSettings(page:Page,section:string){
  await closeMenu(page);
  const settings=page.locator('.sidebar, #shell-nav').getByRole('button',{name:'Settings',exact:true}).first();
  if(!await settings.isVisible())await drawer(page);
  await settings.click();
  await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:section,exact:true}).click();
}
async function choose(page:Page,link:string){
  const menu=await openMenu(page);
  await menu.locator(`[data-mailbox-item][data-link="${link}"]`).click();
  await expect(page.locator('button[data-account-indicator="picker"]')).toHaveAttribute('aria-expanded','false');
}
async function shot(page:Page,info:{outputPath(name:string):string;project:{name:string}},name:string){
  await page.screenshot({path:info.outputPath(name),fullPage:true});
  const dir=process.env['MW_SCREENSHOT_DIR'];
  if(dir)await page.screenshot({path:`${dir}/${info.project.name}-${name}`,fullPage:true});
}

test('native mailbox link switch, live updates, private settings and unlink keep the parent session',async({page},info)=>{
  page.setDefaultTimeout(15000);
  const errors:string[]=[];page.on('pageerror',error=>errors.push(error.message));
  let loads=0;page.on('load',()=>loads++);
  await page.goto('/');
  await page.getByLabel('Email or username',{exact:true}).fill('parent@example.org');
  await page.getByLabel('Password',{exact:true}).fill('parent-pass');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  await expect(page.getByLabel('Email or username',{exact:true})).toHaveCount(0);
  await expect(page.locator('[data-account-indicator]').first()).toBeAttached();
  const primary=await (await page.request.get('/api/me')).json() as {accountId:string;storageAuthority:string;username:string};
  // A run that stopped half-way leaves its link behind; start from one mailbox.
  const ownerHeaders={'x-mailwoman-storage-authority':primary.storageAuthority};
  const stale=(await (await page.request.get('/api/accounts',{headers:ownerHeaders})).json()).accounts as {id:string;revision:string}[];
  if(stale.length){
    expect((await page.request.post('/api/security/step-up',{data:{password:'parent-pass'}})).ok()).toBe(true);
    for(const account of stale)expect((await page.request.delete(`/api/accounts/${account.id}`,{headers:{...ownerHeaders,'if-match':JSON.stringify(account.revision)}})).ok()).toBe(true);
    await page.reload();
  }
  const loadsAtStart=loads;
  await expect(page.locator('[data-account-address]').first()).toBeAttached();
  // One mailbox: the plain address, no picker.
  await expect(page.locator('button[data-account-indicator="picker"]')).toHaveCount(0);

  await openSettings(page,'Linked mailboxes');
  await page.getByRole('button',{name:'Link a mailbox',exact:true}).click();
  const editor=page.getByRole('region',{name:'Link a mailbox',exact:true});
  await editor.getByLabel('Username or email address',{exact:true}).fill('child@child.test');
  await editor.getByLabel('Password',{exact:true}).fill('child-pass');
  await editor.getByLabel('Display name (optional)',{exact:true}).fill(`Child ${info.project.name}`);
  await editor.getByRole('button',{name:'Verify and link',exact:true}).click();
  // The step-up dialog appears unless a recent confirmation still holds (the
  // stale-link cleanup above confirms once).
  const proof=page.getByRole('dialog',{name:'Confirm it is you'});
  if(!stale.length){
    await expect(proof).toBeVisible();
    await proof.getByLabel('Password',{exact:true}).fill('parent-pass');
    await proof.getByRole('button',{name:'Confirm and continue',exact:true}).click();
  }
  await expect(page.getByRole('heading',{name:'Linked mailboxes',exact:true})).toBeVisible();
  const listing=await (await page.request.get('/api/accounts')).json() as {accounts:{id:string;revision:string;address?:string;status:string}[];policy:{childRealtime?:boolean}};
  const linked=listing.accounts.find(account=>account.address==='child@child.test');expect(linked).toBeTruthy();
  expect(linked!.status).toBe('ready');
  expect(listing.policy.childRealtime).toBe(true);
  const link=linked!.id;
  await page.keyboard.press('Escape');

  // Live, while the parent is open: mail delivered to the child moves the
  // child's count in the picker, with no reload.
  const before=await serverUnread(page,link);
  const stamp=`${info.project.name}-${Date.now()}`;
  const first=`Linked live ${stamp} one`;
  await deliver('child@child.test',first);
  await expect(async()=>{expect(await badge(page,link)).toBe(before+1);}).toPass({timeout:30000});
  await shot(page,info,'linked-picker-live-count.png');
  await expect(page.getByText('Live updates are unavailable',{exact:false})).toHaveCount(0);

  // A real slow account switch reports its destination through one shared spinner.
  await page.route('**/jmap/session',async route=>{await new Promise(done=>setTimeout(done,1200));await route.continue();},{times:1});
  await choose(page,link);
  const loading=page.getByTestId('mail-loading-toast');
  await expect(loading).toHaveCount(1);await expect(loading).toContainText('child@child.test');
  await expect(loading.locator('[data-toast-spinner]')).toBeVisible();
  await shot(page,info,'linked-switch-loading.png');
  await expect(loading).toHaveCount(0,{timeout:20000});
  await expect(page.getByText(first).first()).toBeVisible({timeout:20000});
  // Live while the child is open: a second message appears without a reload.
  const second=`Linked live ${stamp} two`;
  await deliver('child@child.test',second);
  await expect(page.getByText(second).first()).toBeVisible({timeout:30000});
  await expect(page.getByText('Live updates are unavailable',{exact:false})).toHaveCount(0);
  await shot(page,info,'linked-child-live-list.png');

  const effective=await (await page.request.get('/api/me',{headers:{'x-mailwoman-link':link}})).json() as {accountId:string;storageAuthority:string;username:string};
  expect(effective.username).toBe('child@child.test');expect(effective.storageAuthority).not.toBe(primary.storageAuthority);
  expect((await (await page.request.get('/api/me')).json()).username).toBe(primary.username);
  const childHeaders={'x-mailwoman-link':link,'x-mailwoman-storage-authority':effective.storageAuthority};
  const signatureName=`linked-${info.project.name}`;
  const signature=await page.request.post('/api/account/signatures',{headers:childHeaders,data:{name:signatureName,body:'Only this child',text:'Only this child',isDefault:true}});expect(signature.ok()).toBe(true);
  const childIdentityName='Child identity '+info.project.name;
  const identityWrite=await page.request.post('/api/account/identities',{headers:childHeaders,data:{id:'',name:childIdentityName,email:'child@child.test',signatureName}});
  expect(identityWrite.ok(),await identityWrite.text()).toBe(true);
  const identities=await page.request.get('/api/account/identities',{headers:childHeaders});expect(identities.ok()).toBe(true);
  const childIdentities=(await identities.json()).identities as {email:string}[];
  expect(childIdentities.every(identity=>identity.email!=='parent@example.org')).toBe(true);
  expect(childIdentities.some(identity=>identity.email==='child@child.test')).toBe(true);
  const discovery=await (await page.request.get('/jmap/session',{headers:childHeaders})).json();
  const boxes=await (await page.request.post('/jmap/api',{headers:childHeaders,data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Mailbox/get',{accountId:effective.accountId},'b']]}})).json();
  const inbox=boxes.methodResponses[0][1].list.find((box:{role?:string})=>box.role==='inbox');expect(inbox).toBeTruthy();
  expect(discovery.primaryAccounts['urn:ietf:params:jmap:mail']).toBe(effective.accountId);
  const mapping=await page.request.put('/api/settings/mail.folderRole.inbox',{headers:childHeaders,data:{value:{id:inbox.id}}});expect(mapping.ok(),String(mapping.status())+' '+await mapping.text()).toBe(true);
  await openSettings(page,'Signatures');
  await expect(page.getByRole('region',{name:'Signatures',exact:true}).getByRole('listitem').filter({hasText:signatureName})).toBeVisible();
  await shot(page,info,'linked-child-settings.png');
  await page.keyboard.press('Escape');

  // Back on the parent: the child's count still moves live.
  await page.route('**/jmap/session',async route=>{await new Promise(done=>setTimeout(done,1200));await route.continue();},{times:1});
  await choose(page,'');
  await expect(loading).toHaveCount(1);await expect(loading).toContainText('parent@example.org');
  await expect(loading).toHaveCount(0,{timeout:20000});
  const parentBefore=await serverUnread(page,link);
  await deliver('child@child.test',`Linked live ${stamp} three`);
  await expect(async()=>{expect(await badge(page,link)).toBe(parentBefore+1);}).toPass({timeout:30000});
  await closeMenu(page);
  expect(loads,'no page reload during switching and live updates').toBe(loadsAtStart);

  const parentSignatures=(await (await page.request.get('/api/account/signatures')).json()).signatures as {name:string}[];
  expect(parentSignatures.some(signature=>signature.name===signatureName)).toBe(false);
  const parentIdentities=(await (await page.request.get('/api/account/identities')).json()).identities as {name:string}[];
  expect(parentIdentities.some(identity=>identity.name===childIdentityName)).toBe(false);
  const parentSettings=await (await page.request.get('/api/settings')).json();
  expect(JSON.stringify(parentSettings)).not.toContain(inbox.id);
  await openSettings(page,'Linked mailboxes');
  // t48: unlink is an icon button and confirms in the shared ConfirmDialog.
  await page.getByRole('button',{name:/^Unlink .*child@child\.test/}).click();
  const unlinkDialog=page.getByRole('alertdialog',{name:/^Unlink \W?child@child\.test\W?\?$/});
  await expect(unlinkDialog.getByText('This removes access through your primary account. It does not delete the mailbox or its messages.')).toBeVisible();
  await unlinkDialog.getByRole('button',{name:'Confirm unlink',exact:true}).click();
  await page.keyboard.press('Escape');
  await expect(page.locator('button[data-account-indicator="picker"]')).toHaveCount(0);
  expect((await page.request.get('/api/me',{headers:{'x-mailwoman-link':link}})).status()).toBe(409);
  expect((await (await page.request.get('/api/me')).json()).username).toBe(primary.username);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
  await shot(page,info,'linked-primary-unlinked.png');
  expect(errors).toEqual([]);
});
