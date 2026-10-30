import {test,expect} from '../csrf.ts';
import {sendRawSmtp} from '../crypto-helpers.ts';
import {postJmap} from '../helpers.ts';

test('moving a message to Trash shows one progress toast then the same result with Undo',async({page},info)=>{
 const subject=`Trash progress ${info.project.name} ${Date.now()}`,sender=`trash-${Date.now()}@example.org`;
 await sendRawSmtp({from:sender,to:'testuser@example.org',message:`From: ${sender}\r\nTo: testuser@example.org\r\nSubject: ${subject}\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <${sender}>\r\n\r\nDisposable progress fixture.\r\n`});
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
 const session=await(await page.request.get('/jmap/session')).json(),accountId=session.primaryAccounts['urn:ietf:params:jmap:mail'];
 const call=async(name:string,args:Record<string,unknown>)=>{const response=await postJmap(page.request,{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[[name,{accountId,...args},'c']]}});expect(response.ok()).toBe(true);const body=await response.json();expect(body.methodResponses[0][0]).toBe(name);return body.methodResponses[0][1];};
 let boxes=(await call('Mailbox/get',{})).list as {id:string;role:string|null}[];
 if(!boxes.some(box=>box.role==='trash')){await call('Mailbox/set',{create:{trash:{name:'Trash'}}});boxes=(await call('Mailbox/get',{})).list;}
 const trash=boxes.find(box=>box.role==='trash')!,inbox=boxes.find(box=>box.role==='inbox')!;
 expect(trash).toBeTruthy();await expect.poll(async()=>(await call('Email/query',{filter:{from:sender}})).ids.length).toBe(1);
 const id=(await call('Email/query',{filter:{from:sender}})).ids[0] as string;
 let release!:()=>void;const held=new Promise<void>(resolve=>{release=resolve;});let intercept=true;
 await page.route('**/jmap/api',async route=>{
  const body=route.request().postDataJSON() as {methodCalls?:[string,{update?:Record<string,{mailboxIds?:Record<string,boolean>}>},string][]};
  if(intercept&&body.methodCalls?.some(([name,args])=>name==='Email/set'&&args.update?.[id]?.mailboxIds?.[trash.id])){intercept=false;await held;}
  await route.continue();
 });
 try{
  await page.reload();const row=page.locator('.list__row').filter({hasText:subject});await expect(row).toBeVisible();
  if(info.project.name==='phone')await row.locator('..').getByTestId('msg-more-toggle').click();else await row.click({button:'right'});
  await page.getByRole('menuitem',{name:'Delete',exact:true}).click();
  const progress=page.getByTestId('mail-move-progress');await expect(progress).toHaveAttribute('data-busy','true');await expect(progress).toContainText('Trash');await expect(progress).toContainText('one message');
  const toastId=await progress.getAttribute('data-toast-id');await page.screenshot({path:info.outputPath('trash-progress.png'),fullPage:true});release();
  const result=page.locator(`[data-toast-id="${toastId}"]`);await expect(result.getByRole('button',{name:'Undo',exact:true})).toBeVisible();await expect(result).not.toHaveAttribute('data-busy','true');
  await expect(page.getByTestId('mail-move-progress')).toHaveCount(0);
  await result.getByRole('button',{name:'Undo',exact:true}).click();await expect(row).toBeVisible();
 }finally{release();await page.unroute('**/jmap/api');await call('Email/set',{update:{[id]:{mailboxIds:{[inbox.id]:true}}}});}
});
