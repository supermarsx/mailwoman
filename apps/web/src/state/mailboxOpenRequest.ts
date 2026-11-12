import {createSignal} from 'solid-js';
import {preferenceAccount,preferenceAccountIsCurrent,type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
const [request,setRequest]=createSignal<{owner:PreferenceAccount}>();
export const mailboxOpenRequest=request;
export function requestMailboxOpen():void{const owner=preferenceAccount();if(owner)setRequest({owner});}
export function takeMailboxOpenRequest():boolean{const value=request();setRequest(undefined);return !!value&&preferenceAccountIsCurrent(value.owner);}
