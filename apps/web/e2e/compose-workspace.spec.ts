import {test,expect} from '@playwright/test';
import {build,preview,loadConfigFromFile,type PreviewServer} from 'vite';
test.describe('compose and settings workspace',()=>{
 test.describe.configure({mode:'default'});
 let server:PreviewServer,origin:string;
 test.beforeAll(async({browserName})=>{
  test.setTimeout(180000);
  const options={outDir:process.env['MW_COMPOSE_FIXTURE_DIR']??`.orchestration/compose-workspace-${browserName}-${process.pid}`,emptyOutDir:true};
  const config=(await loadConfigFromFile({command:'build',mode:'production'}))!.config;
  const plugins=(config.plugins??[]).flat().filter(plugin=>!(plugin&&typeof plugin==='object'&&'name' in plugin&&['mw-boot-script','mw-asset-integrity','mw-bundle-graph'].includes(String(plugin.name))));
  if(!process.env['MW_COMPOSE_FIXTURE_DIR'])await build({...config,configFile:false,logLevel:'warn',build:{...config.build,...options,rollupOptions:{}},plugins:[...plugins,{name:'compose-workspace-fixture',transformIndexHtml:{order:'pre',handler:()=>'<html data-theme="dark"><head><meta name="viewport" content="width=device-width,initial-scale=1"></head><body><div id="root"></div><script type="module" src="/e2e/composeWorkspaceHarness.tsx"></script></body></html>'}}]});
  server=await preview({build:options,preview:{host:'127.0.0.1',port:0}});origin=server.resolvedUrls!.local[0]!;
 });
 test.afterAll(async()=>{if(!server)return;if('closeAllConnections' in server.httpServer)server.httpServer.closeAllConnections();await new Promise<void>(resolve=>server.httpServer.close(()=>resolve()));});
 test('compose fields and actions are compact, one header toggle works, and body clearing is undoable',async({page,browserName})=>{
  await page.route('**/api/**',route=>route.fulfill({json:route.request().url().includes('/settings')?{settings:{}}:{}}));
  await page.goto(origin+'?compose-workspace');
  const compose=page.locator('form.compose');await expect(compose).toBeVisible();
  const to=compose.getByRole('combobox',{name:'To',exact:true}),subject=compose.getByLabel('Subject',{exact:true});
  expect((await subject.boundingBox())!.height).toBeLessThanOrEqual(30);
  expect((await to.locator('..').boundingBox())!.height).toBeLessThanOrEqual(32);
  for(const button of await compose.locator('.compose__header .btn').all())expect((await button.boundingBox())!.height).toBeLessThanOrEqual(30);
  const toggle=compose.getByTestId('compose-header-toggle');await expect(compose.locator('[data-testid="compose-header-corner"]')).toHaveCount(0);
  await toggle.click();await expect(compose.getByTestId('compose-header-body')).toBeHidden();
  await toggle.click();await expect(subject).toBeVisible();
  await subject.fill('Compact composer');await to.fill('recipient@example.org');await to.press('Enter');
  const editor=compose.frameLocator('iframe[title="Visual editor"]').locator('#mw-native-editor');
  await expect(editor).toContainText('Mailwoman Team');await expect(editor.locator('strong')).toHaveText('Mailwoman Team');
  const fromToggle=compose.getByRole('button',{name:'From',exact:true});
  if(!await fromToggle.isVisible())await compose.getByTestId('compose-group-recipients').getByRole('button',{name:'More Recipients commands'}).click();
  await fromToggle.click();await subject.click();
  await compose.locator('.compose__from-row select').selectOption('work');
  await expect(editor).toContainText('Work Team');await expect(editor).not.toContainText('Mailwoman Team');
  await compose.locator('.compose__from-row select').selectOption('identity');await expect(editor).toContainText('Mailwoman Team');
  await expect(compose.getByTestId('compose-auto-signature')).toHaveCount(0);
  await editor.press('Control+Home');await editor.pressSequentially('A message before the signature. ');
  await compose.getByRole('tab',{name:'Format text',exact:true}).click();
  const clear=compose.getByTestId('compose-clear-body');
  if(!await clear.isVisible())await compose.getByTestId('compose-group-clear').getByRole('button',{name:'More Editing commands'}).click();
  await expect(clear).toBeVisible();await clear.click();
  await expect(editor).toHaveText('');await expect(subject).toHaveValue('Compact composer');
  await page.getByTestId('toast-region').getByRole('button',{name:'Undo',exact:true}).click();
  await expect(editor).toContainText('Mailwoman Team');await expect(editor).toContainText('A message before the signature.');
  await page.screenshot({path:`.orchestration/compose-compact-${browserName}.png`});
 });
 test('embedded Settings scrolls its content without scrolling the workspace pane',async({page})=>{
  await page.route('**/api/**',route=>route.fulfill({json:route.request().url().includes('/settings')?{settings:{}}:{}}));
  for(const width of [1280,390]){
   await page.setViewportSize({width,height:600});await page.goto(origin+'?settings-workspace');
   const pane=page.getByTestId('settings-workspace-pane');await expect(pane).toBeAttached();
   await pane.evaluate(node=>{(node as HTMLElement).style.height='500px';(node as HTMLElement).style.width='100%';});
   await expect(pane).toBeVisible();
   const content=pane.locator('[data-settings-scroll]');await expect(content).toBeVisible();
   await expect.poll(()=>content.evaluate(node=>node.scrollHeight>node.clientHeight)).toBe(true);
   expect(await pane.evaluate(node=>getComputedStyle(node).overflowY)).toBe('hidden');
   expect(await pane.evaluate(node=>node.scrollHeight<=node.clientHeight+1)).toBe(true);
   await content.evaluate(node=>{node.scrollTop=node.scrollHeight;});expect(await content.evaluate(node=>node.scrollTop)).toBeGreaterThan(0);
   expect(await pane.evaluate(node=>node.scrollTop)).toBe(0);
  }
 });

});
