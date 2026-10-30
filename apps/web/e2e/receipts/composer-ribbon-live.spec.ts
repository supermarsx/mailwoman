import {test,expect} from '../csrf.ts';
import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {pinComposerRibbonOnPhone,revealRibbonControl} from '../helpers.ts';

// 9dab0fe7 (D4): below 600 px the ribbon starts in auto-hide; these legs are about
// what its controls do, so the phone project pins it for the test (helpers.ts).
let unpinRibbon:()=>Promise<void>=async()=>{};
test.beforeEach(async({playwright,baseURL},info)=>{unpinRibbon=await pinComposerRibbonOnPhone(playwright.request,baseURL,info.project.name);});
test.afterEach(async()=>{await unpinRibbon();});
// Optional current SPA overlay on the isolated mail fixture. API calls remain real;
// server shell routing is covered separately by t32_workspace_popup.
test.beforeEach(async({context})=>{
 const configured=process.env['MW_DETACHED_DIST'];if(!configured)return;
 const dist=path.resolve(configured);
 await context.route('**/*',async route=>{
  const pathname=new URL(route.request().url()).pathname;
  if(pathname!=='/'&&pathname!=='/workspace-popup.html'&&!pathname.startsWith('/assets/'))return route.continue();
  const file=path.resolve(dist,pathname==='/'?'index.html':pathname.slice(1));
  if(!file.startsWith(dist+path.sep))return route.abort();
  const body=await readFile(file);
  const response=await route.fetch();
  await route.fulfill({response,status:200,body,contentType:file.endsWith('.html')?'text/html':file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':'application/octet-stream'});
 });
});
// The actual fixture keeps the production three-second initial-login cooldown.
test.beforeEach(async()=>{await new Promise(resolve=>setTimeout(resolve,3100));});
test('keeps compact ribbon controls above the retained editor and round-trips source',async({page},info)=>{
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
 await page.locator('#shell-nav').getByRole('button',{name:'Compose',exact:true}).click();
 const compose=page.locator('.workspace-compose').last();await compose.getByLabel('Subject',{exact:true}).fill(`Ribbon ${info.project.name}`);
 // On a phone the ribbon folds groups into dropdowns (t38-e5); open one when its control is folded away.
 const unfold=async(_group:string,control:import('@playwright/test').Locator)=>{await revealRibbonControl(page,compose.locator('.compose-ribbon'),control);};
 const editor=compose.locator('.ProseMirror');await editor.fill('Generated rich HTML');await editor.press('Control+a');await unfold('Font',compose.getByRole('button',{name:'Bold',exact:true}));await compose.getByRole('button',{name:'Bold',exact:true}).click();await expect(editor.locator('strong')).toHaveText('Generated rich HTML');
 expect(await compose.locator('.compose-rt > .compose-rt__chrome').count()).toBe(0);
 await expect(compose.getByRole('button',{name:'Cancel',exact:true})).toHaveCount(0);await expect(compose.getByRole('button',{name:'Close',exact:true})).toHaveCount(0);
 await unfold('Message type',compose.getByTestId('format-toggle'));await compose.getByTestId('format-toggle').selectOption('html');const source=compose.getByLabel('HTML source',{exact:true});await expect(source).toHaveValue(/<strong>Generated rich HTML<\/strong>/);
 await source.fill('<h2>Edited source</h2><p><em>Preserved</em></p>');await unfold('Message type',compose.getByTestId('format-toggle'));await compose.getByTestId('format-toggle').selectOption('rich');await expect(editor.locator('h2')).toHaveText('Edited source');await expect(editor.locator('em')).toHaveText('Preserved');
 const markers=compose.getByRole('button',{name:'Paragraph marks',exact:true});await unfold('Paragraph',markers);await markers.click();await expect(markers).toHaveAttribute('aria-pressed','true');await expect(compose.locator('.compose-rt__footer').getByRole('button',{name:'Paragraph marks'})).toHaveCount(0);
 // Every visible ribbon group is exactly three rows tall (t38-e5: 3*--mw-ribbon-row + 2*--mw-ribbon-gap; 88 px at 28 px rows),
 // nothing inside the ribbon scrolls, and the ribbon ends above the message body.
 for(const name of ['Format text','Insert','Table']){await compose.getByRole('tab',{name,exact:true}).click();const g=await compose.evaluate(node=>{const bar=node.querySelector('[data-bar="composerRibbon"]')!;const cs=getComputedStyle(bar);const expected=3*parseFloat(cs.getPropertyValue('--mw-ribbon-row'))+2*parseFloat(cs.getPropertyValue('--mw-ribbon-gap'));const panel=[...bar.querySelectorAll('[data-ribbon-tab]')].find(p=>!(p as HTMLElement).hidden)!;const hs=[...panel.querySelectorAll('[data-ribbon-body],[data-ribbon-controls]')].filter(e=>!e.closest('[hidden]')).map(e=>e.getBoundingClientRect().height);const over=[...bar.querySelectorAll('*')].filter(e=>!e.closest('[hidden]')&&e.clientHeight>0&&e.scrollHeight>e.clientHeight+1).length;const body=node.querySelector('.compose > .field--grow')!;return {expected,hs,over,ribbonBottom:bar.getBoundingClientRect().bottom,bodyTop:body.getBoundingClientRect().top,pageOverflow:document.documentElement.scrollWidth-innerWidth};});expect(g.hs.length).toBeGreaterThan(0);for(const h of g.hs)expect(Math.abs(h-g.expected)).toBeLessThanOrEqual(0.5);expect(g.over).toBe(0);expect(g.ribbonBottom).toBeLessThanOrEqual(g.bodyTop+0.5);expect(g.pageOverflow).toBeLessThanOrEqual(1);}
 await compose.locator('.compose-ribbon__content').evaluate(node=>{node.scrollLeft=0;});await page.screenshot({path:info.outputPath('composer-ribbon.png'),fullPage:true});
});
