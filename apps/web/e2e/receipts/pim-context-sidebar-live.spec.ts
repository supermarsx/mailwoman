import {test,expect} from '../csrf.ts';
import {login} from '../helpers.ts';

test('sidebar context menus offer relevant actions and stay inside the viewport',async({page})=>{
 await login(page);
 for(const [label,id] of [['Calendar','calendar'],['Tasks','tasks'],['Notes','notes'],['Contacts','contacts'],['Keys','keys'],['Attachments','attachments'],['Events','events'],['Outbox','outbox']]){
  const trigger=['calendar','tasks','notes','contacts','keys'].includes(id!)?page.getByTestId(`nav-${id}`):id==='events'?page.getByTestId('sidebar-events'):page.getByRole('button',{name:label!,exact:true});
  await trigger.click({button:'right'});
  const menu=page.getByRole('menu');await expect(menu.getByRole('menuitem',{name:'Open',exact:true})).toBeVisible();
  await expect(menu.getByRole('menuitem',{name:'Refresh',exact:true})).toBeVisible();
  if(['calendar','tasks','notes','contacts','keys'].includes(id!))await expect(menu.getByRole('menuitem',{name:'New item',exact:true})).toBeVisible();
  if(id==='events')await expect(menu.getByRole('menuitemcheckbox',{name:'Upcoming only'})).toBeVisible();
  const box=await menu.boundingBox(),viewport=page.viewportSize();expect(box).toBeTruthy();expect(viewport).toBeTruthy();
  expect(box!.x).toBeGreaterThanOrEqual(0);expect(box!.y).toBeGreaterThanOrEqual(0);expect(box!.x+box!.width).toBeLessThanOrEqual(viewport!.width);expect(box!.y+box!.height).toBeLessThanOrEqual(viewport!.height);
  expect(await menu.getByRole('menuitem').first().locator('svg').count()).toBeGreaterThan(0);
  await page.keyboard.press('Escape');await expect(menu).toHaveCount(0);
 }
 const keys=page.getByTestId('nav-keys');await keys.focus();await page.keyboard.press('Shift+F10');await expect(page.getByRole('menu')).toBeVisible();await page.keyboard.press('Escape');await expect(keys).toBeFocused();
});
