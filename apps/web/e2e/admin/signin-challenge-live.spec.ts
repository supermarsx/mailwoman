import {test,expect} from '@playwright/test';

// Shared loopback fixture deliberately respects the production issuance bucket.
test.beforeEach(async()=>{await new Promise(resolve=>setTimeout(resolve,6500));});

test('self-hosted verification is solved locally, consumed and reset before another login @phone',async({page,baseURL},testInfo)=>{
  test.setTimeout(90_000);
  const origin=new URL(baseURL!).origin;
  const external:string[]=[];
  const workers:string[]=[];
  let logins=0;
  page.on('worker',worker=>workers.push(worker.url()));
  page.on('request',request=>{
    const url=request.url();
    if (/^https?:/.test(url)&&new URL(url).origin!==origin)external.push(new URL(url).origin);
    if(url.includes('pbkdf2')&&url.endsWith('.js'))workers.push(url);
    if(new URL(url).pathname==='/admin/login'&&request.method()==='POST')logins++;
  });
  await page.goto('/admin/');
  const form=page.getByRole('form',{name:'Admin sign in',exact:true});
  await expect(form).toBeVisible();
  const submit=form.getByRole('button',{name:'Sign in',exact:true});
  await expect(submit).toBeDisabled();
  await form.getByLabel('Admin username').fill('root');
  await form.getByLabel('Password',{exact:true}).fill('incorrect-fixture-password');
  const widget=form.locator('altcha-widget');
  await widget.getByRole('checkbox').focus();
  await page.keyboard.press('Space');
  await expect(submit).toBeEnabled({timeout:30_000});
  expect(logins).toBe(0);
  expect(await widget.evaluate(element=>(element as HTMLElement & {getConfiguration():{humanInteractionSignature:boolean}}).getConfiguration().humanInteractionSignature)).toBe(false);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
  await page.screenshot({path:testInfo.outputPath('verification-solved.png'),fullPage:true});
  const first=page.waitForResponse(response=>new URL(response.url()).pathname==='/admin/login'&&response.request().method()==='POST');
  await submit.click();const rejected=await first;expect(rejected.status()).toBe(401);
  const payload=JSON.parse(Buffer.from(rejected.request().postDataJSON().altcha as string,'base64').toString('utf8')) as object;
  expect(Object.keys(payload).sort()).toEqual(['challenge','solution']);
  await expect(submit).toBeDisabled();
  await expect(widget.getByRole('checkbox')).not.toBeChecked();
  await page.waitForTimeout(500);
  expect(logins).toBe(1);
  await form.getByLabel('Password',{exact:true}).fill('hunter2');
  if(testInfo.project.name==='phone')await widget.getByRole('checkbox').tap();else await widget.getByRole('checkbox').click();
  await expect(submit).toBeEnabled({timeout:30_000});
  const second=page.waitForResponse(response=>new URL(response.url()).pathname==='/admin/login'&&response.request().method()==='POST');
  await submit.click();expect((await second).status()).toBe(200);
  await expect(page.getByRole('navigation',{name:'Admin sections'})).toBeVisible();
  expect(workers.length).toBeGreaterThan(0);
  expect(external).toEqual([]);
  await page.screenshot({path:testInfo.outputPath('verified-admin-session.png'),fullPage:true});
});


test('mailbox sign-in solves the same locally served verification before credentials @phone',async({page},testInfo)=>{
  test.setTimeout(90_000);
  await page.goto('/');
  const form=page.getByRole('form',{name:'Sign in',exact:true});
  const submit=form.getByRole('button',{name:'Sign in',exact:true});
  await expect(submit).toBeDisabled();
  await form.getByLabel('Email or username').fill('testuser');
  await form.getByLabel('Password',{exact:true}).fill('testpass');
  if(testInfo.project.name==='phone')await form.locator('altcha-widget').getByRole('checkbox').tap();else await form.locator('altcha-widget').getByRole('checkbox').click();
  await expect(submit).toBeEnabled({timeout:30_000});
  await page.screenshot({path:testInfo.outputPath('mailbox-verification-solved.png'),fullPage:true});
  const signedIn=page.waitForResponse(response=>new URL(response.url()).pathname==='/api/login'&&response.request().method()==='POST');
  await submit.click();expect((await signedIn).status()).toBe(200);
  await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
});
