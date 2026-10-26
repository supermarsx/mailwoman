import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {test,expect} from '../csrf.ts';

test('compact alias rows retain accessible edit, clone and remove actions @phone',async({page},info)=>{
 const dist=path.resolve('../../.orchestration/reports/codex-alias-compact-web');
 await page.route('**/*',async route=>{const url=new URL(route.request().url());if(url.pathname!=='/admin'&&!url.pathname.startsWith('/assets/'))return route.continue();const file=path.resolve(dist,url.pathname==='/admin'?'index.html':url.pathname.slice(1));if(!file.startsWith(dist+path.sep))return route.abort();try{const body=await readFile(file);await route.fulfill({status:200,body,contentType:file.endsWith('.html')?'text/html':file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':'application/octet-stream'});}catch{await route.continue();}});
 await page.goto('/admin');await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill(process.env['MW_ALIAS_TEST_PASSWORD']??'AliasFixture-2026!');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.locator('#admin-main')).toBeVisible();
 const name=`compact-${Date.now()}.example.test`;const aliases=[`first-${name}`,`a-long-but-valid-alias-name-that-wraps-on-small-screens.${name}`];
 expect((await page.request.put(`/admin/domains/${name}`,{data:{enabled:false,upstream:{version:2,backends:[]}}})).ok()).toBe(true);
 try{
  for(const alias of aliases){const detail=await(await page.request.get(`/admin/domains/${name}`)).json();expect((await page.request.put(`/admin/domains/${name}/aliases/${alias}`,{headers:{'if-match':detail.mappingEtag}})).ok()).toBe(true);}
  await page.goto(`/admin#/domains/${name}/aliases`);
  const table=page.getByRole('table',{name:'Domain aliases'});await expect(table).toBeVisible();
  for(const alias of aliases){const row=table.getByRole('row').filter({has:page.getByRole('rowheader',{name:alias,exact:true})});await expect(row.getByRole('button',{name:'Edit alias',exact:true})).toBeVisible();await expect(row.getByRole('button',{name:'Clone alias',exact:true})).toBeVisible();await expect(row.getByRole('button',{name:`Remove ${alias}`,exact:true})).toBeVisible();const box=await row.getByRole('button',{name:'Edit alias',exact:true}).boundingBox();expect(box!.height).toBeGreaterThanOrEqual(info.project.name==='phone'?44:40);}
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.screenshot({path:info.outputPath('compact-aliases.png'),fullPage:true});
  await table.getByRole('row').filter({has:page.getByRole('rowheader',{name:aliases[0]!,exact:true})}).getByRole('button',{name:'Clone alias',exact:true}).click();await expect(page.getByRole('textbox',{name:'Alias domain'})).toHaveValue(aliases[0]!);
 }finally{const response=await page.request.get(`/admin/domains/${name}`);if(response.ok())expect((await page.request.delete(`/admin/domains/${name}`,{headers:{'if-match':response.headers()['etag']!}})).ok()).toBe(true);}
});
