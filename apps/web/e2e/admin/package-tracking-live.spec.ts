import {test,expect} from '../csrf.ts';
import {signInManually,drawerButton} from '../helpers.ts';

test('package provider and account opt-in use separate editors without automatic export @phone',async({page,browser},info)=>{
  await page.goto('/admin');await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');
  const signedIn=page.waitForResponse(response=>response.url().endsWith('/admin/login')&&response.request().method()==='POST');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();expect((await signedIn).status()).toBe(200);
  const original=await(await page.request.get('/admin/package-tracking')).json();
  let mailbox:Awaited<ReturnType<typeof browser.newContext>>|undefined;
  let account='';let previousEnabled=false;let id='';
  try {
    await page.goto('/admin#/package-tracking');
    const allow=page.getByLabel('Allow opted-in accounts to contact this provider');await expect(allow).toBeVisible();await allow.check();
    await page.getByLabel(/^Minimum refresh interval in seconds/).fill(String((original.provider?.refreshSecs??900)===901?902:901));
    if(!original.provider?.configured)await page.getByLabel(/^Provider API key/).fill('local-acceptance-not-a-real-provider-key');
    await page.getByRole('button',{name:'Save provider configuration'}).click();
    const proof=page.getByRole('dialog');await expect(proof).toBeVisible();await proof.getByLabel('Password',{exact:true}).fill('hunter2');await proof.getByRole('button',{name:'Confirm and continue'}).click();
    await expect(page.getByText(/Provider configuration saved/)).toBeVisible();
    await page.screenshot({path:info.outputPath('package-provider.png'),fullPage:true});
    await page.waitForTimeout(3500);
    mailbox=await browser.newContext({baseURL:info.project.use.baseURL??new URL(page.url()).origin,viewport:info.project.use.viewport??{width:1280,height:720},locale:'en-US',serviceWorkers:'block'});
    const user=await mailbox.newPage();await signInManually(user,{username:'testuser',password:'testpass'});await expect(user.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
    const initial=await(await user.request.get('/api/account/packages')).json();account=initial.accountId;previousEnabled=initial.state.enabled;
    if(info.project.name==='phone')await drawerButton(user).click();await user.locator('.sidebar__settings').click();await user.getByRole('button',{name:'Package tracking',exact:true}).click();
    await user.getByLabel('Enable package tracking for my account').check();await expect(user.getByRole('button',{name:'Add package'})).toBeEnabled();
    await user.getByRole('button',{name:'Add package'}).click();await expect(user.getByRole('textbox',{name:'Tracking number',exact:true})).toBeVisible();await expect(user.getByRole('table')).toHaveCount(0);
    const number=String(Date.now());await user.getByRole('textbox',{name:'Label',exact:true}).fill('Local delivery test');await user.getByLabel('Tracking number',{exact:true}).fill(number);await user.getByLabel('Carrier code',{exact:true}).fill('ups');await user.getByRole('button',{name:'Save package'}).click();
    await expect(user.getByRole('table')).toBeVisible();await expect(user.getByRole('textbox',{name:'Tracking number',exact:true})).toHaveCount(0);
    const saved=await(await user.request.get('/api/account/packages')).json();const added=saved.state.packages.find((row:{trackingNumber:string})=>row.trackingNumber===number);expect(added).toBeTruthy();expect(added.attemptedAt).toBeNull();id=added.id;
    await user.reload();await expect(user.getByRole('button',{name:'Compose',exact:true})).toBeVisible();if(info.project.name==='phone')await drawerButton(user).click();await user.locator('.sidebar__settings').click();await user.getByRole('button',{name:'Package tracking',exact:true}).click();await expect(user.getByRole('cell').filter({hasText:number})).toBeVisible();
    expect(await user.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);expect(await user.getByRole('table').evaluate(table=>table.getBoundingClientRect().width)).toBeGreaterThanOrEqual(600);await user.screenshot({path:info.outputPath('account-packages.png'),fullPage:true});
  }finally{
    if(mailbox&&account){const user=mailbox.pages()[0]!;const csrf=(await mailbox.cookies()).find(cookie=>cookie.name==='mw_csrf')?.value;let current=await(await user.request.get('/api/account/packages')).json();if(id){const removed=await user.request.delete(`/api/account/packages/${encodeURIComponent(id)}`,{headers:{'X-CSRF-Token':csrf??''},data:{accountId:account,revision:current.revision}});expect(removed.ok(),await removed.text()).toBe(true);current=await(await user.request.get('/api/account/packages')).json();}const restored=await user.request.put('/api/account/packages',{headers:{'X-CSRF-Token':csrf??''},data:{accountId:account,revision:current.revision,enabled:previousEnabled}});expect(restored.ok(),await restored.text()).toBe(true);await mailbox.close();}
    const current=await page.request.get('/admin/package-tracking');const restored=await page.request.put('/admin/package-tracking',{headers:{'If-Match':current.headers()['etag']!},data:{enabled:original.provider?.enabled??false,refreshSecs:original.provider?.refreshSecs??900}});expect(restored.ok(),await restored.text()).toBe(true);
  }
});
