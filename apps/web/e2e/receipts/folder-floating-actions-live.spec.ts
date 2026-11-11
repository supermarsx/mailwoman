import {test,expect} from '../csrf.ts';
import {login} from '../helpers.ts';

test('folder bulk actions float within the sidebar and settings with reachable overflow',async({page})=>{
 page.setDefaultTimeout(15000);
 await page.route(/\/jmap(?:\/api)?$/,async route=>{
  const response=await route.fetch(),body=await response.json();
  for(const [method,args] of body.methodResponses??[])if(method==='Mailbox/get'&&args.list?.length){
   const sample=args.list[0];
   args.list.push(...Array.from({length:60},(_,index)=>({...sample,id:`floating-folder-${index}`,name:`Floating folder ${String(index).padStart(2,'0')}`,role:null,parentId:null,sortOrder:index+100,totalEmails:0,unreadEmails:0})));
  }
  await route.fulfill({response,json:body});
 });
 await login(page);
 const sidebar=page.locator('.sidebar');
 const folder=sidebar.locator('[data-folder-id="floating-folder-0"]').first();
 await folder.click({modifiers:['Control']});
 const bar=page.getByTestId('folder-selection-bar');await expect(bar).toBeVisible();
 const bounds=await bar.boundingBox(),pane=await sidebar.locator('.sidebar__scroll').boundingBox();
 expect(bounds!.y).toBeGreaterThan(pane!.y+pane!.height-75);
 expect(await bar.getByTestId('bulk-count').evaluate(element=>element.scrollWidth-element.clientWidth)).toBeLessThanOrEqual(1);
 expect(bounds!.x).toBeGreaterThanOrEqual(pane!.x);expect(bounds!.x+bounds!.width).toBeLessThanOrEqual(pane!.x+pane!.width+1);
 await bar.getByTestId('bulk-more').click();
 await expect(page.getByRole('menuitem',{name:'Select all',exact:true})).toBeVisible();
 await page.getByRole('menuitem',{name:/^(Pin|Unpin)$/}).click();await expect(bar).toBeVisible();
 await bar.getByTestId('bulk-more').click();await page.getByRole('menuitem',{name:/^(Pin|Unpin)$/}).click();
 await bar.getByTestId('bar-clear').click();await expect(bar).toHaveCount(0);
 await sidebar.getByRole('button',{name:'Settings',exact:true}).click();
 await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Folders and sidebar',exact:true}).click();
 const card=page.locator('#folders-card-list');await card.getByRole('button',{name:'Select all folders',exact:true}).click();
 const settingsBar=page.getByTestId('settings-folder-selection-bar');await expect(settingsBar).toBeVisible();
 const settingsBounds=await settingsBar.boundingBox(),viewport=page.viewportSize()!;
 expect(settingsBounds!.y).toBeGreaterThan(viewport.height-180);expect(settingsBounds!.y+settingsBounds!.height).toBeLessThan(viewport.height);
 await settingsBar.getByTestId('bulk-clear').click();await expect(settingsBar).toHaveCount(0);
});
