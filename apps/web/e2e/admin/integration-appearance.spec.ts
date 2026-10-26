import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {test,expect} from '../csrf.ts';
import {adminLogin} from './admin-login.ts';

test('integration settings keep long rows, actions and loading bounded @phone',async({page},info)=>{
 const dist=path.resolve('../../.orchestration/reports/codex-admin-integrations-web');
 await page.route('**/*',async route=>{
  const url=new URL(route.request().url());if(url.pathname!=='/admin'&&!url.pathname.startsWith('/assets/'))return route.continue();
  const file=path.resolve(dist,url.pathname==='/admin'?'index.html':url.pathname.slice(1));if(!file.startsWith(dist+path.sep))return route.abort();
  try{const body=await readFile(file);await route.fulfill({status:200,body,contentType:file.endsWith('.html')?'text/html':file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':file.endsWith('.woff2')?'font/woff2':'application/octet-stream'});}catch{await route.continue();}
 });


 await page.route('**/admin/nextcloud/instances',route=>route.fulfill({json:{instances:[{id:'cloud',name:'Company Nextcloud with a long descriptive instance name',baseUrl:'https://cloud.example.test/'+('long-installation-path-'.repeat(18)),enabled:true,etag:'"one"'}],maxInstances:64,maxFileBytes:8388608}}));
 let release!:()=>void;const hold=new Promise<void>(resolve=>{release=resolve;});
 await page.route('**/admin/package-tracking',async route=>{await hold;await route.fulfill({headers:{etag:'"one"'},json:{provider:{enabled:false,refreshSecs:900,configured:true,revision:'one'}}});});
 await adminLogin(page);await page.goto('/admin#/nextcloud');await expect(page.getByRole('table',{name:'Nextcloud instances'})).toBeVisible();
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
 expect(await page.getByRole('table',{name:'Nextcloud instances'}).locator('tbody tr').first().evaluate(element=>element.getBoundingClientRect().height)).toBeLessThan(140);
 const edit=page.getByRole('button',{name:'Open Company Nextcloud with a long descriptive instance name'});await page.keyboard.press('Tab');await edit.focus();await expect(page.getByRole('tooltip')).toContainText('Edit Nextcloud instance');await page.keyboard.press('Escape');await expect(page.getByRole('tooltip')).toHaveCount(0);
 await page.screenshot({path:info.outputPath('nextcloud-list.png'),fullPage:true});await edit.click();await expect(page.getByLabel(/^HTTPS installation URL/)).toHaveValue(/^https:/);
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);const save=await page.getByRole('button',{name:'Save',exact:true}).boundingBox();expect(save!.width).toBeLessThan(200);await page.screenshot({path:info.outputPath('nextcloud-editor.png'),fullPage:true});
 await page.goto('/admin#/package-tracking');await expect(page.getByTestId('async-pending')).toBeVisible();await page.screenshot({path:info.outputPath('package-pending.png'),fullPage:true});release();await expect(page.getByLabel(/^Provider API key/)).toBeVisible();expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);await page.screenshot({path:info.outputPath('package-editor.png'),fullPage:true});
});
