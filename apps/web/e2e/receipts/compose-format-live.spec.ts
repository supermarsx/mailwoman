import {test,expect} from '../csrf.ts';
import {readFile,unlink} from 'node:fs/promises';
import path from 'node:path';
import {pinComposerRibbonOnPhone,revealRibbonControl} from '../helpers.ts';

// 9dab0fe7 (D4): below 600 px the ribbon starts in auto-hide; these legs are about
// what its controls do, so the phone project pins it for the test (helpers.ts).
let unpinRibbon:()=>Promise<void>=async()=>{};
test.beforeEach(async({playwright,baseURL},info)=>{unpinRibbon=await pinComposerRibbonOnPhone(playwright.request,baseURL,info.project.name);});
test.afterEach(async()=>{await unpinRibbon();});
const capture=process.env['MW_DSN_SMTP_CAPTURE']??'test-results/receipts-smtp';
type Accepted={commands:string[];sha256:string;replyDropped:boolean};
async function accepted():Promise<Accepted[]>{try{return (await readFile(path.join(capture,'accepted.jsonl'),'utf8')).trim().split('\n').filter(Boolean).map(line=>JSON.parse(line) as Accepted);}catch{return [];}}
test('plain compose preserves literal text and sends Cc and Bcc without exposing Bcc headers',async({page},info)=>{
 await unlink(path.join(capture,'drop-final')).catch(()=>{});
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');
 await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 await page.getByRole('button',{name:'Compose',exact:true}).click();
 const toggle=page.getByTestId('format-toggle');
 // The message type is a select (e0c99a79); on a phone its ribbon group is folded into a dropdown (t38-e5).
 await revealRibbonControl(page,page.locator('.compose-ribbon').last(),toggle);
 await toggle.selectOption('plain');
 await page.getByTestId('compose-show-cc-bcc').click();
 await page.getByLabel('To',{exact:true}).fill('to@example.org');
 await page.getByLabel('Cc',{exact:true}).fill('copy@example.org');
 await page.getByLabel('Bcc',{exact:true}).fill('hidden@example.org');
 const subject=`Plain compose ${info.project.name} ${Date.now()}`;
 await page.getByLabel('Subject',{exact:true}).fill(subject);
 await page.getByLabel('Body',{exact:true}).fill('Literal <strong>not HTML</strong> & text.\nSecond line.');
 const start=(await accepted()).length;
 await page.screenshot({path:info.outputPath('plain-compose-recipients.png'),fullPage:true});
 await page.getByRole('button',{name:'Send',exact:true}).click();
 await expect.poll(async()=> (await accepted()).length,{timeout:35000}).toBe(start+1);
 const sent=(await accepted())[start]!;expect(sent.replyDropped).toBe(false);
 for(const address of ['to@example.org','copy@example.org','hidden@example.org'])expect(sent.commands.some(command=>command.startsWith('RCPT TO:')&&command.includes(address))).toBe(true);
 const raw=await readFile(path.join(capture,`${sent.sha256}.eml`),'utf8');
 expect(raw).toContain(subject);expect(raw).toMatch(/Content-Type: text\/plain/i);expect(raw).not.toMatch(/Content-Type: text\/html/i);
 expect(raw).not.toMatch(/^Bcc:/mi);expect(raw).not.toContain('hidden@example.org');
 expect(raw).toContain('Literal <strong>not HTML</strong> & text.');
 expect(raw).toMatch(/^Cc:.*copy@example.org/mi);
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
});
