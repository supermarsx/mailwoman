import {test,expect} from '../csrf.ts';
import {signInManually} from '../helpers.ts';
test('signature and identity editors use dedicated saved pages and guarded navigation @phone',async({page})=>{
 await page.goto('/');await signInManually(page,{username:'testuser',password:'testpass'});
 const show=page.getByRole('button',{name:'Show folders sidebar'});if(await show.isVisible())await show.click();
 if(test.info().project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
 await page.getByRole('button',{name:'Settings',exact:true}).click();const sections=page.getByRole('navigation',{name:'Settings sections'});
 await sections.getByRole('button',{name:'Signatures',exact:true}).click();await page.getByRole('button',{name:'New signature',exact:true}).click();
 await expect(page.getByTestId('signature-list')).toHaveCount(0);const name=`editor-${test.info().project.name}-${Date.now()}`;
 await page.getByTestId('signature-editor').getByLabel('Name',{exact:true}).fill(name);await page.getByLabel('Signature',{exact:true}).fill('Saved per-account signature');
 await sections.getByRole('button',{name:'Identities',exact:true}).click();const guard=page.getByRole('alertdialog',{name:'Discard unsaved changes?'});await expect(guard).toBeVisible();await guard.getByRole('button',{name:'Keep editing'}).click();await expect(page.getByLabel('Signature',{exact:true})).toHaveValue('Saved per-account signature');
 await page.screenshot({path:test.info().outputPath('dedicated-signature-editor.png'),fullPage:true});
 await page.getByTestId('signature-save').click();await expect(page.getByTestId('signature-list')).toContainText(name);
 await sections.getByRole('button',{name:'Identities',exact:true}).click();await page.getByRole('button',{name:'New identity',exact:true}).click();await expect(page.getByTestId('identity-list')).toHaveCount(0);
 await page.getByLabel('Display name',{exact:true}).fill(name);await page.getByLabel('Email address',{exact:true}).fill('testuser@example.org');await page.getByTestId('identity-save').click();await expect(page.getByTestId('identity-list')).toContainText(name);
 const identity=page.getByTestId('identity-list').locator('li').filter({hasText:name});await identity.getByRole('button',{name:'Delete',exact:true}).click();await expect(identity).toHaveCount(0);
 await sections.getByRole('button',{name:'Signatures',exact:true}).click();const signature=page.getByTestId('signature-list').locator('li').filter({hasText:name});await signature.getByRole('button',{name:'Delete',exact:true}).click();await expect(signature).toHaveCount(0);
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
});
