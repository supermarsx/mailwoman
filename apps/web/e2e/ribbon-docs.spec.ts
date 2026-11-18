import {test,expect} from '@playwright/test';
import {build,preview,type PreviewServer} from 'vite';
import {createServer,type Server} from 'node:http';
import {readFile} from 'node:fs/promises';
import {resolve,extname} from 'node:path';
test.describe('ribbon and documentation navigation',()=>{
 let app:PreviewServer,docs:Server,appOrigin:string,docsOrigin:string;
 test.beforeAll(async()=>{
  test.setTimeout(180000);
  const options={outDir:'.orchestration/ribbon-browser',emptyOutDir:true};
  await build({build:options,plugins:[{name:'ribbon-fixture',transformIndexHtml:{order:'pre',handler:html=>html.replace('/src/main.tsx','/e2e/ribbonHarness.tsx')}}]});
  app=await preview({build:options,preview:{port:0,host:'127.0.0.1'}});appOrigin=app.resolvedUrls!.local[0]!;
  const root=resolve('../../docs/.vitepress/dist');
  docs=createServer(async(req,res)=>{try{const raw=new URL(req.url!,'http://localhost').pathname;let path=resolve(root,'.'+decodeURIComponent(raw));if(path!==root&&!path.startsWith(root+'/')&&!path.startsWith(root+'\\'))throw Error();if(!extname(path))path=raw.endsWith('/')?resolve(path,'index.html'):path+'.html';const body=await readFile(path);res.setHeader('Content-Type',({'.html':'text/html','.js':'application/javascript','.css':'text/css','.json':'application/json'} as Record<string,string>)[extname(path)]??'application/octet-stream');res.end(body);}catch{res.writeHead(404).end();}});
  await new Promise<void>(done=>docs.listen(0,'127.0.0.1',done));docsOrigin=`http://127.0.0.1:${(docs.address() as {port:number}).port}`;
 });
 test.afterAll(async()=>{await Promise.all([new Promise<void>(done=>app.httpServer.close(()=>done())),new Promise<void>(done=>docs.close(()=>done()))]);});
 test('uses dedicated Search and toggles a dropdown before centralizing the composer',async({page})=>{
  await page.goto(appOrigin);await page.getByRole('tab',{name:'Search',exact:true}).click();await expect(page.getByRole('searchbox',{name:'Search mail'})).toBeVisible();
  await page.getByRole('tab',{name:'Workspace layout',exact:true}).click();const choice=page.getByRole('button',{name:'Arrangement',exact:true});await choice.click();await expect(page.getByRole('menu')).toBeVisible();await choice.click();await expect(page.getByRole('menu')).toHaveCount(0);
  await page.getByRole('tab',{name:'Home',exact:true}).click();await page.getByRole('button',{name:'Compose',exact:true}).click();
  await expect(page.locator('main > div [data-bar="composerRibbon"]')).toBeVisible();await expect(page.locator('.compose__header')).toHaveCount(0);
  const send=page.getByRole('button',{name:'Send',exact:true});await expect(send).toBeVisible();expect(await send.evaluate((button:HTMLButtonElement)=>button.form?.classList.contains('compose'))).toBe(true);
  await page.getByLabel('Subject',{exact:true}).fill('Keep this draft');await page.getByRole('tab',{name:'Insert',exact:true}).click();await page.getByRole('tab',{name:'Format text',exact:true}).click();await expect(page.getByLabel('Subject',{exact:true})).toHaveValue('Keep this draft');
  await expect(send).toBeVisible();await expect(page.locator('#mw-boot')).toHaveCount(0);
  await page.screenshot({path:'.orchestration/ribbon-composer.png'});
 });
 test('concentrates file titles, download and live text controls in the ribbon',async({page})=>{
  await page.goto(appOrigin);await page.getByRole('button',{name:'Open file preview'}).click();
  await expect(page.getByRole('tab',{name:'File previews',exact:true})).toHaveAttribute('aria-selected','true');
  const ribbon=page.locator('[data-bar="mailRibbon"]');await expect(ribbon.getByText('notes.txt',{exact:true})).toBeVisible();
  const wrap=ribbon.getByTestId('text-wrap');await expect(wrap).toBeVisible();await wrap.uncheck();
  await expect(page.locator('[data-text-state="text"]')).toHaveAttribute('data-wrap','false');
  await expect(page.locator('[data-testid="preview-file"] .mw-text__bar')).toBeHidden();
  await page.screenshot({path:'.orchestration/file-command-ribbon.png'});
 });
 test('docs offer dark navigation, full-text search and a mobile topic menu',async({page})=>{
  await page.goto(docsOrigin);await expect(page.locator('html')).toHaveClass(/dark/);await expect(page.locator('.VPSidebar')).toBeVisible();
  await page.getByRole('button',{name:/Search/}).first().click();const search=page.locator('.VPLocalSearchBox input');await search.fill('retention');await expect(page.locator('.VPLocalSearchBox .result').first()).toBeVisible();await page.screenshot({path:'.orchestration/docs-search-dark.png'});await page.keyboard.press('Escape');
  await page.setViewportSize({width:390,height:844});await page.getByRole('button',{name:'Menu',exact:true}).click();await expect(page.locator('.VPSidebar')).toBeVisible();await page.screenshot({path:'.orchestration/docs-sidebar-mobile.png'});
 });
});
