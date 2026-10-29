import path from 'node:path';
import { ADMIN } from './admin-path.ts';
import {test,expect} from '../csrf.ts';
test('global branding publishes local images across admin and mailbox sign-in @phone',async({page,context})=>{
 await page.goto(ADMIN);await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.locator('#admin-main')).toBeVisible();
 // This isolated fixture tests branding, not the independently tested login cooldown.
 await expect.poll(async()=>(await context.cookies()).some(cookie=>cookie.name==='mw_csrf')).toBe(true);
 const csrf=(await context.cookies()).find(cookie=>cookie.name==='mw_csrf')?.value;expect(csrf).toBeTruthy();
 const cooldown=await page.request.put(`${ADMIN}/settings/signin.retryCooldownSecs`,{headers:{'X-CSRF-Token':csrf!},data:{globalValue:0}});expect(cooldown.ok(),await cooldown.text()).toBe(true);
 const initial=await page.request.get(`${ADMIN}/branding`);const initialBody=await initial.json();expect((await page.request.put(`${ADMIN}/branding`,{headers:{'X-CSRF-Token':csrf!,'If-Match':initial.headers()['etag']!},data:{brandName:initialBody.brandName,loginNotice:initialBody.loginNotice,images:{logoId:null,iconMasterId:null,loginBackgroundId:null}}})).ok()).toBe(true);
 await page.goto(`${ADMIN}#/branding`);await page.getByLabel('Product name').fill('Example Team Mail');await page.getByLabel('Sign-in notice',{exact:true}).fill('Welcome to the team mailbox.');
 const uploads=page.getByLabel('Choose a local image');
 const icons=['apple-touch-icon.png','apple-touch-icon.png','apple-touch-icon.png','icons/icon-192.png','icons/icon-512.png','icons/icon-maskable-512.png'];
 for(let i=0;i<icons.length;i++){await uploads.nth(i).setInputFiles(path.resolve('public',icons[i]!));await expect(page.getByRole('button',{name:'Save branding'})).toBeEnabled({timeout:60000});}
 await page.getByRole('button',{name:'Save branding'}).click();await expect(page.getByText('Branding saved.',{exact:true})).toBeVisible();
 const manifestResponse=await page.request.get('/manifest.webmanifest');expect(manifestResponse.ok()).toBe(true);const manifest=await manifestResponse.json();expect(manifest.name).toBe('Example Team Mail');expect(manifest.icons).toHaveLength(3);for(const icon of manifest.icons){expect(icon.src).toMatch(/^api\/assets\/[a-f0-9]{64}$/);expect((await page.request.get('/'+icon.src)).ok()).toBe(true);}
 await expect.poll(()=>page.title()).toMatch(/Example Team Mail$/);
 await expect.poll(()=>page.locator('link[rel="icon"]').first().getAttribute('href')).toMatch(/\/api\/assets\/[a-f0-9]{64}$/);
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 await page.locator('#admin-main').evaluate(node=>node.scrollTo({top:0}));await page.screenshot({path:test.info().outputPath('branding-editor.png'),fullPage:true,animations:'disabled'});
 await context.clearCookies();await page.goto(ADMIN);await expect(page.getByRole('heading',{name:'Example Team Mail'})).toBeVisible();await expect.poll(()=>page.getByAltText('Example Team Mail').evaluate(node=>(node as HTMLImageElement).naturalWidth)).toBeGreaterThan(0);
 const background=page.locator('main > img[aria-hidden="true"]');await expect(background).toBeVisible();expect(await background.evaluate(node=>({fit:getComputedStyle(node).objectFit,loaded:(node as HTMLImageElement).naturalWidth>0}))).toEqual({fit:'cover',loaded:true});expect(await page.getByAltText('Example Team Mail').evaluate(node=>node.getBoundingClientRect().width)).toBeLessThanOrEqual(192);await page.screenshot({path:test.info().outputPath('branding-admin-login.png'),fullPage:true,animations:'disabled'});
 await page.goto('/');await expect(page.getByRole('heading',{name:'Example Team Mail'})).toBeVisible();await expect(page.getByText('Welcome to the team mailbox.',{exact:true})).toBeVisible();await expect.poll(()=>page.getByAltText('Example Team Mail').evaluate(node=>(node as HTMLImageElement).naturalWidth)).toBeGreaterThan(0);
 await page.screenshot({path:test.info().outputPath('branding-mailbox-login.png'),fullPage:true,animations:'disabled'});
 await page.getByLabel('Email or username').fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('button',{name:'Sign in',exact:true})).toHaveCount(0,{timeout:60000});await expect(page.locator('.sidebar__brand')).toContainText('Example Team Mail',{timeout:60000});
 await page.goto(ADMIN);await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.locator('#admin-main')).toBeVisible();await page.goto(`${ADMIN}#/branding`);
 await expect(page.getByRole('button',{name:'Restore default',exact:true})).toHaveCount(6);for(const button of await page.getByRole('button',{name:'Restore default',exact:true}).all())await button.click();await page.getByRole('button',{name:'Save branding'}).click();await expect(page.getByText('Branding saved.',{exact:true})).toBeVisible();await expect.poll(()=>page.locator('link[rel="icon"]').first().getAttribute('href')).not.toMatch(/\/api\/assets\//);
});
