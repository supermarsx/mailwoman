import {test,expect} from '../csrf.ts';
import {readFile,unlink} from 'node:fs/promises';
import path from 'node:path';
import {pinComposerRibbonOnPhone,revealRibbonControl} from '../helpers.ts';
let restoreRibbon:()=>Promise<void>=async()=>{};
test.beforeEach(async({playwright,baseURL},info)=>{restoreRibbon=await pinComposerRibbonOnPhone(playwright.request,baseURL,info.project.name,{username:'parent@example.org',password:'parent-pass'});});
test.afterEach(async()=>{await restoreRibbon();});
const capture=process.env['MW_DSN_SMTP_CAPTURE']??'F:/Projects/mailwoman/.orchestration/reports/codex-outgoing-dsn-smtp';
type Accepted={commands:string[];sha256:string;replyDropped:boolean};
async function accepted():Promise<Accepted[]>{try{return (await readFile(path.join(capture,'accepted.jsonl'),'utf8')).trim().split('\n').filter(Boolean).map(line=>JSON.parse(line) as Accepted);}catch{return [];}}
test('authors HTML and independent plain signature and sends each selected format through SMTP',async({page},info)=>{
 await unlink(path.join(capture,'drop-final')).catch(()=>{});
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('parent@example.org');await page.getByLabel('Password',{exact:true}).fill('parent-pass');
 await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.locator(info.project.name==='phone'?'header.shell__bar button.shell__bar-compose':'button.sidebar__compose')).toBeVisible();
 if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
 await page.getByRole('button',{name:'Settings',exact:true}).click();await page.getByRole('button',{name:'Signatures',exact:true}).click();
 await page.getByRole('button',{name:'New signature',exact:true}).click();
 const name=`Formats ${info.project.name} ${Date.now()}`;
 await page.getByLabel('Name',{exact:true}).fill(name);await (await revealRibbonControl(page,page.locator('[data-testid=signature-editor]'),page.getByRole('combobox',{name:'Signature format'}))).selectOption('html');
 await page.getByRole('textbox',{name:'HTML source'}).fill('<p>Rich <strong>team signature</strong><script>evil()</script></p>');
 await page.getByLabel('Plain-text alternative').fill('Independent plain <signature> & team.');
 await page.screenshot({path:info.outputPath('signature-format-editor.png'),fullPage:true});
 await page.getByTestId('signature-save').click();await expect(page.getByTestId('signature-list')).toContainText(name);
 const templates=await (await page.request.get('/api/account/signatures')).json() as {signatures:{name:string;html:string;text:string}[]};
 const template=templates.signatures.find(value=>value.name===name)!;expect(template.html).toContain('<strong>team signature</strong>');expect(template.html).not.toContain('<script');
 expect(template.text).toBe('Independent plain <signature> & team.');
 const me=await (await page.request.get('/api/me')).json() as {storageAuthority:string};const identityResponse=await page.request.post('/api/account/identities',{headers:{'x-mailwoman-storage-authority':me.storageAuthority},data:{name,email:'parent@example.org',signatureName:name}});expect(identityResponse.ok()).toBe(true);
 const identity=await identityResponse.json() as {id:string};
 // e0c99a79: the From picker is hidden unless mail.showFrom is on (or the ribbon's From button is pressed).
 const authority={'x-mailwoman-storage-authority':me.storageAuthority};
 expect((await page.request.put('/api/settings/mail.showFrom',{headers:authority,data:{value:true}})).ok()).toBe(true);
 try{
 await page.reload();
 for(const format of ['plain','html'] as const){
  await page.locator(info.project.name==='phone'?'header.shell__bar button.shell__bar-compose':'button.sidebar__compose').click();await page.getByRole('combobox',{name:'From',exact:true}).selectOption(identity.id);
  await (await revealRibbonControl(page,page.locator('.compose-ribbon').last(),page.getByTestId('format-toggle'))).selectOption(format==='plain'?'plain':'rich');
  await page.getByLabel('To',{exact:true}).fill('recipient@example.org');
  const subject=`${format} ${name}`;await page.getByLabel('Subject',{exact:true}).fill(subject);await page.getByLabel('Body',{exact:true}).fill('Hello recipient.');
  const start=(await accepted()).length;await page.getByRole('button',{name:'Send',exact:true}).click();
  await expect.poll(async()=> (await accepted()).length,{timeout:35000}).toBe(start+1);
  const sent=(await accepted())[start]!;const raw=await readFile(path.join(capture,`${sent.sha256}.eml`),'utf8');expect(raw).toContain(subject);expect(raw).not.toContain('<script');
  if(format==='plain'){expect(raw).toMatch(/Content-Type: text\/plain/i);expect(raw).toContain('Independent plain <signature> & team.');expect(raw).not.toContain('<strong>');}
  else{expect(raw).toMatch(/Content-Type: text\/html/i);expect(raw).toContain('<strong>team signature</strong>');expect(raw).not.toContain('Independent plain');}
 }
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 }finally{expect((await page.request.delete('/api/settings/mail.showFrom',{headers:authority})).ok()).toBe(true);expect((await page.request.delete(`/api/account/identities/${encodeURIComponent(identity.id)}`,{headers:authority})).ok()).toBe(true);expect((await page.request.delete(`/api/account/signatures/${encodeURIComponent(name)}`,{headers:authority})).ok()).toBe(true);}
});
