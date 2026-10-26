import {test,expect} from '../csrf.ts';
import {signInManually,messageRow,sidebarInbox,drawerButton} from '../helpers.ts';
import {sendRawSmtp} from '../crypto-helpers.ts';
test('print action opens an isolated document suitable for PDF without remote resources @phone',async({page})=>{
 const subject=`Printable message ${Date.now()}`;
 await sendRawSmtp({from:'sender@example.test',message:`From: sender@example.test\r\nTo: testuser@example.org\r\nSubject: ${subject}\r\nMessage-ID: <print-${Date.now()}@example.test>\r\nDate: ${new Date().toUTCString()}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nA printable invoice with a private local PDF destination.\r\n`});
 await signInManually(page,{username:'testuser',password:'testpass'});
 await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 await expect(async()=>{if(test.info().project.name==='phone')await drawerButton(page).click();await sidebarInbox(page).click();await expect(messageRow(page,subject)).toBeVisible({timeout:3000});}).toPass({timeout:30000});
 await messageRow(page,subject).click();
 // Observe the platform print invocation without opening an interactive OS dialog.
 await page.evaluate(()=>{
  const host=window as typeof window&{printDocument?:string};
  new MutationObserver(records=>{for(const record of records)for(const node of record.addedNodes){
   if(node instanceof HTMLIFrameElement&&node.getAttribute('aria-hidden')==='true'&&node.contentWindow){
    node.contentWindow.print=()=>{host.printDocument=node.contentDocument?.documentElement.outerHTML??'';};
   }
  }}).observe(document.body,{childList:true});
 });
 await page.getByRole('button',{name:'Print / Save as PDF',exact:true}).click();
 await expect.poll(()=>page.evaluate(()=>(window as typeof window&{printDocument?:string}).printDocument??'')).toContain('A printable invoice');
 const printed=await page.evaluate(()=>(window as typeof window&{printDocument?:string}).printDocument!);
 expect(printed).toContain(subject);expect(printed).toContain("default-src 'none'");
 expect(printed).toContain("script-src 'none'");
 const printFrame=page.locator('iframe[aria-hidden="true"]').last();
 expect(await printFrame.getAttribute('sandbox')).toBe('allow-same-origin allow-modals');
 if(test.info().project.name==='chromium'){
  const output=await page.context().newPage();await output.setContent(printed);await output.pdf({path:test.info().outputPath('message.pdf'),format:'A4'});await output.close();
 }
});
