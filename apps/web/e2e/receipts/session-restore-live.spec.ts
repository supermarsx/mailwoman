import {test,expect} from '../csrf.ts';
import {connect} from 'node:net';
import {mkdirSync} from 'node:fs';
import type {BrowserContext,Page,TestInfo} from '@playwright/test';

// t109: a reload puts everything back, also in a linked mailbox: the mailbox,
// its folder, the open tabs and the active one, the reading-pane message, the
// list's scroll position (by top message) and the search. The address bar
// names the mailbox (link id), folder and message. Unlinking in another tab
// and reloading opens the sign-in account with a notice; logging out forgets
// it all. Stack: node scripts/e2e-receipts-stack.mjs (Greenmail without sign-in
// checks, mailbox linking on). Chromium and Firefox; 1440 and 390 set here.
const SHOTS=process.env['MW_SCREENSHOT_DIR'];
const SIZES=[{name:'1440',width:1440,height:900},{name:'390',width:390,height:844}] as const;
const COUNT=36;
const BASELINE=process.env['MW_T109_BASELINE']==='1';
const USING=['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'];

const isPhone=(page:Page)=>(page.viewportSize()?.width??1440)<600;
async function drawer(page:Page){
  const open=page.getByRole('button',{name:'Open folders and apps',exact:true});
  if(await open.isVisible()&&await open.getAttribute('aria-expanded')!=='true'){await open.click();await page.waitForTimeout(400);}
}
async function openMenu(page:Page){
  const trigger=page.locator('button[data-account-indicator="picker"]').first();
  await expect(async()=>{await drawer(page);await expect(trigger).toBeVisible({timeout:1500});}).toPass({timeout:20000});
  if(await trigger.getAttribute('aria-expanded')!=='true')await trigger.click();
  const menu=page.getByRole('menu',{name:'Mailbox account',exact:true});
  await expect(menu).toBeVisible();
  return menu;
}
async function signIn(page:Page){
  await page.goto('/');
  await page.getByLabel('Email or username',{exact:true}).fill('parent@example.org');
  await page.getByLabel('Password',{exact:true}).fill('parent-pass');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  await expect(page.getByLabel('Email or username',{exact:true})).toHaveCount(0);
  await expect(page.locator('[data-account-indicator]').first()).toBeAttached();
}
async function openSettingsSection(page:Page,section:string){
  await drawer(page);
  await page.locator('#shell-nav').getByRole('button',{name:'Settings',exact:true}).first().click();
  await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:section,exact:true}).click();
}
const listing=async(page:Page)=>(await (await page.request.get('/api/accounts')).json() as {accounts:{id:string;address?:string;status:string}[]}).accounts;
/** Unlink what earlier t109 runs left (each run links fresh users; the parent's link count is capped). */
async function dropOldLinks(page:Page,keep?:string):Promise<void>{
  const all=(await (await page.request.get('/api/accounts')).json() as {accounts:{id:string;address?:string;revision?:unknown}[]}).accounts;
  const me=await (await page.request.get('/api/me')).json() as {storageAuthority:string};
  if(all.some(account=>account.address?.startsWith('t109-')&&account.address!==keep))
    await page.request.post('/api/security/step-up',{data:{password:'parent-pass'}});
  for(const account of all)if(account.address?.startsWith('t109-')&&account.address!==keep)
    await page.request.delete(`/api/accounts/${encodeURIComponent(account.id)}`,{headers:{'if-match':JSON.stringify(account.revision),'x-mailwoman-storage-authority':me.storageAuthority}}).catch(()=>undefined);
}
/** A link to `address` (made through Settings › Linked mailboxes when it is missing). Each test links
 * a fresh Greenmail user, so the engine's connection to it is new (an idle one can be dropped under it). */
async function ensureLink(page:Page,address:string):Promise<string>{
  await dropOldLinks(page,address);
  const found=(await listing(page)).find(account=>account.address===address&&account.status==='ready');
  if(found)return found.id;
  await openSettingsSection(page,'Linked mailboxes');
  await page.getByRole('button',{name:'Link a mailbox',exact:true}).click();
  const editor=page.getByRole('region',{name:'Link a mailbox',exact:true});
  await editor.getByLabel('Username or email address',{exact:true}).fill(address);
  await editor.getByLabel('Password',{exact:true}).fill('child-pass');
  await editor.getByRole('button',{name:'Verify and link',exact:true}).click();
  const proof=page.getByRole('dialog',{name:'Confirm it is you'});
  const done=page.getByRole('heading',{name:'Linked mailboxes',exact:true});
  await expect(proof.or(done).first()).toBeVisible();
  if(await proof.isVisible()){
    await proof.getByLabel('Password',{exact:true}).fill('parent-pass');
    await proof.getByRole('button',{name:'Confirm and continue',exact:true}).click();
  }
  await expect.poll(async()=>(await listing(page)).find(account=>account.address===address)?.status).toBe('ready');
  return (await listing(page)).find(account=>account.address===address)!.id;
}
/** JMAP in the child mailbox, through the parent session (the link header). */
async function childJmap(page:Page,link:string,calls:unknown[]):Promise<Array<[string,Record<string,unknown>,string]>>{
  for(let attempt=0;;attempt++){
    const res=await page.request.post('/jmap/api',{headers:{'x-mailwoman-link':link},data:{using:USING,methodCalls:calls}});
    if(res.ok())return (await res.json() as {methodResponses:Array<[string,Record<string,unknown>,string]>}).methodResponses;
    if(attempt>20)throw new Error(`JMAP ${res.status()} ${await res.text()}`);
    await page.waitForTimeout(250);
  }
}
const CHILD={current:'child@child.test'};
const IMAP=process.env['MW_T49_IMAP']??'127.0.0.1:3143';
type ImapStep=string|{append:string;body:string};
/** Run IMAP commands as `user` straight against Greenmail (auth disabled), so
 * the seed does not go through the engine's write path. Returns the replies. */
async function imapRun(user:string,steps:ImapStep[]):Promise<string>{
  const [host,port]=IMAP.split(':');
  return new Promise((resolve,reject)=>{
    const socket=connect(Number(port),host!);
    const queue:ImapStep[]=[`LOGIN "${user}" "x"`,...steps,'LOGOUT'];
    let buffer='',all='',tag=0,literal:string|null=null,greeted=false;
    const next=():void=>{
      const step=queue.shift();if(step===undefined){socket.end();resolve(all);return;}
      tag+=1;
      if(typeof step==='string'){socket.write(`t${tag} ${step}\r\n`);return;}
      literal=step.body;
      socket.write(`t${tag} APPEND "${step.append}" {${Buffer.byteLength(step.body,'utf8')}}\r\n`);
    };
    socket.setEncoding('utf8');socket.on('error',reject);
    socket.on('data',(chunk:string)=>{
      buffer+=chunk;all+=chunk;
      if(!greeted){if(buffer.includes('* OK')){greeted=true;buffer='';next();}return;}
      if(literal!==null&&/^\+/m.test(buffer)){socket.write(`${literal}\r\n`);literal=null;buffer='';return;}
      const done=new RegExp(`^t${tag} (OK|NO|BAD)`,'m').exec(buffer);
      if(done){if(done[1]!=='OK'&&queue.length>0){socket.destroy();reject(new Error(`imap: ${buffer}`));return;}buffer='';next();}
    });
  });
}
const message=(subject:string,index:number)=>[`From: Restore Sender <sender@restore.test>`,`To: child@child.test`,`Subject: ${subject}`,
  `Date: ${new Date(Date.UTC(2026,9,1,8,index)).toUTCString()}`,`Message-ID: <${subject.replace(/\W/g,'-')}@restore.test>`,'',`Restore body ${subject}.`,''].join('\r\n');
/** A folder in the child mailbox holding COUNT messages "<tag> NN" (newest NN first):
 * appended to the INBOX straight in Greenmail, then moved by the engine. */
async function seedChildFolder(page:Page,link:string,tag:string):Promise<{folder:string;accountId:string}>{
  const session=await (await page.request.get('/jmap/session',{headers:{'x-mailwoman-link':link}})).json() as {primaryAccounts:Record<string,string>};
  const accountId=session.primaryAccounts['urn:ietf:params:jmap:mail']!;
  let folder:string|undefined,created:unknown;
  for(let attempt=0;attempt<4&&!folder;attempt++){
    if(attempt>0)await page.waitForTimeout(1500);
    const reply=await childJmap(page,link,[['Mailbox/set',{accountId,create:{f:{name:tag}}},'c']]);created=reply;
    folder=(reply[0]![1].created as Record<string,{id:string}>|undefined)?.f?.id;
  }
  expect(folder,JSON.stringify(created)).toBeTruthy();
  await imapRun(CHILD.current,Array.from({length:COUNT},(_,index)=>({append:'INBOX',body:message(`${tag} ${String(index+1).padStart(2,'0')}`,index)})));
  let ids:string[]=[];
  await expect.poll(async()=>{
    const found=await childJmap(page,link,[['Email/query',{accountId,filter:{subject:tag},limit:100},'q']]);
    ids=(found[0]![1].ids as string[])??[];return ids.length;
  },{timeout:90000}).toBe(COUNT);
  await childJmap(page,link,[['Email/set',{accountId,update:Object.fromEntries(ids.map(id=>[id,{mailboxIds:{[folder!]:true}}]))},'m']]);
  return {folder:folder!,accountId};
}
const rows=(page:Page)=>page.locator('.list__scroll .list__row');
const paneSubject=(page:Page)=>page.getByTestId('reading-pane').locator('.reader__subject');
async function topRow(page:Page):Promise<string|null>{
  return page.locator('.list__scroll').evaluate(el=>{
    const top=el.getBoundingClientRect().top;let best:Element|null=null,distance=1e9;
    for(const row of el.querySelectorAll('.list__row')){const y=row.getBoundingClientRect().top-top;if(y>=-4&&y<distance){distance=y;best=row;}}
    return best?.querySelector('.list__subject')?.textContent??null;
  });
}
async function tabs(page:Page):Promise<{titles:string[];active:string|null}>{
  const all=page.locator('.subtab-strip [role=tab]');
  const titles=(await all.allTextContents()).map(text=>text.trim());
  const active=await page.locator('.subtab-strip [role=tab][aria-selected=true]').first().textContent().catch(()=>null);
  return {titles,active:active?.trim()??null};
}
async function activeFolder(page:Page):Promise<string|null>{
  return page.locator('#shell-nav .sidebar__box--active').first().getAttribute('aria-label').catch(()=>null)
    .then(async label=>label??(await page.locator('#shell-nav .sidebar__box--active').first().textContent().catch(()=>null)));
}
async function openFolder(page:Page,name:string){
  await drawer(page);
  await page.locator('#shell-nav .sidebar__nav').getByRole('button',{name:new RegExp(`^${name}`)}).first().click();
  if(isPhone(page))await expect(page.locator('#shell-nav')).not.toHaveClass(/sidebar--open/);
  await page.waitForTimeout(400);
}
async function mailTab(page:Page){await page.getByRole('tab',{name:'Mail',exact:true}).click();}
async function shot(page:Page,info:TestInfo,name:string){if(SHOTS)await page.screenshot({path:`${SHOTS}/${info.project.name}-${name}.png`});}
/** What is on screen, for the record and the comparison. */
async function snapshot(page:Page){
  const url=new URL(page.url());
  return {
    linked:await page.locator('button[data-account-indicator="picker"]').first().getAttribute('data-linked').catch(()=>null),
    folder:await activeFolder(page),
    ...(await tabs(page)),
    pane:await paneSubject(page).textContent({timeout:1000}).catch(()=>null),
    top:await topRow(page).catch(()=>null),
    search:await page.getByRole('searchbox').first().inputValue({timeout:1000}).catch(()=>null),
    query:url.search,
  };
}

test('a reload in a linked mailbox restores the mailbox, folder, tabs, pane, scroll and search',async({page},info)=>{
  test.skip(info.project.name==='phone','the spec sets 1440 and 390 itself');
  test.setTimeout(420000);
  page.setDefaultTimeout(20000);
  const errors:string[]=[];page.on('pageerror',error=>{if(!error.message.includes('serviceWorker'))errors.push(error.message);});
  if(SHOTS)mkdirSync(SHOTS,{recursive:true});
  await signIn(page);
  const child=`t109-${info.project.name}-${Date.now()}@child.test`;
  CHILD.current=child;
  const link=await ensureLink(page,child);
  const records:Record<string,unknown>={};

  for(const size of SIZES){
    await page.setViewportSize(size);
    const tag=`Restore ${info.project.name} ${size.name} ${Date.now()}`;
    const {folder,accountId}=await seedChildFolder(page,link,tag);
    // A fresh start in the sign-in account: no tab state yet.
    await page.evaluate(()=>sessionStorage.clear());
    await page.goto('/');
    await expect(page.locator('button[data-account-indicator="picker"]')).toBeAttached();
    if(await page.locator('button[data-account-indicator="picker"]').getAttribute('data-linked')==='true'){
      const menu=await openMenu(page);await menu.locator('[data-mailbox-item][data-link=""]').click();
    }
    // Switch to the linked mailbox and open its folder.
    const menu=await openMenu(page);
    await menu.locator(`[data-mailbox-item][data-link="${link}"]`).first().click();
    await expect(page.locator('button[data-account-indicator="picker"]').first()).toHaveAttribute('data-linked','true');
    await openFolder(page,tag);
    await expect(rows(page).nth(8)).toBeAttached({timeout:30000});
    const subjectAt=async(index:number)=>(await rows(page).nth(index).locator('.list__subject').textContent())!.trim();
    // A message in its own tab, back to Mail.
    const tabbed=await subjectAt(2);
    if(isPhone(page))await rows(page).filter({hasText:tabbed}).click();
    else await rows(page).filter({hasText:tabbed}).dblclick();
    await expect(page.locator('[role=tab][id^="subtab-tab-message:"]')).toHaveCount(1);
    await mailTab(page);
    // The reading pane's message (there is none at phone width).
    const paned=await subjectAt(4);
    if(!isPhone(page)){
      await rows(page).filter({hasText:paned}).click();
      await expect(paneSubject(page)).toHaveText(paned);
    }
    // Settings in a tab, then back to Mail.
    await drawer(page);
    await page.locator('#shell-nav').getByRole('button',{name:'Settings',exact:true}).first().click();
    await expect(page.locator('[role=tab][id^="subtab-tab-settings:"]')).toHaveCount(1);
    await mailTab(page);
    // Scroll the list: the 15th row on top.
    await page.locator('.list__scroll').evaluate(el=>{const row=el.querySelector('.list__row') as HTMLElement;el.scrollTop=row.offsetHeight*15;el.dispatchEvent(new Event('scroll'));});
    await page.waitForTimeout(600);
    const before=await snapshot(page);
    expect(before.top).not.toBeNull();
    if(!BASELINE)expect(before.query).toContain(`mailbox=${encodeURIComponent(link)}`);
    if(!BASELINE)expect(before.query).toContain(`folder=${encodeURIComponent(folder)}`);
    await shot(page,info,`${size.name}-1-before-reload`);

    // Watch every frame of the reload: a mail row that is not the linked folder's
    // (the sign-in account's inbox) must never be on screen.
    await page.addInitScript((needle:string)=>{
      const w=window as unknown as {__t109Foreign?:Record<string,string[]>};w.__t109Foreign??={};const seen:string[]=w.__t109Foreign[needle]=[];
      const tick=()=>{for(const el of document.querySelectorAll('.list__scroll .list__subject')){const text=el.textContent??'';if(!text.includes(needle)&&!seen.includes(text))seen.push(text);}requestAnimationFrame(tick);};
      requestAnimationFrame(tick);
    },tag);
    await page.reload();
    if(BASELINE){
      // Record what a reload loses (run against a build without t109).
      await page.waitForTimeout(8000);
      const lost=await snapshot(page);records[size.name]={before,after:lost};
      console.log(`t109 baseline ${info.project.name} ${size.name}: ${JSON.stringify({before,after:lost})}`);
      await shot(page,info,`${size.name}-0-baseline-after-reload`);
      continue;
    }
    // No frame of the sign-in account's list: the linked mailbox is up before any mail rows.
    await expect(page.locator('button[data-account-indicator="picker"]').first()).toHaveAttribute('data-linked','true',{timeout:30000});
    await expect(rows(page).filter({hasText:tag}).first()).toBeAttached({timeout:30000});
    await expect.poll(async()=>(await snapshot(page)).top,{timeout:20000}).toBe(before.top);
    if(!isPhone(page))await expect(paneSubject(page)).toHaveText(paned);
    expect(await page.evaluate((needle)=>(window as unknown as {__t109Foreign?:Record<string,string[]>}).__t109Foreign?.[needle],tag),'no frame shows rows of another mailbox').toEqual([]);
    const after=await snapshot(page);
    records[`${size.name}`]={before,after};
    await shot(page,info,`${size.name}-2-after-reload`);
    expect(after.linked).toBe('true');
    expect(after.folder).toBe(before.folder);
    expect(after.titles).toEqual(before.titles);
    expect(after.active).toBe(before.active);
    expect(after.pane).toBe(before.pane);
    expect(after.query).toBe(before.query);

    // A search comes back too, in the same folder.
    if(!isPhone(page)){
      const query=`"${paned}"`;
      const box=page.getByRole('searchbox').first();
      await box.fill(query);await box.press('Enter');
      await expect(rows(page)).toHaveCount(1,{timeout:30000});
      await expect(rows(page).first()).toContainText(paned);
      await page.reload();
      await expect(page.getByRole('searchbox').first()).toHaveValue(query,{timeout:30000});
      await expect(rows(page)).toHaveCount(1,{timeout:30000});
      await expect(rows(page).first()).toContainText(paned);
      await shot(page,info,`${size.name}-3-search-after-reload`);
      await box.fill('');await box.press('Enter');
    }

    // A message that is gone is skipped: delete the pane's message, reload.
    if(!isPhone(page)){
      const found=await childJmap(page,link,[['Email/query',{accountId,filter:{subject:paned}},'q']]);
      const doomed=found[0]![1].ids as string[];expect(doomed).toHaveLength(1);
      const destroyed=await childJmap(page,link,[['Email/set',{accountId,destroy:doomed},'d']]);
      expect(destroyed[0]![1].destroyed,JSON.stringify(destroyed)).toEqual(doomed);
      await page.reload();
      await expect(rows(page).filter({hasText:tag}).first()).toBeAttached({timeout:30000});
      await page.waitForTimeout(1500);
      await expect(paneSubject(page)).toHaveCount(0);
      await expect(page.getByRole('alert')).toHaveCount(0);
      await shot(page,info,`${size.name}-4-gone-message-skipped`);
    }
  }
  await info.attach('restore-records',{body:JSON.stringify(records,null,2),contentType:'application/json'});
  console.log(`t109 records ${info.project.name}: ${JSON.stringify(records)}`);
  expect(errors).toEqual([]);
});

async function unlinkIn(other:Page,address:string){
  await other.goto('/');
  await expect(other.locator('[data-account-indicator]').first()).toBeAttached();
  await openSettingsSection(other,'Linked mailboxes');
  const escaped=address.replace(/[.]/g,'[.]');
  await other.getByRole('button',{name:new RegExp(`^Unlink .*${escaped}`)}).click();
  const dialog=other.getByRole('alertdialog',{name:new RegExp(`^Unlink [^\\w]?${escaped}[^\\w]?[?]$`)});
  await dialog.getByRole('button',{name:'Confirm unlink',exact:true}).click();
  const proof=other.getByRole('dialog',{name:'Confirm it is you'});
  if(await proof.isVisible({timeout:3000}).catch(()=>false)){
    await proof.getByLabel('Password',{exact:true}).fill('parent-pass');
    await proof.getByRole('button',{name:'Confirm and continue',exact:true}).click();
  }
  await expect.poll(async()=>(await listing(other)).some(account=>account.address===address)).toBe(false);
}

test('unlinked in another tab: a reload opens the sign-in account with a notice; logging out forgets the tab state',async({page,context},info)=>{
  test.skip(info.project.name==='phone','the spec sets 1440 and 390 itself');
  test.setTimeout(300000);
  page.setDefaultTimeout(20000);
  if(SHOTS)mkdirSync(SHOTS,{recursive:true});
  for(const size of SIZES){
    await page.setViewportSize(size);
    await signIn(page).catch(async()=>{await page.goto('/');});
    const child=`t109-${info.project.name}-${size.name}-${Date.now()}@child.test`;
    const link=await ensureLink(page,child);
    await page.goto('/');
    const menu=await openMenu(page);
    await menu.locator(`[data-mailbox-item][data-link="${link}"]`).first().click();
    await expect(page.locator('button[data-account-indicator="picker"]').first()).toHaveAttribute('data-linked','true');
    await expect.poll(()=>new URL(page.url()).searchParams.get('mailbox')).toBe(link);

    const other=await (context as BrowserContext).newPage();
    await other.setViewportSize(size);
    await unlinkIn(other,child);
    await other.close();

    await page.reload();
    await expect(page.getByText('The mailbox open before the page reloaded is no longer linked or is turned off, so your sign-in account opened instead.')).toBeVisible({timeout:30000});
    await expect(page.locator('button[data-account-indicator="picker"]')).toHaveCount(0);
    await expect(rows(page).first()).toBeAttached({timeout:30000});
    await shot(page,info,`${size.name}-5-unlinked-fallback`);
    expect(new URL(page.url()).searchParams.get('mailbox')).toBeNull();

    // Log out: the tab's restore record and the address-bar target are gone.
    await drawer(page);
    await page.getByRole('button',{name:'Log out',exact:true}).first().click();
    await expect(page.getByLabel('Email or username',{exact:true})).toBeVisible();
    expect(await page.evaluate(()=>sessionStorage.getItem('mw.session.v1'))).toBeNull();
    expect(new URL(page.url()).search).toBe('');
    await shot(page,info,`${size.name}-6-logged-out`);
  }
});
