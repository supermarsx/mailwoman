import {test,expect} from '../csrf.ts';
import {signInManually} from '../helpers.ts';

test('stored attachment policy blocks renamed downloads and requires warning acknowledgement @phone',async({page})=>{
  await page.goto('/');await signInManually(page,{username:'testuser',password:'testpass'});
  await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible({timeout:60_000});
  const subject=process.env['ATTACHMENT_POLICY_SUBJECT'];expect(subject).toBeTruthy();
  await page.getByText(subject!,{exact:true}).first().click();
  await page.getByRole('option',{name:/policy-danger\.exe/}).click();
  await expect(page.getByRole('alert').filter({hasText:'administrator has blocked'})).toBeVisible();
  await page.screenshot({path:test.info().outputPath('blocked-attachment.png'),fullPage:true});
  // Return to the original reader tab, keeping the blocked attachment tab open.
  await page.getByRole('tab',{name:subject!}).click();
  await page.getByRole('option',{name:/policy-safe\.txt/}).click();
  await expect(page.getByRole('button',{name:'I understand — continue'})).toBeVisible();
  await page.getByRole('button',{name:'I understand — continue'}).click();
  await expect(page.getByRole('link',{name:'Download file'})).toBeVisible();
  await page.screenshot({path:test.info().outputPath('acknowledged-attachment.png'),fullPage:true});
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.goto('/admin');await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');
  const login=page.waitForResponse(response=>new URL(response.url()).pathname==='/admin/login'&&response.request().method()==='POST');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();expect((await login).status()).toBe(200);
  await page.goto('/admin#/attachments');
  await expect(page.getByRole('heading',{name:'Delivery and preview rules'})).toBeVisible();
  await expect(page.getByRole('heading',{name:'Blocked filename extensions'})).toBeVisible();
  await page.screenshot({path:test.info().outputPath('admin-attachment-policy.png'),fullPage:true});
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
});
