import type {OutboundItem} from '../contracts/offline.ts';
import type {OutboxStore} from './outbox.ts';
import {offlineVault,type OfflineVault} from './vault.ts';

/** The durable queue is encrypted independently of optional message caching.
 * Pending work is never evicted to make space for downloaded mail. */
export function encryptedOutboxStore(legacy:OutboxStore,storage:()=>Promise<OfflineVault>=()=>offlineVault('indexeddb')):OutboxStore {
 const namespace='outbound-queue';
 async function migrate():Promise<OfflineVault>{
  const vault=await storage();
  for(const item of await legacy.all()){
   const existing=await vault.get<OutboundItem>(namespace,item.id);
   if(!existing)await vault.put(namespace,item.id,item);
   // A plaintext legacy row survives any encryption/storage failure.
   await legacy.delete(item.id);
  }
  return vault;
 }
 return {
  async all(){return (await (await migrate()).list<OutboundItem>(namespace)).map(row=>row.value).sort((a,b)=>a.createdAt-b.createdAt);},
  async add(item){await (await migrate()).put(namespace,item.id,item);},
  async put(item){await (await migrate()).put(namespace,item.id,item);},
  async delete(id){await (await migrate()).remove(namespace,id);await legacy.delete(id);},
 };
}
