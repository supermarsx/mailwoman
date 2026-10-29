import {test,expect} from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import {adminLogin} from './admin-login.ts';

test('inbound policy editor confirms conditional disabled-only changes @phone',async({page},info)=>{

 let revision='"one"';let config={admin:{enabled:false,cidrs:[],hostnames:[]},webmail:{enabled:false,cidrs:[],hostnames:[]},dns:{servers:[],timeoutMs:2000,cacheSeconds:300,maxHosts:64,maxAddresses:16,concurrency:4}};let saves=0;
 await page.route(`**${ADMIN}/security/inbound-allowlist`,async route=>{if(route.request().method()==='PUT'){const next=route.request().postDataJSON();expect(next.admin.enabled).toBe(false);expect(next.webmail.enabled).toBe(false);expect(route.request().headers()['if-match']).toBe(revision);config=next;revision='"two"';saves++;}await route.fulfill({headers:{etag:revision},json:{config,status:{hosts:[],refreshing:false}}});});
 await adminLogin(page);await page.goto(`${ADMIN}#/inbound-allowlist`);await expect(page.getByRole('heading',{name:'Inbound access',exact:true})).toBeVisible();
 const timeout=page.getByLabel('Lookup timeout (milliseconds)',{exact:true});await timeout.fill('1500');await page.getByRole('button',{name:'Refresh saved policy and DNS status'}).click();await expect(timeout).toHaveValue('1500');
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);const review=page.getByRole('button',{name:'Review changes'});expect((await review.boundingBox())!.width).toBeLessThan(250);await page.screenshot({path:info.outputPath('inbound-form.png'),fullPage:true});
 await review.click();const dialog=page.getByRole('alertdialog');await expect(dialog).toContainText('no automatic loopback');const apply=dialog.getByRole('button',{name:'Apply confirmed policy'});await expect(apply).toBeDisabled();expect(saves).toBe(0);await dialog.getByRole('textbox').fill('APPLY');await page.screenshot({path:info.outputPath('inbound-confirm.png'),fullPage:true});await apply.click();await expect(page.getByText('The server accepted the inbound access policy.',{exact:true})).toBeVisible();expect(saves).toBe(1);await page.reload();await expect(timeout).toHaveValue('1500');await expect(page.getByRole('checkbox',{name:'Restrict this surface to the allowlist'}).first()).not.toBeChecked();await expect(page.getByRole('checkbox',{name:'Restrict this surface to the allowlist'}).last()).not.toBeChecked();
});

// t39 N7: the same editor against the real server, no route mocks. A disabled
// policy with a rescue CIDR, a hostname and a DNS bound is applied through the
// typed confirmation, stored with a new revision, shown after a reload, and the
// original policy is put back.
test('inbound policy saves to the live server and survives a reload @phone',async({page},info)=>{
 await adminLogin(page);
 const before=await page.request.get(`${ADMIN}/security/inbound-allowlist`);expect(before.status(),await before.text()).toBe(200);
 const original=(await before.json()) as {config:Record<string,unknown>};
 test.skip(((original.config['admin'] as {enabled:boolean}).enabled||(original.config['webmail'] as {enabled:boolean}).enabled),'an enabled policy is in place on this stack; not touching it');
 const host=`rescue-${info.project.name}.example.org`;
 try{
  await page.goto(`${ADMIN}#/inbound-allowlist`);await expect(page.getByRole('heading',{name:'Inbound access',exact:true})).toBeVisible();
  const webmail=page.getByRole('region',{name:'Webmail and account APIs',exact:true});
  await webmail.getByLabel('Rescue IPs and CIDRs').fill('192.0.2.0/24');
  await webmail.getByLabel('Allowed DNS hostnames').fill(host);
  const timeout=page.getByLabel('Lookup timeout (milliseconds)',{exact:true});await timeout.fill('1750');
  await page.getByRole('button',{name:'Review changes'}).click();
  const dialog=page.getByRole('alertdialog');const apply=dialog.getByRole('button',{name:'Apply confirmed policy'});await expect(apply).toBeDisabled();
  await dialog.getByRole('textbox').fill('APPLY');
  // The first PUT may answer 403 step_up_required; the client asks for the password and repeats it.
  const saved=page.waitForResponse(r=>r.url().endsWith(`${ADMIN}/security/inbound-allowlist`)&&r.request().method()==='PUT'&&r.status()!==403);
  await apply.click();
  const step=page.getByRole('dialog',{name:'Confirm it is you'});
  if(await step.waitFor({state:'visible',timeout:5000}).then(()=>true,()=>false)){await step.getByLabel('Password',{exact:true}).fill('hunter2');await step.getByRole('button',{name:'Confirm and continue'}).click();}
  const reply=await saved;expect(reply.status(),await reply.text()).toBe(200);
  expect(reply.headers()['etag']).not.toBe(before.headers()['etag']);
  await expect(page.getByText('The server accepted the inbound access policy.',{exact:true})).toBeVisible();
  const stored=(await (await page.request.get(`${ADMIN}/security/inbound-allowlist`)).json()) as {config:{webmail:{enabled:boolean;cidrs:string[];hostnames:string[]};admin:{enabled:boolean};dns:{timeoutMs:number}}};
  expect(stored.config.webmail).toMatchObject({enabled:false,cidrs:['192.0.2.0/24'],hostnames:[host]});
  expect(stored.config.admin.enabled).toBe(false);expect(stored.config.dns.timeoutMs).toBe(1750);
  await page.reload();
  await expect(timeout).toHaveValue('1750');
  await expect(webmail.getByLabel('Allowed DNS hostnames')).toHaveValue(host);
  await expect(webmail.getByLabel('Rescue IPs and CIDRs')).toHaveValue('192.0.2.0/24');
  // A disabled policy grants nothing, so the DNS status lists no hostname.
  await expect(page.getByText('No hostnames are active in enabled policies.',{exact:true})).toBeVisible();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
  await page.screenshot({path:info.outputPath('inbound-live.png'),fullPage:true});
 }finally{
  // Restoring is a write like any other: it needs a fresh step-up proof.
  expect((await page.request.post(`${ADMIN}/step-up`,{data:{password:'hunter2'}})).status()).toBeLessThan(300);
  const current=await page.request.get(`${ADMIN}/security/inbound-allowlist`);
  const restored=await page.request.put(`${ADMIN}/security/inbound-allowlist`,{data:original.config,headers:{'if-match':current.headers()['etag']!}});
  expect(restored.status(),await restored.text()).toBe(200);
 }
});
