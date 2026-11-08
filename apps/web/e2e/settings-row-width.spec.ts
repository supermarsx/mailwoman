import {test,expect,type Page} from '@playwright/test';
import {CREDS,login,signInManually} from './helpers.ts';

test.use({serviceWorkers:'block'});
test.setTimeout(90_000);

async function section(page:Page,id:string):Promise<void>{
  const nav=page.getByRole('navigation',{name:'Settings sections'});
  const entry=nav.locator(`[data-settings-entry="${id}"]`);
  if(await entry.isVisible())await entry.click();
  else{
    const label=(await entry.textContent())!.trim();
    await nav.locator('[data-settings-picker]').click();
    await page.getByRole('menuitemradio',{name:label,exact:true}).click();
  }
  await expect(page.locator(`[data-settings-section="${id}"]`)).toBeVisible();
}

for(const width of [1920,2560,390])test(`settings rows remain bounded and usable at ${width}px`,async({page},info)=>{
  page.setDefaultTimeout(15000);
  await page.setViewportSize({width,height:width===390?844:1080});
  await page.addLocatorHandler(page.getByRole('dialog',{name:'Allow notifications?',exact:true}),async dialog=>{
    await dialog.getByRole('button',{name:'Not now',exact:true}).click();
  });
  if(width===390){
    await signInManually(page,CREDS);
    await page.getByRole('button',{name:'Open folders and apps',exact:true}).click();
  }
  else await login(page);
  await page.locator('#shell-nav').getByRole('button',{name:'Settings',exact:true}).click();
  const observations=[];
  for(const id of ['appearance','reader','compose']){
    await section(page,id);
    const panel=page.locator(`[data-settings-section="${id}"]`);
    await expect(panel.locator('[data-setting-row]').first()).toBeVisible();
    const geometry=await panel.evaluate(node=>{
      const rem=parseFloat(getComputedStyle(document.documentElement).fontSize);
      const rows=[...node.querySelectorAll<HTMLElement>('[data-setting-row]')].filter(e=>e.getBoundingClientRect().width>0).map(row=>{
        const box=row.getBoundingClientRect(),label=row.querySelector(':scope > label,:scope > span');
        const control=row.querySelector('input:not([type="hidden"]),select,textarea,button');
        const b=control?.getBoundingClientRect();
        const labelBox=label?.getBoundingClientRect();
        const column=control?[...row.children].find(child=>child.contains(control))?.getBoundingClientRect():undefined;
        return {width:box.width,control:b?{width:b.width,left:b.left-box.left,right:b.right-box.right}:null,label:label?.textContent?.slice(0,70),sameLineGap:labelBox&&column&&Math.abs(labelBox.top-column.top)<20?column.left-labelBox.right:null};
      });
      return {section:node.getAttribute('data-settings-section'),rem,viewport:innerWidth,pageWidth:document.documentElement.scrollWidth,panelWidth:node.getBoundingClientRect().width,rows};
    });
    expect(geometry.rows.length).toBeGreaterThan(2);
    for(const row of geometry.rows){
      expect(row.width,row.label).toBeLessThanOrEqual(52*geometry.rem+1);
      if(row.control&&row.control.width>0){
        expect(row.control.left,row.label).toBeGreaterThanOrEqual(-1);
        expect(row.control.right,row.label).toBeLessThanOrEqual(1);
      }
      if(row.sameLineGap!==null)expect(row.sameLineGap,row.label).toBeLessThanOrEqual(2*geometry.rem);
    }
    expect(geometry.pageWidth).toBeLessThanOrEqual(width+1);
    observations.push(geometry);
    if(id==='compose'){
      const control=panel.locator('#show-from');await control.scrollIntoViewIfNeeded();
      if(await control.isEnabled()){await control.focus();await expect(control).toBeFocused();}
      await page.screenshot({path:info.outputPath(`settings-compose-${width}.png`)});
      // Exercise translation/user-label pressure without changing any setting.
      const label=panel.locator('label[for="show-from"]');
      await label.evaluate(node=>{node.textContent='A long translated setting label explaining how the sender field behaves in every composed message '.repeat(8)+'UnbrokenLocalizationToken'.repeat(8);});
      const wrap=await label.evaluate(node=>({width:node.getBoundingClientRect().width,height:node.getBoundingClientRect().height,lineHeight:parseFloat(getComputedStyle(node).lineHeight),scrollWidth:node.scrollWidth,clientWidth:node.clientWidth}));
      expect(wrap.height).toBeGreaterThan(wrap.lineHeight*1.5);
      expect(wrap.scrollWidth).toBeLessThanOrEqual(wrap.clientWidth+1);
      await page.screenshot({path:info.outputPath(`settings-long-label-${width}.png`)});
    }else if(id==='reader')await page.screenshot({path:info.outputPath(`settings-reader-${width}.png`)});
  }
  // Signature name uses the legacy Settings field layout, outside SettingRow.
  await section(page,'signatures');
  await page.getByRole('button',{name:'New signature',exact:true}).click();
  const editor=page.getByTestId('signature-editor');await expect(editor).toBeVisible();
  const name=editor.getByRole('textbox',{name:'Name',exact:true});
  const legacy=await name.evaluate(input=>{
    const field=input.closest('label')!;return {width:field.getBoundingClientRect().width,rem:parseFloat(getComputedStyle(document.documentElement).fontSize),controlWidth:input.getBoundingClientRect().width,pageWidth:document.documentElement.scrollWidth};
  });
  expect(legacy.width).toBeLessThanOrEqual(52*legacy.rem+1);
  expect(legacy.controlWidth).toBeLessThanOrEqual(legacy.width+1);
  expect(legacy.pageWidth).toBeLessThanOrEqual(width+1);
  await name.fill('Width smoke');await expect(name).toHaveValue('Width smoke');
  observations.push({legacy});
  await info.attach('settings-row-geometry.json',{body:JSON.stringify(observations,null,2),contentType:'application/json'});
  console.log('SETTINGS_ROW_WIDTH',JSON.stringify({width,maxRows:observations.filter(x=>'rows' in x).map(x=>'rows' in x?{section:x.section,count:x.rows.length,max:Math.max(...x.rows.map(r=>r.width))}:null),legacy}));
});
