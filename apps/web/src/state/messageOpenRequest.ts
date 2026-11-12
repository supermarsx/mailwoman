import {createSignal} from 'solid-js';
import {preferenceAccount,preferenceAccountIsCurrent,type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
const [request,setRequest]=createSignal<{messageId:string;owner:PreferenceAccount}>();
export const messageOpenRequest=request;
export function requestMessageOpen(messageId:string,owner=preferenceAccount()):void{if(owner&&preferenceAccountIsCurrent(owner))setRequest({messageId,owner});}
export function takeMessageOpenRequest():string|null{const value=request();setRequest(undefined);return value&&preferenceAccountIsCurrent(value.owner)?value.messageId:null;}
