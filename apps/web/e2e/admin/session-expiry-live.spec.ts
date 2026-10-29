import { test,expect } from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
test('expired admin session returns to sign in and preserves the safe console route @phone',async({page})=>{
 await new Promise(resolve=>setTimeout(resolve,3100));
 const signIn=async():Promise<void>=>{await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect.poll(async()=>(await page.request.get(`${ADMIN}/session`)).status()).toBe(200);};
 await page.goto(ADMIN);await signIn();
 await page.goto(`${ADMIN}#/diagnostics/security`);
 await expect(page.getByRole('table',{name:'Deployment security assessment'})).toBeVisible();
 // Remove the real server session outside the SPA; its next protected request
 // must handle the same401 that idle/absolute expiry produces.
 expect((await page.request.post(`${ADMIN}/logout`)).ok()).toBe(true);
 await page.getByRole('button',{name:'Refresh assessment'}).click();
 await expect(page.getByRole('form',{name:'Admin sign in'})).toBeVisible();
 await expect(page.getByTestId('toast')).toContainText('Sign in again');
 await expect(page).toHaveURL(new RegExp(`${ADMIN}#/diagnostics/security$`));
 await page.reload();
 await expect(page.getByTestId('toast')).toBeVisible();
 await page.screenshot({path:test.info().outputPath('administrator-session-expired.png'),fullPage:true});
 await new Promise(resolve=>setTimeout(resolve,3100));await signIn();
 await expect(page.getByRole('table',{name:'Deployment security assessment'})).toBeVisible();
 await page.route(`**${ADMIN}/security/deployment`,route=>route.fulfill({status:403,contentType:'application/json',body:'{"code":"forbidden","detail":"Permission denied"}'}));
 await page.getByRole('button',{name:'Refresh assessment'}).click();
 await expect(page.getByRole('alert')).toContainText('Your role does not allow this action');
 await expect(page.getByRole('form',{name:'Admin sign in'})).toHaveCount(0);
 await page.unroute(`**${ADMIN}/security/deployment`);
 await page.getByRole('button',{name:'Refresh assessment'}).click();
 await expect(page.getByRole('table',{name:'Deployment security assessment'})).toBeVisible();
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
});
