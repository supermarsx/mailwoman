// Stats of the open folder dialog (one request, shared by the header tiles and the tabs),
// honouring `properties.folder.sizeCompute`: with `off` nothing is requested until the user asks.
import {createResource,createSignal} from 'solid-js';
import {resourceValue} from '../resourceValue.ts';
import {useApp} from '../../state/context.ts';
import {createConfiguredClient} from '../../api/transport.ts';
import type {Mailbox} from '../../api/jmap-types.ts';
import {fetchFolderStats,forgetFolderStats,type FolderStats} from './folderStats.ts';
import {sizeCompute} from './policy.ts';

export interface StatsState {
 stats:()=>FolderStats|undefined;loading:()=>boolean;failed:()=>boolean;
 /** True when the policy leaves the size to the user and they have not asked yet. */
 idle:()=>boolean;compute:()=>void;retry:()=>void;mode:()=>string;
}

export function useFolderStats(box:()=>Mailbox):StatsState {
 const app=useApp(),client=createConfiguredClient();
 const [wanted,setWanted]=createSignal(sizeCompute()==='on-open'||sizeCompute()==='background');
 const [stats,{refetch}]=createResource(()=>wanted()&&box().mailwomanSearchQuery===undefined?{account:app.accountId(),id:box().id}:null,async key=>{
  if(!key?.account)return undefined;
  return await fetchFolderStats(client,key.account,key.id);
 });
 return {
  stats:()=>resourceValue(stats),loading:()=>stats.loading,failed:()=>stats.state==='errored',
  idle:()=>!wanted()&&box().mailwomanSearchQuery===undefined,
  compute:()=>{if(wanted())void refetch();else setWanted(true);},
  retry:()=>{forgetFolderStats(box().id);void refetch();},
  mode:()=>sizeCompute(),
 };
}
