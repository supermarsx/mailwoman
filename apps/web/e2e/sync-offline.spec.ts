import {test,expect,chromium} from '@playwright/test';
import {build} from 'vite';
import {createServer,type Server} from 'node:http';
import {createHash} from 'node:crypto';
import {readFile,mkdtemp} from 'node:fs/promises';
import {resolve,extname,sep} from 'node:path';
import type {AddressInfo} from 'node:net';
import type {Socket} from 'node:net';
import type {harness} from './syncOfflineHarness.ts';
declare global {interface Window {syncHarness:typeof harness}}
test.describe('encrypted sync and multi-window browser integration, loopback fixture',()=>{
 test.describe.configure({mode:'serial'});
 let server:Server,origin:string,offline=false,principal='verified_browser_owner_123456',setCount=0,submissions=0,uploads=0;
 let notes:Record<string,Record<string,unknown>>={};
 const sockets=new Set<Socket>();
 const output=resolve('.orchestration/sync-browser-build');
 let mailboxes:Record<string,boolean>={inbox:true};
 test.beforeAll(async()=>{
  test.setTimeout(180000);
  await build({logLevel:'warn',plugins:[{name:'sync-browser-entry',transformIndexHtml:{order:'pre',handler:html=>html.replace('/src/main.tsx','/e2e/syncOfflineHarness.ts')}}],build:{outDir:output,emptyOutDir:true}});
  const row=()=>({id:'m',threadId:'thread',subject:'Encrypted offline browser message',receivedAt:new Date().toISOString(),keywords:{},mailboxIds:mailboxes,bodyValues:{body:{value:'Offline browser body'}},htmlBody:[{partId:'body',type:'text/html',blobId:'body',size:20}]});
  server=createServer(async(req,res)=>{
   if(offline){res.destroy();return;}
   const path=new URL(req.url??'/',origin).pathname;
   const json=(value:unknown)=>{res.writeHead(200,{'content-type':'application/json','cache-control':'no-store'});res.end(JSON.stringify(value));};
   if(path==='/api/me'){res.setHeader('set-cookie','mw_csrf=csrf; SameSite=Lax; Path=/');return json({accountId:'a',username:'a@example.test',storageAuthority:principal,csrfToken:'csrf'});}
   if(path==='/jmap/session')return json({accounts:{a:{name:'Browser fixture',isPersonal:true,isReadOnly:false,accountCapabilities:{}}},primaryAccounts:{'urn:ietf:params:jmap:mail':'a'},capabilities:{},state:'s',username:'a@example.test',apiUrl:'/jmap/api',downloadUrl:'/jmap/download/{accountId}/{blobId}/{name}',uploadUrl:'/jmap/upload/{accountId}',eventSourceUrl:'/jmap/eventsource'});
   if(path==='/jmap/api'){
    let body='';for await(const part of req)body+=part;
    const request=JSON.parse(body) as {methodCalls:[string,Record<string,unknown>,string][]};
    if(req.headers['x-csrf-token']!=='csrf'){res.writeHead(403);res.end();return;}
    if(req.headers['x-mailwoman-storage-authority']&&req.headers['x-mailwoman-storage-authority']!==principal){res.writeHead(409,{'content-type':'application/json'});res.end('{"code":"accountChanged"}');return;}
    const methodResponses=request.methodCalls.map(([method,args,id])=>{
     if(method==='Email/query')return [method,{accountId:'a',queryState:'q1',canCalculateChanges:true,position:0,total:mailboxes['inbox']?1:0,ids:mailboxes['inbox']?['m']:[]},id];
     if(method==='Email/get')return [method,{accountId:'a',state:'e1',list:[row()],notFound:[]},id];
     if(method==='CalendarEvent/get')return [method,{accountId:'a',state:'ce1',list:[{id:'event',title:'Weekly meeting',start:'2026-03-01T09:00:00',timeZone:'Europe/Lisbon',duration:'PT1H',recurrenceRules:[{rrule:'FREQ=WEEKLY;BYDAY=SU'}],recurrenceOverrides:{},excludedRecurrenceDates:[]}],notFound:[]},id];
     if(method==='Note/get')return [method,{accountId:'a',state:'n1',list:[{id:'note',title:'Offline note',body:'Saved note body'},...Object.values(notes)],notFound:[]},id];
     if(method==='Note/set'){const created:Record<string,{id:string}>={};for(const [key,value] of Object.entries((args['create']??{}) as Record<string,object>)){const noteId='note-'+Object.keys(notes).length;notes[noteId]={...value,id:noteId};created[key]={id:noteId};}for(const [key,value] of Object.entries((args['update']??{}) as Record<string,object>)){if(!notes[key])throw new Error('Missing replayed note');notes[key]={...notes[key],...value};}return [method,{accountId:'a',created,updated:Object.fromEntries(Object.keys((args['update']??{}) as object).map(key=>[key,null])),destroyed:[],notCreated:null,notUpdated:null,notDestroyed:null},id];}
     if(method==='Email/set'){setCount++;if(args['update'])mailboxes={archive:true};return [method,{accountId:'a',oldState:'e1',newState:'e2',updated:{m:null},notUpdated:null,created:{draft:{id:'sent'}},notCreated:null},id];}
     if(method==='EmailSubmission/set'){submissions++;return [method,{accountId:'a',created:{send:{id:'submission'}},notCreated:null},id];}
     return ['error',{type:'unknownMethod'},id];
    });return json({sessionState:'s',methodResponses});
   }
   if(path==='/jmap/download/a/pdf/doc'){res.writeHead(200,{'content-type':'application/pdf'});res.end('0123456789');return;}
   if(path==='/jmap/upload/a'){uploads++;return json({accountId:'a',blobId:'uploaded',size:26,type:'text/plain'});}
   const file=resolve(output,path==='/'?'index.html':path.slice(1));
   if(!file.startsWith(output+sep)&&file!==output){res.writeHead(404);res.end();return;}
   try{const content=await readFile(file);const type=({'.js':'text/javascript','.html':'text/html','.css':'text/css','.wasm':'application/wasm','.woff2':'font/woff2'} as Record<string,string>)[extname(file)]??'application/octet-stream';res.writeHead(200,{'content-type':type,'cache-control':'no-cache'});res.end(content);}catch{res.writeHead(404);res.end();}
  });
  server.on('upgrade',(req,socket)=>{
   if(offline){socket.destroy();return;}
   const accept=createHash('sha1').update(String(req.headers['sec-websocket-key'])+'258EAFA5-E914-47DA-95CA-C5AB0DC85B11').digest('base64');
   socket.write(`HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ${accept}\r\n\r\n`);
   sockets.add(socket as Socket);socket.on('data',data=>{if((data[0]!&15)===8)socket.end(Buffer.from([0x88,0]));});socket.on('close',()=>sockets.delete(socket as Socket));socket.on('error',()=>undefined);
  });
  await new Promise<void>(done=>server.listen(0,'127.0.0.1',done));origin=`http://127.0.0.1:${(server.address() as AddressInfo).port}`;
 });
 test.beforeEach(()=>{offline=false;principal='verified_browser_owner_123456';mailboxes={inbox:true};setCount=0;submissions=0;uploads=0;notes={};});
 test.afterAll(async()=>{for(const socket of sockets)socket.destroy();server?.closeAllConnections();await new Promise<void>(done=>server?.close(()=>done()));});
 test('one push for two windows; encrypted cold reload; owned replay; refuse a changed principal',async({context})=>{
  test.setTimeout(120000);
  const left=await context.newPage(),right=await context.newPage();
  await left.goto(origin);await left.waitForFunction(()=>!!window.syncHarness);await left.evaluate(()=>window.syncHarness.start());
  await left.evaluate(async()=>{await navigator.serviceWorker.ready;});
  await right.goto(origin);await right.waitForFunction(()=>!!window.syncHarness);await right.evaluate(()=>window.syncHarness.start());
  await expect.poll(()=>sockets.size).toBe(1);
  expect(await left.evaluate(()=>window.syncHarness.mode())).toBe('shared-worker');
  await left.evaluate(()=>window.syncHarness.select(['m']));await expect.poll(()=>right.evaluate(()=>window.syncHarness.selection())).toEqual(['m']);
  await left.evaluate(()=>window.syncHarness.notify());await expect.poll(()=>right.evaluate(()=>window.syncHarness.notifications())).toEqual([{id:'cross-window',message:'Shared browser notification',action:'Apply'}]);
  await right.evaluate(()=>window.syncHarness.invokeNotification());await expect.poll(()=>left.evaluate(()=>window.syncHarness.actions())).toBe(1);
  await expect.poll(()=>left.evaluate(()=>window.syncHarness.notifications())).toEqual([]);
  await left.evaluate(async()=>{await window.syncHarness.body();await window.syncHarness.note();await window.syncHarness.attachment();await window.syncHarness.calendarData();});
  offline=true;for(const socket of sockets)socket.destroy();
  await context.setOffline(true);
  await right.close();
  await left.reload({waitUntil:'domcontentloaded'});await left.waitForFunction(()=>!!window.syncHarness);
  const cached=await left.evaluate(()=>window.syncHarness.start());
  expect(JSON.stringify(cached)).toContain('Encrypted offline browser message');
  const pdf=await left.evaluate(()=>window.syncHarness.pdfDocument());expect(pdf.html).toContain('../bootstrap.mjs');expect(pdf.policy).toContain("frame-ancestors 'self'");
  const sanitized=await left.evaluate(()=>window.syncHarness.sanitize());expect(sanitized).toContain('Offline readable body');expect(sanitized).not.toContain('<script');
  expect(await left.evaluate(()=>window.syncHarness.offline())).toBe(true);
  expect(JSON.stringify(await left.evaluate(()=>window.syncHarness.body()))).toContain('Offline browser body');
  expect(JSON.stringify(await left.evaluate(()=>window.syncHarness.note()))).toContain('Saved note body');
  const expanded=await left.evaluate(()=>window.syncHarness.calendar());expect((expanded.methodResponses[0]![1]['list'] as Record<string,unknown>[]).map(row=>row['instanceStart'])).toEqual(['2030-03-24T09:00:00.000Z','2030-03-31T08:00:00.000Z']);
  expect(await left.evaluate(()=>window.syncHarness.attachment('bytes=2-5'))).toEqual({status:206,range:'bytes 2-5/10',text:'2345'});
  const local=await left.evaluate(()=>window.syncHarness.file());expect(local.blobId).toMatch(/^mw-local:/);
  const created=await left.evaluate(()=>window.syncHarness.createNote());
  const localNote=(created.methodResponses[0]![1]['created'] as Record<string,{id:string}>)['new']!.id;
  await left.evaluate(id=>window.syncHarness.editNote(id),localNote);
  expect(JSON.stringify(await left.evaluate(()=>window.syncHarness.note()))).toContain('Edited offline');
  await left.evaluate(()=>window.syncHarness.queueMove());
  await expect(left.evaluate(()=>window.syncHarness.replay())).rejects.toThrow();expect(setCount).toBe(0);
  offline=false;await context.setOffline(false);
  const replay=await left.evaluate(()=>window.syncHarness.replay());expect(replay,JSON.stringify(await left.evaluate(()=>window.syncHarness.debug()))).toEqual({sent:3,failed:0});expect(Object.values(notes)[0]?.['title']).toBe('Edited offline');expect(setCount).toBe(1);
  principal='different_browser_owner_123456';
  await expect(left.evaluate(()=>window.syncHarness.query())).rejects.toThrow();
  expect(submissions).toBe(0);
 });
 test('BroadcastChannel elects one owner and replaces it after the leader window closes',async({context})=>{
  await context.addInitScript(()=>Object.defineProperty(window,'SharedWorker',{value:undefined,configurable:true}));
  const left=await context.newPage(),right=await context.newPage();
  await left.goto(origin);await left.waitForFunction(()=>!!window.syncHarness);await left.evaluate(()=>window.syncHarness.start());
  await right.goto(origin);await right.waitForFunction(()=>!!window.syncHarness);await right.evaluate(()=>window.syncHarness.start());
  expect(await right.evaluate(()=>window.syncHarness.mode())).toBe('leader');
  await expect.poll(()=>sockets.size).toBe(1);await left.close();
  await expect.poll(()=>sockets.size,{timeout:10000}).toBe(1);
  await expect(right.evaluate(()=>window.syncHarness.query())).resolves.toBeDefined();
 });
 test('replays a queued send with every Mailwoman tab closed using browser Background Sync',async()=>{
  test.setTimeout(60000);
  const profile=await mkdtemp(resolve('.orchestration/sync-profile-'));
  const context=await chromium.launchPersistentContext(profile,{channel:'chromium',headless:true,ignoreDefaultArgs:['--disable-background-networking']});
  try{
  const page=await context.newPage();await page.goto(origin);await page.waitForFunction(()=>!!window.syncHarness);await page.evaluate(()=>window.syncHarness.start());
  await page.evaluate(async()=>{await navigator.serviceWorker.ready;});
  offline=true;await context.setOffline(true);
  await page.evaluate(()=>window.syncHarness.createNote());
  const supported=await page.evaluate(()=>window.syncHarness.queueSend());expect(supported).toBe(true);
  expect(submissions).toBe(0);await page.close();expect(context.pages().every(value=>!value.url().startsWith(origin))).toBe(true);
  offline=false;await context.setOffline(false);
  await expect.poll(()=>submissions,{timeout:30000}).toBe(1);expect(uploads).toBe(1);expect(Object.values(notes)).toHaveLength(1);
  offline=true;await context.setOffline(true);
  const reopened=await context.newPage();await reopened.goto(origin,{waitUntil:'domcontentloaded'});await reopened.waitForFunction(()=>!!window.syncHarness);await reopened.evaluate(()=>window.syncHarness.start());
  expect(JSON.stringify(await reopened.evaluate(()=>window.syncHarness.note()))).toContain('Created offline');
  }finally{await context.close();}
 });

});
