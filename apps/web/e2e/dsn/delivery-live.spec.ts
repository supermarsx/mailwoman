import { test, expect } from '../csrf.ts';
import { messageRow } from '../helpers.ts';
test('received delivery report is shown as unverified delivery state',async({page},info)=>{
  const engine=info.project.name.startsWith('engine');
  await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill(engine?'testuser':'testuser@example.org');
  await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
  const row=messageRow(page,'Delivery failure fixture');await expect(row).toBeVisible({timeout:90000});
  const response=page.waitForResponse(r=>engine?r.request().postData()?.includes('mailwomanDsnReport')===true:r.url().endsWith('/api/mail/delivery-status'));
  await row.click();expect((await response).ok()).toBe(true);
  const report=page.getByRole('region',{name:'Delivery status report'});
  await expect(report).toBeVisible();await expect(report.getByText('Reported failure')).toBeVisible();await expect(report.getByText('4.4.7')).toBeVisible();
  await expect(report).toContainText('missing@example.org');await expect(report).toContainText('retries exhausted');await expect(report).toContainText('Delivery does not mean the message was read');
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await report.screenshot({path:info.outputPath('delivery-report.png'),animations:'disabled'});
});
