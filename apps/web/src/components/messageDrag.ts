import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent,type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {dragToFolderEnabled} from './mailInteractions.ts';
export const MESSAGE_DRAG_TYPE='application/x-mailwoman-message-selection';
interface DragSelection {token:string;owner:PreferenceAccount;ids:string[];expiresAt:number}
let selection:DragSelection|null=null;
observePreferenceAccount(()=>{selection=null;});
export function beginMessageDrag(event:DragEvent,accountId:string|null,ids:readonly string[]):boolean {
 selection=null;
 const owner=preferenceAccount();
 if(!dragToFolderEnabled()||!event.dataTransfer||!owner||owner.id!==accountId||!ids.length||ids.length>1000){event.preventDefault();return false;}
 const token=crypto.randomUUID();selection={token,owner,ids:[...new Set(ids)],expiresAt:Date.now()+30000};
 event.dataTransfer.effectAllowed='move';event.dataTransfer.setData(MESSAGE_DRAG_TYPE,token);
 // No subjects, bodies, addresses or arbitrary externally supplied IDs.
 return true;
}
export function clearMessageDrag():void {selection=null;}
export function peekMessageDrag(event:DragEvent,accountId:string|null):readonly string[]|null {
 return dragToFolderEnabled()&&selection&&selection.expiresAt>Date.now()&&selection.owner.id===accountId&&preferenceAccountIsCurrent(selection.owner)&&event.dataTransfer?.types.includes(MESSAGE_DRAG_TYPE)?selection.ids:null;
}
export function takeMessageDrag(event:DragEvent,accountId:string|null):readonly string[]|null {
 const ids=peekMessageDrag(event,accountId);
 if(!ids||event.dataTransfer?.getData(MESSAGE_DRAG_TYPE)!==selection?.token)return null;
 selection=null;return ids;
}
