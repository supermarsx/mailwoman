import { test, expect, type Page, type APIRequestContext } from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import { deflateSync } from 'node:zlib';
import { sendRawSmtp } from '../crypto-helpers.ts';
import { createServer as createHttpServer } from 'node:http';
import type { Socket } from 'node:net';
import { signInManually, messageRow, sidebarInbox, drawerButton, storageAuthorityHeaders } from '../helpers.ts';
const adminPath=ADMIN;
function png(size=8):Buffer {
  const crc=(b:Buffer)=>{let c=0xffffffff;for(const v of b){c^=v;for(let i=0;i<8;i++)c=c&1?(c>>>1)^0xedb88320:c>>>1;}return(c^0xffffffff)>>>0;};
  const chunk=(name:string,data:Buffer)=>{const bytes=Buffer.concat([Buffer.from(name),data]),len=Buffer.alloc(4),sum=Buffer.alloc(4);len.writeUInt32BE(data.length);sum.writeUInt32BE(crc(bytes));return Buffer.concat([len,bytes,sum]);};
  const head=Buffer.alloc(13);head.writeUInt32BE(size);head.writeUInt32BE(size,4);head[8]=8;head[9]=6;const data=Buffer.alloc(size*(1+size*4));for(let y=0;y<size;y++)for(let x=0;x<size;x++){const p=y*(1+size*4)+1+x*4;data[p]=30;data[p+1]=130;data[p+2]=210;data[p+3]=255;}return Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),chunk('IHDR',head),chunk('IDAT',deflateSync(data)),chunk('IEND',Buffer.alloc(0))]);
}
const hostileSvg='<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" onload="parent.postMessage(\'SVG_EXEC\',\'*\')"><rect width="16" height="16" fill="blue"/><script>parent.postMessage("SVG_EXEC","*")</script><foreignObject width="16" height="16"><body xmlns="http://www.w3.org/1999/xhtml" onload="parent.postMessage(\'SVG_EXEC\',\'*\')">X</body></foreignObject><a href="javascript:parent.postMessage(\'SVG_EXEC\',\'*\')">link</a><image href="https://svg-leak.invalid/pixel"/><style>@import url(https://svg-leak.invalid/style.css);</style></svg>';
async function seed(subject:string,html:string,parts=false):Promise<void> {
  const boundary='privacy-'+Date.now();const headers=`From: Sender <sender@example.test>\r\nTo: testuser@example.org\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <${boundary}@example.test>\r\nSubject: ${subject}\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary="${boundary}"\r\n\r\n`;
  let body=`--${boundary}\r\nContent-Type: text/html; charset=utf-8\r\n\r\n${html}\r\n`;
  if(parts) body+=`--${boundary}\r\nContent-Type: image/png\r\nContent-ID: <logo@test>\r\nContent-Disposition: inline; filename="logo.png"\r\nContent-Transfer-Encoding: base64\r\n\r\n${png().toString('base64')}\r\n--${boundary}\r\nContent-Type: image/svg+xml\r\nContent-Disposition: attachment; filename="hostile.svg"\r\nContent-Transfer-Encoding: base64\r\n\r\n${Buffer.from(hostileSvg).toString('base64')}\r\n`;
  await sendRawSmtp({from:'sender@example.test',message:headers+body+`--${boundary}--\r\n`});
}
// The server runs in a container: the proxy fixture listens on every host interface and the server reaches it at
// host.docker.internal (docker-compose.dev.yml maps it to host-gateway on Linux). MW_E2E_FIXTURE_HOST overrides it.
const fixtureHost=process.env['MW_E2E_FIXTURE_HOST']??'host.docker.internal';
/** An HTTP proxy that tunnels CONNECT 1.2.3.4:80 to a fixture origin serving the message's remote resources. */
async function proxyFixture(){
  const sockets=new Set<Socket>();const tunnels:string[]=[];
  const origin=createHttpServer((request,response)=>{
    const path=new URL(request.url??'/','http://1.2.3.4').pathname;
    if(path==='/fixture.png'){response.writeHead(200,{'content-type':'image/png'});response.end(png());return;}
    if(path==='/tracker.png'){response.writeHead(200,{'content-type':'image/png'});response.end(png(1));return;}
    if(path==='/document.txt'){response.writeHead(200,{'content-type':'text/plain'});response.end('linked document');return;}
    response.writeHead(404);response.end();
  });
  const proxy=createHttpServer((request,response)=>{tunnels.push(`${request.method} ${request.url}`);origin.emit('request',request,response);});
  proxy.on('connect',(request,socket:Socket,head)=>{
    tunnels.push(`CONNECT ${request.url}`);sockets.add(socket);socket.on('close',()=>sockets.delete(socket));socket.on('error',()=>undefined);
    if(request.url!=='1.2.3.4:80'){socket.destroy();return;}
    socket.write('HTTP/1.1 200 Connection Established\r\n\r\n');if(head.length)socket.unshift(head);origin.emit('connection',socket);
  });
  proxy.on('connection',socket=>{sockets.add(socket);socket.on('close',()=>sockets.delete(socket));});
  await new Promise<void>(done=>proxy.listen(0,'0.0.0.0',done));
  return {tunnels,port:(proxy.address() as {port:number}).port,close:async()=>{for(const socket of sockets)socket.destroy();await new Promise<void>(done=>proxy.close(()=>done()));origin.close();}};
}
async function adminLogin(api:APIRequestContext):Promise<void>{
  expect((await api.post(`${adminPath}/login`,{data:{username:process.env['MW_ADMIN_USER']??'root',password:process.env['MW_ADMIN_PASSWORD']??'hunter2'}})).status()).toBe(200);
  expect((await api.post(`${adminPath}/step-up`,{data:{password:process.env['MW_ADMIN_PASSWORD']??'hunter2'}})).status()).toBe(200);
}
/** Set a global setting and return a function that puts back what was there. */
async function setGlobal(api:APIRequestContext,key:string,value:unknown,locked=false):Promise<()=>Promise<void>>{
  const before=await api.get(`${adminPath}/settings/${key}`);expect(before.status()).toBe(200);const saved=await before.json() as {globalValue:unknown;globalLocked:boolean};
  const response=await api.put(`${adminPath}/settings/${key}`,{data:{globalValue:value,locked},headers:{'if-match':before.headers()['etag']!}});expect(response.ok(),await response.text()).toBe(true);
  return async()=>{const current=await api.get(`${adminPath}/settings/${key}`);const put=await api.put(`${adminPath}/settings/${key}`,{data:{globalValue:saved.globalValue??null,locked:saved.globalLocked},headers:{'if-match':current.headers()['etag']!}});expect(put.ok(),await put.text()).toBe(true);};
}
// Personal-settings writes carry the session's storage authority (0a11b581).
async function mailbox(page:Page,mine:Record<string,unknown>):Promise<void>{await page.waitForTimeout(3100);await signInManually(page,{username:'testuser',password:'testpass'});await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();for(const [key,value] of Object.entries(mine)){const response=await page.request.put(`/api/settings/${key}`,{data:{value},headers:await storageAuthorityHeaders(page.request)});expect(response.ok(),await response.text()).toBe(true);}await page.reload();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();}
async function resetMailbox(page:Page,mine:Record<string,unknown>):Promise<void>{for(const key of Object.keys(mine)){const response=await page.request.delete(`/api/settings/${key}`,{headers:await storageAuthorityHeaders(page.request)});expect(response.ok(),await response.text()).toBe(true);}}
async function openMessage(page:Page,subject:string):Promise<void>{await expect(async()=>{if(page.viewportSize()!.width<761){await drawerButton(page).click();await sidebarInbox(page).click();}else await sidebarInbox(page).click();await expect(messageRow(page,subject)).toBeVisible({timeout:3000});}).toPass({timeout:45000});await messageRow(page,subject).click();await expect(page.locator('iframe.reader__frame')).toBeVisible();}

test('signed proxy and CID images load without frame cookies; hostile SVG remains an image @phone',async({page,playwright,baseURL},info)=>{
  const admin=await playwright.request.newContext({baseURL:baseURL??''});const fixture=await proxyFixture();
  // The adaptive colour mode (the default) drops sender background images, CID ones included; read in the
  // original mode so the CID background is there to check.
  const mine={'privacy.remoteContent':'proxy','reader.colorMode':'original'};
  const id=`privacy-fixture-${info.project.name}-${Date.now()}`;let active:string|undefined;let configured=false;let signedIn=false;
  try{
  await page.waitForTimeout(3100);await adminLogin(admin);
  active=((await(await admin.get(`${adminPath}/egress/proxies`)).json()) as {proxies:{id:string;active:boolean}[]}).proxies.find(row=>row.active)?.id;
  const created=await admin.post(`${adminPath}/egress/proxies`,{data:{id,scheme:'http',host:fixtureHost,port:fixture.port,username:'',allowPlaintext:true,trafficClasses:['remote_content'],scope:'global'}});expect(created.ok(),await created.text()).toBe(true);configured=true;
  expect((await admin.post(`${adminPath}/egress/proxies/${id}/activate`)).ok()).toBe(true);
  const subject=`Privacy images ${info.project.name} ${Date.now()}`;
  await seed(subject,'<p>Privacy fixture</p><img alt="remote" src="http://1.2.3.4/fixture.png"><img alt="inline" src="cid:logo@test"><a href="http://1.2.3.4/document.txt">document</a><a href="http://1.2.3.4/fixture.png">unclassified download</a><img alt="tracker" src="http://1.2.3.4/tracker.png"><img alt="cid candidates" srcset="cid:logo@test 1x"><p style="background-image:url(cid:logo@test)">CID background</p>',true);
  const leaks:string[]=[];page.on('request',r=>{if(r.url().includes('svg-leak.invalid'))leaks.push(r.url());});await page.route('**://svg-leak.invalid/**',r=>r.abort());
  await page.addInitScript(()=>{(window as unknown as {svgRan:boolean}).svgRan=false;addEventListener('message',e=>{if(e.data==='SVG_EXEC')(window as unknown as {svgRan:boolean}).svgRan=true;});});
  await mailbox(page,mine);signedIn=true;const signedResponses:{url:string;status:number}[]=[];page.on('response',r=>{if(r.url().includes('/api/rc/i/')||r.url().includes('/api/rc/p/'))signedResponses.push({url:r.url(),status:r.status()});});
  await openMessage(page,subject);const frame=page.frameLocator('iframe.reader__frame').frameLocator('iframe');
  await expect(frame.getByAltText('remote')).toHaveJSProperty('naturalWidth',8,{timeout:30000});await expect(frame.getByAltText('inline')).toHaveJSProperty('naturalWidth',8);
  await expect(frame.getByAltText('cid candidates')).toHaveJSProperty('naturalWidth',8);
  await expect(frame.getByAltText('tracker')).toHaveJSProperty('naturalWidth',1);
  await expect(frame.getByText('CID background')).toHaveAttribute('style', /api\/rc\/p\//);
  await expect(page.getByTestId('remote-content-bar')).toContainText(/1 tracker removed/,{timeout:15000});
  expect(signedResponses.some(r=>r.status===200 && r.url.includes('/api/rc/i/'))).toBe(true);expect(signedResponses.some(r=>r.status===200 && r.url.includes('/api/rc/p/'))).toBe(true);
  await expect(page.locator('.mw-thumb__img').first()).toHaveJSProperty('naturalWidth',16);
  await page.getByRole('option',{name:/hostile.svg/}).click();
  await expect(page.frameLocator('iframe[title="hostile.svg"]').locator('img')).toHaveJSProperty('naturalWidth',16);
  expect(await page.evaluate(()=>(window as unknown as {svgRan:boolean}).svgRan)).toBe(false);expect(leaks).toEqual([]);
  await page.screenshot({path:info.outputPath('svg-preview.png'),fullPage:true});
  // 3da161a0 opens an attachment in its own workspace tab; closing it returns to the message tab.
  await page.getByRole('button',{name:'Close hostile.svg',exact:true}).click();await expect(page.getByRole('tab',{name:subject,exact:true})).toHaveAttribute('aria-selected','true');
  await page.getByText('Files linked in this message',{exact:true}).click();const download=page.waitForEvent('download');await page.getByRole('button',{name:'Download through the server'}).first().click();expect((await download).suggestedFilename()).toBe('document.txt');
  await page.getByLabel('Download another message link as a file').selectOption('http://1.2.3.4/fixture.png');
  const arbitraryDownload=page.waitForEvent('download'); await page.getByRole('button',{name:'Download through the server'}).last().click(); expect((await arbitraryDownload).suggestedFilename()).toBe('fixture.png');
  }finally{
    try{
      if(signedIn)await resetMailbox(page,mine);
      if(configured){expect((await admin.post(active?`${adminPath}/egress/proxies/${active}/activate`:`${adminPath}/egress/proxies/deactivate`)).ok()).toBe(true);expect((await admin.post(`${adminPath}/egress/proxies/${id}/delete`)).ok()).toBe(true);}
    }finally{await admin.dispose();await fixture.close();}
  }
});

test('permitted direct content uses a trusted wrapper and still cannot execute message scripts',async({page,playwright,baseURL},info)=>{
  const admin=await playwright.request.newContext({baseURL:baseURL??''});const restore:(()=>Promise<void>)[]=[];let signedIn=false;
  try{
  await page.waitForTimeout(3100);await adminLogin(admin);restore.push(await setGlobal(admin,'privacy.allowDirect',true));restore.push(await setGlobal(admin,'cap.remote_content.direct',true));
  const subject=`Privacy direct ${info.project.name} ${Date.now()}`;await seed(subject,'<p>Direct fixture</p><img alt="direct" src="https://images.test/direct.png"><img alt="plaintext" src="http://images.test/plain.png">');
  let https=0;let plaintext=0;await page.route('https://images.test/direct.png',r=>{https++;return r.fulfill({contentType:'image/png',body:png()});});await page.route('http://images.test/plain.png',r=>{plaintext++;return r.abort();});
  await mailbox(page,{'privacy.remoteContent':'direct'});signedIn=true;expect((await (await page.request.get('/api/rc/policy')).json() as {mode:string}).mode).toBe('direct');await openMessage(page,subject);
  const outer=page.locator('iframe.reader__frame');await expect(outer).toHaveAttribute('sandbox','allow-scripts');await expect(outer).toHaveAttribute('src',/\/api\/rc\/frame#/);
  const content=page.frameLocator('iframe.reader__frame').frameLocator('iframe');await expect(content.getByAltText('direct')).toHaveJSProperty('naturalWidth',8);
  expect(https).toBe(1);expect(plaintext).toBe(0);
  const header=await page.request.get('/api/rc/frame');expect(header.headers()['x-frame-options']).toBe('SAMEORIGIN');expect(header.headers()['content-security-policy']).not.toContain(',');
  await page.evaluate(()=>{const f=document.querySelector<HTMLIFrameElement>('iframe.reader__frame')!;const token=new URLSearchParams(new URL(f.src).hash.slice(1)).get('token');(window as unknown as {payloadRan:boolean}).payloadRan=false;addEventListener('message',e=>{if(e.data==='MESSAGE_EXEC')(window as unknown as {payloadRan:boolean}).payloadRan=true;});f.contentWindow!.postMessage({type:'mw-reader-frame',token,html:'<p>Sandbox test</p><script>top.postMessage("MESSAGE_EXEC","*")</script>'},'*');});
  await expect(content.getByText('Sandbox test')).toBeVisible();expect(await page.evaluate(()=>(window as unknown as {payloadRan:boolean}).payloadRan)).toBe(false);
  }finally{
    try{if(signedIn)await resetMailbox(page,{'privacy.remoteContent':'direct'});for(const undo of restore.reverse())await undo();}finally{await admin.dispose();}
  }
});

test('privacy and outbound sections show real policy and process diagnostics @phone', async ({page},info) => {
  await page.goto(adminPath);
  await page.getByLabel('Admin username').fill(process.env['MW_ADMIN_USER'] ?? 'root');
  await page.getByLabel('Password', {exact:true}).fill(process.env['MW_ADMIN_PASSWORD'] ?? 'hunter2');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  await expect(page.getByRole('button',{name:'Sign in',exact:true})).toHaveCount(0);
  await page.goto(`${adminPath}#/privacy`);
  await expect(page.getByRole('heading',{name:'Privacy and remote content',exact:true})).toBeVisible();
  await expect(page.getByText('Since this server process started',{exact:true})).toBeVisible();
  await expect(page.getByText('Cached images',{exact:true})).toBeVisible();
  await expect(page.getByRole('link',{name:'Edit policy'})).toHaveCount(7);
  await page.screenshot({path:info.outputPath('privacy-policy.png'),fullPage:true});
  await page.goto(`${adminPath}#/egress`);
  await expect(page.getByRole('heading',{name:'Outbound connections',exact:true})).toBeVisible();
  await expect(page.getByText('A failed configured route blocks the request. It never falls back to a direct connection.',{exact:true})).toBeVisible();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
});
