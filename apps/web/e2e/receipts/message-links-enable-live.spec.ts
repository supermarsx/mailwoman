import {test,expect} from '../csrf.ts';
import {injectViaSmtp,messageRow} from '../helpers.ts';

test('message links require consent, open isolated destinations, and reset on another message',async({page,context,baseURL},info)=>{
 const token=`links-${info.project.name}-${Date.now()}`;
 const destination=new URL(`/consent-fixture/${token}`,baseURL!).href;
 let destinationHeaders:Record<string,string>|undefined;
 await context.route(destination,async route=>{destinationHeaders=route.request().headers();await route.fulfill({contentType:'text/html',body:'<!doctype html><title>Link destination</title><p>Controlled destination</p>'});});
 for(const suffix of ['first','second'])await injectViaSmtp({from:'consent@example.org',subject:`${token} ${suffix}`,text:'Link fixture',html:`<p><a href="${destination}">Controlled link</a></p>`});
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(messageRow(page,`${token} first`)).toBeVisible({timeout:60000});await messageRow(page,`${token} first`).click();
 const reader=page.locator('section.reader').filter({visible:true}).first();
 const outer=reader.frameLocator('iframe.reader__frame');
 await expect(outer.locator('body')).toBeVisible();
 const body=(await reader.locator('iframe.reader__frame').getAttribute('src'))?.includes('/api/rc/frame')?outer.frameLocator('iframe'):outer;
 await expect(body.getByText('Controlled link',{exact:true})).not.toHaveAttribute('href');
 await reader.getByRole('button',{name:'Enable links',exact:true}).click();
 await expect(body.getByRole('link',{name:'Controlled link',exact:true})).toHaveAttribute('target','_blank');
 const opened=context.waitForEvent('page');await body.getByRole('link',{name:'Controlled link',exact:true}).click();const popup=await opened;
 await expect(popup).toHaveTitle('Link destination');expect(await popup.evaluate(()=>window.opener)).toBeNull();expect(destinationHeaders?.['referer']).toBeUndefined();await popup.close();
 await reader.getByRole('button',{name:'Disable links',exact:true}).click();await expect(body.getByText('Controlled link',{exact:true})).not.toHaveAttribute('href');
 await reader.getByRole('button',{name:'Enable links',exact:true}).click();
 await page.screenshot({path:info.outputPath('message-links-enabled.png'),fullPage:true});
 // Reload is a new reader instance and must never persist this message consent.
 await page.reload();await expect(page.getByRole('button',{name:'Enable links',exact:true})).toBeVisible();
 await page.getByRole('tab',{name:'Mail',exact:true}).click();await expect(messageRow(page,`${token} second`)).toBeVisible();await messageRow(page,`${token} second`).click();
 await expect(page.getByRole('button',{name:'Enable links',exact:true})).toBeVisible();
});
