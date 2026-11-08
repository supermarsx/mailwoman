import {test,expect,type Page} from '../csrf.ts';
import {request as pwRequest} from '@playwright/test';
import net,{createServer,type Socket} from 'node:net';
import {deflateSync} from 'node:zlib';
import {ADMIN} from '../admin/admin-path.ts';
import {sendRawSmtp} from '../crypto-helpers.ts';
import {signInManually,messageRow,composeSurface,withUserSetting} from '../helpers.ts';

/**
 * Reply with "Include images" end to end: a newsletter with three remote images
 * (png, jpeg, gif), two cid images in multipart/related, a data: image and a 1x1
 * tracking pixel is replied to under every `mail.replyImages` policy. The remote
 * images come through the real server image proxy: the egress proxy configured
 * here is a CONNECT tunnel to this file's own HTTP server.
 */
const crc=(b:Buffer)=>{let c=0xffffffff;for(const v of b){c^=v;for(let i=0;i<8;i++)c=c&1?(c>>>1)^0xedb88320:c>>>1;}return(c^0xffffffff)>>>0;};
const chunk=(name:string,data:Buffer)=>{const bytes=Buffer.concat([Buffer.from(name),data]),len=Buffer.alloc(4),sum=Buffer.alloc(4);len.writeUInt32BE(data.length);sum.writeUInt32BE(crc(bytes));return Buffer.concat([len,bytes,sum]);};
function png(w:number,h:number,rgb:[number,number,number]):Buffer {
 const head=Buffer.alloc(13);head.writeUInt32BE(w);head.writeUInt32BE(h,4);head[8]=8;head[9]=6;
 const row=1+w*4,data=Buffer.alloc(h*row);for(let y=0;y<h;y++)for(let x=0;x<w;x++){const p=y*row+1+x*4;data[p]=rgb[0];data[p+1]=rgb[1];data[p+2]=rgb[2];data[p+3]=255;}
 return Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),chunk('IHDR',head),chunk('IDAT',deflateSync(data)),chunk('IEND',Buffer.alloc(0))]);
}
/** A solid-colour GIF: 2-colour palette, LZW with a clear code before every pixel. */
function gif(w:number,h:number):Buffer {
 const bits:number[]=[];const put=(code:number)=>{for(let i=0;i<3;i++)bits.push((code>>i)&1);};
 for(let i=0;i<w*h;i++){put(4);put(1);}put(4);put(5);
 const bytes:number[]=[];for(let i=0;i<bits.length;i+=8){let v=0;for(let j=0;j<8;j++)v|=(bits[i+j]??0)<<j;bytes.push(v);}
 const blocks:number[]=[];for(let i=0;i<bytes.length;i+=255){const part=bytes.slice(i,i+255);blocks.push(part.length,...part);}
 const header=Buffer.alloc(13);header.write('GIF89a');header.writeUInt16LE(w,6);header.writeUInt16LE(h,8);header[10]=0x80;
 return Buffer.concat([header,Buffer.from([0,200,0,0,0,255,0x2c,0,0,0,0]),Buffer.from([w&255,w>>8,h&255,h>>8,0,2]),Buffer.from(blocks),Buffer.from([0,0x3b])]);
}
const JPEG_SOURCE='jpeg';
async function jpeg(page:Page,w:number,h:number):Promise<Buffer> {
 const b64=await page.evaluate(([w,h])=>{const c=document.createElement('canvas');c.width=w!;c.height=h!;const x=c.getContext('2d')!;x.fillStyle='#d83';x.fillRect(0,0,w!,h!);return c.toDataURL('image/jpeg',0.8).split(',')[1]!;},[w,h]);
 return Buffer.from(b64!,'base64');
}
async function request(page:Page,path:string,method='GET',data?:unknown,etag?:string){return page.evaluate(async({path,method,data,etag})=>{const token=document.cookie.split('; ').find(v=>v.startsWith('mw_csrf='))?.slice(8);const response=await fetch(path,{method,headers:{...(data===undefined?{}:{'content-type':'application/json'}),...(token?{'x-csrf-token':decodeURIComponent(token)}:{}),...(etag?{'if-match':etag}:{})},body:data===undefined?null:JSON.stringify(data)});const text=await response.text();return{status:response.status,etag:response.headers.get('etag'),body:text?JSON.parse(text):null};},{path,method,data,etag});}

/** Every message in `address`'s INBOX whose Subject contains `subject`, as raw source (Greenmail: the address is login and password). */
function inboxSource(address:string,subject:string):Promise<string[]> {
 return new Promise((resolve,reject)=>{
  const sock=net.createConnection({host:'127.0.0.1',port:Number(process.env['MW_E2E_IMAP_PORT']??3143)});sock.setEncoding('latin1');
  let buf='',step=0,ids:string[]=[];const out:string[]=[];const fail=(e:Error)=>{sock.destroy();reject(e);};
  const send=(line:string)=>{step++;buf='';sock.write(`a${step} ${line}\r\n`);};
  sock.setTimeout(15000,()=>fail(new Error('IMAP timeout')));sock.on('error',fail);
  sock.on('data',(chunk:string)=>{buf+=chunk;if(step===0){if(!buf.includes('\r\n'))return;send(`LOGIN "${address}" "${address}"`);return;}
   const done=new RegExp(`(^|\r\n)a${step} (OK|NO|BAD)[^\r\n]*\r\n$`).exec(buf);if(!done)return;
   if(done[2]!=='OK'){if(step===1){sock.end();return resolve([]);}return fail(new Error(buf.slice(-200)));}
   const reply=buf;if(step===1)return send('SELECT INBOX');if(step===2)return send(`SEARCH SUBJECT "${subject}"`);
   if(step===3)ids=(/\* SEARCH([^\r\n]*)/.exec(reply)?.[1]??'').trim().split(/\s+/).filter(v=>v);
   else{const lit=/\{(\d+)\}\r\n/.exec(reply);if(lit)out.push(reply.slice(lit.index+lit[0].length,lit.index+lit[0].length+Number(lit[1])));}
   const next=ids[step-3];if(next!==undefined)return send(`FETCH ${next} BODY.PEEK[]`);sock.end();resolve(out);});
 });
}
async function editorImages(page:Page){
 return page.evaluate(()=>{const docs:Document[]=[];for(const f of Array.from(document.querySelectorAll('iframe')))try{if(f.contentDocument)docs.push(f.contentDocument);}catch{/* cross-origin */}
  return docs.flatMap(d=>Array.from(d.querySelectorAll('img')).map(img=>({alt:img.alt,w:img.naturalWidth,src:img.getAttribute('src')?.slice(0,40)??''})));});
}
const fixtureHost=process.env['MW_E2E_FIXTURE_HOST']??'127.0.0.1';

test('reply with Include images keeps every image, in the editor, the draft and the sent mail',async({page,browser,baseURL},info)=>{
 test.setTimeout(480_000);
 const sockets=new Set<Socket>(),requests:string[]=[];
 const assets:Record<string,{type:string;body:Buffer}>={
  '/a.png':{type:'image/png',body:png(120,60,[200,40,40])},
  '/b.gif':{type:'image/gif',body:gif(90,50)},
  '/pixel.gif':{type:'image/gif',body:gif(1,1)},
 };
 const probe=await (await browser.newContext()).newPage();await probe.setContent('<p>x</p>');assets['/c.jpg']={type:'image/jpeg',body:await jpeg(probe,100,70)};await probe.context().close();
 const proxy=createServer(socket=>{sockets.add(socket);socket.on('error',()=>undefined);socket.on('close',()=>sockets.delete(socket));let buffer='';let connected=false;socket.on('data',data=>{buffer+=data.toString('latin1');for(;;){const end=buffer.indexOf('\r\n\r\n');if(end<0)return;const head=buffer.slice(0,end);buffer=buffer.slice(end+4);if(!connected){if(!head.startsWith('CONNECT ')){socket.destroy();return;}connected=true;socket.write('HTTP/1.1 200 Connection Established\r\n\r\n');continue;}requests.push(head.split('\r\n')[0]!);const path=/^GET (\/[^ ?]*)/.exec(head)?.[1]??'';const asset=assets[path];if(!asset){socket.end('HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n');return;}socket.write(Buffer.concat([Buffer.from(`HTTP/1.1 200 OK\r\nContent-Type: ${asset.type}\r\nContent-Length: ${asset.body.length}\r\nConnection: close\r\n\r\n`),asset.body]));socket.end();}});});
 await new Promise<void>(resolve=>proxy.listen(0,'0.0.0.0',resolve));const address=proxy.address();if(!address||typeof address==='string')throw new Error('proxy address');
 const adminContext=await browser.newContext({baseURL:baseURL!}),admin=await adminContext.newPage();
 const id=`reply-images-${info.project.name}-${Date.now()}`;let active:string|undefined;let configured=false;let savedMode:Awaited<ReturnType<typeof request>>|undefined;
 const restore:Array<()=>Promise<void>>=[];
 try{
  await admin.goto(ADMIN);await admin.getByLabel('Admin username').fill('root');await admin.getByLabel('Password',{exact:true}).fill('hunter2');const signedIn=admin.waitForResponse(r=>r.url().endsWith(`${ADMIN}/login`)&&r.request().method()==='POST');await admin.getByRole('button',{name:'Sign in',exact:true}).click();expect((await signedIn).status()).toBe(200);await admin.waitForFunction(()=>document.cookie.includes('mw_csrf='));
  expect((await request(admin,`${ADMIN}/step-up`,'POST',{password:'hunter2'})).status).toBe(200);
  const proxies=await request(admin,`${ADMIN}/egress/proxies`);active=(proxies.body.proxies as {id:string;active:boolean}[]).find(row=>row.active)?.id;
  expect((await request(admin,`${ADMIN}/egress/proxies`,'POST',{id,scheme:'http',host:fixtureHost,port:address.port,allowPlaintext:true,trafficClasses:['remote_content'],scope:'global'})).status).toBe(200);configured=true;
  expect((await request(admin,`${ADMIN}/egress/proxies/${id}/activate`,'POST')).status).toBe(200);
  savedMode=await request(admin,`${ADMIN}/settings/privacy.remoteContent`);
  expect((await request(admin,`${ADMIN}/settings/privacy.remoteContent`,'PUT',{globalValue:process.env['MW_RI_MODE']??'proxy'},savedMode.etag!)).status).toBe(200);

  const subject=`Reply images ${id}`;
  const dataImage=png(30,20,[40,40,200]).toString('base64');
  const html=`<div><h2>Newsletter</h2><p>Remote: <img alt="r-png" src="http://1.2.3.4/a.png" width="120" height="60"> <img alt="r-jpg" src="http://1.2.3.4/c.jpg" width="100" height="70"> <img alt="r-gif" src="http://1.2.3.4/b.gif" width="90" height="50"></p><p>Inline: <img alt="cid-one" src="cid:one@x.test" width="40" height="40"> <img alt="cid-two" src="cid:two@x.test" width="50" height="30"></p><p>Data: <img alt="data-img" src="data:image/png;base64,${dataImage}"></p><img alt="tracker" src="http://1.2.3.4/pixel.gif" width="1" height="1"></div>`;
  const cid=(name:string,body:Buffer)=>`--rel\r\nContent-Type: image/png; name="${name}.png"\r\nContent-Transfer-Encoding: base64\r\nContent-ID: <${name}@x.test>\r\nContent-Disposition: inline; filename="${name}.png"\r\n\r\n${body.toString('base64').replace(/(.{76})/g,'$1\r\n')}\r\n`;
  await sendRawSmtp({from:'news@example.test',message:`From: News <news@example.test>\r\nTo: testuser@example.org\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <${id}@example.test>\r\nSubject: ${subject}\r\nMIME-Version: 1.0\r\nContent-Type: multipart/related; boundary=rel\r\n\r\n--rel\r\nContent-Type: text/html; charset=utf-8\r\n\r\n${html}\r\n${cid('one',png(40,40,[20,160,60]))}${cid('two',png(50,30,[160,60,160]))}--rel--\r\n`});
  await page.waitForTimeout(3500);await signInManually(page,{username:'testuser',password:'testpass'});
  await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
  const consoleErrors:string[]=[],failures:string[]=[];page.on('console',m=>{if(m.type()==='error'||m.type()==='warning')consoleErrors.push(m.text());});page.on('requestfailed',r=>failures.push(`${r.url()} ${r.failure()?.errorText}`));page.on('response',r=>{if(r.status()>=400)failures.push(`${r.status()} ${r.url()}`);});

  for(const policy of (process.env['MW_RI_POLICIES']??'ask,deny,allow').split(',') as Array<'ask'|'allow'|'deny'|'trusted'>){
   const undo=await withUserSetting(pwRequest,baseURL!,'mail.replyImages',policy);restore.push(undo);
   await page.goto('/');
   await expect(async()=>{await page.getByRole('navigation',{name:'Mailboxes'}).getByRole('button',{name:'Inbox',exact:true}).click();await expect(messageRow(page,subject)).toBeVisible({timeout:3000});}).toPass({timeout:60000});
   await messageRow(page,subject).click();
   await page.getByRole('button',{name:'Reply',exact:true}).first().click();
   const compose=composeSurface(page);await expect(compose).toBeVisible();
   const body=compose.locator('[contenteditable="true"], iframe').first();await expect(body).toBeVisible();
   if(policy==='ask'){
    const offer=compose.getByTestId('compose-image-offer');await expect(offer).toBeVisible();
    await compose.getByTestId('image-offer-include').click();
    await expect(offer).toBeHidden({timeout:60000});
   }
   await page.waitForTimeout(2500);
   await page.screenshot({path:info.outputPath(`reply-${policy}.png`),fullPage:true});
   const report=await page.evaluate(async()=>{
    const docs:Document[]=[document];for(const f of Array.from(document.querySelectorAll('iframe')))try{if(f.contentDocument)docs.push(f.contentDocument);}catch{/* cross-origin */}
    const out:Array<{alt:string;src:string;w:number;complete:boolean}>=[];
    for(const d of docs)for(const img of Array.from(d.querySelectorAll('img')))if(img.closest('[role="dialog"],[role="region"]')||d!==document)out.push({alt:img.alt,src:img.src.slice(0,60),w:img.naturalWidth,complete:img.complete});
    return out;
   });
   info.annotations.push({type:`images-${policy}`,description:JSON.stringify(report)});
   console.log(policy,JSON.stringify(report));
   if(policy==='deny')expect(report.filter(r=>r.w>0).length).toBe(0);
   else expect(report.filter(r=>r.w>0).length,JSON.stringify(report)).toBeGreaterThanOrEqual(5);
   if(policy==='allow'){
    await expect.poll(()=>page.evaluate(()=>localStorage.getItem('mw.compose.drafts.v2')??''),{timeout:60000}).toContain('cid:');
    await page.screenshot({path:info.outputPath('reply-allow-light.png'),fullPage:true});
    await page.reload();
    await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
    if(!await page.getByLabel('Subject',{exact:true}).isVisible().catch(()=>false)){await page.getByRole('button',{name:'Compose',exact:true}).click();await page.getByTestId('open-drafts').last().click();await page.getByRole('button',{name:`Re: ${subject}`,exact:true}).first().click();}
    await expect(page.getByLabel('Subject',{exact:true})).toHaveValue(`Re: ${subject}`);
    await expect(async()=>{const r=await editorImages(page);expect(r.filter(i=>i.w>0).length,JSON.stringify(r)).toBeGreaterThanOrEqual(5);}).toPass({timeout:30000});
    await page.screenshot({path:info.outputPath('reply-allow-resumed.png'),fullPage:true});
    await page.getByRole('button',{name:'Send',exact:true}).first().click();
    // The seeded sender is intentionally outside trusted domains. Complete the
    // normal send review instead of polling SMTP while its modal blocks send.
    const review=page.getByRole('alertdialog',{name:'Check before sending',exact:true});
    await expect(async()=>{
     if(await review.isVisible())await review.getByRole('button',{name:'Send anyway',exact:true}).click();
     await expect(composeSurface(page)).toBeHidden();
    }).toPass({timeout:30000});
    await page.waitForTimeout(4000);await page.screenshot({path:info.outputPath('after-send.png'),fullPage:true});console.log('AFTER-SEND',(await page.locator('[role=alert],[role=status],[role=dialog]').allInnerTexts()).join(' | ').slice(0,600));
    let sent:string[]=[];
    await expect(async()=>{sent=await inboxSource('news@example.test',subject);expect(sent.length).toBeGreaterThan(0);}).toPass({timeout:90000});
    const mime=sent[0]!;
    console.log('SENT-MIME-SUMMARY',JSON.stringify({length:mime.length,contentIds:mime.match(/^Content-ID:.*$/gim),contentTypes:mime.match(/^Content-Type:.*$/gim),imgSrc:mime.match(/<img[^>]*>/gi)?.map(v=>v.slice(0,160))}));
    const body=mime.replace(/=\r?\n/g,'').replace(/=3D/gi,'=');
    const srcs=[...body.matchAll(/<img[^>]*\ssrc="([^"]*)"/gi)].map(m=>m[1]??'');
    expect(srcs.length,'img tags in the sent mail').toBeGreaterThanOrEqual(5);
    for(const src of srcs){expect(src.startsWith('cid:'),`src ${src}`).toBe(true);expect(mime.toLowerCase()).toContain(`content-id: <${src.slice(4).toLowerCase()}>`);}
    return;
   }
   await page.keyboard.press('Escape');
   // Finish the discard confirmation before the next policy. Navigating away
   // with this modal open restores the previous reply, including its images.
   await compose.getByRole('button',{name:'Discard',exact:true}).click();
   const discard=page.getByRole('alertdialog',{name:'Discard this draft?',exact:true});
   await expect(discard).toBeVisible();
   await discard.getByRole('button',{name:'Discard',exact:true}).click();
   await expect(compose).toBeHidden();
   await page.goto('/');
  }
  console.log('console',JSON.stringify(consoleErrors),'failures',JSON.stringify(failures),'proxied',JSON.stringify(requests));
 }finally{
  try{
   const quiet=async<T>(work:Promise<T>)=>work.catch(()=>undefined);
   for(const undo of restore.reverse())await undo().catch(()=>undefined);
   if(configured){await quiet(request(admin,active?`${ADMIN}/egress/proxies/${active}/activate`:`${ADMIN}/egress/proxies/deactivate`,'POST'));await quiet(request(admin,`${ADMIN}/egress/proxies/${id}/delete`,'POST'));}
   if(savedMode?.status===200)await quiet((async()=>{const cur=await request(admin,`${ADMIN}/settings/privacy.remoteContent`);await request(admin,`${ADMIN}/settings/privacy.remoteContent`,'PUT',{globalValue:savedMode.body.globalValue,locked:savedMode.body.globalLocked,allowUserOverride:savedMode.body.allowUserOverride},cur.etag!);})());
  }finally{await adminContext.close();for(const socket of sockets)socket.destroy();await new Promise<void>(resolve=>proxy.close(()=>resolve()));}
 }
 void JPEG_SOURCE;
});
