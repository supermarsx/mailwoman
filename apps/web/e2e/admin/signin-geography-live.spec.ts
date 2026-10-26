import { test,expect } from '../csrf.ts';
test('sign-in country diagnostic reports real source and missing database honestly @phone',async({page})=>{
 // All projects share the fixture source IP; respect its configured 3s pacing.
 await new Promise(resolve=>setTimeout(resolve,3100));
 await page.goto('/admin');await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('button',{name:'Sign in',exact:true})).toHaveCount(0);
 const response=page.waitForResponse(r=>r.url().endsWith('/admin/bans/geography')&&r.request().method()==='GET');
 await page.goto('/admin#/observability/geography');
 const diagnostic=await response;expect(diagnostic.status()).toBe(200);expect(diagnostic.headers()['cache-control']).toBe('private, no-store');
 const body=await diagnostic.json();expect(body.scope).toBe('off');expect(body.database).toBe('unavailable');expect(body.sourceIp).toBe('127.0.0.1');
 await expect(page.getByRole('table',{name:'Sign-in country policy'}).getByText('127.0.0.1',{exact:true})).toBeVisible();
 await expect(page.getByText(/local country database is missing or unreadable/)).toBeVisible();
 await page.screenshot({path:test.info().outputPath('country-policy-diagnostic.png'),fullPage:true});
 await page.getByRole('button',{name:'Configure country policy'}).click();
 await expect(page).toHaveURL(/settings\/security\.signin\.geoScope/);
 await expect(page.locator('[data-field-anchor="settings:security.signin.geoScope"]')).toBeVisible();
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
});
