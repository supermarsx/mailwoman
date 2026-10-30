import manifest from './linked-scope.json';

/** Mirrors the audited server scope: both sides read `linked-scope.json`
 * (t87), and the server refuses any selector outside its `mailbox` rules.
 * Unknown mailbox APIs fail closed while a child is selected; omitting a
 * selector would execute against the primary. */
export type LinkedRoute='mailbox'|'primary'|'public'|'signed'|'unsupported';
type Scope=Exclude<LinkedRoute,'unsupported'>;
interface Rule {path?:string;prefix?:string;segments?:number;suffix?:string;scope:Scope;resource?:boolean}
const rules=manifest.routes as readonly Rule[];
const SCOPES:ReadonlySet<string>=new Set(['mailbox','primary','public','signed']);
for(const rule of rules){
  // A malformed rule must break the build's tests, not silently widen a scope.
  if(!SCOPES.has(rule.scope)||(rule.path===undefined)===(rule.prefix===undefined)
    ||(rule.prefix!==undefined&&!(Number.isInteger(rule.segments)&&rule.segments!>0))
    ||(rule.resource&&rule.scope!=='mailbox'))throw new Error(`Invalid linked scope rule ${JSON.stringify(rule)}`);
}
function matches(rule:Rule,path:string):boolean {
  if(rule.path!==undefined)return path===rule.path;
  if(!path.startsWith(rule.prefix!))return false;
  const tail=path.slice(rule.prefix!.length).split('/');
  return tail.length===rule.segments&&tail.every(Boolean)&&(rule.suffix===undefined||tail[tail.length-1]===rule.suffix);
}
export function linkedUnsignedResource(path:string):boolean {
  return rules.some(rule=>rule.resource===true&&matches(rule,path));
}
export function linkedRoute(path:string):LinkedRoute {
  // `mailbox` wins over the others so a path never leaves the selected mailbox
  // because a broader primary rule also matches it.
  for(const scope of ['mailbox','primary','public','signed'] as const){
    if(rules.some(rule=>rule.scope===scope&&matches(rule,path)))return scope;
  }
  return 'unsupported';
}

/** Only cookie-authenticated unsigned download URLs carry mwLink. Signed URLs
 * and external destinations must never acquire a selector or parent secrets. */
export function linkedResourceUrl(input:string,linkId:string,serverBase:string):string {
  const base=new URL(serverBase);
  const url=new URL(input,base);
  const prefix=base.pathname.replace(/\/$/,'');
  if(url.origin!==base.origin||url.username||url.password||!url.pathname.startsWith(`${prefix}/`))throw new Error('Unsupported linked resource');
  const path=url.pathname.slice(prefix.length);
  if(!linkedUnsignedResource(path)||!linkId||url.searchParams.has('mwLink'))throw new Error('Unsupported linked resource');
  url.searchParams.set('mwLink',linkId);
  return url.href;
}
