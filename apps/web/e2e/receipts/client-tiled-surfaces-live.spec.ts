import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {test,expect} from '../csrf.ts';
test('keeps calendar tasks and notes usable in narrow retained panes',async({page},info)=>{
 const dist=path.resolve('../../.orchestration/reports/codex-client-surfaces-web');
 await page.route('**/*',async route=>{const url=new URL(route.request().url());if(url.pathname!=='/'&&!url.pathname.startsWith('/assets/'))return route.continue();const file=path.resolve(dist,url.pathname==='/'?'index.html':url.pathname.slice(1));if(!file.startsWith(dist+path.sep))return route.abort();try{await route.fulfill({status:200,body:await readFile(file),contentType:file.endsWith('.html')?'text/html':file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':'application/octet-stream'});}catch{await route.continue();}});
 await new Promise(resolve=>setTimeout(resolve,3100));await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();


 const endpoint='/api/account/preferences';const initial=await page.request.get(endpoint);const original=await initial.json() as {version:number;preferences:Record<string,unknown>};const me=await (await page.request.get('/api/me')).json() as {storageAuthority:string};
 const group=page.getByRole('group',{name:'View arrangement'});const observations=[];
 try{
  for(const name of ['Calendar','Tasks','Notes']){if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();await page.locator('#shell-nav').getByRole('button',{name,exact:true}).click();await expect(page.getByRole('tabpanel',{name,exact:true})).toBeVisible();}
  for(const label of ['Side by side','Masonry side by side','Masonry stacked']){
   await group.getByRole('button',{name:label,exact:true}).click();await expect.poll(async()=>(await (await page.request.get(endpoint)).json()).preferences.workspaceLayout).toBe(label==='Side by side'?'side-by-side':label==='Masonry side by side'?'masonry-side-by-side':'masonry-stack');
   for(const name of ['Calendar','Tasks','Notes']){const pane=page.getByRole('tabpanel',{name,exact:true});if(!await pane.isVisible())continue;const width=await pane.evaluate(node=>node.getBoundingClientRect().width);observations.push({label,name,width});if(width>640)continue;
    if(name==='Calendar'){await expect(pane.getByRole('button',{name:'Calendars',exact:true})).toBeVisible();expect(await pane.getByRole('main').last().evaluate(node=>node.getBoundingClientRect().width)).toBeGreaterThan(width*.85);}
    if(name==='Tasks')expect(await pane.getByPlaceholder('Add a task').evaluate(node=>node.getBoundingClientRect().width)).toBeGreaterThan(width*.5);
    if(name==='Notes')expect(await pane.locator('.notes__detail').evaluate(node=>node.getBoundingClientRect().width)).toBeGreaterThan(width*.85);
   }
   expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);await page.screenshot({path:info.outputPath(`pane-${label.replaceAll(' ','-')}.png`),fullPage:true});
  }
  await page.evaluate(()=>document.documentElement.dir='rtl');expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);await page.screenshot({path:info.outputPath('panes-rtl.png'),fullPage:true});
  await group.getByRole('button',{name:'Tabs',exact:true}).click();await expect.poll(async()=>(await (await page.request.get(endpoint)).json()).preferences.workspaceLayout).toBe('tabs');await page.getByRole('tab',{name:'Calendar',exact:true}).click();
  const calendar=page.getByRole('tabpanel',{name:'Calendar',exact:true});await calendar.getByRole('button',{name:'New event',exact:true}).click();const dialog=page.getByRole('dialog',{name:'New event',exact:true});await dialog.getByLabel('Title',{exact:true}).fill('Unsaved appearance audit — long meeting title');expect(await dialog.evaluate(node=>node.getBoundingClientRect().width)).toBeLessThanOrEqual((await page.viewportSize())!.width);await page.screenshot({path:info.outputPath('populated-unsaved-event-rtl.png'),fullPage:true});await dialog.getByRole('button',{name:'Save',exact:true}).scrollIntoViewIfNeeded();await expect(dialog.getByRole('button',{name:'Save',exact:true})).toBeInViewport();await expect(dialog.getByRole('button',{name:'Cancel',exact:true})).toBeInViewport();await page.keyboard.press('Escape');await expect(dialog).not.toBeVisible();
  if(info.project.name!=='phone'){await page.setViewportSize({width:640,height:360});await expect.poll(()=>page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);await page.screenshot({path:info.outputPath('half-size-layout-viewport.png'),fullPage:true});}
  console.log('TILED_CLIENT_AUDIT '+JSON.stringify(observations));
 }finally{const current=await page.request.get(endpoint);const document=await current.json() as {version:number;preferences:Record<string,unknown>};if(original.preferences['workspaceLayout']===undefined)delete document.preferences['workspaceLayout'];else document.preferences['workspaceLayout']=original.preferences['workspaceLayout'];expect((await page.request.put(endpoint,{headers:{'if-match':current.headers()['etag']!,'x-mailwoman-storage-authority':me.storageAuthority},data:document})).status()).toBe(200);}
});
