import {test,expect} from '../csrf.ts';
import {signInManually} from '../helpers.ts';
test('verified personal signatures preserve independent formats through reload and deletion @phone',async({page},info)=>{
 await page.goto('/');await signInManually(page,{username:'testuser',password:'testpass'});
 const openSettings=async()=>{const show=page.getByRole('button',{name:'Show folders sidebar'});if(await show.isVisible())await show.click();if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();await page.getByRole('button',{name:'Settings',exact:true}).click();await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Signatures',exact:true}).click();};
 await openSettings();await page.getByRole('button',{name:'New signature',exact:true}).click();
 const name=`Owned formats ${info.project.name} ${Date.now()}`;
 await page.getByLabel('Name',{exact:true}).fill(name);await page.getByLabel('Signature format').selectOption('html');await page.getByRole('checkbox',{name:'Edit HTML source'}).check();
 await page.getByRole('textbox',{name:'Edit HTML source'}).fill('<p>Rich <strong>personal signature</strong></p>');await page.getByLabel('Plain-text alternative').fill('Independent plain <personal>');
 await page.getByTestId('signature-save').click();await expect(page.getByTestId('signature-list')).toContainText(name);
 await page.reload();await openSettings();const row=page.getByTestId('signature-list').locator('li').filter({hasText:name});await row.getByRole('button',{name:'Edit',exact:true}).click();
 await expect(page.getByLabel('Plain-text alternative')).toHaveValue('Independent plain <personal>');await page.getByRole('checkbox',{name:'Edit HTML source'}).check();await expect(page.getByRole('textbox',{name:'Edit HTML source'})).toHaveValue('<p>Rich <strong>personal signature</strong></p>');
 await page.screenshot({path:info.outputPath('verified-signature-formats.png'),fullPage:true});expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 await page.getByRole('button',{name:'Cancel',exact:true}).click();await row.getByRole('button',{name:'Delete',exact:true}).click();await expect(row).toHaveCount(0);
});
