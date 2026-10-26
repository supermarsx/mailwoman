import {test,expect,type Page} from '../csrf.ts';
async function csrf(page:Page,path:string):Promise<Record<string,string>>{const cookies=await page.context().cookies(new URL(path,page.url()).href);const token=cookies.filter(cookie=>cookie.name==='mw_csrf').sort((a,b)=>b.path.length-a.path.length)[0]?.value;return token?{'x-csrf-token':token}:{};}
async function setting(page:Page,key:string,value:unknown):Promise<unknown>{
 const path=`/admin/settings/${key}`;const current=await page.request.get(path);expect(current.status()).toBe(200);const before=await current.json();const saved=await page.request.put(path,{headers:{...await csrf(page,path),'if-match':current.headers()['etag']!},data:{globalValue:value}});expect(saved.status(), await saved.text()).toBe(200);return before.globalValue;
}
test('adaptive controls show real evidence and clear the current client with fresh proof @phone',async({page})=>{
 test.setTimeout(150_000);await new Promise(resolve=>setTimeout(resolve,3200));await page.goto('/admin');
 await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect.poll(async()=>(await page.request.get('/admin/session')).status()).toBe(200);
 const originals=new Map<string,unknown>();
 const set=async(key:string,value:unknown):Promise<void>=>{const old=await setting(page,key,value);if(!originals.has(key))originals.set(key,old);};
 try{
  for(const [key,value] of Object.entries({adaptiveMode:'off',rateMode:'off',adaptiveAccounts:false,adaptiveAdmins:false,shortAbsoluteAnomalies:1,shortRatioPercent:0,longAbsoluteAnomalies:0,longRatioPercent:0,throttleSecs:60,signals:['malformed']}))await set(`security.protection.${key}`,value);
  await page.goto('/admin#/observability/protection');await expect(page.getByRole('heading',{name:'Aggregate and adaptive protection'})).toBeVisible();
  await expect(page.getByText('Request budgets: Off. Adaptive protection: Off.')).toBeVisible();
  await page.getByText('Configure modes, windows, thresholds, and exemptions').click();
  await page.getByRole('button',{name:'Adaptive request protection',exact:true}).click();
  await page.getByRole('combobox',{name:/^Adaptive request protection — security\.protection\.adaptiveMode$/}).selectOption('observe');
  const saved=page.waitForResponse(response=>response.url().includes('/admin/settings/security.protection.adaptiveMode')&&response.request().method()==='PUT'&&response.ok());
  await page.getByRole('button',{name:'Save',exact:true}).click();await saved;
  await page.reload();await expect(page.getByRole('combobox',{name:/^Adaptive request protection — security\.protection\.adaptiveMode$/})).toHaveValue('observe');
  await new Promise(resolve=>setTimeout(resolve,3200));
  const mailboxLogin=await page.request.post('/api/login',{data:{username:'testuser',password:'testpass'}});expect(mailboxLogin.status(),await mailboxLogin.text()).toBe(200);
  expect((await page.request.post('/jmap/api',{headers:{...await csrf(page,'/jmap/api'),'content-type':'application/json'},data:Buffer.from('{malformed')})).status()).toBe(400);
  await page.goto('/admin#/observability/protection');
  await expect(page.getByText('Request budgets: Off. Adaptive protection: Observe only.')).toBeVisible();
  const history=page.getByRole('table',{name:'Protection decisions'});await expect(history.getByText('Short-window anomaly count').first()).toBeVisible();
  const state=page.getByRole('table',{name:'Current subject state'});const target=state.locator('tbody tr').filter({has:page.getByRole('button',{name:/Clear protection state for Client address/})}).first();
  await expect(target).toBeVisible();await target.getByRole('button',{name:/Clear protection state/}).click();
  await expect(page.getByRole('alertdialog')).toContainText('does not change policy');
  const cleared=page.waitForResponse(response=>response.url().includes('/admin/bans/protection/subjects/')&&response.request().method()==='DELETE'&&response.status()===204);
  await page.getByRole('alertdialog').getByRole('button',{name:'Clear state',exact:true}).click();
  const proof=page.getByRole('dialog',{name:'Confirm it is you'});await expect(proof).toBeVisible();await proof.getByRole('textbox',{name:'Password',exact:true}).fill('hunter2');await proof.getByRole('button',{name:'Confirm and continue'}).click();await cleared;
  await expect(page.getByRole('alertdialog')).toHaveCount(0);await expect(history.getByText('Administrator cleared state').first()).toBeVisible();
  await expect(page.getByRole('button',{name:'Refresh',exact:true})).toBeEnabled();
  await page.screenshot({path:test.info().outputPath('adaptive-protection-evidence.png'),fullPage:true});expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 }finally{
  await setting(page,'security.protection.adaptiveMode','off');
  for(const [key,value] of originals)await setting(page,key,value);
 }
});
