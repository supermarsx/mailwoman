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
