// t39 R6/L15: the Contacts membership, import and merge pages against a live
// server built from the tree, at 1440 and 390 wide in the chromium and firefox
// projects (the phone project is the 390 Chromium leg already, so it skips).
// Membership and merge are checked after a reload (engine-persisted); the
// pages must not scroll sideways, and at 390 the list pane sits above the
// detail pane. Screenshots go to MW_T39_SHOTS when set.
import {test,expect} from '@playwright/test';
import type {Page,TestInfo} from '@playwright/test';
import path from 'node:path';

const shot=(info:TestInfo,name:string)=>{const dir=process.env['MW_T39_SHOTS'];return dir?path.join(dir,`${info.project.name}-${name}.png`):info.outputPath(`${name}.png`);};
const contacts=(page:Page)=>page.locator('[data-module="contacts"]');
const tag=()=>Math.random().toString(36).slice(2,8);

async function signIn(page:Page):Promise<void>{
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('button',{name:'Compose',exact:true}).first()).toBeVisible({timeout:60000});
}
async function openContacts(page:Page):Promise<void>{
 const nav=page.getByTestId('nav-contacts');
 if(!await nav.isVisible()){const menu=page.locator('header.shell__bar button[aria-controls="shell-nav"]').first();if(await menu.isVisible()&&(await menu.getAttribute('aria-expanded'))!=='true')await menu.click();}
 await nav.click();
 await expect(contacts(page)).toBeVisible();
 await expect(contacts(page).getByRole('searchbox',{name:'Search contacts'})).toBeVisible();
}
async function createContact(page:Page,name:string,email:string):Promise<void>{
 await contacts(page).getByRole('button',{name:'New contact'}).click();
 const form=contacts(page).getByRole('form',{name:'New contact'});
 await form.getByLabel('Full name').fill(name);await form.getByLabel('Email 1',{exact:true}).fill(email);
 await form.getByRole('button',{name:'Save'}).click();
 await expect(contacts(page).getByRole('searchbox',{name:'Search contacts'})).toBeVisible();
}
async function noSideScroll(page:Page):Promise<void>{
 expect(await page.evaluate(()=>document.documentElement.scrollWidth-window.innerWidth)).toBeLessThanOrEqual(0);
 const host=contacts(page);
 expect(await host.evaluate(el=>el.scrollWidth-el.clientWidth)).toBeLessThanOrEqual(1);
}
async function stacked(page:Page,testId:string,narrow:boolean):Promise<void>{
 const pageEl=contacts(page).getByTestId(testId);
 const list=await pageEl.getByRole('navigation').first().boundingBox();
 const detail=await pageEl.locator('nav + div').first().boundingBox();
 expect(list&&detail).toBeTruthy();
 if(narrow)expect(detail!.y).toBeGreaterThanOrEqual(list!.y+list!.height-1);
 else expect(detail!.x).toBeGreaterThan(list!.x+list!.width-1);
}

for(const width of [1440,390]){
 test.describe(`at ${width}px`,()=>{
  const narrow=width<600;
  test.beforeEach(async({page},info)=>{
   test.skip(info.project.name==='phone','the phone project is the 390 Chromium leg; these specs size the viewport themselves');
   await page.setViewportSize({width,height:narrow?844:900});
  });

  test('membership page: a member added with the multi-select survives a reload',async({page},info)=>{
   test.setTimeout(180000);
   await signIn(page);await openContacts(page);
   const id=tag();const name=`Dorothy Vaughan ${id}`;const group=`Team ${id}`;
   await createContact(page,name,`dorothy${id}@example.org`);
   await contacts(page).getByRole('button',{name:'New group'}).click();
   await contacts(page).getByLabel('New group name').fill(group);
   await contacts(page).getByRole('button',{name:'Create'}).click();
   await contacts(page).getByRole('button',{name:'Group membership',exact:true}).click();
   const membership=contacts(page).getByTestId('contacts-membership-page');
   await expect(membership.getByRole('heading',{level:1,name:'Group membership'})).toBeFocused();
   await membership.getByRole('button',{name:new RegExp(group)}).click();
   const picker=membership.getByRole('combobox',{name:'Members'});
   await picker.click();await picker.pressSequentially(`dorothy${id}`);
   await expect(membership.getByRole('listbox')).toBeVisible();
   await expect(membership.getByRole('option',{name:new RegExp(name)})).toBeVisible();
   await page.keyboard.press('ArrowDown');await page.keyboard.press('ArrowUp');
   await expect(picker).toHaveAttribute('aria-activedescendant',/.+/);
   await page.keyboard.press('Enter');
   await expect(membership.getByRole('button',{name:`Remove ${name}`})).toBeVisible();
   await expect(picker).toBeEnabled();
   await page.keyboard.press('Escape');
   await noSideScroll(page);await stacked(page,'contacts-membership-page',narrow);
   await page.screenshot({path:shot(info,`r6-membership-${width}`)});
   await page.reload();
   await expect(page.getByRole('button',{name:'Compose',exact:true}).first()).toBeVisible({timeout:60000});
   await openContacts(page);
   await contacts(page).getByRole('button',{name:'Group membership',exact:true}).click();
   const after=contacts(page).getByTestId('contacts-membership-page');
   await after.getByRole('button',{name:new RegExp(group)}).click();
   await expect(after.getByRole('button',{name:`Remove ${name}`})).toBeVisible();
   await expect(after.getByText('1 member',{exact:true})).toBeVisible();
  });

  test('merge page: the chosen card is kept and the merge survives a reload',async({page},info)=>{
   test.setTimeout(180000);
   await signIn(page);await openContacts(page);
   const id=tag();const name=`Mary Jackson ${id}`;
   await createContact(page,name,`mary${id}@work.example.org`);
   await createContact(page,name,`mary${id}@home.example.org`);
   await contacts(page).getByRole('button',{name:'Find duplicates'}).click();
   const merge=contacts(page).getByTestId('contacts-merge-page');
   await merge.getByRole('navigation',{name:'Possible duplicates'}).getByRole('button',{name:new RegExp(name)}).click();
   const preview=merge.getByRole('region',{name:'Merged preview'});
   await expect(preview.getByText(`mary${id}@work.example.org`)).toBeVisible();
   await expect(preview.getByText(`mary${id}@home.example.org`)).toBeVisible();
   await expect(merge.getByRole('radio')).toHaveCount(2);
   await noSideScroll(page);await stacked(page,'contacts-merge-page',narrow);
   await page.screenshot({path:shot(info,`r6-merge-${width}`)});
   await merge.getByRole('button',{name:'Merge contacts'}).click();
   await expect(merge.getByRole('status')).toHaveText(`Merged into ${name}.`);
   await page.reload();
   await expect(page.getByRole('button',{name:'Compose',exact:true}).first()).toBeVisible({timeout:60000});
   await openContacts(page);
   await contacts(page).getByLabel('Search contacts').fill(name);
   await expect(contacts(page).getByRole('list',{name:'Contact list'}).getByText(name)).toHaveCount(1);
  });

  test('import page: rows can be left out before importing',async({page},info)=>{
   test.setTimeout(180000);
   await signIn(page);await openContacts(page);
   const id=tag();
   await contacts(page).getByRole('button',{name:'Import…'}).click();
   const imp=contacts(page).getByTestId('contacts-import-page');
   await expect(imp.getByRole('heading',{level:1,name:'Import contacts'})).toBeFocused();
   await imp.getByLabel('Paste vCard or CSV').fill(`Name,Email\nKeep ${id},keep${id}@example.org\nSkip ${id},skip${id}@example.org\n`);
   await imp.getByRole('button',{name:'Preview'}).click();
   await expect(imp.getByLabel('Map column Name')).toHaveValue('fullName');
   const rows=imp.getByRole('list',{name:'Import preview'}).getByRole('checkbox');
   await expect(rows).toHaveCount(2);
   await rows.nth(1).uncheck();
   await noSideScroll(page);await stacked(page,'contacts-import-page',narrow);
   await page.screenshot({path:shot(info,`r6-import-${width}`),fullPage:true});
   await imp.getByRole('button',{name:'Import 1 contact'}).click();
   await expect(imp.getByRole('status')).toHaveText(/Imported 1 contact/);
   await imp.getByRole('button',{name:'Done'}).click();
   await contacts(page).getByLabel('Search contacts').fill(id);
   const list=contacts(page).getByRole('list',{name:'Contact list'});
   await expect(list.getByText(`Keep ${id}`)).toBeVisible();
   await expect(list.getByText(`Skip ${id}`)).toHaveCount(0);
  });
 });
}
