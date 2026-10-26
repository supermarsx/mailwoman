import {test,expect} from '../csrf.ts';
import {readFile} from 'node:fs/promises';
import path from 'node:path';
// Optional current SPA overlay on the isolated mail fixture. API calls remain real;
// server shell routing is covered separately by t32_workspace_popup.
test.beforeEach(async({context})=>{
 const configured=process.env['MW_DETACHED_DIST'];if(!configured)return;
 const dist=path.resolve(configured);
 await context.route('**/*',async route=>{
  const pathname=new URL(route.request().url()).pathname;
  if(pathname!=='/'&&pathname!=='/workspace-popup.html'&&!pathname.startsWith('/assets/'))return route.continue();
  const file=path.resolve(dist,pathname==='/'?'index.html':pathname.slice(1));
  if(!file.startsWith(dist+path.sep))return route.abort();
  const body=await readFile(file);
  const response=await route.fetch();
  await route.fulfill({response,status:200,body,contentType:file.endsWith('.html')?'text/html':file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':'application/octet-stream'});
 });
});
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

test('keeps an unsaved composer in the main window when the popup receives a 404',async({page,context},info)=>{
 await context.route('**/workspace-popup.html',async route=>{const response=await route.fetch();await route.fulfill({response,status:404,contentType:'text/html',body:'<!doctype html><html><body><h1>Error 404</h1><p>This page does not exist.</p></body></html>'});});
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
 await page.locator('#shell-nav').getByRole('button',{name:'Compose',exact:true}).click();
 await page.getByLabel('Subject',{exact:true}).fill('Keep this draft');const editor=page.locator('.ProseMirror').last();await editor.fill('Unsaved body survives');
 const pending=page.waitForEvent('popup');await page.getByRole('button',{name:'Open Keep this draft in a new window',exact:true}).click();const popup=await pending;
 await expect.poll(()=>popup.isClosed()).toBe(true);
 await expect(page.getByLabel('Subject',{exact:true})).toHaveValue('Keep this draft');await expect(editor).toHaveText('Unsaved body survives');
 await expect(page.getByText(/The detached window could not be loaded/)).toBeVisible();
});
