import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {test,expect} from '../csrf.ts';
import {adminLogin} from './admin-login.ts';

test('remaining admin forms keep readable controls and separated actions @phone',async({page},info)=>{
 const dist=path.resolve('../../.orchestration/reports/codex-admin-review-web');
 await page.route('**/*',async route=>{
  const url=new URL(route.request().url());if(url.pathname!=='/admin'&&!url.pathname.startsWith('/assets/'))return route.continue();
  const file=path.resolve(dist,url.pathname==='/admin'?'index.html':url.pathname.slice(1));if(!file.startsWith(dist+path.sep))return route.abort();
  try{const body=await readFile(file);await route.fulfill({status:200,body,contentType:file.endsWith('.html')?'text/html':file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':file.endsWith('.woff2')?'font/woff2':'application/octet-stream'});}catch{await route.continue();}
 });


 await adminLogin(page);
 await page.goto('/admin#/logs');const search=page.getByLabel('Account, attempted username or IP',{exact:true});await expect(search).toBeVisible();expect((await search.boundingBox())!.height).toBeGreaterThanOrEqual(36);expect((await search.boundingBox())!.width).toBeGreaterThanOrEqual(190);await expect(page.getByRole('group',{name:'Authentication logs',exact:true})).toHaveAttribute('tabindex','0');
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);await page.screenshot({path:info.outputPath('logs.png')});
 await page.goto('/admin#/maintenance');const rebuild=page.getByRole('button',{name:'Rebuild search index',exact:true});await expect(rebuild).toBeVisible();const rethread=page.getByRole('heading',{name:'Re-thread mailbox',exact:true});const lower=(await rethread.boundingBox())!,upper=(await rebuild.boundingBox())!;expect(lower.y-(upper.y+upper.height)).toBeGreaterThanOrEqual(16);await page.screenshot({path:info.outputPath('maintenance.png')});
 await page.goto('/admin#/privacy');const edit=page.getByRole('link',{name:'Edit policy',exact:true});await expect(edit.first()).toBeVisible();for(const link of await edit.all())expect((await link.boundingBox())!.height).toBeLessThan(48);expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);await page.screenshot({path:info.outputPath('privacy.png')});
 await page.goto('/admin#/branding');await expect(page.getByLabel('Product name',{exact:true})).toBeVisible();expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);await page.screenshot({path:info.outputPath('branding.png')});
});
