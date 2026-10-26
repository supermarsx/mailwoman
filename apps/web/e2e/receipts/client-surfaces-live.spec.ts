import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {test,expect} from '../csrf.ts';
test('audits remaining client surfaces without modifying account data',async({page},info)=>{
 const dist=path.resolve('../../.orchestration/reports/codex-client-surfaces-web');
 await page.route('**/*',async route=>{const url=new URL(route.request().url());if(url.pathname!=='/'&&!url.pathname.startsWith('/assets/'))return route.continue();const file=path.resolve(dist,url.pathname==='/'?'index.html':url.pathname.slice(1));if(!file.startsWith(dist+path.sep))return route.abort();try{await route.fulfill({status:200,body:await readFile(file),contentType:file.endsWith('.html')?'text/html':file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':'application/octet-stream'});}catch{await route.continue();}});
 await new Promise(resolve=>setTimeout(resolve,3100));await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();

 const observations=[];await page.emulateMedia({reducedMotion:'reduce'});
 for(const name of ['Calendar','Tasks','Notes','Contacts','Keys','Attachments','Outbox']){
  if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
  const button=page.locator('#shell-nav').getByRole('button',{name,exact:true});await expect(button).toBeVisible();await button.click();
  const pane=page.getByRole('tabpanel',{name,exact:true});await expect(pane).toBeVisible();await page.evaluate(()=>new Promise<void>(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolve()))));await expect(pane).not.toContainText('Something went wrong');await expect(pane.getByTestId('async-pending')).toHaveCount(0);
  if(name==='Calendar'&&info.project.name==='phone'){const toggle=pane.getByRole('button',{name:'Calendars',exact:true});await expect(toggle).toHaveAttribute('aria-expanded','false');await expect(pane.getByRole('complementary',{name:'Calendars'})).not.toBeVisible();await toggle.click();await expect(pane.getByRole('complementary',{name:'Calendars'})).toBeVisible();await toggle.click();}
  await expect.poll(()=>page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
  if(info.project.name==='phone'&&name==='Notes')expect(await pane.locator('.notes__detail').evaluate(node=>node.getBoundingClientRect().width)).toBeGreaterThan(300);
  if(info.project.name==='phone'&&name==='Tasks')expect(await pane.getByPlaceholder('Add a task').evaluate(node=>node.getBoundingClientRect().width)).toBeGreaterThan(150);
  observations.push(await pane.evaluate((node,name)=>({name,width:node.getBoundingClientRect().width,viewport:innerWidth,scrollWidth:document.documentElement.scrollWidth,text:node.textContent?.slice(0,200)}),name));
  if(name==='Attachments'){const table=pane.getByRole('table');if(await table.count()){const region=table.locator('..');await region.focus();await expect(region).toBeFocused();await page.keyboard.press('ArrowRight');}}
  await page.screenshot({path:info.outputPath(`client-${name.toLowerCase()}.png`),fullPage:true});
 }
 console.log('CLIENT_SURFACE_AUDIT '+JSON.stringify(observations));
});
