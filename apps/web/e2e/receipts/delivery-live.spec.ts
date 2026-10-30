import {test,expect} from '../csrf.ts';
import {readFile,writeFile,unlink,mkdir} from 'node:fs/promises';
import path from 'node:path';
const capture=process.env['MW_DSN_SMTP_CAPTURE']??'test-results/receipts-smtp';
type Accepted={commands:string[];sha256:string;replyDropped:boolean};
async function accepted():Promise<Accepted[]>{try{return (await readFile(path.join(capture,'accepted.jsonl'),'utf8')).trim().split('\n').filter(Boolean).map(line=>JSON.parse(line) as Accepted);}catch{return [];}}
test('persists the delivery default, sends actual DSN options, and shows uncertain SMTP without resend',async({page},info)=>{
 await mkdir(capture,{recursive:true});await unlink(path.join(capture,'drop-final')).catch(()=>{});
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');
 const login=page.waitForResponse(response=>response.url().endsWith('/api/login')&&response.request().method()==='POST');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 const identity=await (await login).json() as {accountId:string;storageAuthority:string};expect(identity.accountId).toBeTruthy();expect(identity.storageAuthority).toBeTruthy();
 const configured=await page.request.post('/api/account/identities',{headers:{'x-mailwoman-account':identity.accountId,'x-mailwoman-storage-authority':identity.storageAuthority},data:{name:'DSN fixture',email:'testuser'}});expect(configured.ok()).toBe(true);
 if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
 await page.getByRole('button',{name:'Settings',exact:true}).click();await page.getByRole('button',{name:'Read receipts',exact:true}).click();
 await page.getByRole('button',{name:'Edit delivery notification default'}).click();
 const preference=page.getByLabel('Request delivery notifications for new messages when supported');await expect(preference).toBeEnabled();
 if(!await preference.isChecked()){await preference.check();await page.getByRole('button',{name:'Save receipt policy'}).click();}else{await page.getByRole('button',{name:'Back to read receipts',exact:true}).click();}
 await expect(page.getByRole('button',{name:'Edit delivery notification default'})).toBeVisible();
 await page.reload();
 const compose=async(subject:string)=>{
  await page.getByRole('button',{name:'Compose',exact:true}).click();
  await page.getByLabel('To',{exact:true}).fill('recipient@example.org');await page.getByLabel('Subject',{exact:true}).fill(subject);
  await page.getByLabel('Body',{exact:true}).fill('Synthetic delivery notification acceptance fixture.');
  await expect(page.getByTestId('compose-request-delivery')).toBeChecked();await expect(page.getByTestId('compose-request-delivery')).toBeEnabled();
 };
 const start=(await accepted()).length;
 const subject=`DSN browser ${info.project.name} ${Date.now()}`;
 await compose(subject);await page.screenshot({path:info.outputPath('dsn-compose.png'),fullPage:true});await page.getByRole('button',{name:'Send',exact:true}).click();
 await expect.poll(async()=> (await accepted()).length,{timeout:35000}).toBe(start+1);
 const sent=(await accepted())[start]!;expect(sent.commands.find(command=>command.startsWith('MAIL FROM:'))).toContain('RET=HDRS');
 expect(sent.commands.filter(command=>command.startsWith('RCPT TO:')).every(command=>command.includes('NOTIFY=SUCCESS,FAILURE,DELAY'))).toBe(true);
 const raw=await readFile(path.join(capture,`${sent.sha256}.eml`),'utf8');expect(raw).toContain(subject);expect(raw).not.toContain('Disposition-Notification-To:');
 const uncertain=`Uncertain ${info.project.name} ${Date.now()}`;
 await writeFile(path.join(capture,'drop-final'),'1');
 try {
  await compose(uncertain);await page.getByRole('button',{name:'Send',exact:true}).click();
  await expect.poll(async()=> (await accepted()).length,{timeout:35000}).toBe(start+2);
  if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
  await page.getByRole('button',{name:'Outbox',exact:true}).click();
  const row=page.locator('.outbox__row').filter({hasText:uncertain});await expect(row).toHaveAttribute('data-state','uncertain');
  await expect(row).toContainText('Delivery status unknown');await expect(row).toContainText('will not be resent automatically');await expect(row.getByRole('button')).toHaveCount(0);
  await page.waitForTimeout(1500);expect((await accepted()).length).toBe(start+2);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.screenshot({path:info.outputPath('delivery-unknown-outbox.png'),fullPage:true});
 } finally {await unlink(path.join(capture,'drop-final')).catch(()=>{});}
});
