export interface TrashedMessages {accountId:string;mailboxId:string|null;sort:unknown;ids:readonly string[];order:readonly string[]}
const listeners=new Set<(event:TrashedMessages)=>void>();
export function onMessagesTrashed(listener:(event:TrashedMessages)=>void):()=>void {listeners.add(listener);return ()=>listeners.delete(listener);}
export function messagesTrashed(event:TrashedMessages):void {for(const listener of listeners)listener(event);}
