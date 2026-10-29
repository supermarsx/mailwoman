import {test,expect} from '../csrf.ts';
import {readFile,unlink} from 'node:fs/promises';
import path from 'node:path';
const capture=process.env['MW_DSN_SMTP_CAPTURE']??'F:/Projects/mailwoman/.orchestration/reports/codex-outgoing-dsn-smtp';
type Accepted={commands:string[];sha256:string;replyDropped:boolean};
async function accepted():Promise<Accepted[]>{try{return (await readFile(path.join(capture,'accepted.jsonl'),'utf8')).trim().split('\n').filter(Boolean).map(line=>JSON.parse(line) as Accepted);}catch{return [];}}
test('authors HTML and independent plain signature and sends each selected format through SMTP',async({page},info)=>{
 await unlink(path.join(capture,'drop-final')).catch(()=>{});
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');
 await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
 await page.getByRole('button',{name:'Settings',exact:true}).click();await page.getByRole('button',{name:'Signatures',exact:true}).click();
 await page.getByRole('button',{name:'New signature',exact:true}).click();
 const name=`Formats ${info.project.name} ${Date.now()}`;
 await page.getByLabel('Name',{exact:true}).fill(name);await page.getByRole('combobox',{name:'Signature format'}).selectOption('html');
 await page.getByRole('textbox',{name:'HTML source'}).fill('<p>Rich <strong>team signature</strong><script>evil()</script></p>');
 await page.getByLabel('Plain-text alternative').fill('Independent plain <signature> & team.');
 await page.screenshot({path:info.outputPath('signature-format-editor.png'),fullPage:true});
 await page.getByTestId('signature-save').click();await expect(page.getByTestId('signature-list')).toContainText(name);
 const templates=await (await page.request.get('/api/account/signatures')).json() as {signatures:{name:string;html:string;text:string}[]};
 const template=templates.signatures.find(value=>value.name===name)!;expect(template.html).toContain('<strong>team signature</strong>');expect(template.html).not.toContain('<script');
 expect(template.text).toBe('Independent plain <signature> & team.');
 const identityResponse=await page.request.post('/api/account/identities',{data:{name,email:'testuser',signatureName:name}});expect(identityResponse.ok()).toBe(true);
 const identity=await identityResponse.json() as {id:string};
 await page.reload();
 for(const format of ['plain','html'] as const){
  await page.getByRole('button',{name:'Compose',exact:true}).click();await page.getByRole('combobox',{name:'From',exact:true}).selectOption(identity.id);
  await page.getByTestId('format-toggle').selectOption(format==='plain'?'plain':'rich');
  await page.getByLabel('To',{exact:true}).fill('recipient@example.org');
  const subject=`${format} ${name}`;await page.getByLabel('Subject',{exact:true}).fill(subject);await page.getByLabel('Body',{exact:true}).fill('Hello recipient.');
  const start=(await accepted()).length;await page.getByRole('button',{name:'Send',exact:true}).click();
  await expect.poll(async()=> (await accepted()).length,{timeout:35000}).toBe(start+1);
  const sent=(await accepted())[start]!;const raw=await readFile(path.join(capture,`${sent.sha256}.eml`),'utf8');expect(raw).toContain(subject);expect(raw).not.toContain('<script');
  if(format==='plain'){expect(raw).toMatch(/Content-Type: text\/plain/i);expect(raw).toContain('Independent plain <signature> & team.');expect(raw).not.toContain('<strong>');}
  else{expect(raw).toMatch(/Content-Type: text\/html/i);expect(raw).toContain('<strong>team signature</strong>');expect(raw).not.toContain('Independent plain');}
 }
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
});
