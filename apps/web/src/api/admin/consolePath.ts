// t76 — the admin console's own address (`GET/PUT {console}/console-path`).
// Only the console calls this; the mail app never learns the path.
import {adminRequest,type AdminResponse} from '../../screens/Admin/shell/adminApi.ts';
export type ConsolePathSource='env'|'stored'|'generated'|'upgrade'|'default';
export interface ConsolePathView {
  /** The path in force, relative to the deploy prefix (`/ops-…`). */
  path:string;
  source:ConsolePathSource;
  /** A stored path the next restart applies, or null. */
  pending:string|null;
  /** The variable that pins the path (`MW_ADMIN_PATH`), or null. */
  pinnedBy:string|null;
  /** The path in force is the old default `/admin`. */
  legacyDefault:boolean;
  /** Shortest path (characters after the `/`) the server accepts here. */
  minLength:number;
  restartRequired?:boolean;
}
export type ConsolePathChange={path:string}|{generate:true};
export interface ConsolePathApi {
  read():Promise<AdminResponse<ConsolePathView>>;
  save(change:ConsolePathChange,etag:string):Promise<AdminResponse<ConsolePathView>>;
}
export const consolePathApi:ConsolePathApi={
  read:()=>adminRequest('/console-path'),
  save:(change,ifMatch)=>adminRequest('/console-path',{method:'PUT',body:change,ifMatch}),
};
/** The shape the server accepts for a path chosen here (it checks again). */
export function validChosenPath(path:string,min:number):boolean{
  if(!path.startsWith('/'))return false;
  const segments=path.slice(1).split('/');
  if(segments.length>4||path.length-1<min)return false;
  return segments.every(s=>s!==''&&s!=='.'&&s!=='..'&&/^[A-Za-z0-9._~-]+$/.test(s));
}

// t58 — email the console address (`GET/POST {console}/console-path/mail`).
export type ConsolePathMailRecipientKind='signIn'|'operator';
export interface ConsolePathMailStatus {
  /** Outbound system email is enabled and names a sender and a relay. */
  configured:boolean;
  /** Why not, when it is not (`disabled`, `noSender`, `noRelay`, ...). */
  reason:string|null;
  /** Addresses the dialog offers: the admin's sign-in name when it is one, then operator recipients. */
  suggestions:Array<{address:string;kind:ConsolePathMailRecipientKind}>;
  perAdminPerHour:number;
}
export interface ConsolePathMailApi {
  status():Promise<AdminResponse<ConsolePathMailStatus>>;
  send(recipient:string,locale:string):Promise<AdminResponse<{queued:boolean;recipient:string}>>;
}
export const consolePathMailApi:ConsolePathMailApi={
  status:()=>adminRequest('/console-path/mail'),
  send:(recipient,locale)=>adminRequest('/console-path/mail',{method:'POST',body:{recipient,locale}}),
};
/** One plain address, as the server accepts it (it checks again). */
export function validRecipient(raw:string):boolean{
  const a=raw.trim();
  if(a.length===0||a.length>254||/[\s,;<>"()[\]\\]/.test(a))return false;
  const at=a.lastIndexOf('@');
  if(at<1)return false;
  const domain=a.slice(at+1);
  if(a.slice(0,at).includes('@')||!domain.includes('.'))return false;
  return domain.split('.').every(l=>l.length>0&&l.length<=63&&!l.startsWith('-')&&!l.endsWith('-')&&/^[\p{L}\p{N}-]+$/u.test(l));
}
