import {test,expect} from '../csrf.ts';
import {signInManually,composeSurface,ribbonControl} from '../helpers.ts';
import {createServer as httpServer} from 'node:http';
import {request as httpsRequest} from 'node:https';
import {connect} from 'node:net';
import type {Duplex} from 'node:stream';
import {readFileSync} from 'node:fs';

// Large attachment → Nextcloud share link, and a new calendar, against a REAL
// Nextcloud (t39 L7). Needs a disposable Nextcloud behind TLS:
//   MW_E2E_NEXTCLOUD_ORIGIN   host:port of its HTTPS listener (certificate valid for IP 8.8.8.8,
//                             e.g. crates/mw-server/tests/fixtures/tls/nextcloud-cert.pem)
//   MW_E2E_NEXTCLOUD_CA       PEM the server should trust for it
//   MW_E2E_NEXTCLOUD_USER / MW_E2E_NEXTCLOUD_APP_PASSWORD_FILE   an account and its app password
// The server reaches it as https://8.8.8.8/ through an egress route this spec
// adds for the `nextcloud` class (an in-process CONNECT proxy), so the address
// policy is used, not bypassed. The file is larger than the 50 MB attachment
// limit, so the composer offers the share link instead of refusing it.
const origin=process.env['MW_E2E_NEXTCLOUD_ORIGIN'];
const caFile=process.env['MW_E2E_NEXTCLOUD_CA'];
const user=process.env['MW_E2E_NEXTCLOUD_USER'];
const passwordFile=process.env['MW_E2E_NEXTCLOUD_APP_PASSWORD_FILE'];
const fixtureHost=process.env['MW_E2E_FIXTURE_HOST']??'127.0.0.1';
const shots=process.env['MW_E2E_SCREENSHOT_DIR'];

async function connectProxy(){
 const [host,port]=origin!.split(':') as [string,string];const sockets=new Set<Duplex>();
 const proxy=httpServer();proxy.on('connect',(request,socket,head)=>{
  if(request.url!=='8.8.8.8:443'){socket.destroy();return;}
  const upstream=connect(Number(port),host,()=>{socket.write('HTTP/1.1 200 Connection Established\r\n\r\n');if(head.length)upstream.write(head);socket.pipe(upstream);upstream.pipe(socket);});
  sockets.add(socket);sockets.add(upstream);socket.on('close',()=>{sockets.delete(socket);upstream.destroy();});upstream.on('close',()=>{sockets.delete(upstream);socket.destroy();});upstream.on('error',()=>socket.destroy());socket.on('error',()=>upstream.destroy());
 });
 await new Promise<void>(done=>proxy.listen(0,'0.0.0.0',done));
 return {port:(proxy.address() as {port:number}).port,close:()=>new Promise<void>(done=>{for(const socket of sockets)socket.destroy();proxy.close(()=>done());})};
}

/** Ask Nextcloud directly, not through the server under test. */
function cloud(method:string,path:string,password:string|null,headers:Record<string,string>={}):Promise<{status:number;body:Buffer}>{
 const [host,port]=origin!.split(':') as [string,string];
 return new Promise((resolve,reject)=>{const req=httpsRequest({host,port:Number(port),path,method,headers:{host:'8.8.8.8',...(password?{authorization:`Basic ${Buffer.from(`${user}:${password}`).toString('base64')}`}:{}),...headers},ca:readFileSync(caFile!,'utf8'),checkServerIdentity:()=>undefined},response=>{const chunks:Buffer[]=[];response.on('data',chunk=>chunks.push(chunk as Buffer));response.on('end',()=>resolve({status:response.statusCode??0,body:Buffer.concat(chunks)}));});req.on('error',reject);req.end();});
}

test('a file over the attachment limit goes to Nextcloud as a password link; a new calendar is created there',async({page,playwright,baseURL},info)=>{
 test.skip(!origin||!caFile||!user||!passwordFile,'set MW_E2E_NEXTCLOUD_* to run against a live Nextcloud');
 test.setTimeout(240_000);
 const password=readFileSync(passwordFile!,'utf8').trim();
 const stamp=`${Date.now()}-${info.project.name}`;
 const shot=async(name:string)=>{await page.screenshot({path:shots?`${shots}/${info.project.name}-${name}.png`:info.outputPath(`${name}.png`),fullPage:true});};
 const admin=await playwright.request.newContext({...baseURL?{baseURL}:{}});
 const proxy=await connectProxy();const route=`t39share-route-${stamp}`;const instanceId=`t39-share-${info.project.name}`;let previous:string|undefined;
 const name=`Large share ${stamp}.bin`;const size=50_000_000+1_234_567;
 try{
  expect((await admin.post('/admin/login',{data:{username:'root',password:'hunter2'}})).status()).toBe(200);
  expect((await admin.post('/admin/step-up',{data:{password:'hunter2'}})).status()).toBe(200);
  const routes=await(await admin.get('/admin/egress/proxies')).json() as {proxies:Array<{id:string;active:boolean}>};
  previous=routes.proxies.find(item=>item.active)?.id;
  expect((await admin.post('/admin/egress/proxies',{data:{id:route,scheme:'http',host:fixtureHost,port:proxy.port,scope:'global',trafficClasses:['nextcloud']}})).ok()).toBe(true);
  expect((await admin.post(`/admin/egress/proxies/${route}/activate`)).ok()).toBe(true);
  const instances=await(await admin.get('/admin/nextcloud/instances')).json() as {instances:Array<{id:string;etag:string}>};
  const existing=instances.instances.find(item=>item.id===instanceId);
  const approved=await admin.put(`/admin/nextcloud/instances/${instanceId}`,{data:{name:`Share cloud ${info.project.name}`,baseUrl:'https://8.8.8.8/',enabled:true,caPem:readFileSync(caFile!,'utf8')},headers:existing?{'if-match':existing.etag}:{'if-none-match':'*'}});
  expect(approved.ok(),await approved.text()).toBe(true);

  await page.goto('/');await signInManually(page,{username:'testuser',password:'testpass'});
  await expect(page.getByRole('button',{name:'Compose',exact:true}).first()).toBeVisible();
  const menu=page.getByRole('button',{name:'Open folders and apps',exact:true});if(await menu.isVisible())await menu.click();
  const show=page.getByRole('button',{name:'Show folders sidebar'});if(await show.isVisible())await show.click();
  await page.getByRole('button',{name:'Settings',exact:true}).click();
  await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Nextcloud accounts',exact:true}).click();
  const card=page.getByRole('heading',{name:`Share cloud ${info.project.name}`,exact:true}).locator('..');
  await card.getByRole('button',{name:/^(Link account|Verify link again)$/}).click();
  await page.getByLabel('Nextcloud app-password username',{exact:true}).fill(user!);
  await page.getByLabel('Nextcloud app password',{exact:true}).fill(password);
  await page.getByRole('button',{name:'Verify and link',exact:true}).click();
  await expect(card.getByText('Linked and ready',{exact:true})).toBeVisible();

  // A new calendar, created in Nextcloud from Settings and connected here.
  await card.getByRole('button',{name:'Contacts and calendars',exact:true}).click();
  const calendar=`Team plans ${stamp}`;
  await page.getByLabel('Name of the new collection',{exact:true}).fill(calendar);
  await page.getByRole('button',{name:'Create in Nextcloud',exact:true}).click();
  await expect(page.getByRole('status').filter({hasText:'was created in Nextcloud and connected'})).toBeVisible();
  await expect(page.getByText(new RegExp(`${calendar} — Ready`))).toBeVisible();
  await shot('nextcloud-calendar-created');
  const home=await cloud('PROPFIND',`/remote.php/dav/calendars/${encodeURIComponent(user!)}/`,password,{depth:'1'});
  expect(home.status).toBe(207);expect(home.body.toString('utf8')).toContain(calendar);
  await page.getByRole('button',{name:'Back to Nextcloud accounts',exact:true}).click();

  // Compose: the file is over the limit, so the share-link panel takes it.
  await page.getByRole('button',{name:'Compose',exact:true}).first().click();
  const compose=composeSurface(page);
  await compose.getByRole('tab',{name:'Insert',exact:true}).click();
  // On a phone the Attachments group folds into a dropdown (t38-e5).
  const input=await ribbonControl(compose,'Attachments',compose.getByLabel('Attach a file',{exact:true}));
  await expect(input).toBeEnabled();
  await input.setInputFiles({name,mimeType:'application/octet-stream',buffer:Buffer.alloc(size,7)});
  const panel=page.getByTestId('compose-share-link');
  await expect(panel).toContainText('larger than the attachment limit');
  await expect(page.getByTestId('compose-attach').getByRole('alert')).toHaveCount(0);
  // The instance list loads after the panel; one linked instance is chosen for
  // you, with several (one per browser project here) the user picks. The
  // select's accessible name includes its current choice.
  const picker=panel.getByRole('combobox',{name:/^Nextcloud instance/});
  await expect(picker).toBeVisible();await picker.selectOption(instanceId);
  await panel.getByLabel('Protect with a password',{exact:true}).check();
  await panel.getByLabel('Share password').fill('Share-Pass-39-x');
  await panel.getByLabel('Set an expiry date',{exact:true}).check();
  const at=new Date(Date.now()+7*86_400_000);const expiry=`${at.getFullYear()}-${String(at.getMonth()+1).padStart(2,'0')}-${String(at.getDate()).padStart(2,'0')}`;
  await panel.getByLabel('Expiry date',{exact:true}).fill(expiry);
  await shot('nextcloud-share-panel');
  await panel.getByRole('button',{name:'Upload and insert link',exact:true}).click();
  await expect(panel.getByTestId('nc-upload-progress')).toBeVisible();
  await expect(page.getByTestId('compose-share-link')).toHaveCount(0,{timeout:180_000});
  const body=compose.locator('.ProseMirror');
  await expect(body).toContainText(`${name} (51.2 MB): https://8.8.8.8/s/`);
  await expect(body).toContainText(`The link works until ${expiry}. The link is password-protected.`);
  const href=await body.locator('a[href^="https://8.8.8.8/s/"]').getAttribute('href');expect(href).toBeTruthy();
  await expect(page.getByTestId('compose-attachments')).toHaveCount(0);
  await shot('nextcloud-share-link-inserted');

  // Nextcloud holds the whole file, and the link it made has the password and expiry.
  const stored=await cloud('PROPFIND',`/remote.php/dav/files/${encodeURIComponent(user!)}/Mail%20attachments/${encodeURIComponent(name)}`,password,{depth:'0'});
  expect(stored.status).toBe(207);expect(stored.body.toString('utf8')).toContain(`<d:getcontentlength>${size}</d:getcontentlength>`);
  const shares=JSON.parse((await cloud('GET',`/ocs/v2.php/apps/files_sharing/api/v1/shares?format=json&path=${encodeURIComponent(`/Mail attachments/${name}`)}`,password,{'OCS-APIRequest':'true'})).body.toString('utf8')) as {ocs:{data:Array<{url:string;expiration:string|null;password?:string|null;share_with?:string|null}>}};
  const share=shares.ocs.data.find(row=>row.url===href);expect(share,JSON.stringify(shares)).toBeTruthy();
  expect(share!.expiration?.startsWith(expiry)).toBe(true);
  expect(Boolean(share!.password)||Boolean(share!.share_with)).toBe(true);
  const token=href!.split('/').pop()!;
  const anonymous=await cloud('GET',`/s/${token}/download`,null);
  expect(anonymous.body.length).not.toBe(size);
 }finally{
  await admin.post(previous?`/admin/egress/proxies/${previous}/activate`:'/admin/egress/proxies/deactivate');
  await admin.post(`/admin/egress/proxies/${route}/delete`);
  await proxy.close();await admin.dispose();
 }
});
