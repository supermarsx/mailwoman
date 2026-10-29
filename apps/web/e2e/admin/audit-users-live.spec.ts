import { test, expect } from '../csrf.ts';
import { ADMIN } from './admin-path.ts';

test('successful mailbox users and audit actors resolve to readable identity pages @phone', async ({ page }) => {
  await page.goto(ADMIN);
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('link',{name:'Customize dashboard'})).toBeVisible();
  const signed=await page.request.post('/api/login',{data:{username:'testuser',password:'testpass'}});
  expect(signed.status(),await signed.text()).toBe(200);
  const proof=await page.request.post('/api/security/step-up',{data:{password:'testpass'}});
  expect(proof.status(),await proof.text()).toBe(200);
  const users=await (await page.request.get(`${ADMIN}/users`)).json();
  const user=users.find((row:{username:string})=>row.username==='testuser');
  expect(user).toBeTruthy();expect(user.lastSigninAt).toBeGreaterThan(0);
  const audit=await (await page.request.get(`${ADMIN}/audit?limit=100`)).json();
  const event=audit.find((row:{actor:string;action:string})=>row.actor===user.accountId && row.action==='step-up-granted');
  expect(event.actorLabel).toBe('testuser');expect(event.actorLink).toEqual({section:'users',id:user.accountId});
  await page.goto(`${ADMIN}#/dashboard`);
  await page.reload(); // Manual-refresh dashboards must fetch the event written after first paint.
  const recent=page.getByRole('link',{name:'testuser',exact:true}).first();
  await expect(recent).toBeVisible();await recent.click();
  await expect(page.getByRole('heading',{name:'User identity',exact:true})).toBeVisible();
  await expect(page.getByRole('heading',{name:'testuser',exact:true})).toBeVisible();
  await expect(page.getByRole('form')).toHaveCount(0);
  await page.goto(`${ADMIN}#/observability`);
  await expect(page.getByRole('link',{name:'testuser',exact:true}).first()).toBeVisible();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.screenshot({path:test.info().outputPath('readable-audit.png'),fullPage:false});
});
