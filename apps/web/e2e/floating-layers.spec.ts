import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {test,expect,type Locator} from '@playwright/test';
import {build,preview,type PreviewServer} from 'vite';

test.describe('floating controls above native dialogs and popovers',()=>{
 let server:PreviewServer,origin:string;
 test.beforeAll(async()=>{
  test.setTimeout(180000);
  const buildOptions={outDir:'.orchestration/floating-browser-build',emptyOutDir:true};
  await build({logLevel:'warn',build:buildOptions,plugins:[{name:'floating-layer-fixture',transformIndexHtml:{order:'pre',handler:()=>'<html data-theme="dark"><body><div id="root"></div><script type="module" src="/e2e/floatingLayerHarness.tsx"></script></body></html>'}}]});
  server=await preview({build:buildOptions,preview:{host:'127.0.0.1',port:0}});origin=server.resolvedUrls!.local[0]!;
 });
 test.afterAll(async()=>{await new Promise<void>((resolve,reject)=>server?.httpServer.close(error=>error?reject(error):resolve())??resolve());});
 async function above(element:Locator){
  await expect(element).toBeVisible();
  await expect.poll(()=>element.evaluate(node=>{const rect=node.getBoundingClientRect();return node.contains(node.ownerDocument.elementFromPoint(rect.left+rect.width/2,rect.top+rect.height/2));})).toBe(true);
 }
 test('privacy action floats below the unchanged login card and opens a themed readable policy',async({page,context})=>{
  const script=readFileSync('../../crates/mw-server/src/public-policy-theme.js','utf8').replaceAll('\r\n','\n');
  const source=readFileSync('../../crates/mw-server/src/compliance.rs','utf8');const css=/const PAGE_CSS: &str = r#"([\s\S]*?)"#;/.exec(source)![1]!;
  const csp=`default-src 'none'; script-src 'sha256-${createHash('sha256').update(script).digest('base64')}'; style-src 'unsafe-inline'`;
  await context.route('**/api/**',route=>route.fulfill({json:route.request().url().includes('/cookies/config')?{enabled:false,policyPages:true,categories:[],inventory:[],rules:[]}:route.request().url().includes('/sso/providers')?{providers:[]}:{}}));
  await context.route('**/privacy*',route=>route.fulfill({contentType:'text/html',headers:{'Content-Security-Policy':csp},body:`<!doctype html><html><head><meta name="viewport" content="width=device-width,initial-scale=1"><style>${css}</style><script>${script}</script></head><body><main><h1>Privacy policy</h1><p>Version 1</p><nav aria-label="On this page"><a href="#rights">Your rights</a></nav><section id="rights"><h2>Your rights</h2><p>Contact the operator to exercise your data rights.</p></section></main></body></html>`}));
  for(const width of [1440,390]){
   await page.setViewportSize({width,height:900});await page.goto(origin+'?login');
   const form=page.getByRole('form',{name:'Sign in'});await expect(form).toBeVisible();const footer=page.getByTestId('login-policy-links');await expect(footer.getByRole('link')).toBeVisible();
   const before=(await form.boundingBox())!;const box=(await footer.boundingBox())!;expect(Math.abs(box.x+box.width/2-width/2)).toBeLessThan(2);expect(box.y+box.height).toBeLessThanOrEqual(900);expect(box.y).toBeGreaterThan(before.y+before.height);
   const expected=await page.evaluate(()=>getComputedStyle(document.documentElement).getPropertyValue('--mw-color-bg').trim());
   const opened=context.waitForEvent('page');await footer.getByRole('link').click();const popup=await opened;await popup.waitForLoadState();
   await expect(popup.getByRole('heading',{name:'Privacy policy'})).toBeVisible();expect(await popup.evaluate(()=>location.hash)).toBe('');
   // The actual theme palette is carried by the link, including generated contract names.
   expect(await popup.evaluate(()=>document.documentElement.style.getPropertyValue('--policy-bg'))).not.toBe('');
   if(expected)expect(await popup.evaluate(()=>document.documentElement.style.getPropertyValue('--policy-bg'))).toBe(expected);
   await popup.getByRole('link',{name:'Your rights'}).click();await expect(popup.locator('section')).toBeInViewport();expect(await popup.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);await popup.close();
   await footer.evaluate(node=>node.remove());const after=(await form.boundingBox())!;expect(after.y).toBeCloseTo(before.y,1);
  }
 });
 for(const owner of ['dialog','popover'])test(`tooltips, themed dropdowns and submenus work inside a native ${owner}`,async({page})=>{
  await page.goto(origin);await page.getByRole('button',{name:`Open ${owner}`,exact:true}).click();
  const panel=owner==='dialog'?page.locator('dialog'):page.locator('[popover="manual"]').filter({has:page.getByRole('button',{name:'Close popover'})});
  await panel.getByRole('button',{name:'Help',exact:true}).hover();await above(page.getByRole('tooltip'));
  await panel.getByRole('combobox',{name:'Choice'}).click();
  await above(page.getByRole('menuitemradio',{name:'Two',exact:true}));
  await page.getByRole('menuitemradio',{name:'Two',exact:true}).click();await expect(panel.getByRole('combobox',{name:'Choice'})).toHaveValue('Two');
  await expect(page.getByRole('menu')).toHaveCount(0);
  await panel.getByRole('button',{name:'Actions',exact:true}).click();await page.getByRole('menuitem',{name:'More',exact:true}).click();
  const nested=page.getByRole('menuitem',{name:'Nested action',exact:true});await above(nested);await nested.hover();await above(page.getByRole('tooltip'));await nested.click();
  await expect(page.getByRole('menu')).toHaveCount(0);
  await expect(panel.getByRole('button',{name:'Actions',exact:true})).toBeFocused();
  await panel.getByRole('combobox',{name:'Choice'}).click();await page.keyboard.press('Escape');await expect(page.getByRole('menu')).toHaveCount(0);await expect(panel).toBeVisible();
  await panel.getByRole('combobox',{name:'Choice'}).click();await panel.getByRole('button',{name:'Help',exact:true}).click();await expect(page.getByRole('menu')).toHaveCount(0);
  await panel.getByRole('combobox',{name:'Choice'}).click();await expect(page.getByRole('menu')).toHaveCount(1);
  await panel.evaluate((node,kind)=>{if(kind==='dialog')(node as HTMLDialogElement).close();else (node as HTMLElement).hidePopover();},owner);
  await expect(page.getByRole('menu')).toHaveCount(0);
  await page.getByRole('button',{name:`Open ${owner}`,exact:true}).click();await panel.getByRole('button',{name:'Help',exact:true}).hover();await expect(page.getByRole('tooltip')).toBeVisible();
  await panel.evaluate((node,kind)=>{if(kind==='dialog')(node as HTMLDialogElement).close();else (node as HTMLElement).hidePopover();},owner);await expect(page.getByRole('tooltip')).toHaveCount(0);
 });
 test('searchable and multiple-choice lists escape clipped transformed containers',async({page})=>{
  await page.goto(origin);await page.getByRole('button',{name:'Open dialog',exact:true}).click();
  await page.getByRole('combobox',{name:'Region'}).fill('');await above(page.getByRole('option',{name:'Paris',exact:true}));await page.getByRole('option',{name:'Paris',exact:true}).click();
  await expect(page.getByRole('combobox',{name:'Region'})).toHaveValue('Europe/Paris');
  await page.getByRole('combobox',{name:'Accounts'}).click();const account=page.getByRole('option',{name:'Account C',exact:true});await above(account);await account.click();await expect(account).toHaveAttribute('aria-selected','true');
  await page.keyboard.press('Escape');await expect(page.getByRole('listbox')).toHaveCount(0);await expect(page.locator('dialog')).toBeVisible();
 });
 test('long dropdowns stay visible when their position is clamped to the viewport edge',async({page})=>{
  await page.goto(origin);await page.locator('#root').evaluate(node=>{(node as HTMLElement).style.minHeight='100vh';});
  await page.getByRole('combobox',{name:'Long choices'}).click();
  const menu=page.getByRole('menu');
  await expect.poll(()=>menu.evaluate(node=>{const rect=node.getBoundingClientRect();return rect.top>=7&&rect.bottom<=innerHeight-7;})).toBe(true);
  const option=page.getByRole('menuitemradio',{name:'Choice 0',exact:true});await above(option);await option.click();
  await expect(menu).toHaveCount(0);
 });
 test('folder search selection stays inside its editor and applies the selected scope',async({page})=>{
  await page.goto(origin);await page.locator('[data-kind="scope"] button').first().click();
  const editor=page.getByTestId('search-chip-editor');await editor.getByRole('combobox').click();
  const option=page.getByRole('menuitemradio',{name:'Projects',exact:true});await option.hover();await page.mouse.down();await expect(editor).toBeVisible();await page.mouse.up();
  await expect(editor.getByRole('combobox')).toHaveValue('projects');await editor.getByRole('button',{name:'Apply',exact:true}).click();
  await expect(page.getByTestId('folder-search-query')).toContainText('budget');await expect(page.getByTestId('folder-search-query')).toContainText('in:');
 });
 test('optional mailbox icons show live status with readable hover help inside the picker',async({page})=>{
  await page.goto(origin);await expect(page.locator('[data-mailbox-connection]')).toHaveCount(0);
  await page.getByRole('button',{name:'Show connection icons',exact:true}).click();
  await expect(page.locator('[data-account-indicator="picker"] [data-mailbox-connection]')).toHaveCount(1);
  await page.locator('[data-account-indicator="picker"]').click();
  const status=page.locator('[data-mailbox-menu] [data-mailbox-connection="auth-expired"]');await above(status);await status.hover();await above(page.getByRole('tooltip'));await expect(page.getByRole('tooltip')).toHaveText('Sign-in required');
  await page.getByRole('button',{name:'Hide connection icons',exact:true}).click();await expect(page.locator('[data-mailbox-connection]')).toHaveCount(0);
 });
});
