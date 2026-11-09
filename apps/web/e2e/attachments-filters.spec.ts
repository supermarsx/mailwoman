// Response-only metadata pages: no real message mutation or attachment download.
import {test,expect,type Page} from '@playwright/test';
import {login} from './helpers.ts';
test.use({serviceWorkers:'block'});test.setTimeout(120_000);
const files=[
 {name:'report-2026.pdf',mime:'application/pdf',sender:'Alice',subject:'Quarterly invoice',date:'2026-11-08T10:00:00Z',size:250000,folder:'af-inbox'},
 {name:'logo-brand.png',mime:'image/png',sender:'Bob',subject:'Brand refresh',date:'2026-09-02T10:00:00Z',size:1500000,folder:'af-archive'},
 {name:'proposal.docx',mime:'application/vnd.openxmlformats-officedocument.wordprocessingml.document',sender:'Alice',subject:'Project proposal',date:'2026-08-03T10:00:00Z',size:2000000,folder:'af-inbox'},
 {name:'budget.xlsx',mime:'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',sender:'Carol',subject:'Finance workbook',date:'2025-06-03T10:00:00Z',size:3000000,folder:'af-finance'},
 {name:'records.zip',mime:'application/zip',sender:'Alice',subject:'Historic records',date:'2026-01-02T10:00:00Z',size:12000000,folder:'af-archive'},
];
async function fixture(page:Page){
 const state={positions:[]as number[],downloads:0,blockedWrites:0};
 await page.addLocatorHandler(page.getByRole('dialog',{name:'Allow notifications?',exact:true}),async dialog=>{await dialog.getByRole('button',{name:'Not now',exact:true}).click();});
 await page.route(/\/jmap\/download\//,async route=>{state.downloads++;await route.abort();});
 await page.route(/\/jmap(?:\/api)?$/,async route=>{
  if(route.request().method()!=='POST')return route.continue();
  const request=route.request().postDataJSON()as{methodCalls:[string,Record<string,unknown>,string][]};
  if(request.methodCalls.some(([method])=>method.endsWith('/set'))){state.blockedWrites++;await route.fulfill({json:{sessionState:'fixture',methodResponses:request.methodCalls.map(([method,args,id])=>[method,{accountId:args['accountId'],oldState:'fixture',newState:'fixture',updated:Object.fromEntries(Object.keys((args['update']??{})as object).map(key=>[key,null])),created:{},destroyed:args['destroy']??[]},id])}});return;}
  const query=request.methodCalls.find(([, ,id])=>id==='aq');
  if(query){
   const offset=Number(query[1]['position']??0);state.positions.push(offset);const file=files[offset];const ids=file?[`af-message-${offset}`]:[];
   const get=request.methodCalls.find(([, ,id])=>id==='ag')!;expect(get[1]['properties']).toEqual(expect.arrayContaining(['attachments','mailboxIds']));expect(get[1]['properties']).not.toContain('bodyValues');
   const email=file?{id:ids[0],blobId:`af-message-blob-${offset}`,from:[{name:file.sender,email:`${file.sender.toLowerCase()}@example.org`}],subject:file.subject,receivedAt:file.date,size:file.size+100,hasAttachment:true,mailboxIds:{[file.folder]:true},attachments:[{partId:'2',blobId:`af-file-${offset}`,name:file.name,type:file.mime,size:file.size,disposition:'attachment'}]}:null;
   await route.fulfill({json:{sessionState:'fixture',methodResponses:[['Email/query',{accountId:query[1]['accountId'],queryState:'fixture',canCalculateChanges:false,position:offset,ids,total:files.length},'aq'],['Email/get',{accountId:query[1]['accountId'],state:'fixture',list:email?[email]:[],notFound:[]},'ag']]}});return;
  }
  const response=await route.fetch();const body=await response.json()as{methodResponses:[string,{list?:Record<string,unknown>[]},string][]};
  for(const[method,args]of body.methodResponses)if(method==='Mailbox/get')args.list=[...(args.list??[]),...[
   {id:'af-inbox',name:'Reports',parentId:null},{id:'af-work',name:'Work',parentId:null},{id:'af-archive',name:'Archive',parentId:'af-work'},{id:'af-finance',name:'Finance',parentId:null},
  ].map(box=>({...box,role:null,totalEmails:0,unreadEmails:0,sortOrder:1,isSubscribed:true,myRights:{mayReadItems:true,mayAddItems:false,mayRemoveItems:false,maySetSeen:false,maySetKeywords:false,mayCreateChild:false,mayRename:false,mayDelete:false}}))];
  await route.fulfill({response,json:body});
 });return state;
}
for(const width of [1280,390])test(`friendly attachment filters scan all metadata pages and fit ${width}px`,async({page},info)=>{
 const state=await fixture(page);await login(page);if(width===390){await page.setViewportSize({width,height:844});await page.getByRole('button',{name:'Open folders and apps',exact:true}).click();}
 await page.locator('#shell-nav').getByRole('button',{name:'Attachments',exact:true}).click();
 const screen=page.locator('section.mw-attach');await expect(screen).toBeVisible();const form=screen.getByRole('form',{name:'Attachment filters'});await expect(form).toBeVisible();
 const row=(name:string)=>screen.getByRole('row').filter({hasText:name});await expect(row('report-2026.pdf')).toBeVisible();
 await expect(form.getByRole('combobox',{name:'Search in',exact:true})).toHaveValue('all');
 // Only page0 has been fetched. The normal type control starts the account scan.
 expect([...new Set(state.positions)]).toEqual([0]);await form.getByRole('combobox',{name:'Filter by type',exact:true}).selectOption('image');await expect(row('logo-brand.png')).toBeVisible();expect(state.positions).toContain(1);
 await form.getByLabel('Sender',{exact:true}).fill('bob');await form.getByLabel('Message subject',{exact:true}).fill('Brand');await form.getByRole('combobox',{name:'Folder',exact:true}).selectOption('af-archive');
 await form.getByRole('combobox',{name:'Received',exact:true}).selectOption('custom');await form.getByLabel('Received from',{exact:true}).fill('2026-09-01');await form.getByLabel('Received through',{exact:true}).fill('2026-09-30');
 await form.getByRole('combobox',{name:'File size',exact:true}).selectOption('custom');await form.getByLabel('Minimum size (MB)',{exact:true}).fill('1');await form.getByLabel('Maximum size (MB)',{exact:true}).fill('2');
 await expect(row('logo-brand.png')).toBeVisible();await expect(screen.locator('tbody tr')).toHaveCount(1);await expect(form.getByRole('group',{name:'Active attachment filters'}).getByRole('button')).toHaveCount(6);
 const geometry=await form.evaluate(element=>({width:element.getBoundingClientRect().width,controls:[...element.querySelectorAll('input,select')].filter(node=>node.getClientRects().length>0).map(node=>{const box=node.getBoundingClientRect();return {left:box.left,right:box.right,width:box.width};}),viewport:innerWidth,document:document.documentElement.scrollWidth}));
 expect(geometry.document).toBeLessThanOrEqual(geometry.viewport+1);for(const control of geometry.controls){expect(control.width).toBeGreaterThan(30);expect(control.left).toBeGreaterThanOrEqual(0);expect(control.right).toBeLessThanOrEqual(geometry.viewport+1);}
 await page.screenshot({path:info.outputPath(`attachment-filters-${width}.png`)});
 await form.getByRole('button',{name:/^Remove filter: Size:/}).click();await expect(form.getByRole('combobox',{name:'File size',exact:true})).toHaveValue('any');await expect(form.getByLabel('Minimum size (MB)',{exact:true})).toHaveCount(0);
 await form.getByRole('button',{name:'Remove filter: Sender: bob',exact:true}).click();await expect(form.getByLabel('Sender',{exact:true})).toHaveValue('');
 await form.getByRole('button',{name:'Clear filters',exact:true}).click();await expect(form.getByRole('group',{name:'Active attachment filters'})).toHaveCount(0);await expect(row('report-2026.pdf')).toBeVisible();
 for(const[type,name]of [['document','proposal.docx'],['spreadsheet','budget.xlsx'],['archive','records.zip']]){await form.getByRole('combobox',{name:'Filter by type',exact:true}).selectOption(type!);await expect(row(name!)).toBeVisible();await expect(screen.locator('tbody tr')).toHaveCount(1);}
 await form.getByText('More filters',{exact:true}).click();await form.getByLabel('File name',{exact:true}).fill('records');await form.getByLabel('File extensions',{exact:true}).fill('zip');await expect(row('records.zip')).toBeVisible();
 if(width===390){const region=screen.getByRole('region',{name:'Attachments',exact:true});await region.scrollIntoViewIfNeeded();expect(await region.evaluate(element=>element.scrollWidth>element.clientWidth)).toBe(true);await region.hover();await page.mouse.wheel(400,0);await expect.poll(()=>region.evaluate(element=>element.scrollLeft)).toBeGreaterThan(0);}
 expect(state.downloads).toBe(0);expect(state.positions).toContain(4);
});
