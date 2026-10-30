import {test,expect} from '../csrf.ts';
import {readFile,unlink,writeFile} from 'node:fs/promises';
import path from 'node:path';
import {withUserSetting,revealRibbonControl} from '../helpers.ts';

type Accepted={commands:string[];sha256:string};
type Session={smtpUtf8Offered:boolean;commands:string[]};
const credentials={username:'parent@example.org',password:'parent-pass'};

// Standalone wire acceptance: run on its own receipts stack with
// MW_CAPTURE_SMTPUTF8=1 and MW_STACK_CAPTURE_RELAY=off. No external delivery.
test('SMTPUTF8 preserves Unicode recipients and header-only addresses and refuses absent capability @phone',async({page,playwright,baseURL},info)=>{
 test.setTimeout(180_000);
 expect(process.env['MW_CAPTURE_SMTPUTF8']).toBe('1');
 expect(process.env['MW_STACK_CAPTURE_RELAY']).toBe('off');
 const capture=process.env['MW_DSN_SMTP_CAPTURE'];expect(capture).toBeTruthy();
 const directory=capture!;
 const rows=async<T>(file:string):Promise<T[]>=>{try{return (await readFile(path.join(directory,file),'utf8')).trim().split('\n').filter(Boolean).map(line=>JSON.parse(line) as T);}catch(error){if((error as NodeJS.ErrnoException).code==='ENOENT')return [];throw error;}};
 const restore:Array<()=>Promise<void>>=[];
 let identityId:string|undefined;
 let headers:Record<string,string>|undefined;
 try{
  for(const [key,value] of [['mail.showFrom',true],['mail.composeWindow','tab'],['mail.draftSaveMode','manual'],['ui.bar.composerRibbon','pinned']] as const){
   restore.push(await withUserSetting(playwright.request,baseURL!,key,value,credentials));
  }
  await unlink(path.join(directory,'no-smtputf8')).catch(()=>{});
  await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill(credentials.username);await page.getByLabel('Password',{exact:true}).fill(credentials.password);await page.getByRole('button',{name:'Sign in',exact:true}).click();
  const composeButton=page.locator(info.project.name==='phone'?'header.shell__bar button.shell__bar-compose':'button.sidebar__compose');await expect(composeButton).toBeVisible();
  const me=await (await page.request.get('/api/me')).json() as {storageAuthority:string};headers={'x-mailwoman-storage-authority':me.storageAuthority};
  const created=await page.request.post('/api/account/identities',{headers,data:{name:`SMTPUTF8 ${info.project.name}`,email:credentials.username,replyTo:'réponse@例子.测试'}});expect(created.ok()).toBe(true);identityId=(await created.json() as {id:string}).id;
  await page.reload();
  const send=async(recipient:string,label:string,selectedIdentity?:string,beforeSubmit?:()=>Promise<void>)=>{
   await composeButton.click();
   if(selectedIdentity)await page.getByRole('combobox',{name:'From',exact:true}).selectOption(selectedIdentity);
   await (await revealRibbonControl(page,page.locator('.compose-ribbon').last(),page.getByTestId('format-toggle'))).selectOption('plain');
   await page.getByLabel('To',{exact:true}).fill(recipient);await page.getByLabel('Subject',{exact:true}).fill(label);await page.getByLabel('Body',{exact:true}).fill('Local SMTPUTF8 wire acceptance.');
   // The independent DSN probe may authenticate; finish it before measuring submission refusal.
   await expect(page.getByTestId('compose-request-delivery')).toBeEnabled();
   await beforeSubmit?.();
   await page.getByRole('button',{name:'Send',exact:true}).click();
  };
  const recipient='"用户,>A"@例子.测试';
  for(const [to,headerOnly] of [[recipient,false],['recipient@example.org',true]] as const){
   const start=(await rows<Accepted>('accepted.jsonl')).length;
   const subject=`SMTPUTF8 ${headerOnly?'header':'recipient'} ${info.project.name} ${Date.now()}`;
   await send(to,subject,headerOnly?identityId:undefined);
   await expect.poll(async()=> (await rows<Accepted>('accepted.jsonl')).length,{timeout:40_000}).toBe(start+1);
   const accepted=(await rows<Accepted>('accepted.jsonl'))[start]!;
   expect(accepted.commands.find(command=>command.startsWith('MAIL FROM:'))).toMatch(/\bSMTPUTF8\b/);
   expect(accepted.commands.find(command=>command.startsWith('MAIL FROM:'))).toContain('BODY=8BITMIME');
   expect(accepted.commands.find(command=>command.startsWith('RCPT TO:'))).toContain(`<${to}>`);
   const raw=await readFile(path.join(directory,`${accepted.sha256}.eml`),'utf8');expect(raw).toContain(subject);
   if(headerOnly){expect(raw).toContain('réponse@例子.测试');expect(accepted.commands.every(command=>Array.from(command).every(char=>char.charCodeAt(0)<128))).toBe(true);}
   else expect(raw).toContain(recipient);
  }
  const before=(await rows<Accepted>('accepted.jsonl')).length;
  let sessionsBefore=0;
  const rejected=`SMTPUTF8 refused ${info.project.name} ${Date.now()}`;await send(recipient,rejected,undefined,async()=>{
   sessionsBefore=(await rows<Session>('sessions.jsonl')).length;
   await writeFile(path.join(directory,'no-smtputf8'),'1');
  });
  if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
  await page.locator('#shell-nav').getByRole('button',{name:'Outbox',exact:true}).click();
  const row=page.locator('.outbox__row').filter({hasText:rejected});await expect(row).toHaveAttribute('data-state','failed',{timeout:40_000});
  await expect.poll(async()=> (await rows<Session>('sessions.jsonl')).slice(sessionsBefore).some(session=>!session.smtpUtf8Offered&&session.commands.includes('EHLO')),{timeout:20_000}).toBe(true);
  const refused=(await rows<Session>('sessions.jsonl')).slice(sessionsBefore).filter(session=>!session.smtpUtf8Offered&&session.commands.includes('EHLO'));
  expect(refused.every(session=>session.commands.every(command=>command!=='AUTH'&&!command.startsWith('MAIL FROM:')&&!command.startsWith('RCPT TO:')))).toBe(true);
  expect((await rows<Accepted>('accepted.jsonl')).length).toBe(before);
  await page.screenshot({path:info.outputPath('smtputf8-refused-without-capability.png'),fullPage:true});
 }finally{
  await unlink(path.join(directory,'no-smtputf8')).catch(()=>{});
  if(identityId&&headers)expect((await page.request.delete(`/api/account/identities/${encodeURIComponent(identityId)}`,{headers})).ok()).toBe(true);
  for(const undo of restore.reverse())await undo();
 }
});
