import {createSignal} from 'solid-js';
import type {Email} from '../api/jmap-types.ts';
import {t} from '../i18n';
/** A request to scroll a message into view and flash it; MessageList consumes it once the row exists. */
export interface FocusRequest {emailId:string;nonce:number}
const [focusRequest,setFocusRequest]=createSignal<FocusRequest|null>(null);
export {focusRequest};
let nonce=0;
export const FOCUS_MAX_PAGES=40;
interface FocusApp {
 accountId:()=>string|null|undefined;selectedMailboxId:()=>string|null;searchActive:()=>boolean;
 selectMailbox:(id:string)=>Promise<void>;messages:()=>Email[];hasMore:()=>boolean;loadMore:()=>Promise<void>;
 mailboxes:()=>{id:string;name:string}[];showToast:(tone:'success'|'error'|'info',message:string)=>void;openMessage?:(id:string)=>Promise<void>;
}
/** Go to `folderId`, page the list forward (capped) until `emailId` is loaded, then scroll it to the centre and flash it.
 * Returns false (with a toast) when the message is not in that folder any more, the account changed, or the page cap was hit. */
export async function focusMessageInFolder(app:FocusApp,folderId:string,emailId:string,options:{open?:boolean;maxPages?:number}={}):Promise<boolean> {
 const account=app.accountId(),name=app.mailboxes().find(box=>box.id===folderId)?.name??folderId;
 const fail=()=>{app.showToast('error',t('mail-focus-not-found',{folder:name}));return false;};
 if(!account)return false;
 if(app.selectedMailboxId()!==folderId||app.searchActive())await app.selectMailbox(folderId);
 for(let pages=0;pages<=(options.maxPages??FOCUS_MAX_PAGES);pages++){
  if(app.accountId()!==account)return false;
  if(app.messages().some(message=>message.id===emailId))break;
  if(!app.hasMore())return fail();
  if(pages===(options.maxPages??FOCUS_MAX_PAGES))return fail();
  await app.loadMore();
 }
 if(app.accountId()!==account||app.selectedMailboxId()!==folderId)return false;
 setFocusRequest({emailId,nonce:++nonce});
 if(options.open&&app.openMessage)await app.openMessage(emailId);
 return true;
}
export const clearFocusRequest=():void=>{setFocusRequest(null);};
