import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {test,expect,type Locator} from '@playwright/test';
import {build,preview,loadConfigFromFile,type PreviewServer} from 'vite';

test.describe('floating controls above native dialogs and popovers',()=>{
 let server:PreviewServer,origin:string;
 test.beforeAll(async({browserName})=>{
  test.setTimeout(180000);
  const buildOptions={outDir:`.orchestration/floating-${browserName}-build`,emptyOutDir:true};
  const config=(await loadConfigFromFile({command:'build',mode:'production'}))!.config;
  // This component harness has no mailbox/login preload graph or app offline tiers.
  const plugins=(config.plugins??[]).flat().filter(plugin=>!(plugin&&typeof plugin==='object'&&'name' in plugin&&['mw-boot-script','mw-asset-integrity'].includes(String(plugin.name))));
  await build({...config,configFile:false,logLevel:'warn',build:{...config.build,...buildOptions},plugins:[...plugins,{name:'floating-layer-fixture',transformIndexHtml:{order:'pre',handler:()=>'<html data-theme="dark"><head><meta name="viewport" content="width=device-width,initial-scale=1"></head><body><div id="root"></div><script type="module" src="/e2e/floatingLayerHarness.tsx"></script></body></html>'}}]});
  server=await preview({build:buildOptions,preview:{host:'127.0.0.1',port:0}});origin=server.resolvedUrls!.local[0]!;
 });
 test.afterAll(async()=>{await new Promise<void>((resolve,reject)=>server?.httpServer.close(error=>error?reject(error):resolve())??resolve());});
 async function above(element:Locator){
  await expect(element).toBeVisible();
  await expect.poll(()=>element.evaluate(node=>{const rect=node.getBoundingClientRect();return node.contains(node.ownerDocument.elementFromPoint(rect.left+rect.width/2,rect.top+rect.height/2));})).toBe(true);
 }
 test('notifications stay on one line, expand with pointer and keyboard, and dismiss the whole queue',async({page,browserName})=>{
  for(const width of [1280,390]){
   await page.setViewportSize({width,height:900});await page.goto(origin+'?settings');
   const inlineBar=page.getByTestId('compact-inline-bar');await expect(inlineBar).toBeVisible();expect((await inlineBar.boundingBox())!.height).toBeLessThanOrEqual(30);
   await page.getByRole('button',{name:'Show detailed notification',exact:true}).click();
   const row=page.getByTestId('toast');const message=row.locator('span[id]');
   await expect(row).toHaveAttribute('data-expanded','false');
   await expect.poll(()=>message.evaluate(node=>node.scrollWidth>node.clientWidth)).toBe(true);
   const compact=(await row.boundingBox())!.height;
   await row.getByRole('button',{name:'Expand notification'}).click();
   await expect(row).toHaveAttribute('data-expanded','true');
   await expect.poll(async()=>(await row.boundingBox())!.height).toBeGreaterThan(compact);
   expect(await message.evaluate(node=>node.scrollWidth<=node.clientWidth+1)).toBe(true);
   await page.getByRole('button',{name:'Update detailed notification',exact:true}).click();
   await expect(row).toHaveAttribute('data-expanded','true');await expect(message).toContainText('Updated.');
   await page.screenshot({path:`.orchestration/notifications-expanded-${browserName}-${width}.png`});
   await row.getByRole('button',{name:'Collapse notification'}).press('Space');
   await expect(row).toHaveAttribute('data-expanded','false');
   expect(await row.evaluate(node=>{const box=node.getBoundingClientRect();return box.left>=0&&box.right<=innerWidth;})).toBe(true);
   await page.getByRole('button',{name:'Show notification burst',exact:true}).click();
   await expect(page.getByText('5 notifications',{exact:true})).toBeVisible();
   await page.getByRole('button',{name:'Dismiss all notifications'}).click();
   await expect(page.getByTestId('toast')).toHaveCount(0);await expect(page.getByText('5 notifications',{exact:true})).toHaveCount(0);
  }
 });
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
 test('settings cards, document pages, warning spacing and reader previews work at desktop and phone sizes',async({page,context,browserName})=>{
  const entries:Record<string,unknown>={};const entry=(value:unknown)=>({value,source:'user',locked:false});
  for(const key of ['mail.focusedInbox','mail.attachmentWarnings','mail.attachmentThumbnails'])entries[key]=entry(true);
  entries['mail.attachmentWarningsRequired']=entry(false);entries['mail.attachmentWarningTypes']=entry(['executable','macro','disguisedName','typeMismatch','activeDocument']);entries['reader.actionStyle']=entry('labels');entries['mail.rowActions']=entry(['quick:delete','quick:pin','more:archive']);
  for(const id of ['reply','reply-all','forward'])entries[`reader.actions.${id}`]=entry('toolbar');
  await context.route('**/api/**',async route=>{
   const url=new URL(route.request().url());
   if(url.pathname==='/api/settings'){await route.fulfill({json:{settings:entries}});return;}
   if(url.pathname.startsWith('/api/settings/')){const key=decodeURIComponent(url.pathname.split('/').at(-1)!);entries[key]=entry(route.request().postDataJSON().value);await route.fulfill({json:entries[key]});return;}
   if(url.pathname==='/api/privacy/processing'){await route.fulfill({json:{controller:{name:'Example operator',address:'Town'},contact:{name:'Privacy',email:'privacy@example.org',url:''},supervisoryAuthority:'Authority',activities:[{id:'mail',name:'Mail delivery',purpose:'Send and receive mail.',categories:'Messages',recipients:'Mail provider',retention:'Until removed',legalBasis:'Contract'}],recipients:[]}});return;}
   await route.fulfill({json:url.pathname.includes('/sessions')?{sessions:[]}:url.pathname==='/api/keys'?{keys:[]}:{}});
  });
  await context.route('**/privacy?*',route=>route.fulfill({json:{version:1,date:'2026-11-16',lang:'en',sections:[{id:'rights',title:'Your rights',body:'Request access to your data.',missing:false}]}}));
  for(const width of [1280,390]){
   await page.setViewportSize({width,height:900});await page.goto(origin+'?settings');await expect(page.getByTestId('yd-categories')).toBeVisible();await page.screenshot({path:`.orchestration/settings-${width}-${browserName}.png`});
   await expect(page.locator('#folders-card-focused')).toHaveCount(1);const wrapper=page.getByRole('region',{name:'Stored data by category'});await expect(wrapper).toBeVisible();expect(await wrapper.evaluate(node=>node.clientWidth<=innerWidth)).toBe(true);
   const table=page.getByTestId('yd-categories');expect(await table.locator('td[data-number]').first().evaluate(node=>getComputedStyle(node).textAlign)).toBe('end');
   await page.getByRole('button',{name:'Privacy policy',exact:true}).click();await expect(page.getByText('Request access to your data.')).toBeVisible();await page.getByRole('button',{name:'Back',exact:true}).click();
   await page.getByRole('button',{name:'Processing records',exact:true}).click();await expect(page.getByRole('heading',{name:'Mail delivery'})).toBeVisible();await page.getByRole('button',{name:'Back',exact:true}).click();
   const warnings=page.locator('fieldset[aria-label="Attachment security warnings"]');await expect(warnings).toHaveCount(1);{const labels=warnings.locator('label');const one=(await labels.nth(0).boundingBox())!,two=(await labels.nth(1).boundingBox())!;expect(two.y>=one.y+one.height+4||two.x>=one.x+one.width+4).toBe(true);}
   const clear=page.getByRole('button',{name:'Clear cached mail now'});await clear.click();expect(await page.locator('body').getAttribute('data-cache-cleared')).toBe('true');
   await page.getByText('Preview your actions ribbon',{exact:true}).click();const preview=page.getByRole('toolbar',{name:'Preview your actions ribbon'});await expect(preview.getByRole('button',{name:'Reply',exact:true})).not.toHaveClass(/reader-action--compact/);
   if(width===1280){const row=page.getByTestId('quick-action-row');await row.hover();await expect(row.getByRole('button',{name:'Delete',exact:true})).toBeVisible();await row.getByRole('button',{name:'More actions',exact:true}).last().click();await page.getByRole('menuitem',{name:'Archive',exact:true}).click();expect(await page.locator('body').getAttribute('data-archived')).toBe('true');}
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
   if(width===390&&browserName==='chromium'){
    await page.getByRole('button',{name:'Show swipe toast'}).click();const toast=page.getByTestId('toast');await expect(toast).toBeVisible();const rect=(await toast.boundingBox())!;const cdp=await context.newCDPSession(page);await cdp.send('Emulation.setTouchEmulationEnabled',{enabled:true});const x=rect.x+30,y=rect.y+rect.height/2;await cdp.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{x,y}]});await cdp.send('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:[{x:x+130,y}]});await cdp.send('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]});await expect(toast).toHaveCount(0);await cdp.detach();
   }
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
  await page.goto(origin);await expect(page.locator('.person-avatar').first()).toHaveCSS('border-radius','0px');await expect(page.locator('[data-mailbox-connection]')).toHaveCount(0);
  await page.getByRole('button',{name:'Show connection icons',exact:true}).click();
  await expect(page.locator('[data-account-indicator="picker"] [data-mailbox-connection]')).toHaveCount(1);
  await page.locator('[data-account-indicator="picker"]').click();
  const status=page.locator('[data-mailbox-menu] [data-mailbox-connection="auth-expired"]');await above(status);await status.hover();await above(page.getByRole('tooltip'));await expect(page.getByRole('tooltip')).toHaveText('Sign-in required');
  await page.getByRole('button',{name:'Hide connection icons',exact:true}).click();await expect(page.locator('[data-mailbox-connection]')).toHaveCount(0);
 });
});
