import {test,expect} from '../csrf.ts';
import type {Browser,Page} from '@playwright/test';
import {revealRibbonControl} from '../helpers.ts';

// t87-e4 (D11): a linked mailbox signs with ITS OWN key, unlocked with its own
// passphrase, and the parent's unlocked key is dropped on the switch.
//
// Stack: node scripts/e2e-receipts-stack.mjs (t46/t48; parent@example.org and
// child@child.test on Greenmail, mailbox linking on). The message the client
// hands the server to send is taken from the wire, and its OpenPGP signature's
// issuer is compared with the two accounts' key fingerprints. (The stack's
// submission fixture accepts the configured parent and child credentials; the
// bytes checked are the ones submitted.)
const PARENT_PASS='parent-key-pass-t87';
const CHILD_PASS='child-key-pass-t87';

/** The armored OpenPGP signature in a message, through QP or base64 transfer encodings. */
function signatureArmor(raw:string):string|null{
  const find=(text:string)=>/-----BEGIN PGP SIGNATURE-----[\s\S]*?-----END PGP SIGNATURE-----/.exec(text)?.[0]??null;
  const direct=find(raw);if(direct)return direct;
  const qp=raw.replace(/=\r?\n/g,'').replace(/=([0-9A-F]{2})/g,(_,h:string)=>String.fromCharCode(parseInt(h,16)));
  const fromQp=find(qp);if(fromQp)return fromQp;
  for(const part of raw.split(/\r?\n\r?\n/)){
    const compact=part.replace(/\s+/g,'');
    if(compact.length<80||!/^[A-Za-z0-9+/=]+$/.test(compact))continue;
    const text=Buffer.from(compact,'base64').toString('latin1');
    const hit=find(text);if(hit)return hit;
  }
  return null;
}

/** The issuer fingerprint (subpacket 33) or key id (subpacket 16) of an armored
 *  v4/v6 OpenPGP signature, upper-case hex. */
function signatureIssuers(armor:string):string[]{
  const body=armor.split(/\r?\n\r?\n/)[1]!.split(/\r?\n=/)[0]!.replace(/-----END PGP SIGNATURE-----/,'').replace(/\s+/g,'');
  const bytes=Buffer.from(body,'base64');
  let at=0;const tagByte=bytes[at++]!;let length:number;
  if(tagByte&0x40){const first=bytes[at++]!;
    if(first<192)length=first;else if(first<224)length=((first-192)<<8)+bytes[at++]!+192;else{length=bytes.readUInt32BE(at);at+=4;}}
  else{const type=tagByte&3;length=type===0?bytes[at++]!:type===1?bytes.readUInt16BE((at+=2)-2):bytes.readUInt32BE((at+=4)-4);}
  const packet=bytes.subarray(at,at+length);
  const version=packet[0]!;
  const issuers:string[]=[];
  const subpackets=(area:Buffer)=>{let i=0;while(i<area.length){let len=area[i++]!;
    if(len>=192&&len<255){len=((len-192)<<8)+area[i++]!+192;}else if(len===255){len=area.readUInt32BE(i);i+=4;}
    const type=area[i]!&0x7f;const data=area.subarray(i+1,i+len);
    if(type===33)issuers.push(data.subarray(1).toString('hex').toUpperCase());
    if(type===16)issuers.push(data.toString('hex').toUpperCase());
    i+=len;}};
  const wide=version===6;let p=4;
  const hashedLen=wide?packet.readUInt32BE(p):packet.readUInt16BE(p);p+=wide?4:2;
  subpackets(packet.subarray(p,p+hashedLen));p+=hashedLen;
  const unhashedLen=wide?packet.readUInt32BE(p):packet.readUInt16BE(p);p+=wide?4:2;
  subpackets(packet.subarray(p,p+unhashedLen));
  return issuers;
}

/** Own key fingerprints and key ids of an account (optionally a linked one). */
async function ownKeyIds(page:Page,headers:Record<string,string>):Promise<string[]>{
  const me=await (await page.request.get('/api/me',{headers})).json() as {accountId:string;storageAuthority:string};
  // A context this spec made itself is not covered by the csrf fixture.
  const csrf=(await page.context().cookies()).find(cookie=>cookie.name==='mw_csrf')?.value;
  const res=await (await page.request.post('/jmap/api',{headers:{...headers,...(csrf?{'x-csrf-token':csrf}:{}),'x-mailwoman-storage-authority':me.storageAuthority},data:{using:['urn:ietf:params:jmap:core','urn:mailwoman:crypto','urn:mailwoman:security'],
    methodCalls:[['CryptoKey/query',{accountId:me.accountId},'q'],['CryptoKey/get',{accountId:me.accountId,'#ids':{resultOf:'q',name:'CryptoKey/query',path:'/ids'},properties:['fingerprint','keyId','isOwn','kind']},'g']]}})).json();
  expect(res.methodResponses?.[1]?.[1]?.list,JSON.stringify(res).slice(0,400)).toBeTruthy();
  const list=res.methodResponses[1][1].list as {fingerprint:string;keyId:string;isOwn:boolean;kind:string}[];
  return list.filter(k=>k.isOwn&&k.kind==='pgp').flatMap(k=>[k.fingerprint.toUpperCase(),k.keyId.toUpperCase()]);
}

async function signIn(page:Page,user:string,password:string){
  await page.goto('/');
  await page.getByLabel('Email or username',{exact:true}).fill(user);
  await page.getByLabel('Password',{exact:true}).fill(password);
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  await expect(page.getByLabel('Email or username',{exact:true})).toHaveCount(0);
  await expect(page.locator('[data-account-indicator]').first()).toBeAttached();
}

/** At phone width the sidebar (picker, Settings) is in the drawer. */
async function drawer(page:Page){
  const open=page.getByRole('button',{name:'Open folders and apps',exact:true});
  if(await open.isVisible()&&await open.getAttribute('aria-expanded')!=='true')await open.click();
}
async function openMenu(page:Page){
  const trigger=page.locator('button[data-account-indicator="picker"]');
  if(!await trigger.isVisible())await drawer(page);
  await expect(trigger).toBeVisible();
  // A hover tooltip (the tab strip's) can sit over the picker; move off it first.
  await page.mouse.move(0,0);
  if(await trigger.getAttribute('aria-expanded')!=='true')await trigger.click();
  const menu=page.getByRole('menu',{name:'Mailbox account',exact:true});
  await expect(menu).toBeVisible();
  return menu;
}
async function choose(page:Page,link:string){
  const menu=await openMenu(page);
  await menu.locator(`[data-mailbox-item][data-link="${link}"]`).click();
  await expect(page.locator('button[data-account-indicator="picker"]')).toHaveAttribute('aria-expanded','false');
}
async function openSettings(page:Page,section:string){
  const settings=page.locator('.sidebar, #shell-nav').getByRole('button',{name:'Settings',exact:true}).first();
  if(!await settings.isVisible())await drawer(page);
  await settings.click();
  await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:section,exact:true}).click();
}
async function shot(page:Page,info:{outputPath(name:string):string;project:{name:string}},name:string){
  await page.screenshot({path:info.outputPath(name),fullPage:true});
  const dir=process.env['MW_SCREENSHOT_DIR'];
  if(dir)await page.screenshot({path:`${dir}/${info.project.name}-${name}`,fullPage:true});
}

/** Generate an own OpenPGP key in the Keys module through the real worker. */
async function generateKey(page:Page,email:string,passphrase:string){
  await drawer(page);
  await page.getByTestId('nav-keys').click();
  await expect(page.locator('[data-module="keys"]')).toBeVisible();
  await page.getByRole('button',{name:'Generate key',exact:true}).click();
  const dialog=page.getByRole('dialog',{name:'Generate a key'});
  await dialog.getByLabel('Key type').selectOption('pgp');
  await dialog.getByLabel('Email',{exact:true}).fill(email);
  await dialog.getByLabel('Key passphrase',{exact:true}).fill(passphrase);
  await dialog.getByRole('button',{name:'Generate',exact:true}).click();
  await expect(dialog).toBeHidden({timeout:60000});
  await expect(page.getByRole('list',{name:'Your keys'}).getByText(email).first()).toBeVisible({timeout:30000});
}

/** The child's own key, made by signing in to the child directly (its keys are its own). */
async function childKey(browser:Browser){
  const context=await browser.newContext();
  const page=await context.newPage();
  await signIn(page,'child@child.test','child-pass');
  if((await ownKeyIds(page,{})).length===0)await generateKey(page,'child@child.test',CHILD_PASS);
  const ids=await ownKeyIds(page,{});
  await context.close();
  return ids;
}

function compose(page:Page){
  return page.getByRole('region',{name:'Compose message',exact:true}).or(page.getByRole('dialog',{name:'Compose message',exact:true}));
}
async function openCompose(page:Page,to:string,subject:string){
  await page.getByRole('button',{name:'Compose',exact:true}).first().click();
  const dialog=compose(page);
  await expect(dialog).toBeVisible();
  await dialog.getByLabel('To',{exact:true}).fill(to);
  await dialog.getByLabel('Subject',{exact:true}).fill(subject);
  await dialog.getByLabel('Body',{exact:true}).fill(`Signed body ${subject}`);
  await dialog.getByRole('tab',{name:'Security',exact:true}).click();
  const pin=dialog.getByRole('button',{name:'Keep the editor ribbon open',exact:true});
  if(await pin.isVisible())await pin.click();
  return dialog;
}

test('a linked mailbox signs with its own key and passphrase; the parent key is dropped on the switch',async({page,browser},info)=>{
  test.setTimeout(240000);
  page.setDefaultTimeout(20000);
  const errors:string[]=[];page.on('pageerror',error=>errors.push(error.message));
  const childIds=await childKey(browser);
  expect(childIds.length).toBeGreaterThan(0);

  await signIn(page,'parent@example.org','parent-pass');
  const primary=await (await page.request.get('/api/me')).json() as {storageAuthority:string};
  const ownerHeaders={'x-mailwoman-storage-authority':primary.storageAuthority};
  // Start from a fresh link, independent of an earlier run's child session.
  const stale=(await (await page.request.get('/api/accounts',{headers:ownerHeaders})).json()).accounts as {id:string;revision:string}[];
  if(stale.length){
    expect((await page.request.post('/api/security/step-up',{data:{password:'parent-pass'}})).ok()).toBe(true);
    for(const account of stale)expect((await page.request.delete(`/api/accounts/${account.id}`,{headers:{...ownerHeaders,'if-match':JSON.stringify(account.revision)}})).ok()).toBe(true);
    // Finish the newly mounted shell's lazy catalog imports before this setup
    // reload; Firefox reports a cancelled import as a page error on unload.
    await page.waitForLoadState('networkidle');
    await page.reload();
    await expect(page.locator('[data-account-indicator]').first()).toBeAttached();
  }
  let link:string|undefined;
  if(!link){
    await openSettings(page,'Linked mailboxes');
    await page.getByRole('button',{name:'Link a mailbox',exact:true}).click();
    const editor=page.getByRole('region',{name:'Link a mailbox',exact:true});
    await editor.getByLabel('Username or email address',{exact:true}).fill('child@child.test');
    await editor.getByLabel('Password',{exact:true}).fill('child-pass');
    await editor.getByRole('button',{name:'Verify and link',exact:true}).click();
    const proof=page.getByRole('dialog',{name:'Confirm it is you'});
    if(await proof.waitFor({state:'visible',timeout:10000}).then(()=>true,()=>false)){
      await proof.getByLabel('Password',{exact:true}).fill('parent-pass');
      await proof.getByRole('button',{name:'Confirm and continue',exact:true}).click();
    }
    await expect(async()=>{
      link=((await (await page.request.get('/api/accounts')).json()).accounts as {id:string;address?:string;status:string}[])
        .find(account=>account.address==='child@child.test'&&account.status==='ready')?.id;
      expect(link).toBeTruthy();
    }).toPass({timeout:20000});
    await page.keyboard.press('Escape');
  }

  // The parent's own key, unlocked in a parent composer.
  if((await ownKeyIds(page,{})).length===0)await generateKey(page,'parent@example.org',PARENT_PASS);
  const parentIds=await ownKeyIds(page,{});
  expect(parentIds.length).toBeGreaterThan(0);
  await page.getByTestId('nav-mail').click().catch(()=>undefined);
  const parentDraft=await openCompose(page,'parent@example.org',`t87 parent ${Date.now()}`);
  await parentDraft.locator('[data-testid="sign-toggle"]').check();
  const parentPanel=parentDraft.locator('[data-testid="compose-sign-unlock"]');
  await expect(parentPanel).toBeVisible();
  await (await revealRibbonControl(page,parentDraft,parentPanel.locator('[data-testid="sign-passphrase"]'))).fill(PARENT_PASS);
  await parentPanel.locator('[data-testid="sign-passphrase"]').press('Enter');
  await expect(parentPanel).toBeHidden({timeout:20000});
  await shot(page,info,'keys-parent-unlocked.png');

  // Switch to the child: the parent's signing session must not survive.
  // A switch that the picker did not take (a click lost to a re-render) is chosen again.
  await expect(async()=>{
    await choose(page,link!);
    await expect(page.locator('button[data-account-indicator="picker"]')).toHaveAttribute('aria-label',/child@child\.test/,{timeout:10000});
  }).toPass({timeout:45000});
  await shot(page,info,'keys-after-switch.png');
  // The switch closes the parent's panes, its composer (and so its unlocked key) with them.
  await expect(page.getByRole('tab',{name:/t87 parent/})).toHaveCount(0,{timeout:20000});
  await expect(async()=>{
    const me=await (await page.request.get('/api/me',{headers:{'x-mailwoman-link':link!}})).json() as {username:string};
    expect(me.username).toBe('child@child.test');
  }).toPass();
  const subject=`t87 child signed ${info.project.name} ${Date.now()}`;
  const dialog=await openCompose(page,'child@child.test',subject);
  await dialog.locator('[data-testid="sign-toggle"]').check();
  const panel=dialog.locator('[data-testid="compose-sign-unlock"]');
  // The child's own passphrase prompt appears: nothing is unlocked for it yet.
  await expect(panel).toBeVisible();
  await expect(panel.locator('[data-testid="sign-passphrase"]')).toHaveValue('');
  await shot(page,info,'keys-child-passphrase-prompt.png');
  // The parent's passphrase does not open the child's key.
  await (await revealRibbonControl(page,dialog,panel.locator('[data-testid="sign-passphrase"]'))).fill(PARENT_PASS);
  await panel.locator('[data-testid="sign-passphrase"]').press('Enter');
  await expect(panel.getByRole('alert')).toBeVisible({timeout:20000});
  await shot(page,info,'keys-child-parent-pass-refused.png');
  await panel.locator('[data-testid="sign-passphrase"]').fill(CHILD_PASS);
  await panel.locator('[data-testid="sign-passphrase"]').press('Enter');
  await expect(panel).toBeHidden({timeout:20000});
  const wire:string[]=[];
  const onRequest=(request:import('@playwright/test').Request)=>{if(request.method()==='POST'&&request.url().includes('/jmap/api'))wire.push(request.postData()??'');};
  page.on('request',onRequest);
  await dialog.locator('form.compose').evaluate(form=>(form as HTMLFormElement).requestSubmit());
  await expect(dialog).toBeHidden({timeout:30000});
  page.off('request',onRequest);
  const sent=wire.find(body=>body.includes('EmailSubmission/set')&&body.includes('BEGIN PGP SIGNED MESSAGE'));
  expect(sent,'the submitted message is clear-signed').toBeTruthy();
  // The armored text sits in a string value of the request (a body value).
  const strings:string[]=[];
  const walk=(value:unknown):void=>{if(typeof value==='string')strings.push(value);else if(value&&typeof value==='object')for(const item of Object.values(value))walk(item);};
  walk(JSON.parse(sent!));
  const text=strings.find(value=>value.includes('BEGIN PGP SIGNATURE'))??'';
  const armor=signatureArmor(text);
  expect(armor,'the sent message carries an OpenPGP signature').not.toBeNull();
  const issuers=signatureIssuers(armor!);
  expect(issuers.length).toBeGreaterThan(0);
  const matches=(ids:string[])=>issuers.some(issuer=>ids.some(id=>id.endsWith(issuer)||issuer.endsWith(id)));
  expect(matches(parentIds),`signature issuers ${issuers.join(',')} must not be the parent's keys ${parentIds.join(',')}`).toBe(false);
  expect(matches(childIds),`signature issuers ${issuers.join(',')} are the child's keys ${childIds.join(',')}`).toBe(true);
  await shot(page,info,'keys-child-sent.png');

  // Settings > Keys in the child lists the child's keys only (when the module is
  // offered to a linked mailbox; t87-e2 gates it).
  const keysNav=page.getByTestId('nav-keys');
  if(await keysNav.isVisible()){
    await keysNav.click();
    const own=page.getByRole('list',{name:'Your keys'});
    await expect(own.getByText('child@child.test').first()).toBeVisible();
    await expect(own.getByText('parent@example.org')).toHaveCount(0);
    await shot(page,info,'keys-child-module.png');
  }
  await choose(page,'');
  expect(errors).toEqual([]);
});
