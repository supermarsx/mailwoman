import {test,expect} from '../csrf.ts';
import {sendRawSmtp} from '../crypto-helpers.ts';
import {postJmap,useTabbedWorkspace,withUserSetting} from '../helpers.ts';

test('reply from a folder tab saves its originating folder instead of the main Inbox',async({page,playwright,baseURL},info)=>{
 const restoreReply=await withUserSetting(playwright.request,baseURL!,'mail.replyCopyToOriginal',true);
 const restoreWindow=await withUserSetting(playwright.request,baseURL!,'mail.composeWindow','tab');
 const restoreDraft=await withUserSetting(playwright.request,baseURL!,'mail.draftSaveMode','manual');
 const subject=`Reply origin ${info.project.name} ${Date.now()}`,folderName=`Projects ${Date.now()}`;
 let restoreLayout=async()=>{},folderId='',emailId='';
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
 const {accountId}=await(await page.request.get('/api/me')).json() as {accountId:string};
 const call=async(name:string,args:Record<string,unknown>)=>{const response=await postJmap(page.request,{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[[name,{accountId,...args},'fixture']]}});expect(response.ok()).toBe(true);const body=await response.json();expect(body.methodResponses[0][0]).toBe(name);return body.methodResponses[0][1];};
 try{
  restoreLayout=await useTabbedWorkspace(page);
  await expect.poll(async()=>(await call('Mailbox/get',{})).mailwomanInitialSync,{timeout:120000}).toBe('ready');
  const inbox=(await call('Mailbox/get',{})).list.find((box:{role:string})=>box.role==='inbox') as {id:string};
  folderId=(await call('Mailbox/set',{create:{projects:{name:folderName}}})).created.projects.id;
  await sendRawSmtp({from:'origin@example.org',to:'testuser@example.org',message:`From: origin@example.org\r\nTo: testuser@example.org\r\nSubject: ${subject}\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <${Date.now()}-origin@example.org>\r\n\r\nReply from the Projects folder tab.\r\n`});
  await expect.poll(async()=>(await call('Email/query',{filter:{subject}})).ids.length,{timeout:30000}).toBe(1);emailId=(await call('Email/query',{filter:{subject}})).ids[0];
  // The engine store admits one mailbox per message. This proves the real
  // Projects gesture; overlapping memberships are covered by the workspace unit.
  const updated=await call('Email/set',{update:{[emailId]:{mailboxIds:{[folderId]:true}}}});expect(updated.notUpdated).toBeUndefined();
  await expect.poll(async()=>(await call('Email/get',{ids:[emailId],properties:['id','mailboxIds']})).list[0]?.mailboxIds).toEqual({[folderId]:true});expect(folderId).not.toBe(inbox.id);
  await page.reload();await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
  const mainFolder=await page.locator('#shell-nav .sidebar__box--active').first().getAttribute('aria-label');expect(mainFolder).toBe('Inbox');
  await page.locator('#shell-nav').getByRole('button',{name:folderName,exact:true}).click({button:'right'});await page.getByRole('menuitem',{name:'Open in new tab',exact:true}).click();
  const pane=page.getByRole('tabpanel',{name:folderName,exact:true});await pane.getByRole('button').filter({hasText:subject}).click();await page.getByTestId('reader-reply').click();
  const compose=page.locator('form.compose').last();await expect(compose).toBeVisible();
  await expect.poll(()=>page.evaluate(expected=>{const drafts=JSON.parse(localStorage.getItem('mw.compose.drafts.v2')??'[]') as {subject:string;sentMailboxId?:string}[];return drafts.find(draft=>draft.subject.includes(expected))?.sentMailboxId;},subject)).toBe(folderId);
  await page.screenshot({path:info.outputPath('reply-copy-origin.png'),fullPage:true});
  await compose.getByRole('button',{name:'Discard account draft',exact:true}).click();await page.getByTestId('compose-discard-confirm').getByRole('button',{name:'Discard',exact:true}).click();await expect(compose).toHaveCount(0);
 }finally{
  if(emailId)await call('Email/set',{destroy:[emailId]});if(folderId)await call('Mailbox/set',{destroy:[folderId]});await restoreLayout();await restoreDraft();await restoreWindow();await restoreReply();
 }
});
