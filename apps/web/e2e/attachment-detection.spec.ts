// Real reader/list/preview UI with a contradictory legacy JMAP flag. Only the
// fixture message's attachment metadata and downloadable bytes are injected;
// security verdict filenames never become attachment IDs.
import {test,expect,type Page} from '@playwright/test';
import {readFile} from 'node:fs/promises';
import {login,messageRow} from './helpers.ts';
import {buildPdf} from './pdf-fixtures.ts';

test.use({serviceWorkers:'block'});test.setTimeout(90_000);
const BLOB='browser-cid-pdf',NAME='iban_Habito.pdf';
const PDF=Buffer.from(buildPdf(2),'latin1');
async function fixture(page:Page){
 page.setDefaultTimeout(15000);
 await page.addLocatorHandler(page.getByRole('dialog',{name:'Allow notifications?',exact:true}),async dialog=>{await dialog.getByRole('button',{name:'Not now',exact:true}).click();});
 const ids=new Set<string>();
 await page.route(/\/jmap\/download\//,async route=>{
  const path=new URL(route.request().url()).pathname;
  if(path.includes(`/${BLOB}/`)){await route.fulfill({status:200,contentType:'application/pdf',body:PDF});return;}
  await route.continue();
 });
 await page.route(/\/jmap(?:\/api)?$/,async route=>{
  if(route.request().method()!=='POST')return route.continue();
  const response=await route.fetch();const body=await response.json() as {methodResponses?:[string,{list?:Record<string,unknown>[]},string][]};
  for(const [method,args] of body.methodResponses??[]){
   if(method!=='Email/get')continue;
   for(const email of args.list??[]){
    if(email['subject']==='Lunch tomorrow?')ids.add(String(email['id']));
    if(!ids.has(String(email['id'])))continue;
    email['hasAttachment']=false;
    email['attachments']=[{partId:'2',blobId:BLOB,type:'application/pdf',name:NAME,size:PDF.length,cid:'invoice@sender',disposition:'inline'}];
   }
  }
  await route.fulfill({response,json:body});
 });
}

test('CID PDF metadata recovers paperclip, menu download, reader files and PDF preview despite a false flag',async({page},info)=>{
 await fixture(page);await login(page);const row=messageRow(page,'Lunch tomorrow?');
 await expect(row.locator('[aria-label="Has attachment"]')).toBeVisible();
 await row.click({button:'right'});
 const download=page.waitForEvent('download');await page.getByRole('menuitem',{name:`Download ${NAME}`,exact:true}).click();
 const file=await download;expect(file.suggestedFilename()).toBe(NAME);expect(await readFile((await file.path())!)).toEqual(PDF);
 await row.click();const bar=page.getByTestId('reader-attachments-bar');await expect(bar).toBeVisible();await expect(bar).toContainText('1 attachment');
 await page.getByTestId('reader-attachments-toggle').click();
 await expect(page.getByTestId('reader-attachments-inline')).toHaveCount(0);
 await page.getByRole('option',{name:`PDF ${NAME}`,exact:true}).click();
 const tab=page.getByRole('tabpanel',{name:NAME,exact:true});await expect(tab).toBeVisible();
 const inner=tab.frameLocator('iframe.mw-viewer__pdf-frame');await expect(inner.locator('.page canvas').first()).toBeVisible({timeout:30000});await expect(inner.locator('#numPages')).toContainText('2');
 await page.screenshot({path:info.outputPath('cid-pdf-detected-preview.png')});
});
