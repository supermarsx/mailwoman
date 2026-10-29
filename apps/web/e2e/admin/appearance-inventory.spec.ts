import {writeFile} from 'node:fs/promises';
import {test,expect} from '../csrf.ts';
import {adminLogin} from './admin-login.ts';

test('remaining admin screens retain bounded read-only layout @phone',async({page},info)=>{
  test.setTimeout(180_000);
  await adminLogin(page);
  const results:Array<{section:string;width:number;documentWidth:number;pending:number;headings:(string|null)[];tables:number;alerts:number}>=[];
  for(const section of ['ai-assistant','branding','diagnostics','logs','maintenance','oauth-clients','plugins','ui-plugins','privacy','roles','security-policy','server-metadata','settings','config','observability']){
    if(process.env['MW_APPEARANCE_SECTION']&&process.env['MW_APPEARANCE_SECTION']!==section)continue;
    await test.step(section,async()=>{
      // A fresh document prevents a previous route's heading satisfying readiness
      // while the next lazy screen is still replacing it with a skeleton.
      await page.goto(`/admin?appearance-audit=${section}#/${section}`);
      await page.locator('main').first().waitFor();
      await expect(page.locator('main h1, main h2').first(),`${section} finished mounting`).toBeVisible();
      await page.waitForLoadState('networkidle',{timeout:8000}).catch(()=>undefined);
      const metrics=await page.evaluate(()=>({width:innerWidth,documentWidth:document.documentElement.scrollWidth,pending:document.querySelectorAll('[data-testid="async-pending"]').length,headings:[...document.querySelectorAll('main h1,main h2')].map(node=>node.textContent),tables:document.querySelectorAll('main table').length,alerts:document.querySelectorAll('main [role="alert"]').length}));
      results.push({section,...metrics});
      await page.screenshot({path:info.outputPath(`${section}.png`)});
      expect.soft(metrics.documentWidth,`${section} document overflow`).toBeLessThanOrEqual(metrics.width+1);
      if(section==='settings'&&process.env['MW_EXPECT_COMPACT_FILTERS']==='1'){
        const filters=page.getByRole('button',{name:'Filters',exact:true});
        await expect(filters).toHaveAttribute('aria-expanded','false');
        await filters.focus();await page.keyboard.press('Enter');
        await expect(page.getByRole('group',{name:'Filters',exact:true})).toBeVisible();
        await page.getByRole('button',{name:'Appearance',exact:true}).click();
        await expect(page.getByRole('button',{name:'Appearance',exact:true})).toHaveAttribute('aria-pressed','true');
        await page.getByRole('button',{name:'Filters (1)',exact:true}).click();
        await expect(page.getByRole('group',{name:'Filters',exact:true})).toHaveCount(0);
      }
    });
  }
  await info.attach('appearance-inventory',{body:JSON.stringify(results,null,2),contentType:'application/json'});
  await writeFile(info.outputPath('inventory.json'),JSON.stringify(results,null,2));
});
