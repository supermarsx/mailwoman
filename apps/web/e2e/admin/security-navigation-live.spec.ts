import {test,expect} from '../csrf.ts';
import {adminLogin} from './admin-login.ts';

test('Security Centre groups existing tools with contained permission-aware navigation @phone',async({page},info)=>{

 await adminLogin(page);
 const bans=page.waitForResponse(response=>new URL(response.url()).pathname==='/admin/bans');
 await page.goto('/admin#/observability/bans');
 expect((await bans).status()).toBe(200);
 const tabs=page.getByRole('navigation',{name:'Security centre',exact:true});
 await expect(tabs.getByRole('link',{name:'IP ban list',exact:true})).toHaveAttribute('aria-current','page');
 await expect(page.getByRole('region',{name:'IP ban list',exact:true})).toBeVisible();
 await expect(page.locator('#admin-main [data-testid="async-pending"]')).toHaveCount(0);
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
 await page.screenshot({path:info.outputPath('security-bans.png'),fullPage:true});
 const add=page.getByRole('button',{name:'Add ban',exact:true});expect((await add.boundingBox())!.width).toBeLessThan(240);await add.click();
 await expect(page.getByLabel('IP address',{exact:true})).toBeVisible();await page.getByRole('button',{name:'Cancel',exact:true}).click();await expect(page).toHaveURL(/#\/security\/bans$/);
 for(const [label,route,api] of [['Temporary sign-in locks','locks','/admin/bans/automatic'],['Sign-in country policy','geography','/admin/bans/geography'],['Aggregate and adaptive protection','protection','/admin/bans/protection'],['Deployment checks','deployment','/admin/security/deployment']] as const){
   const loaded=page.waitForResponse(response=>new URL(response.url()).pathname===api);
   const link=tabs.getByRole('link',{name:label,exact:true});await link.focus();await page.keyboard.press('Enter');expect((await loaded).status()).toBe(200);
   await expect(link).toHaveAttribute('aria-current','page');await expect(page).toHaveURL(new RegExp('#/security/'+route+'$'));
   await expect(page.locator('#admin-main [data-testid="async-pending"]')).toHaveCount(0);
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
   await page.screenshot({path:info.outputPath('security-'+route+'.png'),fullPage:true});
 }
 const sidebar=page.getByRole('navigation',{name:'Admin sections',exact:true});
 for(const label of ['Sessions','Security policy','Inbound access'])await expect(sidebar.getByRole('link',{name:label,exact:true})).toHaveCount(0);
 for(const [legacy,label] of [['sessions','Sessions'],['security-policy','Security policy'],['inbound-allowlist','Inbound access']] as const){await page.goto('/admin#/'+legacy);await expect(tabs.getByRole('link',{name:label,exact:true})).toHaveAttribute('aria-current','page');}
 await page.goto('/admin#/two-factor');await expect(tabs.getByRole('link',{name:'Overview',exact:true})).toHaveAttribute('aria-current','page');
 await page.goto('/admin#/observability');await expect(page.getByRole('heading',{name:'Observability',exact:true})).toBeVisible();await expect(page.getByRole('table',{name:'IP ban list'})).toHaveCount(0);
});
