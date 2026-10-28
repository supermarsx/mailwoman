import {test,expect} from '../csrf.ts';
import {adminLogin} from './admin-login.ts';

test('compact alias rows retain accessible edit, clone and remove actions @phone',async({page},info)=>{
 await adminLogin(page);
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
