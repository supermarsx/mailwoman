// t94: the Focused inbox rules and per-sender overrides against a live server
// built from the tree, over Greenmail.
//
// Seed (unique per run): a newsletter with List-Unsubscribe (Other by header,
// although its name has no automated-sender word); a sender in the contacts
// whose name says "Updates" (Focused: the contact rule beats the keyword
// heuristic); a sender the user wrote to (a message in Sent) whose name says
// "Digest" (Focused); a reply in a thread the user started (Focused); a plain
// person (Focused). Then: the row menu's "Always move to Focused" moves the
// newsletter at once with an Undo toast, Undo puts it back, the reader's More
// menu moves it again, a reload keeps it (the override is on the server), and
// Settings > Folders and sidebar > Manage Focused senders removes it.
// Runs at 1440 and 390 in the chromium and firefox projects. Screenshots go to
// MW_T94_SHOTS.
import {test,expect} from '../csrf.ts';
import type {Page,TestInfo} from '@playwright/test';
import net from 'node:net';
import path from 'node:path';

const smtpPort=Number(process.env['MW_E2E_SMTP_PORT']??42225);
const run=`${Date.now().toString(36)}${Math.random().toString(36).slice(2,6)}`;
const ME='testuser@example.org';
const S={
 news:{from:`Ana Deals <deals.${run}@shop.example>`,subject:`t94 deals ${run}`,extra:`List-Unsubscribe: <mailto:unsub.${run}@shop.example>\r\nList-Id: Deals <deals.${run}.shop.example>\r\n`},
 contact:{from:`Carol Updates <carol.${run}@example.net>`,subject:`t94 contact ${run}`,extra:''},
 wrote:{from:`Dave Digest <dave.${run}@example.net>`,subject:`Re: t94 hello dave ${run}`,extra:''},
 // Its In-Reply-To/References are added once the root is sent (the server picks that Message-ID).
 thread:{from:`Erin Notifications <erin.${run}@example.net>`,subject:`Re: t94 plan ${run}`,extra:''},
 person:{from:`Pat Person <pat.${run}@example.net>`,subject:`t94 person ${run}`,extra:''},
} as const;
type Key=keyof typeof S;
let threadExtra='';
const addr=(k:Key)=>/<(.+)>/.exec(S[k].from)![1]!;

function deliver(m:{from:string;subject:string;extra:string},n:number):Promise<void>{
 return new Promise((resolve,reject)=>{
  const socket=net.connect(smtpPort,'127.0.0.1');
  const from=/<(.+)>/.exec(m.from)![1]!;
  const data=`From: ${m.from}\r\nTo: ${ME}\r\nSubject: ${m.subject}\r\nDate: ${new Date(Date.now()+n*1000).toUTCString().replace('GMT','+0000')}\r\nMessage-ID: <t94-${run}-${n}@example.net>\r\n${m.extra}MIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nFocused senders fixture ${n}.\r\n.\r\n`;
  const steps=['EHLO t94\r\n',`MAIL FROM:<${from}>\r\n`,`RCPT TO:<${ME}>\r\n`,'DATA\r\n',data,'QUIT\r\n'];
  let i=0,buffer='';
  socket.on('data',chunk=>{buffer+=chunk.toString();if(!buffer.endsWith('\r\n'))return;const last=buffer.trim().split('\r\n').at(-1)!;if(/^\d{3}-/.test(last))return;buffer='';if(/^[45]/.test(last)){socket.destroy();reject(new Error(last));return;}if(i<steps.length)socket.write(steps[i++]!);});
  socket.on('close',()=>resolve());socket.on('error',reject);
 });
}

const shot=(info:TestInfo,name:string)=>{const dir=process.env['MW_T94_SHOTS'];return dir?path.join(dir,`${info.project.name}-${name}.png`):info.outputPath(`${name}.png`);};
async function signIn(page:Page):Promise<string>{
 await page.goto('/');
 await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');
 const login=page.waitForResponse(r=>new URL(r.url()).pathname==='/api/login'&&r.request().method()==='POST');
 await page.getByRole('button',{name:'Sign in',exact:true}).click();
 const {accountId}=await (await login).json() as {accountId:string};
 await expect(page.locator('.mail-pane')).toBeVisible({timeout:60000});
 return accountId;
}
async function authority(page:Page):Promise<Record<string,string>>{const me=await (await page.request.get('/api/me')).json() as {storageAuthority:string};return {'x-mailwoman-storage-authority':me.storageAuthority};}
async function jmap(page:Page,calls:unknown[]):Promise<Array<[string,Record<string,unknown>,string]>>{
 for(let attempt=0;;attempt++){
  const res=await page.request.post('/jmap/api',{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail','urn:mailwoman:contacts'],methodCalls:calls}});
  if(res.ok())return (await res.json() as {methodResponses:Array<[string,Record<string,unknown>,string]>}).methodResponses;
  if(attempt>1)throw new Error(`JMAP ${res.status()} ${await res.text()}`);
  await page.waitForTimeout(1000);
 }
}
async function setFocused(page:Page,on:boolean):Promise<void>{
 const h=await authority(page);
 const res=on?await page.request.put('/api/settings/mail.focusedInbox',{data:{value:true},headers:h}):await page.request.delete('/api/settings/mail.focusedInbox',{headers:h});
 expect([200,204,404]).toContain(res.status());
}
async function clearOverrides(page:Page):Promise<void>{
 const h=await authority(page);
 const {senders}=await (await page.request.get('/api/account/focused-senders')).json() as {senders:Array<{email:string}>};
 for(const s of senders)expect((await page.request.put('/api/account/focused-senders',{data:{email:s.email,tab:null},headers:h})).status()).toBe(200);
}

/** Contact, the message to Dave, and the thread root, through the app's own JMAP; then the inbound mail. */
async function seed(page:Page,accountId:string):Promise<void>{
 // On a fresh stack the index is still being built from the 30-message seed;
 // mail that lands during that rebuild can be missing from it for a while.
 // Wait until a search finds the stack's own seed first.
 await expect.poll(async()=>((await jmap(page,[['Email/query',{accountId,filter:{text:'"Question about the API 29"'},calculateTotal:true,limit:1},'q']]))[0]![1]['total'] as number|undefined)??0,{timeout:120000,intervals:[1000,2000,5000],message:'the index answers'}).toBeGreaterThan(0);
 const boxes=(await jmap(page,[['Mailbox/get',{accountId,ids:null},'m']]))[0]![1]['list'] as Array<{id:string;name:string;role:string|null}>;
 let sent=boxes.find(b=>b.role==='sent')?.id;
 if(!sent){
  // A new folder cannot take a special-use role; the server gives "Sent" the sent role by its name.
  await jmap(page,[['Mailbox/set',{accountId,create:{s:{name:'Sent'}}},'c']]);
  const again=(await jmap(page,[['Mailbox/get',{accountId,ids:null},'m']]))[0]![1]['list'] as Array<{id:string;role:string|null}>;
  sent=again.find(b=>b.role==='sent')?.id;
 }
 expect(sent,'a Sent folder').toBeTruthy();
 const contact=(await jmap(page,[['ContactCard/set',{accountId,create:{c:{name:{full:'Carol Updates'},emails:[{context:'work',value:addr('contact'),pref:1}]}}},'c']]))[0]![1];
 expect(Object.keys((contact['created']??{}) as object),JSON.stringify(contact)).toEqual(['c']);
 // Two messages the user sends (the app's own send: a draft, then a submission
 // that files it in Sent): one to Dave, and the start of a thread.
 const send=async(to:{name?:string;email:string},subject:string):Promise<string>=>{
  const res=await jmap(page,[
   ['Email/set',{accountId,create:{draft:{mailboxIds:{[sent!]:true},keywords:{$draft:true,$seen:true},from:[{email:ME}],to:[to],subject,bodyValues:{b:{value:`${subject}.`}},textBody:[{partId:'b',type:'text/plain'}]}}},'set'],
   ['EmailSubmission/set',{accountId,create:{send:{emailId:'#draft',envelope:{mailFrom:{email:ME},rcptTo:[{email:to.email}]}}},onSuccessUpdateEmail:{'#send':{mailboxIds:{[sent!]:true},'keywords/$draft':null}}},'submit'],
  ]);
  const id=((res[0]![1]['created'] as Record<string,{id:string}>|undefined)?.['draft']?.id);
  expect(id,JSON.stringify(res)).toBeTruthy();
  expect(Object.keys((res[1]![1]['created']??{}) as object),JSON.stringify(res[1])).toEqual(['send']);
  const got=(await jmap(page,[['Email/get',{accountId,ids:[id],properties:['messageId','mailboxIds']},'g']]))[0]![1]['list'] as Array<{messageId:string[]}>;
  return got[0]!.messageId[0]!;
 };
 await send({name:'Dave Digest',email:addr('wrote')},`t94 hello dave ${run}`);
 const root=await send({email:`group.${run}@example.net`},`t94 plan ${run}`);
 threadExtra=['In-Reply-To','References'].map(name=>`${name}: <${root}>\r\n`).join('');
 for(const [n,k] of (Object.keys(S) as Key[]).entries())await deliver(k==='thread'?{...S[k],extra:threadExtra}:S[k],n);
}

const tabs=(page:Page)=>page.getByTestId('inbox-tabs');
const row=(page:Page,k:Key)=>page.locator('.list__row').filter({hasText:S[k].subject});
async function openTab(page:Page,name:'Focused'|'Other'):Promise<void>{
 const tab=tabs(page).getByRole('tab',{name:new RegExp(`^${name}`)});
 await expect(page.locator('section.list').filter({visible:true}).first()).toHaveAttribute('aria-busy','false');
 await tab.scrollIntoViewIfNeeded();
 await page.evaluate(()=>new Promise<void>(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolve()))));
 await tab.click();await expect(tab).toHaveAttribute('aria-selected','true');
}
/** Which tab lists each fixture, read from the two tabs. */
async function split(page:Page):Promise<Record<Key,'Focused'|'Other'>>{
 const out={} as Record<Key,'Focused'|'Other'>;
 for(const name of ['Focused','Other'] as const){
  await openTab(page,name);
  await page.waitForTimeout(300);
  for(const k of Object.keys(S) as Key[])if(await row(page,k).count()>0)out[k]=name;
 }
 return out;
}
async function drawer(page:Page):Promise<void>{const menu=page.locator('header.shell__bar button[aria-controls="shell-nav"]').first();if(await menu.isVisible()&&(await menu.getAttribute('aria-expanded'))!=='true')await menu.click();}
async function openFolderSettings(page:Page):Promise<void>{
 const nav=page.getByRole('navigation',{name:'Settings sections'});
 if(!await nav.isVisible()){
  const settings=page.locator('.sidebar, #shell-nav').getByRole('button',{name:'Settings',exact:true}).first();
  if(!await settings.isVisible())await drawer(page);
  await settings.click();
 }
 await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Folders and sidebar',exact:true}).click();
 await expect(page.getByTestId('focused-senders')).toBeVisible();
}
async function backToMail(page:Page):Promise<void>{
 const tab=page.getByRole('tab',{name:'Mail',exact:true});if(await tab.isVisible()){await tab.click();return;}
 const mail=page.locator('.sidebar, #shell-nav').getByRole('button',{name:'Mail',exact:true}).first();if(!await mail.isVisible())await drawer(page);await mail.click();
}

test.describe.configure({timeout:360000});
let seeded=false;
for(const width of [1440,390]){
 test(`Focused inbox rules and Always move at ${width}px`,async({page},info)=>{
  test.skip(info.project.name==='phone','the 390 legs run in the chromium and firefox projects');
  await page.setViewportSize({width,height:900});
  const accountId=await signIn(page);
  await clearOverrides(page);
  await setFocused(page,true);
  if(!seeded){await seed(page,accountId);seeded=true;}
  try{
   // The server's decisions, as Email/get answers them.
   await expect.poll(async()=>{
    // New mail reaches the store when the client syncs; on a fresh account the
    // upstream connection can drop once while the seed creates the Sent folder,
    // and a reload reconnects.
    await page.reload();await expect(page.getByRole('button',{name:'Compose',exact:true}).first()).toBeVisible({timeout:60000});
    // Listed from the Inbox, not searched: on a fresh server the index may still be filling.
    const inbox=((await jmap(page,[['Mailbox/get',{accountId,ids:null},'m']]))[0]![1]['list'] as Array<{id:string;role:string|null}>).find(b=>b.role==='inbox')!.id;
    const q=await jmap(page,[['Email/query',{accountId,filter:{inMailbox:inbox},sort:[{property:'receivedAt',isAscending:false}],limit:100},'q'],['Email/get',{accountId,'#ids':{resultOf:'q',name:'Email/query',path:'/ids'},properties:['subject','mailboxIds','mailwomanFocus']},'g']]);
    const list=q[1]![1]['list'] as Array<{subject:string;mailwomanFocus?:{tab:string;reason:string}}>;
    return Object.fromEntries((Object.keys(S) as Key[]).map(k=>[k,list.find(e=>e.subject===S[k].subject)?.mailwomanFocus?.reason??null]));
   },{timeout:150000,intervals:[2000,5000,10000],message:'mailwomanFocus reasons'}).toEqual({news:'listHeader',contact:'contact',wrote:'correspondent',thread:'thread',person:'default'});
   await page.reload();
   await expect(tabs(page)).toBeVisible({timeout:60000});
   await expect.poll(()=>split(page),{timeout:60000}).toEqual({news:'Other',contact:'Focused',wrote:'Focused',thread:'Focused',person:'Focused'});
   await openTab(page,'Focused');await page.screenshot({path:shot(info,`split-focused-${width}`)});
   await openTab(page,'Other');await page.screenshot({path:shot(info,`split-other-${width}`)});

   // Row menu: Always move to Focused, at once, with Undo.
   await row(page,'news').hover();
   await row(page,'news').click({button:'right'});
   const menu=page.getByRole('menu');
   await expect(menu.getByRole('menuitemcheckbox',{name:'Always move to Focused'})).toBeVisible();
   await page.screenshot({path:shot(info,`row-menu-${width}`)});
   const put=page.waitForResponse(r=>new URL(r.url()).pathname==='/api/account/focused-senders'&&r.request().method()==='PUT');
   await menu.getByRole('menuitemcheckbox',{name:'Always move to Focused'}).click();
   await expect(row(page,'news')).toHaveCount(0);
   expect((await put).status()).toBe(200);
   const toast=page.getByText(`Messages from ${addr('news')} now go to Focused.`);
   await expect(toast).toBeVisible();
   await page.screenshot({path:shot(info,`moved-toast-${width}`)});
   await page.getByRole('button',{name:'Undo',exact:true}).click();
   await expect(row(page,'news')).toBeVisible({timeout:15000});
   await expect.poll(async()=>((await (await page.request.get('/api/account/focused-senders')).json()) as {senders:unknown[]}).senders.length).toBe(0);

   // Reader More menu: the same choice, then a reload keeps it.
   await row(page,'news').click();
   const more=page.getByTestId('reader-more').first();
   await expect(more).toBeVisible({timeout:30000});
   await more.click();
   await page.getByRole('menu').getByRole('menuitemcheckbox',{name:'Always move to Focused'}).click();
   await expect.poll(async()=>((await (await page.request.get('/api/account/focused-senders')).json()) as {senders:Array<{email:string;tab:string}>}).senders).toEqual([expect.objectContaining({email:addr('news'),tab:'focused'})]);
   await page.reload();
   // The message reopens as a workspace tab; at phone width it covers the list.
   await expect(page.getByRole('button',{name:'Compose',exact:true}).first()).toBeVisible({timeout:60000});
   if(!await tabs(page).isVisible())await backToMail(page);
   await expect(tabs(page)).toBeVisible({timeout:60000});
   await expect.poll(()=>split(page),{timeout:60000}).toEqual({news:'Focused',contact:'Focused',wrote:'Focused',thread:'Focused',person:'Focused'});
   await openTab(page,'Focused');await page.screenshot({path:shot(info,`after-reload-${width}`)});

   // Settings: the manage list shows it; Remove sends it back to Other.
   await openFolderSettings(page);
   const item=page.getByTestId('focused-sender').filter({hasText:addr('news')});
   await expect(item).toContainText('Always Focused');
   await page.getByTestId('focused-senders').scrollIntoViewIfNeeded();
   await page.screenshot({path:shot(info,`manage-list-${width}`)});
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
   await item.getByRole('button',{name:`Remove ${addr('news')}`}).click();
   await expect(page.getByTestId('focused-senders-empty')).toBeVisible();
   await page.screenshot({path:shot(info,`manage-empty-${width}`)});
   await backToMail(page);
   await expect(tabs(page)).toBeVisible({timeout:30000});
   await expect.poll(()=>split(page),{timeout:30000}).toMatchObject({news:'Other'});
  }finally{
   await clearOverrides(page);
   await setFocused(page,false);
  }
 });
}
