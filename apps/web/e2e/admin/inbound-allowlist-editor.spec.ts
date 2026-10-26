import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {test,expect} from '../csrf.ts';
import {adminLogin} from './admin-login.ts';

test('inbound policy editor confirms conditional disabled-only changes @phone',async({page},info)=>{
 const dist=path.resolve('../../.orchestration/reports/codex-inbound-ui-web');
 await page.route('**/*',async route=>{
  const url=new URL(route.request().url());if(url.pathname!=='/admin'&&!url.pathname.startsWith('/assets/'))return route.continue();
  const file=path.resolve(dist,url.pathname==='/admin'?'index.html':url.pathname.slice(1));if(!file.startsWith(dist+path.sep))return route.abort();
  try{const body=await readFile(file);await route.fulfill({status:200,body,contentType:file.endsWith('.html')?'text/html':file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':file.endsWith('.woff2')?'font/woff2':'application/octet-stream'});}catch{await route.continue();}
 });


 let revision='"one"';let config={admin:{enabled:false,cidrs:[],hostnames:[]},webmail:{enabled:false,cidrs:[],hostnames:[]},dns:{servers:[],timeoutMs:2000,cacheSeconds:300,maxHosts:64,maxAddresses:16,concurrency:4}};let saves=0;
 await page.route('**/admin/security/inbound-allowlist',async route=>{if(route.request().method()==='PUT'){const next=route.request().postDataJSON();expect(next.admin.enabled).toBe(false);expect(next.webmail.enabled).toBe(false);expect(route.request().headers()['if-match']).toBe(revision);config=next;revision='"two"';saves++;}await route.fulfill({headers:{etag:revision},json:{config,status:{hosts:[],refreshing:false}}});});
 await adminLogin(page);await page.goto('/admin#/inbound-allowlist');await expect(page.getByRole('heading',{name:'Inbound access',exact:true})).toBeVisible();
 const timeout=page.getByLabel('Lookup timeout (milliseconds)',{exact:true});await timeout.fill('1500');await page.getByRole('button',{name:'Refresh saved policy and DNS status'}).click();await expect(timeout).toHaveValue('1500');
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);const review=page.getByRole('button',{name:'Review changes'});expect((await review.boundingBox())!.width).toBeLessThan(250);await page.screenshot({path:info.outputPath('inbound-form.png'),fullPage:true});
 await review.click();const dialog=page.getByRole('alertdialog');await expect(dialog).toContainText('no automatic loopback');const apply=dialog.getByRole('button',{name:'Apply confirmed policy'});await expect(apply).toBeDisabled();expect(saves).toBe(0);await dialog.getByRole('textbox').fill('APPLY');await page.screenshot({path:info.outputPath('inbound-confirm.png'),fullPage:true});await apply.click();await expect(page.getByText('The server accepted the inbound access policy.',{exact:true})).toBeVisible();expect(saves).toBe(1);await page.reload();await expect(timeout).toHaveValue('1500');await expect(page.getByRole('checkbox',{name:'Restrict this surface to the allowlist'}).first()).not.toBeChecked();await expect(page.getByRole('checkbox',{name:'Restrict this surface to the allowlist'}).last()).not.toBeChecked();
});
