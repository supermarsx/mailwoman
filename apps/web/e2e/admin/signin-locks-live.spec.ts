import { request } from 'node:http';
import { ADMIN } from './admin-path.ts';
import { test, expect, type Page } from '../csrf.ts';
function attempt(base:string,username='lock-browser-fixture',password='invalid-fixture-password'):Promise<{status:number,retry:number,code:string}>{
 return new Promise((resolve,reject)=>{
  const body=JSON.stringify({username,password});
  const req=request(new URL(`${ADMIN}/login`,base),{method:'POST',headers:{'content-type':'application/json','content-length':Buffer.byteLength(body)}},res=>{
   let text='';res.setEncoding('utf8');res.on('data',chunk=>{text+=chunk;});
   res.on('end',()=>{let code='';try{code=String((JSON.parse(text) as {code?:unknown}).code??'');}catch{/* not JSON */}resolve({status:res.statusCode??0,retry:Number(res.headers['retry-after']??1),code});});
  });req.on('error',reject);req.end(body);
 });
}
async function signIn(page:Page):Promise<void>{
 await page.goto(ADMIN);await page.getByLabel('Admin username').fill('root');
 await page.getByLabel('Password',{exact:true}).fill('hunter2');
 const login=page.waitForResponse(response=>new URL(response.url()).pathname===`${ADMIN}/login`&&response.request().method()==='POST');
 await page.getByRole('button',{name:'Sign in',exact:true}).click();const result=await login;expect(result.status(),await result.text()).toBe(200);
 await expect(page.getByRole('button',{name:'Sign in',exact:true})).toHaveCount(0);
}
// The lock is per source address, and every client of a NAT-published stack (Docker,
// CI) reaches the server from the same address as this browser. So the administrator
// signs in BEFORE the rejections, proves the lock blocks a fresh sign-in, unlocks it
// from the still-valid session, and the finally block unlocks through the same API if
// any step failed: a leaked lock would refuse every later admin sign-in for 15 min.
test('temporary locks list actual rejected credentials and require proof to unlock @phone',async({page,baseURL})=>{
 test.setTimeout(150_000);
 if(!baseURL)throw new Error('Live fixture base URL required');
 await page.waitForTimeout(3100);await signIn(page);
 // The address the server attributes to this host's connections (the diagnostic reports the caller's own source).
 const diagnostic=await page.request.get(`${ADMIN}/bans/geography`);expect(diagnostic.status()).toBe(200);
 const ip=String((await diagnostic.json()).sourceIp);expect(ip).not.toBe('');
 const unlockLeft=async():Promise<void>=>{
  const listed=await page.request.get(`${ADMIN}/bans/automatic`);if(!listed.ok())return;
  const lock=((await listed.json()).locks as {surface:string,ip:string,revision:string}[]).find(row=>row.surface==='admin'&&row.ip===ip);if(!lock)return;
  expect((await page.request.post(`${ADMIN}/step-up`,{data:{password:'hunter2'}})).ok()).toBe(true);
  const cleared=await page.request.delete(`${ADMIN}/bans/automatic/admin/${encodeURIComponent(ip)}`,{headers:{'if-match':`"${lock.revision}"`}});
  expect(cleared.ok(),await cleared.text()).toBe(true);
 };
 try{
  let locked=false;
  for(let tries=0;tries<16&&!locked;tries++){
   const response=await attempt(baseURL);
   if(response.status===429&&response.code==='signinLocked'){locked=true;break;}
   if(response.status!==401&&response.status!==429)throw new Error(`Unexpected login status ${response.status} ${response.code}`);
   await new Promise(resolve=>setTimeout(resolve,Math.min(4000,Math.max(1,response.retry)*1000+100)));
  }
  expect(locked,'five rejected credentials lock the source').toBe(true);
  // A fresh sign-in from the locked source is refused even with the right password.
  await page.waitForTimeout(3100);const refused=await attempt(baseURL,'root','hunter2');expect(refused.status).toBe(429);expect(refused.code).toBe('signinLocked');
  await page.goto(`${ADMIN}#/observability/locks`);
  await page.getByRole('searchbox',{name:'Search temporary sign-in locks'}).fill(ip);
  const table=page.getByRole('table',{name:'Temporary sign-in locks'});
  await expect(table.getByText(ip,{exact:true})).toBeVisible();await expect(table).toContainText('Administrator');
  await page.screenshot({path:test.info().outputPath('temporary-lock-table.png'),fullPage:true});
  await page.getByRole('button',{name:`Unlock Administrator sign-in for ${ip}`}).click();
  const confirm=page.getByRole('alertdialog');await expect(confirm).toContainText('fresh security check');
  await confirm.getByRole('button',{name:'Unlock',exact:true}).click();
  const proof=page.getByRole('dialog',{name:'Confirm it is you'});
  await proof.getByRole('textbox',{name:'Password',exact:true}).fill('hunter2');
  await proof.getByRole('button',{name:'Confirm and continue'}).click();
  await expect(table.getByText(ip,{exact:true})).toHaveCount(0);
  await page.reload();await expect(page.getByRole('button',{name:`Unlock Administrator sign-in for ${ip}`})).toHaveCount(0);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 }finally{await unlockLeft();}
 // Unlocked: a fresh sign-in from the same source is admitted again.
 await page.context().clearCookies();await page.waitForTimeout(3100);await signIn(page);
});
