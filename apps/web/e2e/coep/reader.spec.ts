import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {createServer} from 'node:http';
import type {AddressInfo} from 'node:net';
import {test,expect} from '../csrf.ts';
let stopExternal:(()=>Promise<void>)|undefined;
test.afterEach(async()=>{await stopExternal?.();stopExternal=undefined;});

for(const blocked of [false,true])test(`message bridge displays with ${blocked?'a blocked initial frame':'normal frame admission'}`,async({page},info)=>{
 const dist=path.resolve('../../.orchestration/reports/codex-coep-web');
 const attempts:string[]=[];
 let externalRequests=0;
 const image=createServer((_request,response)=>{externalRequests++;response.writeHead(200,{'content-type':'image/gif','cross-origin-resource-policy':'cross-origin'});response.end(Buffer.from('R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7','base64'));});
 await new Promise<void>(resolve=>image.listen(0,'127.0.0.1',resolve));
 stopExternal=()=>new Promise<void>((resolve,reject)=>{image.close(error=>error?reject(error):resolve());image.closeAllConnections();});
 const externalUrl=`http://127.0.0.1:${(image.address() as AddressInfo).port}/pixel`;
 await page.request.get(externalUrl);expect(externalRequests).toBe(1);externalRequests=0;
 await page.route('**/*',async route=>{
  const url=new URL(route.request().url());
  if(url.pathname==='/api/rc/frame'){
   attempts.push(url.search);
   // Deterministically reproduce an embedder-policy navigation refusal. Desktop
   // engine builds need not have the same COEP support as the user's phone.
   if(blocked&&!url.search)return route.abort('blockedbyresponse');
   // Current SPA against the shared old binary: mirror the new public endpoint
   // headers here; the native HTTP regression separately checks the real server.
   const response=await route.fetch();const headers=response.headers();
   if(url.searchParams.get('coep')==='require-corp'){
    headers['cross-origin-embedder-policy']='require-corp';
    headers['content-security-policy']=headers['content-security-policy']!.replace("img-src 'self' https: data: blob:","img-src 'self' data: blob:");
   }
   const script=(await readFile('../../crates/mw-server/src/remote-frame.js','utf8')).replace(/\r\n?/g,'\n');
   const html=(await response.text()).replace(/<script>[\s\S]*?<\/script>/,()=>`<script>${script}</script>`);
   const hash=createHash('sha256').update(script).digest('base64');
   headers['content-security-policy']=headers['content-security-policy']!.replace(/'sha256-[^']+'/g,`'sha256-${hash}'`);
   return route.fulfill({response,headers,body:html});
  }
  if(url.pathname==='/'||url.pathname.startsWith('/assets/')){
   const file=path.resolve(dist,url.pathname==='/'?'index.html':url.pathname.slice(1));if(!file.startsWith(dist+path.sep))return route.abort();
   const response=await route.fetch();return route.fulfill({response,status:200,body:await readFile(file),contentType:file.endsWith('.html')?'text/html':file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':'application/octet-stream'});
  }
  return route.continue();
 });
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.locator('.list__row').first()).toBeVisible();await page.locator('.list__row').first().click();
 const outer=page.locator('iframe.reader__frame').filter({visible:true}).first();await expect(outer).toBeVisible();
 await expect(outer).toHaveAttribute('sandbox','allow-scripts');
 const body=outer.contentFrame().frameLocator('iframe').locator('body');await expect(body).not.toHaveText('',{timeout:15000});
 if(blocked){
  expect(attempts).toEqual(['','?coep=require-corp']);
  await expect(page.getByRole('status').filter({hasText:'Compatible viewer'})).toBeVisible();
  await outer.evaluate((node,externalUrl)=>{
   const frame=node as HTMLIFrameElement;const token=new URLSearchParams(new URL(frame.src).hash.slice(1)).get('token');
   const state=window as unknown as {coepProbeRan:boolean};state.coepProbeRan=false;
   addEventListener('message',event=>{if(event.data==='coep-script-ran')state.coepProbeRan=true;});
   frame.contentWindow!.postMessage({type:'mw-reader-frame',token,revision:900,html:'<p>Compatibility probe</p><img alt="embedded probe" src="data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7"><img alt="external probe" src="'+externalUrl+'"><script>top.postMessage("coep-script-ran","*")</script>'},'*');
  },externalUrl);
  await expect(body.getByText('Compatibility probe')).toBeVisible();
  await expect(body.getByAltText('embedded probe')).toHaveJSProperty('naturalWidth',1);
  expect(externalRequests).toBe(0);
  expect(await page.evaluate(()=>(window as unknown as {coepProbeRan:boolean}).coepProbeRan)).toBe(false);
 }else expect(attempts).toEqual(['']);
 if(info.project.name!=='webkit-phone')expect(await page.evaluate(()=>crossOriginIsolated)).toBe(true);
 await expect(outer.contentFrame().locator('iframe')).toHaveAttribute('sandbox','');
 await page.screenshot({path:info.outputPath('reader-coep.png'),fullPage:true});
});
