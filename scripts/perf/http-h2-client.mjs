// HTTP/2 client mode of the load generator. It plugs into measure() as an
// alternative transport that returns fetch-shaped responses (status, ok,
// body.getReader(), json()), so the request templates, response validators,
// 503 classification and byte limits are the single code path used for
// HTTP/1.1 as well. Nothing here changes the default: HTTP/1.1 stays the
// default unless an h2 configuration is passed.
//
// Cleartext (`h2c`) uses prior knowledge on the target's plain listener. `tls`
// negotiates h2 by ALPN and verifies the certificate against a rig CA given
// explicitly; the only way to skip verification is the loudly named
// `rigOnlyInsecureTlsNoVerify` flag.
import http2 from 'node:http2';
import {readFileSync} from 'node:fs';
import {Readable} from 'node:stream';
import zlib from 'node:zlib';
import {performance} from 'node:perf_hooks';

const {NGHTTP2_REFUSED_STREAM,NGHTTP2_CANCEL}=http2.constants;
export const MAX_H2_CONNECTIONS=20000;
const MAX_RECONNECTS=1000;

// browser-like: one pool per simulated client (key = target + client index),
//   `connections` (default 1) HTTP/2 connections per client, many streams each.
// proxy-like: a few connections per target shared by all simulated clients
//   (like an nginx/Caddy/Traefik upstream); the client address still travels in
//   the forwarded header of each request.
export const H2_PRESETS=Object.freeze({
 'browser-like':Object.freeze({perClient:true,connections:1,maxStreams:100}),
 'proxy-like':Object.freeze({perClient:false,connections:4,maxStreams:100}),
});

const integer=(value,min,max)=>Number.isInteger(value)&&value>=min&&value<=max;
export function validateH2(input){
 if(!input||typeof input!=='object')throw new Error('Invalid HTTP/2 configuration');
 const mode=input.mode,preset=input.preset??'proxy-like';
 if(!['h2c','tls'].includes(mode))throw new Error('HTTP/2 mode must be h2c or tls');
 if(!Object.hasOwn(H2_PRESETS,preset)&&preset!=='custom')throw new Error('Unknown HTTP/2 preset');
 const base=H2_PRESETS[preset]??H2_PRESETS['proxy-like'];
 const perClient=preset==='custom'?input.perClient===true:base.perClient;
 const connections=input.connections??base.connections,maxStreams=input.maxStreams??base.maxStreams;
 if(!integer(connections,1,64)||!integer(maxStreams,1,1000))throw new Error('Invalid HTTP/2 connection model');
 const insecure=input.rigOnlyInsecureTlsNoVerify===true;
 if(mode==='tls'){
  const hasCa=typeof input.caPem==='string'&&input.caPem.includes('BEGIN CERTIFICATE');
  if(hasCa===insecure)throw new Error('HTTP/2 over TLS needs exactly one of an explicit rig CA or rigOnlyInsecureTlsNoVerify');
 }else if(input.caPem!==undefined||insecure||input.servername!==undefined)throw new Error('TLS options need HTTP/2 mode tls');
 if(input.servername!==undefined&&!/^[A-Za-z0-9.:-]{1,253}$/.test(input.servername))throw new Error('Invalid TLS server name');
 return {mode,preset,perClient,connections,maxStreams,...(mode==='tls'?{...(insecure?{rigOnlyInsecureTlsNoVerify:true}:{caPem:input.caPem}),...(input.servername===undefined?{}:{servername:input.servername})}:{})};
}

// Campaign environment. Absent MW_LOAD_FIXTURE_HTTP2 means HTTP/1.1.
export function h2FromEnv(env){
 const mode=env.MW_LOAD_FIXTURE_HTTP2;
 if(mode===undefined||mode==='')return undefined;
 const num=name=>env[name]===undefined?undefined:Number(env[name]);
 return validateH2({mode,preset:env.MW_LOAD_FIXTURE_H2_PRESET,connections:num('MW_LOAD_FIXTURE_H2_CONNECTIONS'),maxStreams:num('MW_LOAD_FIXTURE_H2_MAX_STREAMS'),
  ...(env.MW_LOAD_FIXTURE_H2_PRESET==='custom'?{perClient:env.MW_LOAD_FIXTURE_H2_PER_CLIENT==='1'}:{}),
  ...(env.MW_LOAD_FIXTURE_H2_CA_FILE?{caPem:readFileSync(env.MW_LOAD_FIXTURE_H2_CA_FILE,'utf8')}:{}),
  ...(env.MW_LOAD_FIXTURE_H2_SERVERNAME?{servername:env.MW_LOAD_FIXTURE_H2_SERVERNAME}:{}),
  ...(env.MW_LOAD_RIG_ONLY_INSECURE_TLS_NO_VERIFY==='1'?{rigOnlyInsecureTlsNoVerify:true}:{})});
}

const FORBIDDEN=new Set(['host','connection','keep-alive','transfer-encoding','upgrade','proxy-connection']);
const DECODERS={gzip:()=>zlib.createGunzip(),'x-gzip':()=>zlib.createGunzip(),deflate:()=>zlib.createInflate(),br:()=>zlib.createBrotliDecompress()};
function makeResponse(headers,stream){
 const status=Number(headers[':status']);
 const encoding=String(headers['content-encoding']??'').toLowerCase();
 let wire=0;
 // Like fetch, the body handed to validators is decoded; the generator also sees the encoded size, which fetch hides.
 async function* counted(){for await(const chunk of stream){wire+=chunk.length;yield chunk;}}
 const source=()=>DECODERS[encoding]?Readable.from(counted()).pipe(DECODERS[encoding]()):counted();
 const reader=()=>{
  const iterator=source()[Symbol.asyncIterator]();
  return {
   async read(){const next=await iterator.next();return next.done?{done:true,value:undefined}:{done:false,value:next.value};},
   async cancel(){stream.__local=true;if(!stream.closed)stream.close(NGHTTP2_CANCEL);},
   releaseLock(){},
  };
 };
 const text=async()=>{const parts=[],r=reader();for(;;){const next=await r.read();if(next.done)break;parts.push(next.value);}return Buffer.concat(parts).toString('utf8');};
 return {status,ok:status>=200&&status<300,headers:{get:name=>headers[String(name).toLowerCase()]??null},
  body:{getReader:reader,async cancel(){stream.__local=true;if(!stream.closed)stream.close(NGHTTP2_CANCEL);}},
  wireBytes:()=>wire,text,json:async()=>JSON.parse(await text())};
}

export function createH2Client(input,{now=performance.now.bind(performance)}={}){
 const cfg=validateH2(input);
 const pools=new Map(),all=[];
 const stats={opened:0,closed:0,goaway:0,reconnects:0,connectErrors:0,alpnFailures:0,rst:{},refusedStreams:0,localCancels:0,streams:0,peakStreams:0,queued:0,queueWaitMs:0,queueWaitMaxMs:0};
 let inflight=0,closed=false,serverMax=null,reconnectsLeft=MAX_RECONNECTS;
 const tlsOptions=cfg.mode==='tls'?{ALPNProtocols:['h2'],rejectUnauthorized:cfg.rigOnlyInsecureTlsNoVerify!==true,...(cfg.caPem?{ca:cfg.caPem}:{}),...(cfg.servername?{servername:cfg.servername}:{})}:{};

 const free=s=>!s.draining&&s.active<s.limit;
 const pick=pool=>{let best=null;for(const s of pool.sessions)if(free(s)&&(!best||s.active<best.active))best=s;return best;};
 const reserve=s=>{s.active++;s.total++;stats.streams++;peak(++inflight);return s;};
 const peak=n=>{if(n>stats.peakStreams)stats.peakStreams=n;};
 const wake=pool=>{
  while(pool.waiters.length){
   const s=pick(pool);if(!s)return;
   const waiter=pool.waiters.shift();waiter.resolve(reserve(s));
  }
 };
 const retire=(s,pool)=>{
  if(s.retired)return;s.retired=true;s.draining=true;pool.sessions.delete(s);
  if(!closed&&pool.sessions.size<cfg.connections&&reconnectsLeft>0){reconnectsLeft--;stats.reconnects++;pool.sessions.add(open(pool));wake(pool);}
  else if(!pool.sessions.size)for(const waiter of pool.waiters.splice(0))waiter.reject(new Error('No HTTP/2 connection available'));
 };
 function open(pool){
  const session=http2.connect(pool.origin,{settings:{enablePush:false},...tlsOptions});
  const s={session,active:0,total:0,limit:cfg.maxStreams,draining:false,retired:false,counted:false,settled:false};
  all.push(s);stats.opened++;
  // Ready means connected AND the server's SETTINGS (its stream limit) are known, so the limit is respected from the first stream.
  s.ready=new Promise((resolve,reject)=>{
   session.once('connect',()=>{if(cfg.mode==='tls'&&session.alpnProtocol!=='h2'){stats.alpnFailures++;session.destroy();reject(new Error('Target did not negotiate h2'));}});
   session.once('remoteSettings',()=>resolve());
   session.once('error',reject);session.once('close',()=>reject(new Error('closed')));
  });
  s.ready.catch(()=>{});
  session.on('remoteSettings',settings=>{
   const max=settings.maxConcurrentStreams;s.settled=true;
   if(Number.isInteger(max)){serverMax=serverMax===null?max:Math.min(serverMax,max);s.limit=Math.min(cfg.maxStreams,max);wake(pool);}
  });
  session.on('goaway',()=>{stats.goaway++;s.draining=true;retire(s,pool);try{session.close();}catch{/* already closing */}});
  session.on('error',()=>{stats.connectErrors++;retire(s,pool);});
  session.on('close',()=>{if(!s.counted){s.counted=true;stats.closed++;}retire(s,pool);});
  return s;
 }
 const poolFor=(origin,clientIndex)=>{
  const key=cfg.perClient?origin+'|'+(clientIndex??0):origin;
  let pool=pools.get(key);
  if(!pool){
   if((pools.size+1)*cfg.connections>MAX_H2_CONNECTIONS)throw new Error('HTTP/2 connection budget exceeded');
   pool={origin,sessions:new Set(),waiters:[]};pools.set(key,pool);
  }
  while(pool.sessions.size<cfg.connections&&!closed)pool.sessions.add(open(pool));
  return pool;
 };

 const client={
  config:cfg,
  // Open the shared connections before the measured window (proxy-like). Per-client pools open on first use.
  async warm(origins){
   if(cfg.perClient)return;
   const sessions=origins.flatMap(origin=>[...poolFor(origin,0).sessions]);
   await Promise.race([Promise.all(sessions.map(s=>s.ready)),new Promise((_,reject)=>setTimeout(()=>reject(new Error('HTTP/2 connections not ready')),10000).unref())]);
  },
  // Claim a stream slot. When every connection is at its stream limit the call
  // waits in the generator (FIFO); the wait is reported and counts as scheduling lag.
  async acquire(origin,clientIndex,signal){
   if(closed)throw new Error('HTTP/2 client closed');
   signal?.throwIfAborted();
   const pool=poolFor(origin,clientIndex),began=now();
   let s=pick(pool),queuedMs=0;
   if(s){
    reserve(s);
    if(!s.settled){try{await s.ready;}catch(error){s.active--;inflight--;throw error;}}
   }else{
    s=await new Promise((resolve,reject)=>{
     const waiter={resolve,reject};pool.waiters.push(waiter);
     signal?.addEventListener('abort',()=>{const at=pool.waiters.indexOf(waiter);if(at>=0){pool.waiters.splice(at,1);reject(signal.reason);}},{once:true});
    });
    queuedMs=now()-began;stats.queued++;stats.queueWaitMs+=queuedMs;stats.queueWaitMaxMs=Math.max(stats.queueWaitMaxMs,queuedMs);
   }
   let released=false;
   return {s,pool,queuedMs,release(){if(released)return;released=true;s.active--;inflight--;wake(pool);}};
  },
  request(slot,url,init={}){
   return new Promise((resolve,reject)=>{
    const target=new URL(url),headers={':method':init.method??'GET',':path':target.pathname+target.search,':scheme':target.protocol.slice(0,-1),':authority':target.host};
    for(const [key,value] of Object.entries(init.headers??{})){const name=key.toLowerCase();if(!FORBIDDEN.has(name))headers[name]=String(value);}
    // fetch advertises gzip and deflate on its own; the h2 path does the same so both protocols see the same server behaviour.
    if(headers['accept-encoding']===undefined)headers['accept-encoding']='gzip, deflate';
    let stream;
    try{stream=slot.s.session.request(headers,{endStream:init.body===undefined});}catch(error){reject(error);return;}
    stream.on('close',()=>{
     slot.autoRelease&&slot.release();
     const code=stream.rstCode;
     if(code&&!stream.__local){stats.rst[code]=(stats.rst[code]??0)+1;if(code===NGHTTP2_REFUSED_STREAM)stats.refusedStreams++;}
     reject(new Error('stream closed before response'));
    });
    stream.on('error',reject);
    const cancel=()=>{if(stream.closed)return;stream.__local=true;stats.localCancels++;stream.close(NGHTTP2_CANCEL);};
    if(init.signal?.aborted)cancel();else init.signal?.addEventListener('abort',cancel,{once:true});
    stream.once('response',headersIn=>resolve(makeResponse(headersIn,stream)));
    if(init.body!==undefined)stream.end(init.body);
   });
  },
  // A slot that bypasses the generator's stream limit (burst scenario): every stream is opened now and the
  // connection's own flow decides what happens beyond the server's announced limit.
  rawSlot(origin){
   const pool=poolFor(origin,0),s=[...pool.sessions][0];
   if(!s)throw new Error('No HTTP/2 connection available');
   s.total++;stats.streams++;
   return {s,pool,queuedMs:0,autoRelease:false,release(){}};
  },
  // fetch-shaped single request, for preflights, ownership proofs and probes.
  async oneShot(url,init={}){
   const slot=await client.acquire(new URL(url).origin,0,init.signal);slot.autoRelease=true;
   try{return await client.request(slot,url,init);}catch(error){slot.release();throw error;}
  },
  report(){
   const totals=all.map(s=>s.total);
   const rst=Object.fromEntries(Object.entries(stats.rst));
   return {schema:1,protocol:cfg.mode==='tls'?'h2':'h2c',transport:cfg.mode,preset:cfg.preset,perClientConnections:cfg.perClient,connectionsPerPool:cfg.connections,pools:pools.size,
    maxStreamsConfigured:cfg.maxStreams,serverMaxConcurrentStreams:serverMax,connectionsOpened:stats.opened,connectionsClosed:stats.closed,reconnects:stats.reconnects,connectErrors:stats.connectErrors,alpnFailures:stats.alpnFailures,
    goaway:stats.goaway,rstStream:rst,rstStreamTotal:Object.values(rst).reduce((a,b)=>a+b,0),refusedStreams:stats.refusedStreams,localCancels:stats.localCancels,
    streams:stats.streams,peakStreamsInFlight:stats.peakStreams,queuedStreams:stats.queued,queueWaitMs:{total:stats.queueWaitMs,max:stats.queueWaitMaxMs},
    streamsPerConnection:{connections:totals.length,min:totals.length?Math.min(...totals):0,max:totals.length?Math.max(...totals):0,total:totals.reduce((a,b)=>a+b,0)},
    flowControlStalls:'not exposed by node:http2'};
  },
  close(){
   closed=true;
   for(const pool of pools.values()){for(const waiter of pool.waiters.splice(0))waiter.reject(new Error('HTTP/2 client closed'));for(const s of pool.sessions){try{s.session.destroy();}catch{/* gone */}}}
  },
 };
 return client;
}

// Combine per-worker reports. Counters add; the stream limit is the smallest the servers announced.
export function mergeH2Reports(reports){
 const list=reports.filter(Boolean);
 if(!list.length)return undefined;
 if(new Set(list.map(r=>r.protocol+'|'+r.preset+'|'+r.connectionsPerPool+'|'+r.maxStreamsConfigured)).size!==1)throw new Error('Workers used different HTTP/2 models');
 const sum=key=>list.reduce((n,r)=>n+r[key],0),rst={};
 for(const r of list)for(const [code,count] of Object.entries(r.rstStream))rst[code]=(rst[code]??0)+count;
 const maxima=list.map(r=>r.serverMaxConcurrentStreams).filter(Number.isInteger);
 const per=list.map(r=>r.streamsPerConnection).filter(r=>r.connections>0);
 return {...list[0],pools:sum('pools'),serverMaxConcurrentStreams:maxima.length?Math.min(...maxima):null,connectionsOpened:sum('connectionsOpened'),connectionsClosed:sum('connectionsClosed'),reconnects:sum('reconnects'),connectErrors:sum('connectErrors'),alpnFailures:sum('alpnFailures'),
  goaway:sum('goaway'),rstStream:rst,rstStreamTotal:sum('rstStreamTotal'),refusedStreams:sum('refusedStreams'),localCancels:sum('localCancels'),streams:sum('streams'),
  peakStreamsInFlight:sum('peakStreamsInFlight'),peakMethod:'sum of per-worker peaks; upper bound',queuedStreams:sum('queuedStreams'),
  queueWaitMs:{total:list.reduce((n,r)=>n+r.queueWaitMs.total,0),max:Math.max(...list.map(r=>r.queueWaitMs.max))},
  streamsPerConnection:{connections:list.reduce((n,r)=>n+r.streamsPerConnection.connections,0),min:per.length?Math.min(...per.map(r=>r.min)):0,max:per.length?Math.max(...per.map(r=>r.max)):0,total:list.reduce((n,r)=>n+r.streamsPerConnection.total,0)}};
}

// Burst scenario: N distinct GETs issued at the same instant on ONE h2
// connection (the pattern of a cold page load). Like a browser, the generator
// stays within the server's announced stream limit and queues the rest on the
// same connection; `exceedServerLimit:true` deliberately opens every stream at
// once so the server's refusals are observed. Whatever the server then answers
// (200, 503 reasons, refused streams) is what is reported.
const KNOWN_503=new Set(['assetLoadBusy','requestProtectionUnavailable','publishedGuardFenceRefusal','authorizationNotConfirmedBeforeDispatch']);
export async function h2Burst({origin,paths,headers={},h2,timeoutMs=10000,exceedServerLimit=false}){
 if(!Array.isArray(paths)||!paths.length||paths.length>2000||new Set(paths).size!==paths.length||paths.some(p=>typeof p!=='string'||!p.startsWith('/')))throw new Error('A burst needs 1-2000 distinct absolute paths');
 const cfg=validateH2({...h2,preset:'custom',connections:1,maxStreams:paths.length>1000?1000:paths.length,perClient:false});
 const client=createH2Client(cfg),target=new URL(origin).origin;
 try{
  await client.warm([target]);
  const began=performance.now();
  const results=await Promise.all(paths.map(async path=>{
   const started=performance.now(),signal=AbortSignal.timeout(timeoutMs);
   try{
    // One connection, every stream opened now.
    const slot=exceedServerLimit?client.rawSlot(target):await client.acquire(target,0,signal);
    slot.autoRelease=true;
    const response=await client.request(slot,new URL(path,target),{headers,signal});
    const chunks=[],reader=response.body.getReader();let bytes=0;
    for(;;){const part=await reader.read();if(part.done)break;bytes+=part.value.length;if(chunks.length<1&&response.status===503)chunks.push(part.value);}
    let reason;
    if(response.status===503){try{const code=JSON.parse(Buffer.concat(chunks).toString('utf8').slice(0,512)).code;reason=KNOWN_503.has(code)?code:'other503';}catch{reason='other503';}}
    return {path,status:response.status,bytes,ms:performance.now()-started,...(reason?{reason}:{})};
   }catch{return {path,status:'network',bytes:0,ms:performance.now()-started};}
  }));
  const statuses={},reasons={};
  for(const row of results){statuses[row.status]=(statuses[row.status]??0)+1;if(row.reason)reasons[row.reason]=(reasons[row.reason]??0)+1;}
  return {schema:1,scenario:'single-connection-burst',requests:results.length,wallMs:performance.now()-began,statuses,http503Reasons:reasons,results,connection:client.report()};
 }finally{client.close();}
}
