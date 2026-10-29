import {test,expect} from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import {adminLogin} from './admin-login.ts';

test('guided advanced domain persists real backend policies with bounded forms @phone',async({page},info)=>{
 await adminLogin(page);const name=`guided-${info.project.name}-${Date.now()}.example.test`;
 try{
  await page.goto(`${ADMIN}#/domains/new/guided`);
  await page.getByLabel('Domain name',{exact:true}).fill(name);await page.getByRole('button',{name:'Next',exact:true}).click();
  await page.getByLabel('Connection',{exact:true}).selectOption('advanced');
  await page.getByRole('button',{name:'Add backend',exact:true}).click();
  await page.getByLabel(/^Backend ID/).fill('primary');await page.getByLabel('Hostname or IP address',{exact:true}).fill('mail.example.test');
  await page.getByLabel(/^Priority/).fill('10');await page.getByLabel('Preferred for this service',{exact:true}).check();
  await page.getByLabel(/^Authentication username format/).selectOption('domainUser');await page.getByLabel('Authentication domain',{exact:true}).fill('CORP');
  await page.getByLabel('Customize transport security',{exact:true}).check();await page.getByLabel(/^Security selection/).selectOption('auto');await page.getByLabel(/^Method priority 2/).selectOption('starttls');
  await page.getByLabel('Customize authentication methods',{exact:true}).check();await page.getByLabel(/^Authentication selection/).selectOption('auto');await page.getByLabel(/^Authentication priority 1/).selectOption('plain');await page.getByLabel(/^Authentication priority 2/).selectOption('login');
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
  const alignment=await page.getByLabel('Preferred for this service',{exact:true}).evaluate(input=>{const parent=input.parentElement!;const a=input.getBoundingClientRect(),b=parent.getBoundingClientRect();return Math.abs(a.y+a.height/2-(b.y+b.height/2));});expect(alignment).toBeLessThan(3);
  const button=await page.getByRole('button',{name:'Back to backend list',exact:true}).boundingBox();expect(button!.width).toBeLessThan(260);
  await page.screenshot({path:info.outputPath('guided-advanced-form.png'),fullPage:true});
  await page.getByRole('button',{name:'Back to backend list',exact:true}).click();await page.getByRole('button',{name:'Add backend',exact:true}).click();
  await page.getByLabel(/^Protocol and service/).selectOption('smtp');await page.getByLabel('Hostname or IP address',{exact:true}).fill('smtp.example.test');
  await page.getByRole('button',{name:'Back to backend list',exact:true}).click();await expect(page.getByRole('table',{name:'Backend setup drafts'})).toBeVisible();
  await page.getByRole('button',{name:'Next',exact:true}).click();await page.screenshot({path:info.outputPath('guided-advanced-review.png'),fullPage:true});
  expect((await page.request.get(`${ADMIN}/domains/${name}`)).status()).toBe(404);
  await page.getByRole('button',{name:'Create domain',exact:true}).click();await expect(page).toHaveURL(new RegExp(`/domains/${name}/connections$`));
  const saved=await page.request.get(`${ADMIN}/domains/${name}`);expect(saved.ok(),await saved.text()).toBe(true);const data=await saved.json();
  expect(data.upstream).toMatchObject({version:2,backends:[{id:'primary',protocol:'imap',priority:10,preferred:true,username:{format:'domainUser',domain:'CORP'},endpoint:{security:{mode:'auto',allowed:['implicit','starttls'],allowPlaintext:false},authentication:{mode:'auto',mechanisms:['plain','login']}}},{protocol:'smtp',role:'submission'}]});
  await page.reload();await expect(page.getByRole('heading',{name:'primary · IMAP',exact:true})).toBeVisible();expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
  await page.screenshot({path:info.outputPath('guided-advanced-persisted.png'),fullPage:true});
 }finally{const read=await page.request.get(`${ADMIN}/domains/${name}`);if(read.ok())expect((await page.request.delete(`${ADMIN}/domains/${name}`,{headers:{'if-match':read.headers()['etag']!}})).ok()).toBe(true);}
});
