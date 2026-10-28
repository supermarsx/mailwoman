import { test, expect, type Page, type BrowserContext } from '../csrf.ts';
import { signInManually } from '../helpers.ts';

const cooldown = (): Promise<void> => new Promise(resolve => setTimeout(resolve, 3100));
async function loginAdmin(page: Page): Promise<void> {
  await cooldown(); await page.goto('/admin');
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect.poll(async () => (await page.request.get('/admin/session')).status()).toBe(200);
}
async function confirmBatch(page: Page): Promise<void> {
  await page.getByRole('alertdialog').getByRole('button', { name: 'Confirm selected revocations' }).click();
  const proof = page.getByRole('dialog', { name: 'Confirm it is you' });
  if (await proof.isVisible().catch(() => false)) {
    await proof.getByRole('textbox', { name: 'Password', exact: true }).fill('hunter2');
    await proof.getByRole('button', { name: 'Confirm and continue' }).click();
  }
}
test('bulk session selections revoke actual targets and preserve the current authority @phone', async ({ page, browser, baseURL }) => {
  test.setTimeout(180_000);
  const contexts: BrowserContext[] = [];
  const marker = `BulkProof-${Date.now()}`;
  const context = async (suffix: string): Promise<BrowserContext> => {
    const result = await browser.newContext({ ...(baseURL ? { baseURL } : {}), userAgent: `${marker}-${suffix}` });
    contexts.push(result); return result;
  };
  try {
    const targets: { page: Page; id: string }[] = [];
    for (const suffix of ['admin-one', 'admin-two']) {
      const target = await (await context(suffix)).newPage(); await loginAdmin(target);
      const listing = await (await target.request.get('/admin/admin-sessions')).json();
      targets.push({ page: target, id: listing.sessions.find((row: { current: boolean }) => row.current).id });
    }
    await loginAdmin(page); await page.goto('/admin#/admins');
    const admins = page.getByRole('table', { name: 'Administrator sessions' });
    for (const target of targets) await admins.locator(`[data-row-key="${target.id}"]`).getByRole('checkbox').check();
    await page.getByRole('button', { name: 'Revoke selected sessions (2)', exact: true }).click();
    await expect(page.getByRole('alertdialog')).toContainText('only the selected sessions');
    await page.getByRole('alertdialog').getByRole('button', { name: 'Cancel', exact: true }).click();
    for (const target of targets) expect((await target.page.request.get('/admin/session')).status()).toBe(200);
    await page.getByRole('button', { name: 'Revoke selected sessions (2)', exact: true }).click();
    // The existing mutation layer obtains fresh proof, then retries that initial
    // refused request; the batch itself never retries a completed mutation.
    const confirmation = confirmBatch(page);
    await expect.poll(async () => {
      const proof = page.getByRole('dialog', { name: 'Confirm it is you' });
      if (await proof.isVisible()) {
        await proof.getByRole('textbox', { name: 'Password', exact: true }).fill('hunter2');
        await proof.getByRole('button', { name: 'Confirm and continue' }).click();
      }
      return (await targets[0]!.page.request.get('/admin/session')).status();
    }).toBe(401); await confirmation;
    for (const target of targets) expect((await target.page.request.get('/admin/session')).status()).toBe(401);
    expect((await page.request.get('/admin/session')).status()).toBe(200);
    await expect(page.getByText(/Revoked:.*2.*Could not confirm:.*0/)).toBeVisible();
    await page.screenshot({ path: test.info().outputPath('bulk-administrator-sessions.png'), fullPage: true });

    const mailboxTargets: Page[] = [];
    for (const suffix of ['mail-one', 'mail-two']) {
      const target = await (await context(suffix)).newPage(); await cooldown();
      await target.goto('/'); await signInManually(target, { username: 'testuser', password: 'testpass' });
      await expect.poll(async () => (await target.request.get('/api/me')).status()).toBe(200);
      mailboxTargets.push(target);
    }
    await page.goto('/admin#/sessions');
    await page.getByRole('searchbox', { name: 'Search account, device or IP' }).fill(marker);
    await expect(page.getByRole('table', { name: 'Mailbox sessions' }).locator('tbody tr')).toHaveCount(2);
    await page.getByRole('checkbox', { name: 'Select all visible sessions', exact: true }).check();
    await page.getByRole('button', { name: 'Revoke selected sessions (2)', exact: true }).click();
    await confirmBatch(page);
    await expect(page.getByText(/Revoked:.*2.*Could not confirm:.*0/)).toBeVisible();
    for (const target of mailboxTargets) expect((await target.request.get('/api/me')).status()).toBe(401);
    expect((await page.request.get('/admin/session')).status()).toBe(200);
    await page.screenshot({ path: test.info().outputPath('bulk-mailbox-sessions.png'), fullPage: true });
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  } finally { for (const value of contexts) await value.close(); }
});
test('mailbox user selects other sessions without ending the current session @phone',async({page,browser,baseURL})=>{
 test.setTimeout(150_000);const marker=`UserBulk-${Date.now()}`;const contexts:BrowserContext[]=[];
 try{
  for(const suffix of ['one','two']){
   const context=await browser.newContext({...(baseURL?{baseURL}:{}),userAgent:`${marker}-${suffix}`});contexts.push(context);
   await cooldown();const target=await context.newPage();await target.goto('/');await signInManually(target,{username:'testuser',password:'testpass'});
   await expect.poll(async()=>(await target.request.get('/api/me')).status()).toBe(200);
  }
  await cooldown();await page.goto('/');await signInManually(page,{username:'testuser',password:'testpass'});
  await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
  const show=page.getByRole('button',{name:'Show folders sidebar'});if(await show.isVisible())await show.click();
  if(test.info().project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
  await page.getByRole('button',{name:'Settings',exact:true}).click();
  await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Security centre',exact:true}).click();
  // 2b1695c0 split the Security centre into tabs; the sessions table is on "Sessions & access".
  await page.getByRole('tablist',{name:'Security centre'}).getByRole('tab',{name:'Sessions & access',exact:true}).click();
  const section=page.getByRole('region',{name:'Active sessions',exact:true});
  await expect(section.getByText('This session',{exact:true})).toBeVisible();
  await expect(section.locator('tbody tr').filter({has:page.getByText('This session',{exact:true})}).getByRole('checkbox')).toBeDisabled();
  await section.getByRole('searchbox').fill(marker);
  await expect(section.locator('tbody tr')).toHaveCount(2);
  await section.getByRole('checkbox',{name:'Select all visible other sessions'}).check();
  await section.getByRole('button',{name:'Sign out selected sessions (2)',exact:true}).click();
  await expect(page.getByRole('alertdialog')).toContainText('This session stays signed in');
  await page.getByRole('alertdialog').getByRole('button',{name:'Sign out selected sessions',exact:true}).click();
  await expect(section.getByText(/Signed out:.*2.*Could not confirm:.*0/)).toBeVisible();
  for(const context of contexts)expect((await context.request.get('/api/me')).status()).toBe(401);
  expect((await page.request.get('/api/me')).status()).toBe(200);
  await page.screenshot({path:test.info().outputPath('bulk-account-sessions.png'),fullPage:true});
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 }finally{for(const context of contexts)await context.close();}
});
