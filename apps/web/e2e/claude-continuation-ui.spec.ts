import {test,expect} from '@playwright/test';
import {readFile} from 'node:fs/promises';
import {CREDS,composeSurface,login,messageRow} from './helpers.ts';

test.setTimeout(45_000);
test.use({serviceWorkers:'block'});

test('message context menu opens a participant chooser without leaving mail',async({page})=>{
  const ids=new Set<string>();
  await page.route(/\/jmap(\/api)?$/,async route=>{
    if(route.request().method()!=='POST')return route.continue();
    const response=await route.fetch();
    const body=await response.json() as {methodResponses?:[string,{list?:Record<string,unknown>[]},string][]};
    for(const [method,args] of body.methodResponses??[]){
      if(method!=='Email/get')continue;
      for(const email of args.list??[]){
        if(email['subject']==='Lunch tomorrow?')ids.add(String(email['id']));
        if(!ids.has(String(email['id'])))continue;
        email['from']=[{name:'Ada Browser',email:'ada-browser@example.org'}];
        email['to']=[{name:'Grace Browser',email:'grace-browser@example.org'}];
        email['replyTo']=null;email['bcc']=null;
      }
    }
    await route.fulfill({response,json:body});
  });
  await login(page);
  const initialUrl=page.url();
  const row=messageRow(page,'Lunch tomorrow?');
  await row.click({button:'right'});
  await page.getByRole('menuitem',{name:'Save contacts from this message',exact:true}).click();
  await page.getByRole('menuitem',{name:'Choose contacts…',exact:true}).click();
  const dialog=page.getByRole('dialog',{name:'Save contacts from this message'});
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole('checkbox',{name:'Save ada-browser@example.org',exact:true})).toBeVisible();
  await expect(dialog.getByRole('checkbox',{name:'Save grace-browser@example.org',exact:true})).toBeVisible();
  await expect(dialog.getByRole('textbox',{name:'Name'}).first()).toHaveValue('Ada Browser');
  expect(page.url()).toBe(initialUrl);
  await dialog.getByRole('button',{name:'Cancel',exact:true}).click();
  await expect(dialog).toHaveCount(0);
});

test('reader exports CSV from a popup and stays on the message',async({page})=>{
  await login(page);await messageRow(page,'Lunch tomorrow?').click();
  const initialUrl=page.url();
  if(await page.getByTestId('reader-export').isVisible())await page.getByTestId('reader-export').click();
  else {await page.getByTestId('reader-more').click();await page.getByRole('menuitem',{name:'Export',exact:true}).click();}
  const dialog=page.getByTestId('export-dialog');await expect(dialog).toBeVisible();
  expect(page.url()).toBe(initialUrl);
  await dialog.getByTestId('export-format').selectOption('csv');
  const downloadEvent=page.waitForEvent('download');await dialog.getByTestId('export-start').click();
  const download=await downloadEvent;
  expect(download.suggestedFilename()).toMatch(/\.csv$/);
  expect(await readFile((await download.path())!,'utf8')).toContain('Lunch tomorrow?');
  await expect(dialog.getByTestId('export-result')).toBeVisible();
  await dialog.getByTestId('export-close').click();await expect(dialog).toHaveCount(0);
  expect(page.url()).toBe(initialUrl);
});

test('folder properties presents navigable sections and closes with Escape',async({page})=>{
  await login(page);
  const inbox=page.getByRole('button',{name:/^Inbox(?:\s|$)/}).first();
  await inbox.click({button:'right'});
  await page.getByRole('menuitem',{name:'Properties…',exact:true}).click();
  const dialog=page.getByTestId('folder-properties');await expect(dialog).toBeVisible();
  await dialog.getByRole('tab',{name:'Appearance',exact:true}).click();
  await expect(dialog.getByRole('tab',{name:'Appearance',exact:true})).toHaveAttribute('aria-selected','true');
  await dialog.getByRole('tab',{name:'General',exact:true}).click();
  await page.keyboard.press('ArrowDown');
  await expect(dialog.getByRole('tab',{name:'Appearance',exact:true})).toBeFocused();
  await page.keyboard.press('Escape');await expect(dialog).toHaveCount(0);
});


test('reader reuses chrome and body frame while switching messages',async({page})=>{
  await login(page);await messageRow(page,'Welcome to Mailwoman').click();
  const frame=page.locator('iframe[title="Message body"]');await expect(frame).toBeVisible();
  const saved=await Promise.all([page.locator('.reader__toolbar').elementHandle(),page.locator('.reader__subject').elementHandle(),frame.elementHandle()]);
  await page.evaluate(()=>{
    document.body.dataset['readerSmokeOverlays']='0';
    new MutationObserver(records=>{for(const record of records)for(const node of record.addedNodes){if(node instanceof Element&&(node.matches('[data-testid="async-pending"]')||node.querySelector('[data-testid="async-pending"]')))document.body.dataset['readerSmokeOverlays']=String(Number(document.body.dataset['readerSmokeOverlays'])+1);}}).observe(document.body,{subtree:true,childList:true});
  });
  for(const subject of ['Lunch tomorrow?','Welcome to Mailwoman','Your invoice is ready','Lunch tomorrow?']){
    await messageRow(page,subject).click();
    await expect(page.locator('.reader__subject')).toHaveText(subject);
    await expect(frame).toBeVisible();
    for(const [index,selector] of ['.reader__toolbar','.reader__subject','iframe[title="Message body"]'].entries())expect(await saved[index]!.evaluate((node,selector)=>node===document.querySelector(selector),selector)).toBe(true);
  }
  expect(await page.locator('body').getAttribute('data-reader-smoke-overlays')).toBe('0');
});

test('composer header folds and ribbon mode toggle works from the corner',async({page})=>{
  await login(page);await page.getByRole('button',{name:'Compose',exact:true}).click();
  const compose=composeSurface(page);await expect(compose).toBeVisible();
  await compose.getByLabel('To',{exact:true}).fill('person@example.org');
  await compose.getByLabel('Subject',{exact:true}).fill('Header smoke');
  const header=compose.getByTestId('compose-header-corner');await expect(header).toBeVisible();
  await expect(header).toHaveAttribute('aria-expanded','true');await header.click();
  await expect(header).toHaveAttribute('aria-expanded','false');await header.click();
  await expect(header).toHaveAttribute('aria-expanded','true');
  const pin=compose.getByTestId('compose-ribbon-pin');await expect(pin).toBeVisible();
  const before=await pin.getAttribute('aria-pressed');await pin.click();
  await expect(pin).toHaveAttribute('aria-pressed',before==='true'?'false':'true');
  await pin.click();await expect(pin).toHaveAttribute('aria-pressed',before!);
});


test('signature manager opens over the mounted composer and restores focus',async({page})=>{
  await page.setViewportSize({width:1280,height:720});
  await login(page);await page.getByRole('button',{name:'Compose',exact:true}).click();
  const compose=composeSurface(page);await expect(compose).toBeVisible();
  await compose.getByLabel('Subject',{exact:true}).fill('Keep this draft mounted');
  const original=await compose.elementHandle();
  const pin=compose.getByTestId('compose-ribbon-pin');if(await pin.getAttribute('aria-pressed')==='false')await pin.click();
  await compose.getByRole('tab',{name:'Insert',exact:true}).click();
  const more=compose.getByRole('button',{name:'More Signature commands',exact:true});await expect(more).toBeVisible();await more.click();
  const menu=compose.getByTestId('signature-menu');await expect(menu).toBeVisible();await menu.click();
  await compose.getByTestId('signature-edit').click();
  const manager=page.getByTestId('signature-manager');await expect(manager).toBeVisible();
  await expect(manager.getByRole('heading',{name:'Signatures',exact:true})).toBeVisible();
  await manager.getByRole('button',{name:'Close',exact:true}).click();await expect(manager).toHaveCount(0);
  await expect(compose.getByLabel('Subject',{exact:true})).toHaveValue('Keep this draft mounted');
  expect(await original!.evaluate(node=>node.isConnected)).toBe(true);
  await expect(more).toBeFocused();
});


for(const pending of [false,true])test(`export request is cleared when logout unmounts mail in the same document (${pending?'lazy gap':'open popup'})`,async({page})=>{
  let releaseChunk=()=>{};
  let chunkRequested=()=>{};
  const requested=new Promise<void>(resolve=>{chunkRequested=resolve;});
  const gate=new Promise<void>(resolve=>{releaseChunk=resolve;});
  if(pending)await page.route(/\/assets\/ExportDialog-[^/]+\.js(?:\?.*)?$/,async route=>{const response=await route.fetch();chunkRequested();await gate;await route.fulfill({response});});
  let first=true;
  await login(page);await messageRow(page,'Lunch tomorrow?').click();
  const openExport=async()=>{
    const direct=page.getByTestId('reader-export');
    if(await direct.isVisible())await direct.click();
    else {await page.getByTestId('reader-more').click();await page.getByRole('menuitem',{name:'Export',exact:true}).click();}
    if(pending&&first){first=false;await Promise.race([requested,page.waitForTimeout(10000).then(()=>{throw new Error('Export lazy chunk was not intercepted');})]);}
    else await expect(page.getByTestId('export-dialog')).toBeVisible();
  };
  await openExport();
  const originalShell=await page.locator('.workspace-shell').elementHandle();
  await page.evaluate(()=>{document.documentElement.dataset['exportLifecycle']='same-document';});
  // The modal traps pointer/focus interaction. Dispatch the existing Logout
  // button's click to exercise the real logout handler and screen disposal;
  // this is a synthetic lifecycle trigger, not a claim about a sidebar gesture.
  // While the JS chunk is held there is no modal: Logout is a real pointer click.
  if(pending)await page.getByRole('button',{name:'Log out',exact:true}).click();
  else await page.evaluate(()=>document.querySelector<HTMLButtonElement>('.sidebar__logout')!.click());
  await expect(page.getByRole('button',{name:'Sign in',exact:true})).toBeVisible();
  expect(await originalShell!.evaluate(node=>node.isConnected)).toBe(false);
  releaseChunk();
  await expect(page.getByTestId('export-dialog')).toHaveCount(0);
  await page.getByLabel('Email or username',{exact:true}).fill(CREDS.username);
  await page.getByLabel('Password',{exact:true}).fill(CREDS.password);
  for(let attempt=0;attempt<4;attempt++){
    const response=page.waitForResponse(r=>r.request().method()==='POST'&&new URL(r.url()).pathname.endsWith('/api/login'));
    await page.getByRole('button',{name:'Sign in',exact:true}).click();
    const signed=await response;
    if(signed.status()!==429){expect(signed.status()).toBe(200);break;}
    const seconds=Number(signed.headers()['retry-after']??'3');
    await page.waitForTimeout(Math.min(Number.isFinite(seconds)?seconds:3,30)*1000+250);
  }
  await expect(messageRow(page,'Lunch tomorrow?')).toBeVisible();
  expect(await page.locator('html').getAttribute('data-export-lifecycle')).toBe('same-document');
  await expect(page.getByTestId('export-dialog')).toHaveCount(0);
  await messageRow(page,'Welcome to Mailwoman').click();await openExport();
  const dialog=page.getByTestId('export-dialog');await dialog.getByTestId('export-format').selectOption('csv');
  const downloaded=page.waitForEvent('download');await dialog.getByTestId('export-start').click();
  expect(await readFile((await (await downloaded).path())!,'utf8')).toContain('Welcome to Mailwoman');
});
