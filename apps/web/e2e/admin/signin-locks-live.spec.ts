import { request } from 'node:http';
import { test, expect } from '../csrf.ts';
function rejection(base:string,ip:string):Promise<{status:number,retry:number}>{
 return new Promise((resolve,reject)=>{
  const target=new URL('/admin/login',base);target.hostname='127.0.0.1';
  const body=JSON.stringify({username:'lock-browser-fixture',password:'invalid-fixture-password'});
  const req=request(target,{method:'POST',localAddress:ip,headers:{'content-type':'application/json','content-length':Buffer.byteLength(body)}},res=>{
   res.resume();res.on('end',()=>resolve({status:res.statusCode??0,retry:Number(res.headers['retry-after']??1)}));
  });req.on('error',reject);req.end(body);
 });
}
test('temporary locks list actual rejected credentials and require proof to unlock @phone',async({page,baseURL})=>{
 test.setTimeout(90000);
 if(!baseURL)throw new Error('Live fixture base URL required');
 const ip=`127.0.0.${test.info().project.name==='chromium'?21:test.info().project.name==='firefox'?22:23}`;
 let failures=0;
 for(let attempt=0;attempt<16&&failures<5;attempt++){
  const response=await rejection(baseURL,ip);
  if(response.status===401)failures++;
  else if(response.status!==429)throw new Error(`Unexpected login status ${response.status}`);
  if(failures<5)await new Promise(resolve=>setTimeout(resolve,Math.min(4000,Math.max(1,response.retry)*1000+100)));
 }
 await page.goto('/admin');await page.getByLabel('Admin username').fill('root');
 await page.getByLabel('Password',{exact:true}).fill('hunter2');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByRole('button',{name:'Sign in',exact:true})).toHaveCount(0);
 await page.goto('/admin#/observability/locks');
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
});
