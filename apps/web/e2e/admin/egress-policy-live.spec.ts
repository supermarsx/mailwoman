import {test,expect} from '../csrf.ts';
import { ADMIN } from './admin-path.ts';

test('outbound route uses a dedicated editor and persists explicit traffic policy @phone',async({page},info)=>{
  await page.waitForTimeout(4000);
  await page.goto(ADMIN);
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password',{exact:true}).fill('hunter2');
  const signedIn=page.waitForResponse(response=>response.url().endsWith(`${ADMIN}/login`)&&response.request().method()==='POST');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();expect((await signedIn).status()).toBe(200);
  const id=`policy-${info.project.name}-${Date.now()}`;
  try {
    await page.goto(`${ADMIN}#/egress/new`);
    await expect(page.getByTestId('egress-id')).toBeVisible();
    await expect(page.getByRole('table')).toHaveCount(0);
    await page.getByTestId('egress-id').fill(id);
    await page.getByTestId('egress-host').fill('proxy.example');
    await page.getByTestId('egress-port').fill('3128');
    await page.getByLabel('Remote message images and files',{exact:true}).uncheck();
    await page.getByLabel('Calendar subscription feeds',{exact:true}).uncheck();
    await page.getByLabel('Nextcloud files and DAV requests',{exact:true}).check();
    await page.getByLabel('Route scope',{exact:false}).fill('global');
    await page.getByTestId('egress-save').click();
    const proof=page.getByRole('dialog',{name:'Confirm it is you'});
    await proof.getByLabel('Password',{exact:true}).fill('hunter2');
    await proof.getByRole('button',{name:'Confirm and continue'}).click();
    await expect(page.getByTestId(`egress-edit-${id}`)).toBeVisible();
    await page.getByTestId(`egress-edit-${id}`).click();
    await page.reload();
    await expect(page.getByLabel('Nextcloud files and DAV requests',{exact:true})).toBeChecked();
    await expect(page.getByLabel('Remote message images and files',{exact:true})).not.toBeChecked();
    await expect(page.getByRole('table')).toHaveCount(0);
    const response=await page.request.get(`${ADMIN}/egress/proxies`);expect(response.ok()).toBe(true);
    const body=await response.json() as {proxies:Array<{id:string;scope:string;trafficClasses:string[];active:boolean}>};
    expect(body.proxies.find(row=>row.id===id)).toMatchObject({scope:'global',trafficClasses:['nextcloud'],active:false});
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
    await page.screenshot({path:info.outputPath('outbound-policy-editor.png'),fullPage:true});
  } finally {
    const cleanup=await page.request.post(`${ADMIN}/egress/proxies/${encodeURIComponent(id)}/delete`);
    expect(cleanup.status()).toBe(200);
  }
});
