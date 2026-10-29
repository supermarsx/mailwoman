import {test,expect} from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import {adminLogin} from './admin-login.ts';

test('configuration transfer keyboard and bounded layout @phone',async({page},info)=>{

 await page.route(`**${ADMIN}/config/export`,route=>route.fulfill({json:{bundle:{format:'mailwoman-settings',version:1,entries:[]},excluded:['secrets']}}));
 await page.route(`**${ADMIN}/config/import`,route=>route.fulfill({json:{changes:[{key:'mail.trustedRecipientDomains',before:{globalValue:['previous.example']},after:{globalValue:['a-very-long-configured-destination-name.example.test','second.example.test']},impact:{}}],revisions:{'mail.trustedRecipientDomains':'"v1"'}}}));
 await adminLogin(page);await page.goto(`${ADMIN}#/configuration-transfer`);
 const spacing=await page.getByRole('tablist',{name:'Configuration & backups'}).evaluate(element=>{const parent=element.closest('section')!;const paragraphs=parent.querySelectorAll(':scope > p');const help=paragraphs[0]!.getBoundingClientRect();return element.getBoundingClientRect().top-help.bottom;});expect(spacing).toBeLessThanOrEqual(26);
 const first=page.getByRole('tab',{name:'Export settings'});await first.focus();await page.keyboard.press('ArrowRight');await expect(page.getByRole('tab',{name:'Import settings'})).toBeFocused();await expect(page.getByRole('tabpanel',{name:'Import settings'})).toBeVisible();
 await page.getByLabel('Settings JSON file').setInputFiles({name:'settings.json',mimeType:'application/json',buffer:Buffer.from(JSON.stringify({format:'mailwoman-settings',version:1,entries:[]}))});
 await expect(page.getByRole('region',{name:'Review settings changes'})).toBeVisible();
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
 expect(await page.getByRole('region',{name:'Review settings changes'}).locator('pre').first().evaluate(element=>element.getBoundingClientRect().width)).toBeGreaterThanOrEqual(200);
 const button=await page.getByRole('button',{name:'Apply reviewed changes'}).boundingBox();expect(button!.width).toBeLessThan(300);
 await page.screenshot({path:info.outputPath('transfer-import.png'),fullPage:true});
 await page.getByRole('tab',{name:'Import settings'}).focus();await page.keyboard.press('End');await expect(page.getByRole('tabpanel',{name:'Backup destinations'})).toBeVisible();expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
 await page.screenshot({path:info.outputPath('transfer-backups.png'),fullPage:true});
 await page.keyboard.press('Home');await expect(first).toBeFocused();const downloaded=page.waitForEvent('download');await page.getByRole('button',{name:'Download settings',exact:true}).click();expect((await downloaded).suggestedFilename()).toBe('mailwoman-settings.json');
});
