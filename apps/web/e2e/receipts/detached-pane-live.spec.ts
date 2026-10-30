import {test,expect} from '../csrf.ts';
import type {Page} from '@playwright/test';
async function detachTab(page:Page,title:string){await page.getByRole('tab',{name:title,exact:true}).press('Shift+F10');await page.getByRole('menuitem',{name:`Open ${title} in a new window`,exact:true}).click();}
import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {pinComposerRibbonOnPhone} from '../helpers.ts';

// 9dab0fe7 (D4): below 600 px the ribbon starts in auto-hide; these legs are about
// what its controls do, so the phone project pins it for the test (helpers.ts).
let unpinRibbon:()=>Promise<void>=async()=>{};
test.beforeEach(async({playwright,baseURL},info)=>{unpinRibbon=await pinComposerRibbonOnPhone(playwright.request,baseURL,info.project.name);});
test.afterEach(async()=>{await unpinRibbon();});
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
 const navigate=async(name:string)=>{await page.mouse.move(1,1);await expect(page.getByRole('tooltip')).toHaveCount(0);if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();await expect(page.getByRole('navigation',{name:'Mailboxes'})).toHaveAttribute('aria-busy','false');await page.locator('#shell-nav').getByRole('button',{name,exact:true}).click();};
 await navigate('Compose');const subject=`Popup draft ${info.project.name}`;await page.getByLabel('Subject',{exact:true}).fill(subject);
 const editor=page.locator('.ProseMirror').last();await editor.fill('Original draft');
 const promise=page.waitForEvent('popup');await detachTab(page,subject);const popup=await promise;
 await expect(popup.locator('.detached-workspace__header')).toHaveCSS('display','flex');
 await expect(popup.locator('.detached-workspace__content')).toHaveCSS('display','flex');
 const remote=popup.locator('.ProseMirror');await expect(remote).toHaveText('Original draft');await remote.click();await remote.press('Control+End');await remote.pressSequentially(' retained');await remote.press('Control+a');
 await popup.getByRole('button',{name:'Bold',exact:true}).click();await expect(remote.locator('strong')).toHaveText('Original draft retained');
 await popup.screenshot({path:info.outputPath('detached-live-composer.png'),fullPage:true});
 await page.getByRole('tab',{name:subject,exact:true}).press('Shift+F10');await page.getByRole('menuitem',{name:'Reattach to main window',exact:true}).click();await expect(editor).toHaveText('Original draft retained');await expect(editor.locator('strong')).toHaveText('Original draft retained');
 const again=page.waitForEvent('popup');await detachTab(page,subject);const second=await again;await expect(second.locator('.ProseMirror')).toHaveText('Original draft retained');await second.close();await expect(editor).toHaveText('Original draft retained');
 await page.screenshot({path:info.outputPath('reattached-live-composer.png'),fullPage:true});
});

test('returns detached actions to a visible dirty-editor confirmation',async({page},info)=>{
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 const navigate=async(name:string)=>{await page.mouse.move(1,1);await expect(page.getByRole('tooltip')).toHaveCount(0);if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();await expect(page.getByRole('navigation',{name:'Mailboxes'})).toHaveAttribute('aria-busy','false');await page.locator('#shell-nav').getByRole('button',{name,exact:true}).click();};
 await navigate('Contacts');await page.getByRole('button',{name:'New contact',exact:true}).click();await page.getByLabel('Full name',{exact:true}).fill('Unsaved detached contact');
 const pending=page.waitForEvent('popup');await detachTab(page,'Contacts');const popup=await pending;
 await expect(popup.getByLabel('Full name',{exact:true})).toHaveValue('Unsaved detached contact');await popup.getByRole('button',{name:'Cancel',exact:true}).click().catch(error=>{if(!popup.isClosed())throw error;});
 await expect(page.getByRole('alertdialog')).toBeVisible();await page.getByRole('button',{name:'Keep editing',exact:true}).click();await expect(page.getByLabel('Full name',{exact:true})).toHaveValue('Unsaved detached contact');
 await page.getByRole('button',{name:'Cancel',exact:true}).click();await page.getByRole('button',{name:'Discard changes',exact:true}).click();
});

test('closes detached content when the verified account logs out',async({page},info)=>{
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 const navigate=async(name:string)=>{await page.mouse.move(1,1);await expect(page.getByRole('tooltip')).toHaveCount(0);if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();await expect(page.getByRole('navigation',{name:'Mailboxes'})).toHaveAttribute('aria-busy','false');await page.locator('#shell-nav').getByRole('button',{name,exact:true}).click();};
 await navigate('Contacts');const pending=page.waitForEvent('popup');await detachTab(page,'Contacts');const popup=await pending;
 await expect(popup.getByRole('button',{name:'Reattach to main window',exact:true})).toBeVisible();
 if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();// Log out sits in the phone's drawer
 await page.getByRole('button',{name:'Log out',exact:true}).click();await expect(page.getByLabel('Email or username',{exact:true})).toBeVisible();await expect.poll(()=>popup.isClosed()).toBe(true);
});

test('keeps an unsaved composer in the main window when the popup receives a 404',async({page,context},info)=>{
 await context.route('**/workspace-popup.html',async route=>{const response=await route.fetch();await route.fulfill({response,status:404,contentType:'text/html',body:'<!doctype html><html><body><h1>Error 404</h1><p>This page does not exist.</p></body></html>'});});
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
 await page.locator('#shell-nav').getByRole('button',{name:'Compose',exact:true}).click();
 await page.getByLabel('Subject',{exact:true}).fill('Keep this draft');const editor=page.locator('.ProseMirror').last();await editor.fill('Unsaved body survives');
 const pending=page.waitForEvent('popup');await detachTab(page,'Keep this draft');const popup=await pending;
 await expect.poll(()=>popup.isClosed()).toBe(true);
 await expect(page.getByLabel('Subject',{exact:true})).toHaveValue('Keep this draft');await expect(editor).toHaveText('Unsaved body survives');
 await expect(page.getByText(/The detached window could not be loaded/)).toBeVisible();
});
