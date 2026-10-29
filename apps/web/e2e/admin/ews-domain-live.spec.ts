import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {test,expect} from '../csrf.ts';
import {adminLogin} from './admin-login.ts';

test('EWS guided and dedicated editors persist actual HTTPS Basic configuration @phone',async({page},info)=>{
 const repo=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../../..');
 await adminLogin(page);const name=`ews-ui-${info.project.name}-${Date.now()}.example.test`;
 const ca=(await readFile(path.join(repo,'crates/mw-server/tests/fixtures/tls/cert1.pem'),'utf8')).replace(/\r\n/gu,'\n');
 try{
  await page.goto('/admin#/domains/new/guided');
  await page.getByLabel('Domain name',{exact:true}).fill(name);await page.getByRole('button',{name:'Next',exact:true}).click();
  await page.getByLabel('Connection',{exact:true}).selectOption('advanced');await page.getByRole('button',{name:'Add backend',exact:true}).click();
  await page.getByLabel(/^Backend ID/).fill('exchange');await page.getByLabel(/^Protocol and service/).selectOption('ews');
  await page.getByLabel(/^Exchange Web Services endpoint/).fill('https://exchange.example.test/EWS/Exchange.asmx');
  await page.getByLabel(/^Additional origin CA/).fill(ca);await page.getByLabel(/^Priority/).fill('7');
  await page.getByLabel('Preferred for this service',{exact:true}).check();
  await expect(page.getByLabel(/^EWS authentication/)).toBeDisabled();
  await expect(page.getByLabel('Hostname or IP address',{exact:true})).toHaveCount(0);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
  const button=await page.getByRole('button',{name:'Back to backend list',exact:true}).boundingBox();expect(button!.width).toBeLessThan(260);
  await page.screenshot({path:info.outputPath('ews-guided.png'),fullPage:true});
  await page.getByRole('button',{name:'Back to backend list',exact:true}).click();await page.getByRole('button',{name:'Next',exact:true}).click();
  await expect(page.getByText(/HTTPS Basic/)).toBeVisible();
  await page.getByRole('button',{name:'Create domain',exact:true}).click();await expect(page).toHaveURL(new RegExp(`/domains/${name}/connections$`));
  let saved=await page.request.get(`/admin/domains/${name}`);expect(saved.ok(),await saved.text()).toBe(true);
  let data=await saved.json();expect(data.upstream.backends).toEqual([{id:'exchange',protocol:'ews',role:'mail',enabled:true,preferred:true,priority:7,username:{format:'fullEmail'},ews:{endpointUrl:'https://exchange.example.test/EWS/Exchange.asmx',authentication:'basic',caPem:ca}}]);
  await page.getByRole('button',{name:'Edit exchange',exact:true}).click();
  await expect(page.getByLabel(/^Exchange Web Services endpoint/)).toHaveValue('https://exchange.example.test/EWS/Exchange.asmx');
  await expect(page.getByLabel(/^Additional origin CA/)).toHaveValue(ca);
  await page.getByLabel(/^Priority/).fill('3');await page.getByLabel(/^Authentication username format/).selectOption('domainUser');await page.getByLabel(/^Authentication domain/).fill('CORP');
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
  await page.screenshot({path:info.outputPath('ews-dedicated.png'),fullPage:true});
  await page.getByRole('button',{name:'Save backend',exact:true}).click();await expect(page).toHaveURL(new RegExp(`/domains/${name}$`));await page.goto(`/admin#/domains/${name}/connections`);await expect(page.getByRole('heading',{name:'exchange · EWS',exact:true})).toBeVisible();
  await page.reload();saved=await page.request.get(`/admin/domains/${name}`);data=await saved.json();expect(data.upstream.backends[0]).toMatchObject({priority:3,username:{format:'domainUser',domain:'CORP'},ews:{caPem:ca}});
  await page.goto(`/admin#/domains/${name}/diagnostics`);await expect(page.getByRole('note')).toContainText('Unauthenticated diagnostics cannot test EWS');
  // Actual server refusal proves this is not a client-only protocol label.
  const invalid={...data.upstream,backends:data.upstream.backends.map((row:Record<string,unknown>)=>({...row,ews:{endpointUrl:'http://exchange.example.test/EWS',authentication:'ntlm'}}))};
  const rejected=await page.request.put(`/admin/domains/${name}`,{headers:{'if-match':saved.headers()['etag']!},data:{upstream:invalid}});expect(rejected.status()).toBe(422);
 }finally{const read=await page.request.get(`/admin/domains/${name}`);if(read.ok())expect((await page.request.delete(`/admin/domains/${name}`,{headers:{'if-match':read.headers()['etag']!}})).ok()).toBe(true);}
});
