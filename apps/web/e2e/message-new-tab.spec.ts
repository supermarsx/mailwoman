import {test,expect} from '@playwright/test';
import {login,messageRow} from './helpers.ts';

test('context menu opens a retained reusable message tab and preserves the reading pane',async({page},info)=>{
  await page.setViewportSize({width:1600,height:1000});
  await page.route(/\/api\/settings(?:\?.*)?$/,async route=>{
    const response=await route.fetch();const body=await response.json();
    body.settings['notifications.askPermission']={value:false,source:'user',locked:false};
    await route.fulfill({response,json:body});
  });
  let prefs:Record<string,unknown>|undefined;
  await page.route(/\/api\/account\/preferences(?:\?.*)?$/,async route=>{
    if(!prefs){const response=await route.fetch();prefs=await response.json() as Record<string,unknown>;}
    if(route.request().method()==='PUT')prefs!['preferences']=(route.request().postDataJSON() as {preferences:unknown}).preferences;
    const preferences={...prefs!['preferences'] as Record<string,unknown>,readingPane:'right',readerCollapsed:false,messageListCollapsed:false,workspaceLayout:'tabs',markRead:'never'};
    await route.fulfill({status:200,headers:{'content-type':'application/json',etag:'"new-tab-browser-only"'},json:{...prefs,preferences}});
  });
  await login(page);
  const first=messageRow(page,'Lunch tomorrow?');await expect(first).toBeVisible();
  const other=page.locator('.list__row').filter({hasNotText:'Lunch tomorrow?'}).first();await expect(other).toBeVisible();
  const otherSubject=(await other.locator('.list__subject').innerText()).trim();
  await first.click({button:'right',position:{x:10,y:10}});
  await page.getByRole('menuitem',{name:'Open message',exact:true}).click();
  const pane=page.getByTestId('reading-pane');
  await expect(pane.locator('.reader__subject')).toHaveText('Lunch tomorrow?');
  const retained=await pane.locator('.reader__subject').elementHandle();
  await expect(page.getByRole('tab',{name:'Mail',exact:true})).toHaveAttribute('aria-selected','true');
  await other.click({button:'right',position:{x:10,y:10}});
  const newTab=page.getByRole('menuitem',{name:'Open in a new tab',exact:true});await expect(newTab).toBeVisible();
  await page.screenshot({path:info.outputPath('message-new-tab-menu.png')});await newTab.click();
  const tab=page.getByRole('tab',{name:otherSubject,exact:true});await expect(tab).toHaveAttribute('aria-selected','true');
  await expect(page.getByRole('tabpanel',{name:otherSubject,exact:true}).locator('.reader__subject')).toHaveText(otherSubject);
  await page.getByRole('tab',{name:'Mail',exact:true}).click();
  await expect(pane.locator('.reader__subject')).toHaveText('Lunch tomorrow?');
  expect(await retained!.evaluate(node=>node.isConnected)).toBe(true);
  expect(await pane.locator('.reader__subject').evaluate((node,original)=>node===original,retained)).toBe(true);
  await other.click({button:'right',position:{x:10,y:10}});
  await page.getByRole('menuitem',{name:'Open in a new tab',exact:true}).click();
  await expect(tab).toHaveAttribute('aria-selected','true');await expect(tab).toHaveCount(1);
  await page.screenshot({path:info.outputPath('message-new-tab-retained.png')});
});
