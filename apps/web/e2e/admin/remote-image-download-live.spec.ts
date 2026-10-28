import {test,expect,type Page} from '../csrf.ts';
import {createServer,type Socket} from 'node:net';
import {deflateSync} from 'node:zlib';
import {sendRawSmtp} from '../crypto-helpers.ts';
import {signInManually,messageRow,drawerButton} from '../helpers.ts';
function png():Buffer {
 const crc=(b:Buffer)=>{let c=0xffffffff;for(const v of b){c^=v;for(let i=0;i<8;i++)c=c&1?(c>>>1)^0xedb88320:c>>>1;}return(c^0xffffffff)>>>0;};
 const chunk=(name:string,data:Buffer)=>{const bytes=Buffer.concat([Buffer.from(name),data]),len=Buffer.alloc(4),sum=Buffer.alloc(4);len.writeUInt32BE(data.length);sum.writeUInt32BE(crc(bytes));return Buffer.concat([len,bytes,sum]);};
 const head=Buffer.alloc(13);head.writeUInt32BE(8);head.writeUInt32BE(8,4);head[8]=8;head[9]=6;const data=Buffer.alloc(8*33);for(let y=0;y<8;y++)for(let x=0;x<8;x++){const p=y*33+1+x*4;data[p]=30;data[p+1]=130;data[p+2]=210;data[p+3]=255;}return Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),chunk('IHDR',head),chunk('IDAT',deflateSync(data)),chunk('IEND',Buffer.alloc(0))]);
}
// The server runs in a container: the proxy fixture listens on every host interface and the server reaches it at
// host.docker.internal (docker-compose.dev.yml maps it to host-gateway on Linux). MW_E2E_FIXTURE_HOST overrides it.
const fixtureHost=process.env['MW_E2E_FIXTURE_HOST']??'host.docker.internal';
async function request(page:Page,path:string,method='GET',data?:unknown,etag?:string){return page.evaluate(async({path,method,data,etag})=>{const token=document.cookie.split('; ').find(v=>v.startsWith('mw_csrf='))?.slice(8);const response=await fetch(path,{method,headers:{...(data===undefined?{}:{'content-type':'application/json'}),...(token?{'x-csrf-token':decodeURIComponent(token)}:{}),...(etag?{'if-match':etag}:{})},body:data===undefined?null:JSON.stringify(data)});const text=await response.text();return{status:response.status,etag:response.headers.get('etag'),body:text?JSON.parse(text):null};},{path,method,data,etag});}
test('manual image download uses owned message membership and the privacy proxy @phone',async({page,browser,baseURL},info)=>{
 const sockets=new Set<Socket>(),requests:string[]=[],tunnels:string[]=[];const bytes=png();
 const proxy=createServer(socket=>{sockets.add(socket);socket.on('error',()=>undefined);socket.on('close',()=>sockets.delete(socket));let buffer='';let connected=false;socket.on('data',data=>{buffer+=data.toString('latin1');const end=buffer.indexOf('\r\n\r\n');if(end<0)return;const head=buffer.slice(0,end);buffer=buffer.slice(end+4);if(!connected){tunnels.push(head.split('\r\n')[0]!);if(!head.startsWith('CONNECT 1.2.3.4:80 ')){socket.destroy();return;}connected=true;socket.write('HTTP/1.1 200 Connection Established\r\n\r\n');return;}requests.push(head);socket.end(Buffer.concat([Buffer.from(`HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: ${bytes.length}\r\nConnection: close\r\n\r\n`),bytes]));});});
 await new Promise<void>(resolve=>proxy.listen(0,'0.0.0.0',resolve));const address=proxy.address();if(!address||typeof address==='string')throw new Error('proxy address');
 const adminContext=await browser.newContext({baseURL:baseURL!}),admin=await adminContext.newPage();
 const id=`image-download-${info.project.name}-${Date.now()}`,keys=['privacy.remoteImageDownloads','privacy.remoteContent'];
 let saved:Awaited<ReturnType<typeof request>>[]=[];let active:string|undefined;let configured=false;
 try{
  await admin.waitForTimeout(3500);await admin.goto('/admin');await admin.getByLabel('Admin username').fill('root');await admin.getByLabel('Password',{exact:true}).fill('hunter2');const signedIn=admin.waitForResponse(r=>r.url().endsWith('/admin/login')&&r.request().method()==='POST');await admin.getByRole('button',{name:'Sign in',exact:true}).click();expect((await signedIn).status()).toBe(200);await admin.waitForFunction(()=>document.cookie.includes('mw_csrf='));
  const proof=await request(admin,'/admin/step-up','POST',{password:'hunter2'});expect(proof.status,JSON.stringify(proof.body)).toBe(200);
  saved=await Promise.all(keys.map(key=>request(admin,`/admin/settings/${key}`)));for(const value of saved)expect(value.status).toBe(200);
  const proxies=await request(admin,'/admin/egress/proxies');active=(proxies.body.proxies as {id:string;active:boolean}[]).find(row=>row.active)?.id;
  expect((await request(admin,'/admin/egress/proxies','POST',{id,scheme:'http',host:fixtureHost,port:address.port,allowPlaintext:true,trafficClasses:['remote_content'],scope:'global'})).status).toBe(200);configured=true;
  expect((await request(admin,`/admin/egress/proxies/${id}/activate`,'POST')).status).toBe(200);
  for(let i=0;i<keys.length;i++)expect((await request(admin,`/admin/settings/${keys[i]}`,'PUT',{globalValue:i===0?true:'block',locked:true},saved[i]!.etag!)).status).toBe(200);
  const subject=`Manual proxy image ${id}`;
  await sendRawSmtp({from:'sender@example.test',message:`From: sender@example.test\r\nTo: testuser@example.org\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <${id}@example.test>\r\nSubject: ${subject}\r\nMIME-Version: 1.0\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<p>Manual download fixture</p><img alt="manual image" src="http://1.2.3.4/${id}.png">\r\n`});
  await page.waitForTimeout(3500);await signInManually(page,{username:'testuser',password:'testpass'});await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
  const direct:string[]=[];page.on('request',r=>{if(r.url().startsWith('http://1.2.3.4/'))direct.push(r.url());});await page.route('http://1.2.3.4/**',r=>r.abort());
  await expect(async()=>{if(page.viewportSize()!.width<761){await drawerButton(page).click();await page.getByRole('navigation',{name:'Mailboxes'}).getByRole('button',{name:'Inbox',exact:true}).click();}else await page.getByRole('navigation',{name:'Mailboxes'}).getByRole('button',{name:'Inbox',exact:true}).click();await expect(messageRow(page,subject)).toBeVisible({timeout:3000});}).toPass({timeout:60000});await messageRow(page,subject).click();
  const section=page.getByRole('region',{name:'Manual remote-image downloads'});await expect(section).toBeVisible();await section.locator('summary').click();expect(requests).toHaveLength(0);
  const download=page.waitForEvent('download',{timeout:45000});const fetched=page.waitForResponse(r=>r.url().includes('/api/rc/d/'),{timeout:40000});await section.getByRole('button',{name:'Download through the server'}).click();const result=await fetched.catch(error=>{throw new Error(`${String(error)}; tunnels=${JSON.stringify(tunnels)} requests=${requests.length}`);});expect(result.status(),await result.text()).toBe(200);expect((await download).suggestedFilename()).toBe('message-image.png');await expect.poll(()=>requests.length).toBe(1);expect(requests[0]).toContain(`GET /${id}.png `);expect(requests[0]?.toLowerCase()).not.toContain('cookie:');expect(direct).toEqual([]);
  await section.scrollIntoViewIfNeeded();await page.screenshot({path:info.outputPath('manual-image-download.png'),fullPage:true});
 }finally{
  try{
  if(configured){expect((await request(admin,active?`/admin/egress/proxies/${active}/activate`:'/admin/egress/proxies/deactivate','POST')).status).toBe(200);expect((await request(admin,`/admin/egress/proxies/${id}/delete`,'POST')).status).toBe(200);}
  for(let i=0;i<saved.length;i++){const before=saved[i]!;if(before.status!==200)continue;const path=`/admin/settings/${keys[i]}`,current=await request(admin,path);expect((await request(admin,path,'PUT',{globalValue:before.body.globalValue,locked:before.body.globalLocked,allowUserOverride:before.body.allowUserOverride},current.etag!)).status).toBe(200);}
  }finally{await adminContext.close();for(const socket of sockets)socket.destroy();await new Promise<void>(resolve=>proxy.close(()=>resolve()));}
 }
});
