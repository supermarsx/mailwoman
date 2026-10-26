import {expect,type Page} from '@playwright/test';

/** These suites share one source address and preserve the fixture's 3s cooldown. */
export async function adminLogin(page:Page):Promise<void>{
 await page.goto('/admin');
 await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');
 await page.waitForTimeout(3100);
 const login=page.waitForResponse(response=>response.url().endsWith('/admin/login')&&response.request().method()==='POST');
 await page.getByRole('button',{name:'Sign in',exact:true}).click();const result=await login;expect(result.status(),await result.text()).toBe(200);await expect(page.locator('#admin-main')).toBeVisible();
}
