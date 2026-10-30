import {test,expect,type Page} from '../csrf.ts';
import {messageRow,signInManually} from '../helpers.ts';

// Two devices, one account (t39 L8): what device A does to messages and folders
// shows on device B through the realtime channel, with no reload on B. Device A
// acts through its own UI (the list's context menu); the folder create and rename
// go through A's signed-in JMAP session (the web app has no folder-create UI).
// B is never reloaded: a marker set on its window must survive to the end.
const shots=process.env['MW_E2E_SCREENSHOT_DIR'];

async function shot(page:Page,name:string,info:{project:{name:string};outputPath:(name:string)=>string}):Promise<void>{
 await page.screenshot({path:shots?`${shots}/${info.project.name}-${name}.png`:info.outputPath(`${name}.png`),fullPage:true});
}

test('read, flag, move, delete and folder changes on device A show live on device B',async({page,browser,baseURL,playwright},info)=>{
 test.skip(info.project.name==='phone','two desktop browser contexts');
 const stamp=`${Date.now()}-${info.project.name}`;
 const subjects={read:`Sync read ${stamp}`,open:`Sync open ${stamp}`,gone:`Sync delete ${stamp}`};
 const other=await browser.newContext({...(baseURL?{baseURL}:{}),viewport:{width:1280,height:900}});const b=await other.newPage();
 let call:((name:string,args:Record<string,unknown>)=>Promise<Record<string,unknown>>)|undefined;let folderId:string|undefined;
 try{
  await page.goto('/');await signInManually(page,{username:'testuser',password:'testpass'});
  await expect(page.getByRole('button',{name:'Compose',exact:true}).first()).toBeVisible();
  const session=await(await page.request.get('/jmap/session')).json() as {primaryAccounts:Record<string,string>};const accountId=session.primaryAccounts['urn:ietf:params:jmap:mail']!;
  call=async(name,args)=>{const send=()=>page.request.post('/jmap/api',{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[[name,{accountId,...args},'c']]}});let response=await send();
   // The server caps concurrent JMAP requests per account; like the web client, retry a 429.
   for(let attempt=0;response.status()===429&&attempt<40;attempt++){await new Promise(done=>setTimeout(done,50));response=await send();}
   expect(response.ok(),`${name} ${response.status()} ${await response.text()}`).toBe(true);const body=await response.json() as {methodResponses:[string,Record<string,unknown>][]};expect(body.methodResponses[0]![0],JSON.stringify(body)).toBe(name);return body.methodResponses[0]![1];};
  let boxes=(await call('Mailbox/get',{})).list as {id:string;role:string|null;name:string}[];
  // Greenmail has only INBOX; the engine gives Archive and Trash their roles by name.
  const missing=Object.fromEntries((['archive','trash'] as const).filter(role=>!boxes.some(box=>box.role===role)).map(role=>[role,{name:role==='archive'?'Archive':'Trash'}]));
  if(Object.keys(missing).length){await call('Mailbox/set',{create:missing});boxes=(await call('Mailbox/get',{})).list as typeof boxes;}
  const inbox=boxes.find(box=>box.role==='inbox')!;const archive=boxes.find(box=>box.role==='archive')!;
  for(const subject of Object.values(subjects)){
   const raw=`From: sync@example.org\r\nTo: testuser@example.org\r\nSubject: ${subject}\r\nMessage-ID: <${subject.replaceAll(' ','-')}@fixture.invalid>\r\nContent-Type: text/plain\r\n\r\nCross-device fixture: ${subject}.\r\n`;
   const upload=await page.request.post(`/jmap/upload/${accountId}`,{data:raw,headers:{'Content-Type':'message/rfc822'}});expect(upload.ok()).toBe(true);const {blobId}=await upload.json() as {blobId:string};
   const imported=await call('Email/import',{emails:{m:{blobId,mailboxIds:{[inbox.id]:true},keywords:{}}}});expect((imported.created as Record<string,{id:string}>).m?.id).toBeTruthy();
  }
  await page.reload();
  await b.goto('/');await signInManually(b,{username:'testuser',password:'testpass'});
  await expect(b.getByRole('button',{name:'Compose',exact:true}).first()).toBeVisible();
  await b.evaluate(()=>{(window as unknown as {__mwNoReload:boolean}).__mwNoReload=true;});
  for(const device of [page,b])for(const subject of Object.values(subjects))await expect(messageRow(device,subject)).toBeVisible();
  await expect(messageRow(b,subjects.read)).toHaveClass(/list__row--unread/);
  const menu=async(subject:string,item:string)=>{await messageRow(page,subject).click({button:'right'});await page.getByRole('menuitem',{name:item,exact:true}).click();};

  // Read and flag on A: B's row changes in place.
  await menu(subjects.read,'Mark read');
  await expect(messageRow(b,subjects.read)).not.toHaveClass(/list__row--unread/,{timeout:15000});
  await menu(subjects.read,'Flag');
  await expect(messageRow(b,subjects.read).getByTestId('row-flagged')).toBeVisible({timeout:15000});
  await shot(b,'sync-read-flag-on-b',info);

  // B is reading a message when A archives it: B keeps the open message and its
  // place in the list, and is told where it went, without a dialog.
  await messageRow(b,subjects.open).click();
  const reader=b.getByRole('heading',{name:subjects.open,exact:true});
  await expect(reader).toBeVisible();
  // Read by a third session, so A's and B's own refreshes do not compete with it.
  // This count-only session belongs to device A. Reuse its user agent so the
  // probe does not introduce a third-device security notice into Inbox mid-move.
  const probe=await playwright.request.newContext({...(baseURL?{baseURL}:{}),userAgent:await page.evaluate(()=>navigator.userAgent)});
  expect((await probe.post('/api/login',{data:{username:'testuser',password:'testpass'}})).ok()).toBe(true);
  // The server caps concurrent JMAP requests; like the web client, retry a 429 at once.
  const counts=async()=>{let response=await probe.post('/jmap/api',{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Mailbox/get',{accountId,ids:[inbox.id,archive.id]},'c']]}});
   for(let attempt=0;response.status()===429&&attempt<40;attempt++){await new Promise(done=>setTimeout(done,50));response=await probe.post('/jmap/api',{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Mailbox/get',{accountId,ids:[inbox.id,archive.id]},'c']]}});}
   expect(response.ok(),await response.text()).toBe(true);const list=(await response.json()).methodResponses[0][1].list as {id:string;totalEmails:number;unreadEmails:number}[];return Object.fromEntries(list.map(box=>[box.id===inbox.id?'inbox':'archive',[box.unreadEmails,box.totalEmails]]));};
  const before=await counts();
  const moved=page.waitForResponse(response=>response.status()===200&&response.url().endsWith('/jmap/api')&&(response.request().postData()??'').includes('Email/set')&&(response.request().postData()??'').includes(archive.id));
  await menu(subjects.open,'Archive');const moveResult=await(await moved).json();const setResult=moveResult.methodResponses.find((value:[string,unknown])=>value[0]==='Email/set')?.[1];expect(setResult?.notUpdated).toBeUndefined();expect(Object.keys(setResult?.updated??{})).toHaveLength(1);
  // The counts the server reports right after the move, before any upstream
  // resync: the message left the Inbox total and joined Archive's (t39-e1's
  // stale-count report; it stayed in the Inbox total until a resync).
  const after=await counts();
  expect(after.inbox![1]).toBe(before.inbox![1]!-1);expect(after.archive![1]).toBe(before.archive![1]!+1);
  await expect(messageRow(b,subjects.open)).toHaveCount(0,{timeout:15000});
  const notice=b.getByTestId('reader-remote-change');
  await expect(notice).toBeVisible({timeout:15000});
  await expect(b.getByTestId('reader-remote-change-region')).toHaveAttribute('role','status');
  await expect(notice).toContainText(/moved to \W?Archive\W? from another device or app/);
  await expect(reader).toBeVisible();
  await expect(b.getByRole('dialog')).toHaveCount(0);
  await shot(b,'sync-moved-while-reading-on-b',info);

  // Delete on A: the row leaves B's list.
  await menu(subjects.gone,'Delete');
  await expect(messageRow(b,subjects.gone)).toHaveCount(0,{timeout:15000});

  // A folder created and renamed on A appears, then renames, in B's folder list.
  const created=await call('Mailbox/set',{create:{f:{name:`Sync folder ${stamp}`}}});folderId=(created.created as Record<string,{id:string}>|undefined)?.f?.id;expect(folderId,JSON.stringify(created)).toBeTruthy();
  const folders=b.getByRole('navigation',{name:'Mailboxes'});
  await expect(folders.getByText(`Sync folder ${stamp}`,{exact:true})).toBeVisible({timeout:15000});
  await call('Mailbox/set',{update:{[folderId!]:{name:`Sync renamed ${stamp}`}}});
  await expect(folders.getByText(`Sync renamed ${stamp}`,{exact:true})).toBeVisible({timeout:15000});
  await expect(folders.getByText(`Sync folder ${stamp}`,{exact:true})).toHaveCount(0);
  await shot(b,'sync-folder-renamed-on-b',info);

  expect(await b.evaluate(()=>(window as unknown as {__mwNoReload?:boolean}).__mwNoReload)).toBe(true);
 }finally{
  if(call&&folderId)await call('Mailbox/set',{destroy:[folderId]}).catch(()=>undefined);
  await other.close();
 }
});
