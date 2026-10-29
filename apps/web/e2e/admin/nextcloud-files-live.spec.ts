import {test,expect} from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import {signInManually,storageAuthorityHeaders,composeSurface,ribbonControl} from '../helpers.ts';
import {createServer as httpsServer} from 'node:https';
import {createServer as httpServer} from 'node:http';
import {connect} from 'node:net';
import type {Duplex} from 'node:stream';
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';

// The TLS fixtures live at the repo root; resolve them from this file, not the cwd (CI runs from apps/web).
const fixture=(name:string):string=>fileURLToPath(new URL(`../../../../crates/mw-server/tests/fixtures/tls/${name}`,import.meta.url));
// The server runs in a container; it reaches this in-process proxy on the runner host through
// host.docker.internal (docker-compose.dev.yml maps it to host-gateway on Linux). MW_E2E_FIXTURE_HOST overrides it.
const fixtureHost=process.env['MW_E2E_FIXTURE_HOST']??'host.docker.internal';

async function cloudFixture(){
  const sockets=new Set<Duplex>();const calls:Array<{method:string;path:string}>=[];
  const cert=readFileSync(fixture('nextcloud-cert.pem'),'utf8');
  const key=readFileSync(fixture('nextcloud-key.pem'),'utf8');
  const origin=httpsServer({cert,key},(request,response)=>{
    request.resume();calls.push({method:request.method??'',path:request.url??''});
    if(request.headers.authorization!==`Basic ${Buffer.from('alice:cloud-password').toString('base64')}`){response.writeHead(401);response.end();return;}
    if(request.method==='PROPFIND'){
      const file=request.headers.depth==='1'?'<d:response><d:href>/nextcloud/remote.php/dav/files/alice/R%C3%A9sum%C3%A9.txt</d:href><d:propstat><d:prop><d:getcontentlength>10</d:getcontentlength><d:getcontenttype>text/plain</d:getcontenttype></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response>':'';
      response.writeHead(207,{'content-type':'application/xml'});response.end(`<d:multistatus xmlns:d="DAV:"><d:response><d:href>/nextcloud/remote.php/dav/files/alice/</d:href><d:propstat><d:prop><d:resourcetype><d:collection/></d:resourcetype></d:prop></d:propstat></d:response>${file}</d:multistatus>`);return;
    }
    response.writeHead(200,{'content-type':'text/plain'});response.end('cloud-file');
  });
  origin.on('connection',socket=>{sockets.add(socket);socket.on('close',()=>sockets.delete(socket));});
  await new Promise<void>(done=>origin.listen(0,'127.0.0.1',done));
  const originPort=(origin.address() as {port:number}).port;
  const proxy=httpServer();proxy.on('connect',(request,socket,head)=>{
    if(request.url!=='8.8.8.8:443'){socket.destroy();return;}
    const upstream=connect(originPort,'127.0.0.1',()=>{socket.write('HTTP/1.1 200 Connection Established\r\n\r\n');if(head.length)upstream.write(head);socket.pipe(upstream);upstream.pipe(socket);});
    sockets.add(socket);sockets.add(upstream);socket.on('close',()=>{sockets.delete(socket);upstream.destroy();});upstream.on('close',()=>{sockets.delete(upstream);socket.destroy();});upstream.on('error',()=>socket.destroy());socket.on('error',()=>upstream.destroy());
  });
  await new Promise<void>(done=>proxy.listen(0,'0.0.0.0',done));
  return {cert,calls,port:(proxy.address() as {port:number}).port,close:async()=>{for(const socket of sockets)socket.destroy();await Promise.all([new Promise<void>(done=>origin.close(()=>done())),new Promise<void>(done=>proxy.close(()=>done()))]);}};
}

test('account-owned Nextcloud link browses and attaches through approved TLS proxy @phone',async({page,playwright,baseURL},info)=>{
  const admin=await playwright.request.newContext({...baseURL?{baseURL}:{}});
  const cloud=await cloudFixture();const id=`browser-cloud-${info.project.name}`;const route=`browser-cloud-route-${Date.now()}`;let active:string|undefined;let etag:string|undefined;
  const value={name:`Browser cloud ${info.project.name}`,baseUrl:'https://8.8.8.8/nextcloud/',enabled:true,caPem:cloud.cert};
  try{
    await page.waitForTimeout(4000);await admin.get(ADMIN);
    expect((await admin.post(`${ADMIN}/login`,{data:{username:'root',password:'hunter2'}})).status()).toBe(200);
    expect((await admin.post(`${ADMIN}/step-up`,{data:{password:'hunter2'}})).status()).toBe(200);
    const routes=await(await admin.get(`${ADMIN}/egress/proxies`)).json() as {proxies:Array<{id:string;active:boolean}>};
    for(const stale of routes.proxies.filter(item=>item.id.startsWith('browser-cloud-route-'))){if(stale.active)await admin.post(`${ADMIN}/egress/proxies/deactivate`);await admin.post(`${ADMIN}/egress/proxies/${stale.id}/delete`);}
    active=routes.proxies.find(item=>item.active&&!item.id.startsWith('browser-cloud-route-'))?.id;
    expect((await admin.post(`${ADMIN}/egress/proxies`,{data:{id:route,scheme:'http',host:fixtureHost,port:cloud.port,scope:'global',trafficClasses:['nextcloud']}})).ok()).toBe(true);
    expect((await admin.post(`${ADMIN}/egress/proxies/${route}/activate`)).ok()).toBe(true);
    const instances=await(await admin.get(`${ADMIN}/nextcloud/instances`)).json() as {instances:Array<{id:string;etag:string}>};
    const previous=instances.instances.find(item=>item.id===id);
    const approved=await admin.put(`${ADMIN}/nextcloud/instances/${id}`,{data:value,headers:previous?{'if-match':previous.etag}:{'if-none-match':'*'}});
    expect(approved.ok(),await approved.text()).toBe(true);etag=(await approved.json() as {etag:string}).etag;
    await page.waitForTimeout(4000);await page.goto('/');await signInManually(page,{username:'testuser',password:'testpass'});
    await expect(page.getByRole('button',{name:'Compose',exact:true}).first()).toBeVisible();
    const menu=page.getByRole('button',{name:'Open folders and apps',exact:true});if(await menu.isVisible())await menu.click();
    const show=page.getByRole('button',{name:'Show folders sidebar'});if(await show.isVisible())await show.click();
    await page.getByRole('button',{name:'Settings',exact:true}).click();
    await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Nextcloud accounts',exact:true}).click();
    const instance=page.getByRole('heading',{name:value.name,exact:true}).locator('..');
    await instance.getByRole('button',{name:'Link account',exact:true}).click();
    await page.getByLabel('Nextcloud app-password username',{exact:true}).fill('alice');
    await page.getByLabel('Nextcloud app password',{exact:true}).fill('cloud-password');
    await page.getByRole('button',{name:'Verify and link',exact:true}).click();
    await expect(instance.getByText('Linked and ready',{exact:true})).toBeVisible();
    await page.getByRole('button',{name:'Compose',exact:true}).click();
    // Attachment sources sit on the composer ribbon's Insert tab (e0c99a79).
    await composeSurface(page).getByRole('tab',{name:'Insert',exact:true}).click();
    // On a phone the Attachments group folds into a dropdown (t38-e5).
    await (await ribbonControl(composeSurface(page),'Attachments',page.getByRole('button',{name:'Attach from Nextcloud',exact:true}))).click();
    await page.getByRole('combobox',{name:'Nextcloud instance',exact:true}).selectOption(id);
    await page.getByRole('button',{name:/^Select .*Résumé\.txt/}).click();
    await page.getByRole('button',{name:'Attach 1 file',exact:true}).click();
    await expect(page.getByTestId('compose-attachments')).toContainText('Résumé.txt');
    expect(cloud.calls.some(call=>call.method==='GET'&&call.path.includes('R%C3%A9sum%C3%A9.txt'))).toBe(true);
    await page.screenshot({path:info.outputPath('nextcloud-actual-file-attachment.png'),fullPage:true});
  }finally{try{
    const accountResponse=await page.request.get('/api/account/nextcloud');
    if(accountResponse.ok()){const account=await accountResponse.json() as {accountId:string;links:Array<{instanceId:string;etag:string}>};const link=account.links.find(item=>item.instanceId===id);if(link)expect((await page.request.delete(`/api/account/nextcloud/links/${id}`,{data:{accountId:account.accountId},headers:{'if-match':link.etag,...await storageAuthorityHeaders(page.request)}})).ok()).toBe(true);}
    if(etag)expect((await admin.put(`${ADMIN}/nextcloud/instances/${id}`,{data:{...value,enabled:false},headers:{'if-match':etag}})).ok()).toBe(true);
    await admin.post(active?`${ADMIN}/egress/proxies/${active}/activate`:`${ADMIN}/egress/proxies/deactivate`);
    await admin.post(`${ADMIN}/egress/proxies/${route}/delete`);
    }finally{try{
      await admin.post(active?`${ADMIN}/egress/proxies/${active}/activate`:`${ADMIN}/egress/proxies/deactivate`);
      await admin.post(`${ADMIN}/egress/proxies/${route}/delete`);
    }finally{await cloud.close();await admin.dispose();}}
  }
});
