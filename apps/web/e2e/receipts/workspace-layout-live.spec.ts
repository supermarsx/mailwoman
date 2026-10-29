import {test,expect} from '../csrf.ts';

test('five persisted workspace modes retain panes and fit the viewport',async({page},info)=>{
  await page.goto('/');
  await page.getByLabel('Email or username',{exact:true}).fill('testuser');
  await page.getByLabel('Password',{exact:true}).fill('testpass');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
  const endpoint='/api/account/preferences';
  const original=await (await page.request.get(endpoint)).json() as {version:number;preferences:Record<string,unknown>};
  const me=await (await page.request.get('/api/me')).json() as {storageAuthority:string};
  const group=page.getByRole('group',{name:'View arrangement'});
  const modes=[['tabs','Tabs'],['side-by-side','Side by side'],['stack','Stacked'],['masonry-side-by-side','Masonry side by side'],['masonry-stack','Masonry stacked']] as const;
  try{
    await expect(group.getByRole('button')).toHaveCount(5);
    for(const control of await group.getByRole('button').all()){
      await expect(control).toHaveText('');await expect(control.locator('svg')).toHaveCount(1);
      expect(await control.getAttribute('data-tooltip')).toBeTruthy();
    }
    if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
    const footer=page.locator('.sidebar__footer');
    await expect(footer.getByRole('button',{name:'Settings',exact:true})).toBeVisible();
    const positions=await footer.evaluate(element=>{const settings=element.querySelector('.sidebar__settings')!.getBoundingClientRect();const logout=element.querySelector('.sidebar__logout')!.getBoundingClientRect();const sidebar=element.closest('.sidebar')!.getBoundingClientRect();return {settingsBottom:settings.bottom,logoutTop:logout.top,footerBottom:element.getBoundingClientRect().bottom,sidebarBottom:sidebar.bottom};});
    expect(positions.settingsBottom).toBeLessThanOrEqual(positions.logoutTop+1);
    expect(positions.sidebarBottom-positions.footerBottom).toBeLessThan(32);
    if(info.project.name==='phone')await page.keyboard.press('Escape');
    for(const name of ['Contacts','Tasks','Notes','Settings']){
      if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
      await page.locator('#shell-nav').getByRole('button',{name,exact:true}).click();
      await expect(page.getByRole('tab',{name,exact:true})).toHaveAttribute('aria-selected','true');
    }
    const panes=page.locator('.workspace-panes > .workspace-pane');
    await expect(panes).toHaveCount(4);
    await panes.evaluateAll(elements=>elements.forEach((element,index)=>element.setAttribute('data-retained-proof',String(index))));
    for(const [mode,label] of modes){
      await group.getByRole('button',{name:label,exact:true}).click();
      await expect.poll(async()=>(await (await page.request.get(endpoint)).json()).preferences.workspaceLayout).toBe(mode);
      const actual=info.project.name==='phone'?'tabs':mode;
      await expect(page.locator('.workspace-panes')).toHaveAttribute('data-layout',actual);
      await expect(panes).toHaveCount(4);
      expect(await panes.evaluateAll(elements=>elements.every((element,index)=>element.getAttribute('data-retained-proof')===String(index)))).toBe(true);
      await expect(page.locator('.workspace-panes > .workspace-pane:visible')).toHaveCount(actual==='tabs'?1:actual.startsWith('masonry')?4:2);
      if(actual==='side-by-side'||actual==='stack'){
        const first=await page.locator('.workspace-pane[data-view-slot="0"]:visible').boundingBox();
        const second=await page.locator('.workspace-pane[data-view-slot="1"]:visible').boundingBox();
        expect(first).not.toBeNull();expect(second).not.toBeNull();
        if(actual==='side-by-side'){expect(Math.abs(first!.y-second!.y)).toBeLessThan(2);expect(second!.x).toBeGreaterThanOrEqual(first!.x+first!.width);}
        else{expect(Math.abs(first!.x-second!.x)).toBeLessThan(2);expect(second!.y).toBeGreaterThanOrEqual(first!.y+first!.height);}
        await expect(page.getByRole('separator',{name:'Resize workspace panes'})).toBeVisible();
      }
      if(actual.startsWith('masonry')){
        const boxes=await panes.evaluateAll(elements=>elements.map(element=>{const r=element.getBoundingClientRect();return {width:r.width,height:r.height};}));
        if(actual==='masonry-side-by-side')expect(boxes[0]!.height).toBeGreaterThan(boxes[1]!.height*1.5);
        else expect(boxes[0]!.width).toBeGreaterThan(boxes[1]!.width*1.5);
      }
      expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
      expect(await page.evaluate(()=>document.documentElement.scrollHeight<=innerHeight+1)).toBe(true);
    }
    await page.reload();
    await expect(group.getByRole('button',{name:'Masonry stacked',exact:true})).toHaveAttribute('aria-pressed','true');
    await expect(page.getByRole('tab',{name:'Settings',exact:true})).toBeVisible();
    await expect(panes).toHaveCount(4);
    if(info.project.name!=='phone'){
      await group.getByRole('button',{name:'Side by side',exact:true}).click();
      await page.getByRole('tab',{name:'Tasks',exact:true}).dragTo(page.locator('.workspace-pane[data-view-slot="0"]:visible'));
      await expect(page.locator('.workspace-pane[data-view-slot="0"]:visible')).toHaveAttribute('aria-label','Tasks');
      await page.reload();
      await expect(page.locator('.workspace-pane[data-view-slot="0"]:visible')).toHaveAttribute('aria-label','Tasks');
      const button=group.getByRole('button',{name:'Masonry stacked',exact:true});
      await button.hover();await expect(page.getByRole('tooltip')).toContainText('Tile all open views');await button.click();
      await expect.poll(async()=>(await (await page.request.get(endpoint)).json()).preferences.workspaceLayout).toBe('masonry-stack');
    }
    await page.screenshot({path:info.outputPath('five-workspace-layouts.png'),fullPage:true});
  }finally{
    const current=await page.request.get(endpoint);
    if(current.ok()){
      const document=await current.json() as {version:number;preferences:Record<string,unknown>};
      if(original.preferences['workspaceLayout']===undefined)delete document.preferences['workspaceLayout'];else document.preferences['workspaceLayout']=original.preferences['workspaceLayout'];
      const restored=await page.request.put(endpoint,{headers:{'if-match':current.headers()['etag']!,'x-mailwoman-storage-authority':me.storageAuthority},data:document});
      expect(restored.status()).toBe(200);
    }
  }
});
