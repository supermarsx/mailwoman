// `Mailbox/stats`: what the server knows about one folder (size, dates, remote
// path and counters). A Mailwoman extension method (crates/mw-engine/src/folder_stats.rs);
// a server that does not know it answers an error and the dialog shows what the
// folder list already holds.
import type {Client} from '../../api/client.ts';
import {CAP_CORE,CAP_MAIL} from '../../api/jmap-types.ts';
import type {RemoteLocation} from './policy.ts';

export interface FolderStats {
 accountId:string;mailboxId:string;role:string|null;totalEmails:number;unreadEmails:number;cachedEmails:number;
 /** True while fewer messages are cached than the folder holds: sizes then cover only those. */
 approximate:boolean;sizeBytes:number;flaggedEmails:number;draftEmails:number;
 oldest:string|null;newest:string|null;bodiesCached:number;largest:{id:string;size:number}[];remote:RemoteLocation;
}

const cache=new Map<string,{at:number;stats:FolderStats}>();
const TTL_MS=60_000;
/** Forget cached answers (after emptying a folder or clearing its cache). */
export function forgetFolderStats(mailboxId?:string):void {if(mailboxId)cache.delete(mailboxId);else cache.clear();}

export async function fetchFolderStats(client:Pick<Client,'jmap'>,accountId:string,mailboxId:string,signal?:AbortSignal,now:number=Date.now()):Promise<FolderStats> {
 const key=`${accountId}:${mailboxId}`,hit=cache.get(key);
 if(hit&&now-hit.at<TTL_MS)return hit.stats;
 const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Mailbox/stats',{accountId,mailboxId},'stats']]},signal?{signal}:undefined);
 const found=response.methodResponses.find(item=>item[2]==='stats');
 if(!found||found[0]==='error')throw new Error((found?.[1] as {type?:string}|undefined)?.type??'No Mailbox/stats response');
 const stats=found[1] as unknown as FolderStats;
 if(stats.mailboxId!==mailboxId||typeof stats.sizeBytes!=='number')throw new Error('Invalid Mailbox/stats response');
 cache.set(key,{at:now,stats});
 return stats;
}

/** `1.4 MB`, `812 B`. */
export function formatBytes(bytes:number):string {
 if(!Number.isFinite(bytes)||bytes<0)return '';
 const units=['B','KB','MB','GB','TB'];let value=bytes,unit=0;
 while(value>=1024&&unit<units.length-1){value/=1024;unit++;}
 return `${unit===0?value:value.toFixed(value>=100?0:1)} ${units[unit]}`;
}
