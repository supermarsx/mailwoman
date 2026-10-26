// Only a same-console 401 ends the administrator session. No credential material
// is stored; the tab-local marker explains an expired session after a reload.
import { adminUrl } from './adminPath.ts';
let generation=0;
let invalidated:number|undefined;
let memory:'active'|'expired'|null=null;
const listeners=new Set<()=>void>();
function key():string{return `mailwoman.admin-session:${adminUrl('')}`;}
function saved(): 'active'|'expired'|null {
  try{const value=sessionStorage.getItem(key());return value==='active'||value==='expired'?value:memory;}catch{return memory;}
}
function remember(value:'active'|'expired'|null):void{
  memory=value;
  try{if(value===null)sessionStorage.removeItem(key());else sessionStorage.setItem(key(),value);}catch{/* Storage may be disabled. */}
}
export function adminSessionGeneration():number{return generation;}
export function hadAdminSession():boolean{return saved()!==null;}
export function markAdminSessionActive():void{generation++;invalidated=undefined;remember('active');}
export function forgetAdminSession():void{generation++;invalidated=undefined;remember(null);}
export function markAdminSessionEnded():void{
  if(saved()==='expired')return;
  invalidated=generation++;remember('expired');
}
export function adminResponseIsCurrent(epoch:number):boolean{return epoch===generation||epoch===invalidated;}
export function onAdminSessionEnded(listener:()=>void):()=>void{listeners.add(listener);return()=>listeners.delete(listener);}
export function observeAdminResponse(input:string,status:number,epoch:number,sessionProbe=false):void{
  if(status!==401||epoch!==generation||!hadAdminSession())return;
  const base=new URL(adminUrl(''),globalThis.location?.href??'http://localhost');
  const target=new URL(input,base);
  if(target.origin!==base.origin||!target.pathname.startsWith(`${base.pathname}/`))return;
  const suffix=target.pathname.slice(base.pathname.length);
  if(suffix==='/login'||suffix.startsWith('/login/')||suffix==='/logout'||(suffix==='/session'&&!sessionProbe))return;
  markAdminSessionEnded();
  for(const listener of listeners)listener();
}
