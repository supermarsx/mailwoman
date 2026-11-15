import {offlineVault,type OfflineVault} from './vault.ts';
import {rememberJmap} from './jmapCache.ts';
import {replayLedger,type Acknowledgement} from './replayCache.ts';
export async function importReplay(vault:OfflineVault,namespace:string,source:()=>Promise<OfflineVault>=()=>offlineVault('indexeddb')):Promise<void>{
 const held=await source();
 const records=await held.list<Acknowledgement>(replayLedger(namespace));
 records.sort((a,b)=>a.value.at-b.value.at);
 for(const {key,value} of records){
  await rememberJmap(vault,namespace,value.request,value.response);
  await held.remove(replayLedger(namespace),key);
 }
}
