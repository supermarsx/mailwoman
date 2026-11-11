import {test,expect} from '../csrf.ts';
import {login,messageRow} from '../helpers.ts';

test('expanded attachments follow the next message and inline images have space around them',async({page})=>{
 await page.route('**/api/settings',async route=>{
  const response=await route.fetch();const body=await response.json();
  body.settings={...body.settings,'reader.attachmentsCollapsed':{value:false,source:'user',locked:false}};
  await route.fulfill({response,json:body});
 });
 await page.route('**/api/attachments/policy',route=>route.fulfill({json:{previewAllowed:true,downloadAllowed:true,maxBytes:0,allowedExtensions:[],blockedExtensions:[],warnExtensions:[],allowedMimeTypes:[],blockedMimeTypes:[],warnMimeTypes:[]}}));
 await page.route(/\/jmap(?:\/api)?$/,async route=>{
  const response=await route.fetch();const body=await response.json();
  for(const [name,args] of body.methodResponses??[])if(name==='Email/get')for(const row of args.list??[]){
   const stem=row.subject==='Lunch tomorrow?'?'lunch':'other';row.hasAttachment=true;
   row.attachments=[{partId:'fixture-file',blobId:`fixture-${stem}`,name:`${stem}.pdf`,type:'application/pdf',size:1024,disposition:'attachment'},
    {partId:'fixture-inline',blobId:`fixture-${stem}-logo`,name:`${stem}-logo.png`,type:'image/png',size:99,disposition:'inline',cid:'fixture-logo@example.org'}];
  }
  await route.fulfill({response,json:body});
 });
 await login(page);await messageRow(page,'Lunch tomorrow?').click();
 const bar=page.getByTestId('reader-attachments');
 await expect(bar.locator('[data-policy="ready"]')).toBeVisible();await expect(bar.getByRole('option',{name:/lunch\.pdf/})).toBeVisible();
 const inline=page.getByTestId('reader-attachments-inline');await inline.locator('summary').click();
 await expect(bar.getByRole('option',{name:/lunch-logo\.png/})).toBeVisible();
 const fileBox=await bar.locator('[data-policy="ready"] > .mw-thumbs').boundingBox(),summaryBox=await inline.locator('summary').boundingBox();
 expect(summaryBox!.y-(fileBox!.y+fileBox!.height)).toBeGreaterThanOrEqual(12);
 const summaryEnd=summaryBox!.y+summaryBox!.height,inlineBox=await inline.locator('.mw-thumbs').boundingBox();expect(inlineBox!.y-summaryEnd).toBeGreaterThanOrEqual(8);
 await page.locator('.list__subject').filter({hasNotText:'Lunch tomorrow?'}).first().click();
 await expect(bar.locator('[data-policy="ready"]')).toBeVisible();await expect(bar.getByRole('option',{name:/other\.pdf/})).toBeVisible();
 await expect(bar.getByRole('option',{name:/lunch\.pdf/})).toHaveCount(0);await expect(page.getByTestId('reader-attachments-toggle')).toHaveAttribute('aria-expanded','true');
});
