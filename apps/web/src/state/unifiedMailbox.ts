import {createSignal} from 'solid-js';
import {preferenceAccount,preferenceAccountIsCurrent,type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import type {UnifiedRole} from '../api/workspaceMail.ts';
const [request,setRequest]=createSignal<{owner:PreferenceAccount}>();
export const unifiedMailboxRequest=request;
export const [unifiedMailboxActive,setUnifiedMailboxActive]=createSignal(false);
export const [unifiedVirtualMode,setUnifiedVirtualMode]=createSignal(false);
export const [unifiedFolderRole,setUnifiedFolderRole]=createSignal<UnifiedRole>('inbox');
export const [unifiedFolderCounts,setUnifiedFolderCounts]=createSignal<Partial<Record<UnifiedRole,{total:number|null;unread:number|null}>>>({});
export function requestUnifiedMailbox(role=unifiedFolderRole()):void{const owner=preferenceAccount();if(owner){setUnifiedVirtualMode(true);setUnifiedFolderRole(role);setRequest({owner});}}
export function takeUnifiedMailboxRequest():boolean{const value=request();setRequest(undefined);return !!value&&preferenceAccountIsCurrent(value.owner);}
