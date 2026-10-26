import {test,expect} from '../csrf.ts';
import {signInManually} from '../helpers.ts';

test('creates a contact group on a dedicated page and reloads its confirmed result @phone',async({page},info)=>{
 await signInManually(page,{username:'testuser',password:'testpass'});await page.goto('/#/contacts');
 await page.getByRole('button',{name:'New group',exact:true}).click();await expect(page.getByRole('heading',{name:'New group',exact:true})).toBeVisible();await expect(page.getByRole('searchbox',{name:'Search contacts'})).toHaveCount(0);
 const name=`Group ${info.project.name} ${Date.now()}`;await page.getByLabel('New group name',{exact:true}).fill(name);
 await page.getByRole('button',{name:'Cancel',exact:true}).click();await expect(page.getByRole('alertdialog')).toBeVisible();await page.getByRole('button',{name:'Keep editing',exact:true}).click();await expect(page.getByLabel('New group name',{exact:true})).toHaveValue(name);
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);await page.screenshot({path:info.outputPath('group-exclusive-editor.png'),fullPage:true});
 await page.getByRole('button',{name:'Create',exact:true}).click();await expect(page.getByRole('heading',{name:'New group',exact:true})).toHaveCount(0);await expect(page.getByRole('button',{name:new RegExp(`^${name}`)})).toBeVisible();await page.reload();await expect(page.getByRole('button',{name:new RegExp(`^${name}`)})).toBeVisible();
 await page.getByRole('button',{name:`Edit group ${name}`,exact:true}).click();await expect(page.getByRole('heading',{name:'Edit contact group'})).toBeVisible();await expect(page.getByRole('searchbox',{name:'Search contacts'})).toHaveCount(0);await page.getByLabel('Group name',{exact:true}).fill(name+' renamed');await page.getByRole('button',{name:'Save',exact:true}).click();await expect(page.getByRole('heading',{name:'Edit contact group'})).toHaveCount(0);await page.reload();await expect(page.getByRole('button',{name:new RegExp(`^${name} renamed`)})).toBeVisible();
});
