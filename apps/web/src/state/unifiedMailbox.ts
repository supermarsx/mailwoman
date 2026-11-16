import {activeUnifiedGroup,setActiveUnifiedGroup,unifiedDefaultGroup} from './unifiedAccountPreferences.ts';
import {settings} from '../api/settings.ts';
import {createSignal} from 'solid-js';
import {preferenceAccount,preferenceAccountIsCurrent,type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import type {UnifiedRole} from '../api/workspaceMail.ts';
export const [unifiedAvailable,setUnifiedAvailable]=createSignal(false);
export const [unifiedStartup,setUnifiedStartup]=createSignal(false);
const [request,setRequest]=createSignal<{owner:PreferenceAccount}>();
export const unifiedMailboxRequest=request;
export const [unifiedMailboxActive,setUnifiedMailboxActive]=createSignal(false);
export const [unifiedVirtualMode,setUnifiedVirtualMode]=createSignal(false);
export const [unifiedFolderRole,setUnifiedFolderRole]=createSignal<UnifiedRole>('inbox');
export const [unifiedFolderCounts,setUnifiedFolderCounts]=createSignal<Partial<Record<UnifiedRole,{total:number|null;unread:number|null}>>>({});
export function requestUnifiedMailbox(role?:UnifiedRole,groupId?:string|null):void{
 const owner=preferenceAccount();
 if(owner&&unifiedAvailable()&&settings()['mail.unifiedAccount']?.value===true){
  const selectedGroup=groupId!==undefined?groupId:role!==undefined?activeUnifiedGroup():unifiedDefaultGroup();
  if(activeUnifiedGroup()!==selectedGroup)setUnifiedFolderCounts({});
  setActiveUnifiedGroup(selectedGroup);setUnifiedVirtualMode(true);setUnifiedFolderRole(role??unifiedFolderRole());setRequest({owner});
 }
}
export function takeUnifiedMailboxRequest():boolean{const value=request();setRequest(undefined);return !!value&&preferenceAccountIsCurrent(value.owner);}
