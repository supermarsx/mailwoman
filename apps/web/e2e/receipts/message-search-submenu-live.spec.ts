import {test,expect} from '../csrf.ts';

test('message search opens a keyboard accessible second level with exact sender-domain search',async({page},info)=>{
 await page.goto('/');
 await page.getByLabel('Email or username',{exact:true}).fill('testuser');
 await page.getByLabel('Password',{exact:true}).fill('testpass');
 await page.getByRole('button',{name:'Sign in',exact:true}).click();
 const row=page.locator('.list__row').first();await expect(row).toBeVisible();
 await row.focus();await row.press('Shift+F10');
 const search=page.getByRole('menuitem',{name:'Search related messages',exact:true});
 await expect(page.getByRole('menuitem',{name:'Search this sender',exact:true})).toHaveCount(0);
 await search.focus();await search.press('ArrowRight');
 const submenu=page.getByRole('menu',{name:'Search related messages',exact:true});
 await expect(submenu).toBeVisible();
 await expect(submenu.getByRole('menuitem',{name:'Search this sender',exact:true})).toBeFocused();
 await expect(submenu.getByRole('menuitem',{name:'Search the same subject',exact:true})).toBeVisible();
 await expect(submenu.getByRole('menuitem',{name:'Search in this folder',exact:true})).toBeVisible();
 const domain=submenu.getByRole('menuitem',{name:'Search messages from this domain',exact:true});await expect(domain).toBeVisible();
 const size=page.viewportSize()!,bounds=await submenu.boundingBox();expect(bounds!.x).toBeGreaterThanOrEqual(0);expect(bounds!.x+bounds!.width).toBeLessThanOrEqual(size.width);
 await page.screenshot({path:info.outputPath('message-search-submenu.png')});
 await submenu.press('Escape');await expect(search).toBeFocused();await expect(submenu).toHaveCount(0);
 await search.click();
 const queried=page.waitForRequest(request=>{if(request.method()!=='POST')return false;try{return (request.postDataJSON()?.methodCalls??[]).some((call:unknown[])=>call[0]==='Email/query'&&String((call[1] as {filter?:{text?:string}}).filter?.text).includes('from-domain:'));}catch{return false;}});
 await page.getByRole('menuitem',{name:'Search messages from this domain',exact:true}).click();
 const request=await queried;const calls=request.postDataJSON().methodCalls as [string,{filter?:{text?:string}}][];
 expect(calls.find(call=>call[0]==='Email/query')?.[1].filter?.text).toMatch(/from-domain:"[a-z0-9.-]+"/u);
 await expect(page.getByRole('menu')).toHaveCount(0);
});
