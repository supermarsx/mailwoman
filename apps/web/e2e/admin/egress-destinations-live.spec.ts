import {test,expect} from '../csrf.ts';
import { ADMIN } from './admin-path.ts';

test('private destination grants use a dedicated editor and remain class scoped @phone',async({page},info)=>{
  await page.goto(ADMIN);
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password',{exact:true}).fill('hunter2');
  const signedIn=page.waitForResponse(response=>response.url().endsWith(`${ADMIN}/login`)&&response.request().method()==='POST');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();expect((await signedIn).status()).toBe(200);
  const destination=`ews-${info.project.name.replaceAll(/[^a-z0-9]/gi,'-')}-${Date.now()}.corp.test:443`;
  try {
    await page.goto(`${ADMIN}#/egress/destinations/new`);
    await expect(page.getByLabel(/^Traffic class/)).toBeVisible();
    await expect(page.getByRole('table')).toHaveCount(0);
    await page.getByLabel(/^Traffic class/).selectOption('plugins');
    await page.getByLabel(/^Destination/).fill(destination);
    await page.getByLabel('Administrative note').fill('Browser acceptance fixture');
    await page.screenshot({path:info.outputPath('internal-destination-editor.png'),fullPage:true});
    await page.getByRole('button',{name:'Save',exact:true}).click();
    const proof=page.getByRole('dialog',{name:'Confirm it is you'});
    await proof.getByLabel('Password',{exact:true}).fill('hunter2');
    await proof.getByRole('button',{name:'Confirm and continue'}).click();
    await expect(page.getByRole('cell',{name:destination,exact:true})).toBeVisible();
    await page.reload();
    const row=page.getByRole('row').filter({has:page.getByRole('cell',{name:destination,exact:true})});
    await expect(row).toBeVisible();
    await expect(row).toContainText('Approved plugin and provider requests');
    const response=await page.request.get(`${ADMIN}/egress/destinations`);expect(response.ok()).toBe(true);
    const body=await response.json() as {destinations:Array<{id:string;trafficClass:string;destination:string}>};
    expect(body.destinations.filter(item=>item.destination===destination).map(item=>item.trafficClass)).toEqual(['plugins']);
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
    await row.getByRole('button',{name:'Revoke permission'}).click();
    await row.getByRole('button',{name:'Confirm revocation'}).click();
    await expect(page.getByRole('cell',{name:destination,exact:true})).toHaveCount(0);
  } finally {
    const response=await page.request.get(`${ADMIN}/egress/destinations`);
    if(response.ok()) {
      const body=await response.json() as {destinations:Array<{id:string;destination:string}>};
      for(const row of body.destinations.filter(item=>item.destination===destination)) {
        expect((await page.request.delete(`${ADMIN}/egress/destinations/${encodeURIComponent(row.id)}`)).ok()).toBe(true);
      }
    }
  }
});
