import path from 'node:path';
import {test,expect} from '../csrf.ts';
test('global branding publishes local images across admin and mailbox sign-in @phone',async({page,context})=>{
 await page.goto('/admin');await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.locator('#admin-main')).toBeVisible();
 // This isolated fixture tests branding, not the independently tested login cooldown.
 await expect.poll(async()=>(await context.cookies()).some(cookie=>cookie.name==='mw_csrf')).toBe(true);
 const csrf=(await context.cookies()).find(cookie=>cookie.name==='mw_csrf')?.value;expect(csrf).toBeTruthy();
 const cooldown=await page.request.put('/admin/settings/signin.retryCooldownSecs',{headers:{'X-CSRF-Token':csrf!},data:{globalValue:0}});expect(cooldown.ok(),await cooldown.text()).toBe(true);
 const initial=await page.request.get('/admin/branding');const initialBody=await initial.json();expect((await page.request.put('/admin/branding',{headers:{'X-CSRF-Token':csrf!,'If-Match':initial.headers()['etag']!},data:{brandName:initialBody.brandName,loginNotice:initialBody.loginNotice,images:{logoId:null,iconMasterId:null,loginBackgroundId:null}}})).ok()).toBe(true);
 await page.goto('/admin#/branding');await page.getByLabel('Product name').fill('Example Team Mail');await page.getByLabel('Sign-in notice',{exact:true}).fill('Welcome to the team mailbox.');
 const uploads=page.getByLabel('Choose a local image');
 for(let i=0;i<3;i++){await uploads.nth(i).setInputFiles(path.resolve('public/apple-touch-icon.png'));await expect(page.getByRole('button',{name:'Save branding'})).toBeEnabled({timeout:60000});}
 await page.getByRole('button',{name:'Save branding'}).click();await expect(page.getByText('Branding saved.',{exact:true})).toBeVisible();
 await expect.poll(()=>page.title()).toMatch(/Example Team Mail$/);
 await expect.poll(()=>page.locator('link[rel="icon"]').first().getAttribute('href')).toMatch(/\/api\/assets\/[a-f0-9]{64}$/);
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 await page.locator('#admin-main').evaluate(node=>node.scrollTo({top:0}));await page.screenshot({path:test.info().outputPath('branding-editor.png'),fullPage:true,animations:'disabled'});
 await context.clearCookies();await page.goto('/admin');await expect(page.getByRole('heading',{name:'Example Team Mail'})).toBeVisible();await expect.poll(()=>page.getByAltText('Example Team Mail').evaluate(node=>(node as HTMLImageElement).naturalWidth)).toBeGreaterThan(0);
 await page.goto('/');await expect(page.getByRole('heading',{name:'Example Team Mail'})).toBeVisible();await expect(page.getByText('Welcome to the team mailbox.',{exact:true})).toBeVisible();await expect.poll(()=>page.getByAltText('Example Team Mail').evaluate(node=>(node as HTMLImageElement).naturalWidth)).toBeGreaterThan(0);
 await page.screenshot({path:test.info().outputPath('branding-mailbox-login.png'),fullPage:true,animations:'disabled'});
 await page.getByLabel('Email').fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('button',{name:'Sign in',exact:true})).toHaveCount(0,{timeout:60000});await expect(page.locator('.sidebar__brand')).toContainText('Example Team Mail',{timeout:60000});
 await page.goto('/admin');await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.locator('#admin-main')).toBeVisible();await page.goto('/admin#/branding');
 await expect(page.getByRole('button',{name:'Restore default',exact:true})).toHaveCount(3);for(const button of await page.getByRole('button',{name:'Restore default',exact:true}).all())await button.click();await page.getByRole('button',{name:'Save branding'}).click();await expect(page.getByText('Branding saved.',{exact:true})).toBeVisible();await expect.poll(()=>page.locator('link[rel="icon"]').first().getAttribute('href')).not.toMatch(/\/api\/assets\//);
});
