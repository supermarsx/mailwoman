import {createSignal} from 'solid-js';
import {observePreferenceAccount,preferenceAccountIsCurrent,type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import type {SmartArchiveGroup} from './smartArchive.ts';
export interface SmartArchiveRequest {owner:PreferenceAccount;groups:SmartArchiveGroup[];trigger:HTMLElement|null;finish:(groups:SmartArchiveGroup[]|undefined)=>void}
const [queue,setQueue]=createSignal<SmartArchiveRequest[]>([]);
export const pendingSmartArchive=()=>queue()[0];
export function finishSmartArchive(groups?:SmartArchiveGroup[]):void{const current=queue()[0];if(!current)return;setQueue(old=>old.slice(1));current.finish(groups);}
observePreferenceAccount(()=>{
 const pending=queue();setQueue(pending.filter(value=>preferenceAccountIsCurrent(value.owner)));
 for(const value of pending)if(!preferenceAccountIsCurrent(value.owner))value.finish(undefined);
});
export function requestSmartArchive(owner:PreferenceAccount,groups:SmartArchiveGroup[]):Promise<SmartArchiveGroup[]|undefined>{
 if(!preferenceAccountIsCurrent(owner))return Promise.resolve(undefined);
 return new Promise(resolve=>setQueue(old=>[...old,{owner,groups,trigger:typeof document!=='undefined'&&document.activeElement instanceof HTMLElement?document.activeElement:null,finish:resolve}]));
}
