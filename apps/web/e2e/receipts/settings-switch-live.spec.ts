import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {test,expect} from '../csrf.ts';

test('rapid settings navigation never replaces the workspace with a loading screen',async({page,context},info)=>{
 const dist=path.resolve('../../.orchestration/reports/codex-settings-switch-web');
 await context.route('**/*',async route=>{
  const url=new URL(route.request().url());
  if(url.pathname!=='/'&&!url.pathname.startsWith('/assets/'))return route.continue();
  const file=path.resolve(dist,url.pathname==='/'?'index.html':url.pathname.slice(1));
  if(!file.startsWith(dist+path.sep))return route.abort();
  try{await route.fulfill({status:200,body:await readFile(file),contentType:file.endsWith('.html')?'text/html':file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':'application/octet-stream'});}catch{await route.continue();}
 });
 await page.goto('/');
 await page.getByLabel('Email or username',{exact:true}).fill('testuser');
 await page.getByLabel('Password',{exact:true}).fill('testpass');
 await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
 if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
 await page.locator('.sidebar__footer').getByRole('button',{name:'Settings',exact:true}).click();
 const nav=page.getByRole('navigation',{name:'Settings sections'});
 await expect(nav).toBeVisible();
 let release!:()=>void;
 const gate=new Promise<void>(resolve=>{release=resolve;});
 let requests=0;
 await context.route('**/api/account/signatures',async route=>{
  requests++;await gate;await route.fulfill({json:{signatures:[]}});
 });
 await nav.getByRole('button',{name:'Signatures',exact:true}).click();
 await expect.poll(()=>requests).toBe(1);
 await expect(nav).toBeVisible();
 await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
 for(const name of ['Appearance','Signatures','Appearance','Signatures','Appearance']){
  await nav.getByRole('button',{name,exact:true}).click();
  await expect(nav).toBeVisible();
  await expect(nav.getByRole('button',{name,exact:true})).toHaveAttribute('aria-current','page');
 }
 release();
 await expect(page.locator('[data-settings-section="signatures"] [data-testid="signature-list"]')).toBeAttached();
 await expect(page.locator('[data-settings-section="signatures"]')).not.toBeVisible();
 await nav.getByRole('button',{name:'Signatures',exact:true}).click();
 await expect(page.locator('[data-settings-section="signatures"]')).toBeVisible();
 await expect(page.getByRole('button',{name:'New signature',exact:true})).toBeVisible();
 expect(requests).toBe(1);
 await page.screenshot({path:info.outputPath('settings-switch-stable.png'),fullPage:true});
});
