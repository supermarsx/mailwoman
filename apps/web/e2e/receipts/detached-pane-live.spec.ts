import {test,expect} from '../csrf.ts';
// The actual fixture keeps the production three-second initial-login cooldown.
test.beforeEach(async()=>{await new Promise(resolve=>setTimeout(resolve,3100));});
test('detaches a live composer and preserves rich edits on reattach',async({page},info)=>{
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 const navigate=async(name:string)=>{if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();await page.locator('#shell-nav').getByRole('button',{name,exact:true}).click();};
 await navigate('Compose');const subject=`Popup draft ${info.project.name}`;await page.getByLabel('Subject',{exact:true}).fill(subject);
 const editor=page.locator('.ProseMirror').last();await editor.fill('Original draft');
 const promise=page.waitForEvent('popup');await page.getByRole('button',{name:`Open ${subject} in a new window`,exact:true}).click();const popup=await promise;
 await expect(popup.locator('.detached-workspace__header')).toHaveCSS('display','flex');
 await expect(popup.locator('.detached-workspace__content')).toHaveCSS('display','flex');
 const remote=popup.locator('.ProseMirror');await expect(remote).toHaveText('Original draft');await remote.click();await remote.press('Control+End');await remote.pressSequentially(' retained');await remote.press('Control+a');
 await popup.getByRole('button',{name:'Bold',exact:true}).click();await expect(remote.locator('strong')).toHaveText('Original draft retained');
 await popup.screenshot({path:info.outputPath('detached-live-composer.png'),fullPage:true});
 await popup.getByRole('button',{name:'Reattach to main window',exact:true}).click();await expect(editor).toHaveText('Original draft retained');await expect(editor.locator('strong')).toHaveText('Original draft retained');
 const again=page.waitForEvent('popup');await page.getByRole('button',{name:`Open ${subject} in a new window`,exact:true}).click();const second=await again;await expect(second.locator('.ProseMirror')).toHaveText('Original draft retained');await second.close();await expect(editor).toHaveText('Original draft retained');
 await page.screenshot({path:info.outputPath('reattached-live-composer.png'),fullPage:true});
});

test('returns detached actions to a visible dirty-editor confirmation',async({page},info)=>{
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 const navigate=async(name:string)=>{if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();await page.locator('#shell-nav').getByRole('button',{name,exact:true}).click();};
 await navigate('Contacts');await page.getByRole('button',{name:'New contact',exact:true}).click();await page.getByLabel('Full name',{exact:true}).fill('Unsaved detached contact');
 const pending=page.waitForEvent('popup');await page.getByRole('button',{name:'Open Contacts in a new window',exact:true}).click();const popup=await pending;
 await expect(popup.getByLabel('Full name',{exact:true})).toHaveValue('Unsaved detached contact');await popup.getByRole('button',{name:'Cancel',exact:true}).click().catch(error=>{if(!popup.isClosed())throw error;});
 await expect(page.getByRole('alertdialog')).toBeVisible();await page.getByRole('button',{name:'Keep editing',exact:true}).click();await expect(page.getByLabel('Full name',{exact:true})).toHaveValue('Unsaved detached contact');
 await page.getByRole('button',{name:'Cancel',exact:true}).click();await page.getByRole('button',{name:'Discard changes',exact:true}).click();
});

test('closes detached content when the verified account logs out',async({page},info)=>{
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 const navigate=async(name:string)=>{if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();await page.locator('#shell-nav').getByRole('button',{name,exact:true}).click();};
 await navigate('Contacts');const pending=page.waitForEvent('popup');await page.getByRole('button',{name:'Open Contacts in a new window',exact:true}).click();const popup=await pending;
 await expect(popup.getByRole('button',{name:'Reattach to main window',exact:true})).toBeVisible();
 await page.getByRole('button',{name:'Log out',exact:true}).click();await expect(page.getByLabel('Email or username',{exact:true})).toBeVisible();await expect.poll(()=>popup.isClosed()).toBe(true);
});
