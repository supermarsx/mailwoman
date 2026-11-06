// Which read-cache entries belong to one folder. Mailbox entries are keyed by the
// JSON of the request that made them; message entries by `['message', id]`. A
// folder's own entries are the list requests that name it and the messages the
// client knows to be in it.
import type {MailCacheKind} from './mailReadCache.ts';

/** Predicate for `invalidate()`: the mailbox requests naming `mailboxId` and the cached `messageIds`. */
export function folderCachePredicate(mailboxId:string,messageIds:Iterable<string>):(key:string,kind:MailCacheKind)=>boolean {
 const ids=new Set(messageIds);
 const needle=JSON.stringify(mailboxId);
 return (key,kind)=>{
  if(kind==='mailbox')return key.includes(needle);
  try{const parsed=JSON.parse(key) as unknown;return Array.isArray(parsed)&&parsed[0]==='message'&&typeof parsed[1]==='string'&&ids.has(parsed[1]);}catch{return false;}
 };
}
