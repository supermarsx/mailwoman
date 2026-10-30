import {test,expect} from '../csrf.ts';
import {adminLogin} from '../admin/admin-login.ts';
import {ADMIN} from '../admin/admin-path.ts';

test('diagnostics check tables keep labelled columns and contain overflow on narrow screens',async({page},info)=>{
 await page.setViewportSize({width:info.project.name==='phone'?390:1440,height:900});
 await adminLogin(page);await page.goto(`${ADMIN}#/diagnostics/server`);
 const pages=page.getByRole('navigation',{name:'Diagnostics pages'});
 for(const name of ['Server','Connectivity','Security posture']){
  await pages.getByRole('link',{name,exact:true}).click();
  const table=page.getByRole('table',{name,exact:true});await expect(table).toBeVisible();
  for(const column of ['Check','Status','Result','Duration','Actions'])await expect(table.getByRole('columnheader',{name:column,exact:true})).toBeVisible();
  await expect(table.locator('tbody tr').first()).toBeVisible();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
  await page.screenshot({path:info.outputPath(`${name.replaceAll(' ','-')}.png`),fullPage:true});
 }
 await pages.getByRole('link',{name:'Mail backends',exact:true}).click();
 await expect(pages.getByRole('link',{name:'Mail backends',exact:true})).toHaveAttribute('aria-current','page');
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
});
