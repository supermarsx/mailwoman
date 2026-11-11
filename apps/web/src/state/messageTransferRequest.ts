import {createSignal} from 'solid-js';
import {preferenceAccount,type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {activeLinkedMailbox} from '../api/linkedMailboxSession.ts';
import {settings} from '../api/settings.ts';
import type {Email} from '../api/jmap-types.ts';

export interface MessageTransferRequest {email:Email;sourceLink:string|null;owner:PreferenceAccount}
const [request,setRequest]=createSignal<MessageTransferRequest>();
export const pendingMessageTransfer=request;
export const transfersEnabled=():boolean=>settings()['mail.crossAccountTransfer']?.value===true;
export function requestMessageTransfer(email:Email,sourceLink=activeLinkedMailbox()?.linkId??null):void{
 const owner=preferenceAccount();
 if(owner&&transfersEnabled()&&!request())setRequest({email,sourceLink,owner});
}
export function closeMessageTransfer():void{setRequest(undefined);}
