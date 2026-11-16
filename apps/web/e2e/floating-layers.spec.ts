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
 test('optional mailbox icons show live status with readable hover help inside the picker',async({page})=>{
  await page.goto(origin);await expect(page.locator('[data-mailbox-connection]')).toHaveCount(0);
  await page.getByRole('button',{name:'Show connection icons',exact:true}).click();
  await expect(page.locator('[data-account-indicator="picker"] [data-mailbox-connection]')).toHaveCount(1);
  await page.locator('[data-account-indicator="picker"]').click();
  const status=page.locator('[data-mailbox-menu] [data-mailbox-connection="auth-expired"]');await above(status);await status.hover();await above(page.getByRole('tooltip'));await expect(page.getByRole('tooltip')).toHaveText('Sign-in required');
  await page.getByRole('button',{name:'Hide connection icons',exact:true}).click();await expect(page.locator('[data-mailbox-connection]')).toHaveCount(0);
 });
});
