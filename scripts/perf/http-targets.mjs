// Target rules for the load generator. It may only ever load a system the
// operator owns: the literal loopback fixture of the single-host path, or an
// explicit allow-list of PRIVATE rig addresses, each proven to belong to this
// run by the run's control token. No hostnames, no DNS, no public addresses.
import {isIPv4,isIPv6} from 'node:net';

export const LOOPBACK_ORIGIN_HOST='127.0.0.1';
export const MAX_TARGETS=64;

function ipv4Octets(text){
 if(!isIPv4(text))return null;
 const parts=text.split('.').map(Number);
 return parts.length===4&&parts.every(n=>Number.isInteger(n)&&n>=0&&n<=255)?parts:null;
}
// 'loopback' only for the literal 127.0.0.1 (as before); 'private' for RFC 1918
// IPv4 and IPv6 unique-local (fc00::/7); everything else, including link-local
// (cloud metadata), CGNAT, multicast, hostnames and IPv4-mapped IPv6, is 'refused'.
export function classifyHost(hostname){
 if(typeof hostname!=='string'||!hostname)return 'refused';
 const bare=hostname.startsWith('[')&&hostname.endsWith(']')?hostname.slice(1,-1):hostname;
 const v4=ipv4Octets(bare);
 if(v4){
  if(bare===LOOPBACK_ORIGIN_HOST)return 'loopback';
  const [a,b]=v4;
  if(a===10||a===172&&b>=16&&b<=31||a===192&&b===168)return 'private';
  return 'refused';
 }
 if(isIPv6(bare)&&!bare.includes('.')){
  const first=parseInt(bare.split(':')[0]||'0',16);
  if(Number.isInteger(first)&&(first&0xfe00)===0xfc00)return 'private';
 }
 return 'refused';
}

// Plain http origin with a literal address and an explicit port; nothing else.
export function parseTargetOrigin(text){
 let url;
 try{url=new URL(text);}catch{throw new Error('Invalid target origin');}
 if(url.protocol!=='http:'||!url.port||url.pathname!=='/'||url.search||url.hash||url.username||url.password)throw new Error('Invalid target origin');
 const kind=classifyHost(url.hostname);
 if(kind==='refused')throw new Error('Target address is not a literal loopback or private rig address');
 return {origin:url.origin,host:url.hostname,port:url.port,kind};
}

// Allow-list entries are `address:port` (IPv6 as `[address]:port`).
export function parseAllowList(value){
 const entries=Array.isArray(value)?value:typeof value==='string'&&value?value.split(','):[];
 if(entries.length>MAX_TARGETS)throw new Error('Too many allowed targets');
 const result=new Set();
 for(const entry of entries){
  let parsed;
  try{parsed=parseTargetOrigin('http://'+String(entry).trim()+'/');}catch{throw new Error('Allowed target must be a private rig address with a port');}
  if(parsed.kind!=='private')throw new Error('Allowed target must be a private rig address with a port');
  result.add(parsed.origin);
 }
 return [...result];
}

// Targets for a run. A loopback origin is accepted as today; any other
// address must be private AND listed in the operator's allow-list.
export function validateTargets(origins,allowed=[]){
 if(!Array.isArray(origins)||!origins.length||origins.length>MAX_TARGETS)throw new Error('Invalid target list');
 const allow=new Set(parseAllowList(allowed)),seen=new Set(),result=[];
 for(const text of origins){
  const target=parseTargetOrigin(text);
  if(seen.has(target.origin))throw new Error('Duplicate target');
  if(target.kind==='private'&&!allow.has(target.origin))throw new Error('Private target is not in the explicit allow-list');
  seen.add(target.origin);result.push(target);
 }
 return result;
}

// Ownership proof. The run's control path and token are read back from every
// non-loopback target: the token must be accepted AND a request without it must
// be refused (the endpoint really is token-gated), so a target that merely
// answers 200 to anything is not accepted. Loopback is the run's own fixture.
export async function proveOwnership(targets,{path,token,fetchImpl=fetch,timeoutMs=5000}){
 const proven=[];
 for(const target of targets){
  if(target.kind==='loopback'){proven.push(target.origin);continue;}
  if(typeof path!=='string'||!/^\/__fixture-diagnostic-[a-f0-9-]+$/.test(path)||typeof token!=='string'||token.length<16)throw new Error('Ownership proof needs the run control path and token');
  const url=new URL(path,target.origin);
  const own=await fetchImpl(url,{headers:{'x-fixture-token':token},redirect:'error',signal:AbortSignal.timeout(timeoutMs)}).catch(()=>null);
  if(!own||own.status!==200){await own?.body?.cancel();throw new Error('Target did not accept the run control token');}
  let body;try{body=await own.json();}catch{throw new Error('Target did not return a control snapshot');}
  if(!body||typeof body!=='object'||Array.isArray(body))throw new Error('Target did not return a control snapshot');
  const without=await fetchImpl(url,{redirect:'error',signal:AbortSignal.timeout(timeoutMs)}).catch(()=>null);
  if(!without||without.status===200){await without?.body?.cancel();throw new Error('Target control endpoint is not token-gated');}
  await without.body?.cancel();
  proven.push(target.origin);
 }
 return proven;
}
