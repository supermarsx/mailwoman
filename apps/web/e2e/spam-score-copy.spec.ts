import {test,expect,type Page} from '@playwright/test';
import {login,messageRow} from './helpers.ts';
test.use({serviceWorkers:'block'});test.setTimeout(90_000);
const HEADERS='From: Carlos <carlos@example.org>\r\nSubject: Lunch tomorrow?\r\nX-Spam-Status: Yes, score=7.5 required=5.0 tests=PDF_RULE=4.0,HAM_RULE=-0.5\r\nX-Spam-Report: * 4.0 PDF_RULE PDF attachment heuristic\r\n * -0.5 HAM_RULE Known sender reputation\r\n\r\nBody';
async function fixture(page:Page){
 await page.addLocatorHandler(page.getByRole('dialog',{name:'Allow notifications?',exact:true}),async dialog=>{await dialog.getByRole('button',{name:'Not now',exact:true}).click();});
 await page.route(/\/jmap\/download\//,async route=>{
  if(new URL(route.request().url()).pathname.endsWith('/message.eml')){await route.fulfill({status:200,contentType:'message/rfc822',body:HEADERS});return;}
  await route.continue();
 });
}

test('SpamAssassin Copy breakdown sends the displayed rule details to the clipboard',async({page})=>{
 await page.addInitScript(()=>{
  const state=window as unknown as {spamCopies:string[]};state.spamCopies=[];
  Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async(text:string)=>{state.spamCopies.push(text);}}});
 });
 await fixture(page);await login(page);await messageRow(page,'Lunch tomorrow?').click();
 await page.getByRole('button',{name:/SpamAssassin score 7\.5.*Show the rule breakdown/}).click();
 const breakdown=page.getByTestId('spam-score-breakdown');await expect(breakdown).toBeVisible();await breakdown.getByRole('button',{name:'Copy breakdown',exact:true}).click();
 await expect(breakdown.getByRole('status')).toHaveText('Breakdown copied.');
 const copied=await page.evaluate(()=>(window as unknown as {spamCopies:string[]}).spamCopies);
 expect(copied).toHaveLength(1);expect(copied[0]).toContain('SpamAssassin score breakdown');expect(copied[0]).toContain('Total: 7.5');expect(copied[0]).toContain('Threshold: 5');expect(copied[0]).toContain('PDF_RULE\t+4\tPDF attachment heuristic');expect(copied[0]).toContain('HAM_RULE\t−0.5\tKnown sender reputation');
});
