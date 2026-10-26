import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {test,expect} from '../csrf.ts';
test('retains main views, reorders tabs and resumes an owned draft after reload',async({page},info)=>{
 const dist=path.resolve('../../.orchestration/reports/codex-workspace-tabs-web');
 await page.route('**/*',async route=>{const url=new URL(route.request().url());if(url.pathname!=='/'&&!url.pathname.startsWith('/assets/'))return route.continue();const file=path.resolve(dist,url.pathname==='/'?'index.html':url.pathname.slice(1));if(!file.startsWith(dist+path.sep))return route.abort();try{const body=await readFile(file);await route.fulfill({status:200,body,contentType:file.endsWith('.html')?'text/html':file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':'application/octet-stream'});}catch{await route.continue();}});
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 const navigate=async(name:string)=>{if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();await page.locator('#shell-nav').getByRole('button',{name,exact:true}).click();};
 for(const name of ['Calendar','Tasks','Notes','Contacts','Keys','Attachments','Outbox']){await navigate(name);await expect(page.getByRole('tab',{name,exact:true})).toBeVisible();}
 expect(await page.locator('.workspace-pane').count()).toBe(7);
 await page.getByRole('tab',{name:'Attachments',exact:true}).click();
 const file=page.locator('.mw-attach table tbody tr td:first-child button').first();await expect(file).toBeVisible();const filename=(await file.innerText()).trim();await file.click();await expect(page.getByRole('tab',{name:filename,exact:true})).toBeVisible();
 await page.getByRole('tab',{name:'Contacts',exact:true}).click();await expect(page.locator('[data-surface="contacts"]')).toBeVisible();
 await page.getByRole('tab',{name:'Contacts',exact:true}).press('Control+Shift+ArrowLeft');
 await navigate('Compose');const subject=`Workspace draft ${info.project.name}`;await page.getByLabel('Subject',{exact:true}).fill(subject);await expect(page.getByRole('tab',{name:subject,exact:true})).toBeVisible();
 await page.waitForTimeout(1000);
 const record=await page.evaluate(()=>JSON.parse(localStorage.getItem('mw.workspace.v2')??'null'));expect(record.tabs.length).toBe(10);expect(record.version).toBe(2);expect(record.storageAuthority).toMatch(/^msa1_[A-Za-z0-9_-]+$/);const me=await page.evaluate(async()=>{const response=await fetch('/api/me');return response.json() as Promise<{accountId:string;storageAuthority:string}>;});expect(record.accountId).toBe(me.accountId);expect(record.storageAuthority).toBe(me.storageAuthority);expect(JSON.stringify(record)).not.toContain(subject);
 await page.reload();await expect(page.getByRole('tab',{name:subject,exact:true})).toBeVisible();await expect(page.getByLabel('Subject',{exact:true})).toHaveValue(subject);
 await expect(page.getByRole('tab',{name:filename,exact:true})).toBeVisible();
 for(const name of ['Calendar','Tasks','Notes','Contacts','Keys','Attachments','Outbox'])await expect(page.getByRole('tab',{name,exact:true})).toBeVisible();
 await page.screenshot({path:info.outputPath('retained-workspace.png'),fullPage:true});
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
});
