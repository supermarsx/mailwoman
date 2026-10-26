import {test,expect,type Page} from '../csrf.ts';
async function api(page:Page,method:string,data?:unknown,etag?:string){
  return page.evaluate(async({method,data,etag})=>{
    const token=document.cookie.split('; ').find(value=>value.startsWith('mw_csrf='))?.slice(8);
    const response=await fetch('/admin/telemetry',{method,headers:{...(data?{'content-type':'application/json'}:{}),...(token?{'x-csrf-token':decodeURIComponent(token)}:{}),...(etag?{'if-match':etag}:{})},body:data?JSON.stringify(data):null});
    return {status:response.status,etag:response.headers.get('etag'),data:await response.json()};
  },{method,data,etag});
}
test('dedicated telemetry editor persists bounded configuration and distinguishes active state @phone',async({page},info)=>{
  await page.waitForTimeout(3200);
  await page.goto('/admin');
  await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');
  const login=page.waitForResponse(response=>response.url().endsWith('/admin/login')&&response.request().method()==='POST');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();expect((await login).status()).toBe(200);
  const previous=await api(page,'GET');expect(previous.status).toBe(200);
  try {
    await page.goto('/admin#/observability/telemetry');
    await expect(page.getByRole('heading',{name:'Operational telemetry',exact:true})).toBeVisible();
    const environment=page.getByLabel(/^Environment/);
    await environment.fill('browser-acceptance');
    await page.getByRole('button',{name:'Save telemetry configuration',exact:true}).click();
    const confirmation=page.getByRole('dialog',{name:'Confirm it is you'});
    await confirmation.getByLabel('Password',{exact:true}).fill('hunter2');
    await confirmation.getByRole('button',{name:'Confirm and continue'}).click();
    await expect(page.getByText('Telemetry configuration saved. Restart the server to apply changes.',{exact:true})).toBeVisible();
    await page.reload();await expect(environment).toHaveValue('browser-acceptance');
    const saved=await api(page,'GET');expect(saved.data.config.environment).toBe('browser-acceptance');
    expect(saved.data.active.environment).toBe(previous.data.active.environment);
    expect(saved.data.restartRequired).toBe(true);
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
    await page.screenshot({path:info.outputPath('telemetry-editor.png'),fullPage:true});
  } finally {
    const current=await api(page,'GET');expect((await api(page,'PUT',previous.data.config,current.etag!)).status).toBe(200);
  }
});
