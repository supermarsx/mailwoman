import {test,expect} from '../csrf.ts';
import {signInManually} from '../helpers.ts';

test('analytics configuration and account consent persist without enabling collection @phone',async({page},info)=>{
  await page.goto('/admin');
  await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  await expect(page.getByRole('button',{name:'Sign in',exact:true})).toHaveCount(0);
  const initial=await (await page.request.get('/admin/analytics')).json();
  let originalPreferences:unknown;
  try {
    await page.goto('/admin#/analytics');
    await expect(page.getByRole('heading',{name:'Usage analytics',exact:true})).toBeVisible();
    await page.getByRole('combobox',{name:'Provider',exact:true}).selectOption('matomo');
    await page.getByLabel(/^Collector endpoint/).fill('https://stats.example/matomo.php');
    await page.getByLabel(/^Website or site ID/).fill('7');
    await page.getByLabel('Enable optional analytics collection').uncheck();
    await page.getByRole('button',{name:'Save configuration',exact:true}).click();
    const confirmation=page.getByRole('dialog',{name:'Confirm it is you'});
    await confirmation.getByLabel('Password',{exact:true}).fill('hunter2');
    await confirmation.getByRole('button',{name:'Confirm and continue'}).click();
    await expect(page.getByText('Analytics configuration saved.',{exact:true})).toBeVisible();
    await page.reload();await expect(page.getByLabel(/^Website or site ID/)).toHaveValue('7');
    await expect(page.getByLabel('Enable optional analytics collection')).not.toBeChecked();
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
    await page.screenshot({path:info.outputPath('analytics-admin.png'),fullPage:true});
    await page.goto('/');await signInManually(page,{username:'testuser',password:'testpass'});
    await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
    originalPreferences=await (await page.request.get('/api/account/preferences')).json();
    const folders=page.getByRole('button',{name:'Show folders sidebar'});if(await folders.isVisible())await folders.click();
    await page.getByRole('button',{name:'Settings',exact:true}).click();
    await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Privacy',exact:true}).click();
    const consent=page.getByLabel('Allow optional screen usage counts for this account');
    await expect(consent).toBeEnabled();
    const next=!(await consent.isChecked());
    const saved=page.waitForResponse(response=>response.url().endsWith('/api/account/preferences')&&response.request().method()==='PUT'&&response.status()===200);
    await consent.setChecked(next);await saved;
    const stored=await(await page.request.get('/api/account/preferences')).json();expect(stored.preferences.analyticsConsent).toBe(next);
    const status=await(await page.request.get('/api/analytics/status')).json();expect(status.enabled).toBe(false);
    await page.screenshot({path:info.outputPath('analytics-consent.png'),fullPage:true});
  } finally {
    if(originalPreferences){const response=await page.request.get('/api/account/preferences');await page.request.put('/api/account/preferences',{headers:{'if-match':response.headers()['etag']!},data:originalPreferences});}
    const response=await page.request.get('/admin/analytics');
    await page.request.put('/admin/analytics',{headers:{'if-match':response.headers()['etag']!},data:initial.config});
  }
});
