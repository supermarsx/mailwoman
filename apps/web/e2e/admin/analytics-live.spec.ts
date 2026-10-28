import {test,expect,type Page} from '../csrf.ts';
import {signInManually} from '../helpers.ts';

async function request(page:Page,path:string,method='GET',data?:unknown,etag?:string){
  return page.evaluate(async({path,method,data,etag})=>{
    const token=document.cookie.split('; ').find(value=>value.startsWith('mw_csrf='))?.slice(8);
    // Personal-settings writes repeat the session's storage authority (0a11b581), as the web client does.
    const authority=method!=='GET'&&path.startsWith('/api/account/')?(await (await fetch('/api/me')).json()).storageAuthority as string:undefined;
    const response=await fetch(path,{method,headers:{...(data===undefined?{}:{'content-type':'application/json'}),...(token?{'x-csrf-token':decodeURIComponent(token)}:{}),...(etag?{'if-match':etag}:{}),...(authority?{'x-mailwoman-storage-authority':authority}:{})},body:data===undefined?null:JSON.stringify(data)});
    const body=await response.text();return {ok:response.ok,status:response.status,etag:response.headers.get('etag'),body,json:body?JSON.parse(body):null};
  },{path,method,data,etag});
}

test('analytics configuration and account consent persist without enabling collection @phone',async({page},info)=>{
  await page.waitForTimeout(4000);
  await page.goto('/admin');
  await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');
  const signedIn=page.waitForResponse(response=>response.url().endsWith('/admin/login')&&response.request().method()==='POST');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  const login=await signedIn;expect(login.status(),await login.text()).toBe(200);
  await expect(page.getByRole('button',{name:'Sign in',exact:true})).toHaveCount(0);
  const initialResponse=await request(page,'/admin/analytics');
  expect(initialResponse.ok,initialResponse.body).toBe(true);
  const initial=initialResponse.json;
  const siteId=initial.config.siteId==='7'?'8':'7';
  let originalPreferences:unknown;
  try {
    await page.goto('/admin#/analytics');
    await expect(page.getByRole('heading',{name:'Usage analytics',exact:true})).toBeVisible();
    await page.getByRole('combobox',{name:'Provider',exact:true}).selectOption('matomo');
    await page.getByLabel(/^Collector endpoint/).fill('https://stats.example/matomo.php');
    await page.getByLabel(/^Website or site ID/).fill(siteId);
    await page.getByLabel('Enable optional analytics collection').uncheck();
    await page.getByRole('button',{name:'Save configuration',exact:true}).click();
    const confirmation=page.getByRole('dialog',{name:'Confirm it is you'});
    await confirmation.getByLabel('Password',{exact:true}).fill('hunter2');
    await confirmation.getByRole('button',{name:'Confirm and continue'}).click();
    await expect(page.getByText('Analytics configuration saved.',{exact:true})).toBeVisible();
    await page.reload();await expect(page.getByLabel(/^Website or site ID/)).toHaveValue(siteId);
    await expect(page.getByLabel('Enable optional analytics collection')).not.toBeChecked();
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
    await page.screenshot({path:info.outputPath('analytics-admin.png'),fullPage:true});
    // Separate deliberate logins respect the configured per-client cooldown.
    await page.waitForTimeout(4000);
    await page.goto('/');await signInManually(page,{username:'testuser',password:'testpass'});
    await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
    originalPreferences=(await request(page,'/api/account/preferences')).json;
    const folders=page.getByRole('button',{name:'Show folders sidebar'});if(await folders.isVisible())await folders.click();
    const drawer=page.getByRole('button',{name:'Open folders and apps'});if(await drawer.isVisible())await drawer.click();
    await page.getByRole('button',{name:'Settings',exact:true}).click();
    await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Privacy',exact:true}).click();
    const consent=page.getByLabel('Allow optional screen usage counts for this account');
    await expect(consent).toBeEnabled();
    const next=!(await consent.isChecked());
    const saved=page.waitForResponse(response=>response.url().endsWith('/api/account/preferences')&&response.request().method()==='PUT'&&response.status()===200);
    await consent.setChecked(next);await saved;
    const stored=(await request(page,'/api/account/preferences')).json;expect(stored.preferences.analyticsConsent).toBe(next);
    const status=(await request(page,'/api/analytics/status')).json;expect(status.enabled).toBe(false);
    await page.screenshot({path:info.outputPath('analytics-consent.png'),fullPage:true});
  } finally {
    if(originalPreferences){
      const response=await request(page,'/api/account/preferences');
      const restored=await request(page,'/api/account/preferences','PUT',originalPreferences,response.etag!);
      expect(restored.ok,restored.body).toBe(true);
    }
    const restoredSession=page.waitForResponse(response=>response.url().endsWith('/admin/session'));
    await page.goto('/admin#/analytics');
    await restoredSession;
    await expect(page.getByRole('heading',{name:'Usage analytics',exact:true})).toBeVisible();
    const proof=await request(page,'/admin/step-up','POST',{password:'hunter2'});
    expect(proof.ok,proof.body).toBe(true);
    const response=await request(page,'/admin/analytics');
    // The read model reports apiSecretConfigured; the write model omits it (AnalyticsWrite, 15976cce).
    const write={...initial.config};delete write.apiSecretConfigured;
    const restored=await request(page,'/admin/analytics','PUT',write,response.etag!);
    expect(restored.ok,restored.body).toBe(true);
  }
});
