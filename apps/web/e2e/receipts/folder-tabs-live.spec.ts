import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {test,expect} from '../csrf.ts';
test('folder tabs retain independent selection and scroll without replacing the main folder',async({page,context},info)=>{
 page.setDefaultTimeout(15000);
 const dist=path.resolve('../../.orchestration/reports/codex-folder-tabs-web');
 await context.route('**/*',async route=>{
  const url=new URL(route.request().url());if(url.pathname!=='/'&&!url.pathname.startsWith('/assets/'))return route.continue();
  const file=path.resolve(dist,url.pathname==='/'?'index.html':url.pathname.slice(1));if(!file.startsWith(dist+path.sep))return route.abort();
  try{await route.fulfill({status:200,body:await readFile(file),contentType:file.endsWith('.html')?'text/html':file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':'application/octet-stream'});}catch{await route.continue();}
 });
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
 if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
 const settings=page.locator('.sidebar__footer').getByRole('button',{name:'Settings',exact:true});
 await expect(settings).toHaveText('');await expect(settings).toHaveAttribute('data-tooltip','Settings');
 const settingsBox=await settings.boundingBox(),logoutBox=await page.locator('.sidebar__footer').getByRole('button',{name:'Log out',exact:true}).boundingBox();
 expect(settingsBox).toBeTruthy();expect(logoutBox).toBeTruthy();expect(settingsBox!.x+settingsBox!.width).toBeLessThanOrEqual(logoutBox!.x+1);expect(Math.abs(settingsBox!.y-logoutBox!.y)).toBeLessThan(4);
 if(info.project.name==='phone')await page.keyboard.press('Escape');
 const initial=await page.locator('#shell-nav .sidebar__box--active').first().getAttribute('aria-label');
 async function openFolder(index:number){
  if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
  const buttons=page.locator('#shell-nav nav').first().locator('button.sidebar__box').filter({has:page.locator('.sidebar__box-name')});
  const folder=buttons.nth(index);const name=await folder.getAttribute('aria-label');expect(name).toBeTruthy();
  await folder.click({button:'right'});await page.getByRole('menuitem',{name:'Open in new tab',exact:true}).click();
  return name!;
 }
 const first=await openFolder(0);const firstTab=page.getByRole('tab',{name:first,exact:true});await expect(firstTab).toHaveAttribute('aria-selected','true');
 const pane=page.getByRole('tabpanel',{name:first,exact:true});await expect(pane.getByRole('checkbox').first()).toBeVisible();await pane.getByRole('checkbox').first().check();
 const scroll=await pane.getByRole('list').evaluate(list=>{const scroller=list.parentElement!;scroller.scrollTop=250;scroller.dispatchEvent(new Event('scroll'));return scroller.scrollTop;});
 expect(scroll).toBeGreaterThan(0);
 const second=await openFolder(1);expect(second).not.toBe(first);await expect(page.getByRole('tab',{name:second,exact:true})).toHaveAttribute('aria-selected','true');
 await firstTab.click();await expect(pane.getByRole('checkbox').first()).toBeChecked();
 expect(await pane.getByRole('list').evaluate(list=>list.parentElement!.scrollTop)).toBe(scroll);
 await page.getByRole('tab',{name:'Mail',exact:true}).click();expect(await page.locator('#shell-nav .sidebar__box--active').first().getAttribute('aria-label')).toBe(initial);
 await firstTab.click();await page.reload();await expect(page.getByRole('tab',{name:first,exact:true})).toBeVisible();await expect(page.getByRole('tab',{name:second,exact:true})).toBeVisible();
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
 await page.screenshot({path:info.outputPath('folder-tabs-retained.png'),fullPage:true});
});
