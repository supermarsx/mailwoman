import contract from './websocket-action-contract.json';

export {contract as actionContract};
export type ActionActor='webmail'|'admin';
export interface ActionRoute {actor:ActionActor;path:string;endpoint:string;cookieEndpoint?:string}
const bytes=(value:string)=>new TextEncoder().encode(value).length;
const controls=/[\u0000-\u0020\u007f-\u009f]/;
interface Rule {exact?:string;prefix?:string;methods?:string[]}
function required(rules:readonly Rule[],path:string,method:string):boolean{
 return rules.some(rule=>(rule.methods===undefined||rule.methods.includes(method))&&(rule.exact===path||rule.prefix!==undefined&&path.startsWith(rule.prefix)));
}
/** Validate BEFORE URL's dot-segment normalization. Query data may encode slashes;
 * path components may not introduce routing delimiters or a second decode. */
export function validActionPath(value:string):boolean{
 if(!value.startsWith('/')||value.startsWith('//')||bytes(value)>contract.limits.pathBytes||value.includes('#')||value.includes('\\')||controls.test(value)||/%(?![\da-f]{2})/i.test(value))return false;
 const [path,...query]=value.split('?');
 try{
  const decoded=decodeURIComponent(path!);
  if(decoded.includes('\\')||decoded.includes('%')||/[\u0000-\u001f\u007f-\u009f]/.test(decoded)||/%(?:2f|5c)/i.test(path!))return false;
  if(decoded.split('/').some(part=>part==='.'||part==='..'))return false;
  if(query.length)decodeURIComponent(query.join('?'));
 }catch{return false;}
 return true;
}
export function actionHeaders(input:HeadersInit|undefined):Record<string,string>|undefined{
 try{
  const entries: [string,string][]=input instanceof Headers?[...input.entries()]:Array.isArray(input)?input.map(pair=>[pair[0],pair[1]]):Object.entries(input??{});
  if(entries.length>contract.limits.headers)return;
  const result:Record<string,string>={};let length=0;
  for(const [name,value] of entries){const key=name.toLowerCase();if(!contract.requestHeaders.includes(key)||Object.hasOwn(result,key)||/[\r\n\u0000]/.test(value))return;length+=bytes(key)+bytes(value);if(length>contract.limits.headerBytes)return;result[key]=value;}
  return result;
 }catch{return;}
}
export function actionRoute(input:string,init:RequestInit|undefined,base:string,consolePrefix:string):ActionRoute|undefined{
 try{
  // Reject raw escapes before constructing target, including absolute inputs.
  const raw=input.replace(/^[a-z][a-z\d+.-]*:\/\/[^/?#]*/i,'');
  if(!validActionPath(raw))return;
  const root=new URL(base,globalThis.location?.href),target=new URL(input,globalThis.location?.href);
  if(root.origin!==globalThis.location?.origin||target.origin!==root.origin||target.username||target.password||target.hash)return;
  const prefix=root.pathname.replace(/\/$/,''),method=(init?.method??'GET').toUpperCase();
  if(!target.pathname.startsWith(`${prefix}/`)||!contract.methods.includes(method)||init?.keepalive||init?.credentials==='omit'||init?.body!=null&&typeof init.body!=='string')return;
  const headers=actionHeaders(init?.headers);if(!headers)return;
  if(init?.body&&(bytes(init.body)>contract.limits.bodyBytes||!/^application\/json(?:;|$)/i.test(headers['content-type']??'')))return;
  const path=target.pathname.slice(prefix.length)+target.search,pathname=path.split('?')[0]!;
  const admin=consolePrefix.replace(/\/$/,'');
  let actor:ActionActor='webmail',endpoint=contract.webmail.endpoint,cookieEndpoint:string|undefined;
  if(pathname.startsWith(`${admin}/`)){
   if(required(contract.admin.httpRequired,pathname.slice(admin.length),method))return;
   actor='admin';endpoint=admin+contract.admin.endpointSuffix;cookieEndpoint=prefix+admin+contract.admin.cookieDeliverySuffix;
  }else{
   if(pathname===admin||required(contract.webmail.httpRequired,pathname,method)||!(contract.webmail.jsonExact.includes(pathname)||contract.webmail.jsonPrefixes.some(p=>pathname.startsWith(p))))return;
  }
  const socket=new URL(prefix+endpoint,root.origin);socket.protocol=root.protocol==='https:'?'wss:':'ws:';
  return {actor,path,endpoint:socket.href,...(cookieEndpoint?{cookieEndpoint}: {})};
 }catch{return;}
}
