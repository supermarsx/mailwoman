import type {JmapRequest,JmapResponse} from '../api/jmap-types.ts';
import type {QueuedItem} from './outboxCore.ts';
import {offlineVault} from './vault.ts';
import {offlineNamespace} from './runtime.ts';

export interface Acknowledgement {request:JmapRequest;response:JmapResponse;at:number}
export const replayLedger=(namespace:string)=>`replay-acknowledgements:${namespace}`;
/** The service worker cannot open window-owned OPFS handles. Keep the actual
 * server acknowledgement encrypted in IDB until a window imports it. */
export async function rememberReplay(item:QueuedItem,request:JmapRequest,response:JmapResponse):Promise<void>{
 if(item.type!=='pim'||!item.storageAuthority||!item.cacheBase)return;
 const account=request.methodCalls[0]?.[1]['accountId'];if(typeof account!=='string')return;
 const namespace=offlineNamespace(item.cacheBase,{accountId:account,storageAuthority:item.storageAuthority});
 const vault=await offlineVault('indexeddb');
 await vault.put(replayLedger(namespace),item.id,{request,response,at:Date.now()} satisfies Acknowledgement);
}
