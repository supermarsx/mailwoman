import {test,expect} from '../csrf.ts';
import {login,messageRow} from '../helpers.ts';
import {buildPdf} from '../pdf-fixtures.ts';
import {readFileSync} from 'node:fs';

test('PDF Save downloads the current document and preview actions share compact dimensions',async({page})=>{
 const pdf=Buffer.from(buildPdf(2),'latin1');
 await page.route('**/api/settings',async route=>{const response=await route.fetch();const body=await response.json();body.settings={...body.settings,'reader.attachmentsCollapsed':{value:false,source:'user',locked:false}};await route.fulfill({response,json:body});});
 await page.route('**/api/attachments/policy',route=>route.fulfill({json:{previewAllowed:true,downloadAllowed:true,maxBytes:0,allowedExtensions:[],blockedExtensions:[],warnExtensions:[],allowedMimeTypes:[],blockedMimeTypes:[],warnMimeTypes:[]}}));
 await page.route(/\/jmap\/download\/[^/]+\/fixture-pdf-save\//,route=>route.fulfill({contentType:'application/pdf',body:pdf}));
 await page.route(/\/jmap(?:\/api)?$/,async route=>{const response=await route.fetch();const body=await response.json();for(const [name,args] of body.methodResponses??[])if(name==='Email/get')for(const row of args.list??[])if(row.subject==='Lunch tomorrow?'){row.hasAttachment=true;row.attachments=[{partId:'pdf',blobId:'fixture-pdf-save',name:'save-test.pdf',type:'application/pdf',size:pdf.length,disposition:'attachment'}];}await route.fulfill({response,json:body});});
 await login(page);await messageRow(page,'Lunch tomorrow?').click();await page.getByRole('option',{name:/save-test\.pdf/}).click();
 const tab=page.locator('.attachment-tab');const frame=tab.frameLocator('iframe.mw-viewer__pdf-frame');await expect(frame.locator('.page canvas').first()).toBeVisible({timeout:35000});
 const download=page.waitForEvent('download',{timeout:10000});await frame.locator('#downloadButton').click();const saved=await download;expect(saved.suggestedFilename()).toBe('save-test.pdf');expect(readFileSync((await saved.path())!)).toEqual(pdf);
 const save=await tab.locator('.attachment-tab__download').boundingBox(),info=await tab.getByTestId('attachment-details-button').boundingBox();
 expect(Math.abs(save!.height-info!.height)).toBeLessThan(1);expect(Math.abs(save!.width-info!.width)).toBeLessThan(1);expect(save!.height).toBeLessThanOrEqual(36);
 for(const [appearance,surface,bg,text,accent] of [['light','rgb(249, 244, 234)','rgb(255, 253, 248)','rgb(35, 29, 24)','rgb(148, 59, 38)'],['dark','rgb(34, 51, 68)','rgb(17, 24, 34)','rgb(232, 240, 248)','rgb(103, 200, 176)']]){
  await page.evaluate(({appearance,surface,bg,text,accent})=>{const root=document.documentElement;root.dataset.appearance=appearance;for(const [key,value] of Object.entries({surface,bg,'bg-alt':surface,text,'text-dim':text,border:accent,accent,'accent-text':bg}))root.style.setProperty(`--${key}`,value!);document.body.style.fontFamily='monospace';},{appearance:appearance!,surface:surface!,bg:bg!,text:text!,accent:accent!});
  await expect.poll(()=>frame.locator('#toolbarContainer').evaluate(element=>getComputedStyle(element).backgroundColor)).toBe(surface);
  await expect.poll(()=>frame.locator('body').evaluate(element=>getComputedStyle(element).fontFamily)).toContain('monospace');
  await expect.poll(()=>frame.locator('#documentPropertiesDialog').evaluate(element=>getComputedStyle(element).getPropertyValue('--button-primary-bg-color').trim())).toBe(accent);
  await expect.poll(()=>frame.locator('html').evaluate(element=>getComputedStyle(element).colorScheme)).toBe(appearance);
 }
 await frame.locator('#editorFreeTextButton').click();await frame.locator('.page').first().click({position:{x:140,y:160}});
 const note=frame.locator('.freeTextEditor [contenteditable="true"]').last();await expect(note).toBeVisible();await note.fill('Saved annotation');
 const changed=page.waitForEvent('download',{timeout:10000});await frame.locator('#downloadButton').click();const edited=readFileSync((await (await changed).path())!);
 expect(edited.length).toBeGreaterThan(pdf.length);expect(edited.subarray(0,5).toString()).toBe('%PDF-');
});
