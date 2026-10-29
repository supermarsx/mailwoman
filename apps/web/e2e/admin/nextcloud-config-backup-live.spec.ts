import {test,expect} from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import {createServer as httpServer} from 'node:http';
import {request as httpsRequest} from 'node:https';
import {connect} from 'node:net';
import type {Duplex} from 'node:stream';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';

// Live Nextcloud settings backup (t64). Needs a real, disposable Nextcloud behind TLS:
//   MW_E2E_NEXTCLOUD_ORIGIN   host:port of its HTTPS listener (certificate valid for IP 8.8.8.8)
//   MW_E2E_NEXTCLOUD_CA       PEM file of the CA that signed it
//   MW_E2E_NEXTCLOUD_USER / MW_E2E_NEXTCLOUD_APP_PASSWORD_FILE   an account and its app password
//   MW_E2E_NEXTCLOUD_READONLY  optional: a folder shared read-only with that account
// The server reaches it as https://8.8.8.8/ through an egress proxy route this spec
// adds for the `nextcloud` class, so the address policy is exercised, not bypassed.
const origin=process.env['MW_E2E_NEXTCLOUD_ORIGIN'];
const caFile=process.env['MW_E2E_NEXTCLOUD_CA'];
const user=process.env['MW_E2E_NEXTCLOUD_USER'];
const passwordFile=process.env['MW_E2E_NEXTCLOUD_APP_PASSWORD_FILE'];
const fixtureHost=process.env['MW_E2E_FIXTURE_HOST']??'host.docker.internal';
const screenshots=process.env['MW_E2E_SCREENSHOT_DIR'];

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

/** Read a file straight from Nextcloud's WebDAV, independent of the server under test. */
function cloudGet(path:string,password:string):Promise<{status:number;body:Buffer}>{
  const [host,port]=origin!.split(':') as [string,string];
  return new Promise((resolve,reject)=>{const req=httpsRequest({host,port:Number(port),path,method:'GET',headers:{host:'8.8.8.8',authorization:`Basic ${Buffer.from(`${user}:${password}`).toString('base64')}`},ca:readFileSync(caFile!,'utf8'),checkServerIdentity:()=>undefined},response=>{const chunks:Buffer[]=[];response.on('data',chunk=>chunks.push(chunk as Buffer));response.on('end',()=>resolve({status:response.statusCode??0,body:Buffer.concat(chunks)}));});req.on('error',reject);req.end();});
}

test('settings backup: configure Nextcloud, test the connection, back up and find the verified file in Nextcloud',async({page,playwright,baseURL})=>{
  test.skip(!origin||!caFile||!user||!passwordFile,'set MW_E2E_NEXTCLOUD_* to run against a live Nextcloud');
  const password=readFileSync(passwordFile!,'utf8').trim();
  const folder=`Backups/Mailwoman-e2e-${Date.now()}`;
  const admin=await playwright.request.newContext({...baseURL?{baseURL}:{}});
  const proxy=await connectProxy();const route=`ncbackup-route-${Date.now()}`;let previous:string|undefined;
  try{
    expect((await admin.post(`${ADMIN}/login`,{data:{username:'root',password:'hunter2'}})).status()).toBe(200);
    expect((await admin.post(`${ADMIN}/step-up`,{data:{password:'hunter2'}})).status()).toBe(200);
    const routes=await(await admin.get(`${ADMIN}/egress/proxies`)).json() as {proxies:Array<{id:string;active:boolean}>};
    previous=routes.proxies.find(item=>item.active)?.id;
    expect((await admin.post(`${ADMIN}/egress/proxies`,{data:{id:route,scheme:'http',host:fixtureHost,port:proxy.port,scope:'global',trafficClasses:['nextcloud']}})).ok()).toBe(true);
    expect((await admin.post(`${ADMIN}/egress/proxies/${route}/activate`)).ok()).toBe(true);

    await page.goto(ADMIN);
    await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');
    await page.getByRole('button',{name:'Sign in',exact:true}).click();
    await page.goto(`${ADMIN}#/configuration-transfer`);
    await page.getByRole('tab',{name:'Backup destinations',exact:true}).click();
    const card=page.getByRole('region',{name:'Settings backups to Nextcloud'}).or(page.locator('section[aria-labelledby="ncbackup-title"]')).first();
    await expect(card.getByRole('heading',{name:'Settings backups to Nextcloud'})).toBeVisible();
    await expect(page.getByRole('link',{name:'How to create an app password'})).toHaveAttribute('href',/docs\.nextcloud\.com/);

    // A wrong app password: saved, then the test names the sign-in step.
    await card.getByLabel(/^Nextcloud server URL/).fill('https://8.8.8.8/');
    await card.getByLabel(/^Nextcloud username/).fill(user!);
    await card.getByLabel(/^App password/).fill('Wrong-Wrong-Wrong-Wrong-Wrong');
    await card.getByLabel(/^Target folder/).fill(folder);
    await card.getByLabel(/^Copies to keep/).fill('2');
    await card.getByText('Private certificate authority').click();
    await card.getByLabel('Additional CA certificates (PEM)').fill(readFileSync(caFile!,'utf8'));
    await card.getByRole('button',{name:'Save destination'}).click();
    const confirmation=page.getByRole('dialog',{name:'Confirm it is you'});
    // The browser session has no step-up yet: saving a credential asks for one.
    await expect(confirmation).toBeVisible();await confirmation.getByLabel('Password',{exact:true}).fill('hunter2');await confirmation.getByRole('button',{name:'Confirm and continue'}).click();
    await expect(card.getByText('Nextcloud destination saved.')).toBeVisible();
    await expect(card.getByLabel(/^App password/)).toHaveValue('');
    await card.getByRole('button',{name:'Test connection'}).click();
    await expect(card.getByRole('alert')).toContainText('Sign-in failed: Nextcloud rejected the username or app password.');
    if(screenshots)await page.screenshot({path:`${screenshots}/ncbackup-wrong-password.png`,fullPage:true});

    // The real app password: the test creates the nested folder and cleans up its probe.
    await card.getByLabel(/^App password/).fill(password);
    await card.getByRole('button',{name:'Save destination'}).click();
    await expect(card.getByText('Nextcloud destination saved.')).toBeVisible();
    await card.getByRole('button',{name:'Test connection'}).click();
    await expect(card.getByRole('status')).toContainText(/Connection works: signed in, the folder was created/);
    if(screenshots)await page.screenshot({path:`${screenshots}/ncbackup-test-ok.png`,fullPage:true});
    const body=await page.content();expect(body).not.toContain(password);
    const view=await(await admin.get(`${ADMIN}/config/backup/nextcloud`)).text();
    expect(view).not.toContain(password);expect(view).toContain('"hasAppPassword":true');

    // Three backups with keep=2: each is verified, the oldest is pruned.
    const files:string[]=[];
    for(let index=0;index<3;index++){
      if(index)await page.waitForTimeout(1100);
      await card.getByRole('button',{name:'Back up now'}).click();
      const status=card.getByRole('status');
      await expect(status).toContainText(/Uploaded and verified mailwoman-settings-\d{8}T\d{6}Z\.json/);
      files.push(/(mailwoman-settings-\d{8}T\d{6}Z\.json)/.exec(await status.innerText())![1]!);
    }
    await expect(card.getByText(new RegExp(`^Last backup: ${files[2]!.replace(/\./g,'\\.')}`))).toBeVisible();
    if(screenshots)await page.screenshot({path:`${screenshots}/ncbackup-backed-up.png`,fullPage:true});
    const base=`/remote.php/dav/files/${encodeURIComponent(user!)}/${folder.split('/').map(encodeURIComponent).join('/')}/`;
    const newest=await cloudGet(base+files[2]!,password);
    expect(newest.status).toBe(200);
    const parsed=JSON.parse(newest.body.toString('utf8')) as {bundle:{format:string;version:number}};
    expect(parsed.bundle.format).toBe('mailwoman-settings');
    expect((await cloudGet(base+files[1]!,password)).status).toBe(200);
    expect((await cloudGet(base+files[0]!,password)).status).toBe(404);
    const audit=JSON.parse((await(await admin.get(`${ADMIN}/config/backup/nextcloud`)).text()) as string) as {lastRun:{sha256:string}};
    expect(audit.lastRun.sha256).toBe(createHash('sha256').update(newest.body).digest('hex'));

    // A folder the account may not create is reported as such.
    const readonly=process.env['MW_E2E_NEXTCLOUD_READONLY'];
    if(readonly){
      await card.getByLabel(/^Target folder/).fill(`${readonly}/cannot-create`);
      await card.getByRole('button',{name:'Save destination'}).click();
      await expect(card.getByText('Nextcloud destination saved.')).toBeVisible();
      await card.getByRole('button',{name:'Test connection'}).click();
      await expect(card.getByRole('alert')).toContainText('Folder access failed: The folder is missing and this account may not create it (HTTP 403)');
      if(screenshots)await page.screenshot({path:`${screenshots}/ncbackup-folder-denied.png`,fullPage:true});
    }
    await card.getByRole('button',{name:'Remove destination'}).click();
    await expect(card.getByText('Nextcloud destination removed.')).toBeVisible();
  }finally{
    await admin.post(previous?`${ADMIN}/egress/proxies/${previous}/activate`:`${ADMIN}/egress/proxies/deactivate`);
    await admin.post(`${ADMIN}/egress/proxies/${route}/delete`);
    await proxy.close();await admin.dispose();
  }
});
