import {test,expect} from '@playwright/test';
import {build,preview,type PreviewServer} from 'vite';

test.describe('contacts, notices and inert reader parsing',()=>{
 let server:PreviewServer,origin:string;
 test.beforeAll(async()=>{
  test.setTimeout(180000);
  const options={outDir:'.orchestration/contacts-notices-browser',emptyOutDir:true};
  await build({logLevel:'warn',build:options,plugins:[{name:'contacts-notice-fixture',transformIndexHtml:{order:'pre',handler:()=>'<html data-theme="dark"><head><meta name="viewport" content="width=device-width,initial-scale=1"></head><body><div id="root"></div><script type="module" src="/e2e/contactsNoticeHarness.tsx"></script></body></html>'}}]});
  server=await preview({build:options,preview:{host:'127.0.0.1',port:0}});origin=server.resolvedUrls!.local[0]!;
 });
 test.afterAll(async()=>{await new Promise<void>((resolve,reject)=>server?.httpServer.close(error=>error?reject(error):resolve())??resolve());});
 test('folder icons stay searchable, selectable and visible on narrow screens',async({page})=>{
  await page.goto(origin+'?folder-icons');const search=page.getByRole('searchbox',{name:'Search icons'});
  await expect(page.getByRole('radio')).toHaveCount(160);
  await search.fill('medical');await expect(page.getByRole('radio',{name:'Dental care'})).toBeVisible();
  await page.screenshot({path:'.orchestration/folder-icons-medical.png'});
  await search.fill('telecom router');await page.getByRole('radio',{name:'Internet and broadband'}).click();
  await expect(page.getByRole('status',{name:'Selected icon'})).toHaveText('router');
  await search.fill('');await page.getByRole('radio',{name:'Automatic folder role',exact:true}).focus();
  await page.keyboard.press('End');await expect(page.getByRole('radio',{name:'Puzzles',exact:true})).toBeFocused();
  const visible=await page.getByRole('radio',{name:'Puzzles',exact:true}).evaluate(node=>{
   const rect=node.getBoundingClientRect(),clip=node.closest('[role="radiogroup"]')!.getBoundingClientRect();return rect.top>=clip.top&&rect.bottom<=clip.bottom;
  });expect(visible).toBe(true);
  await page.setViewportSize({width:390,height:844});await search.fill('platforms');
  await expect(page.getByRole('radio',{name:'Linux',exact:true})).toBeVisible();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.screenshot({path:'.orchestration/folder-icons-platforms-mobile.png'});
 });
 test('contacts keep name, email and actions usable on desktop and mobile',async({page})=>{
  await page.goto(origin);
  await expect(page.getByText('Ada Lovelace')).toBeVisible();
  await page.getByText('Ada Lovelace').click();await expect(page.getByRole('article',{name:'Contact Ada Lovelace'})).toBeVisible();
  await page.screenshot({path:'.orchestration/contacts-desktop.png'});
  await page.setViewportSize({width:390,height:844});
  await expect(page.getByRole('article',{name:'Contact Ada Lovelace'})).toBeVisible();
  await expect(page.getByRole('searchbox')).not.toBeVisible();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.screenshot({path:'.orchestration/contacts-mobile.png'});
  await page.getByRole('button',{name:'Back',exact:true}).click();await expect(page.getByRole('searchbox')).toBeVisible();
  await page.getByRole('searchbox').fill('Turing');await expect(page.getByText('Alan Turing')).toBeVisible();await expect(page.getByText('Ada Lovelace')).not.toBeVisible();
 });
 test('notice editor keeps input focus and exposes separate rule controls',async({page})=>{
  await page.goto(origin+'?notices');await page.getByRole('button',{name:'New notice',exact:true}).click();
  await page.getByLabel('Notice name',{exact:true}).fill('Confidential');await page.getByLabel('Notice text',{exact:true}).fill('Only for intended recipients.');
  await page.getByLabel('Subject contains',{exact:true}).fill('private');await page.getByRole('button',{name:'Back',exact:true}).click();
  await expect(page.getByText('Confidential',{exact:true})).toBeVisible();
  await page.getByRole('button',{name:'Edit Confidential',exact:true}).click();await expect(page.getByLabel('Subject contains',{exact:true})).toHaveValue('private');
 });
 test('reader transformations do not load resources or violate the parent style CSP',async({page})=>{
  await page.route(origin+'**',async route=>{if(route.request().resourceType()!=='document')return route.continue();const response=await route.fetch();await route.fulfill({response,headers:{...response.headers(),'content-security-policy':"default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'"}});});
  const problems:string[]=[];page.on('console',message=>{if(/Content.Security.Policy|file:\/\/\//i.test(message.text()))problems.push(message.text());});
  await page.goto(origin);await expect.poll(()=>page.evaluate(()=>typeof (globalThis as unknown as {inertProbe:unknown}).inertProbe)).toBe('function');
  await page.evaluate(()=>{(globalThis as unknown as {violations:number}).violations=0;document.addEventListener('securitypolicyviolation',()=>{(globalThis as unknown as {violations:number}).violations++;});});
  const result=await page.evaluate(()=> (globalThis as unknown as {inertProbe:(value:string)=>string}).inertProbe('<style>p{color:red}</style><p style="padding:10px;background:url(file:///private.png)">Readable</p><img src="file:///private.png">'));
  expect(result).toContain('Readable');await page.waitForTimeout(100);expect(problems).toEqual([]);expect(await page.evaluate(()=>(globalThis as unknown as {violations:number}).violations)).toBe(0);
 });
});
