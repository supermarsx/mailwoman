import {createSignal} from 'solid-js';
import type {Email} from '../api/jmap-types.ts';
import {t} from '../i18n';
import {accountUiPreferences,setAccountUiPreferences} from '../api/accountPreferences.ts';
/** A request to scroll a message into view and flash it; MessageList consumes it once the row exists. */
export interface FocusRequest {emailId:string;nonce:number;/** The target is in the same conversation as the message the jump started from: open the conversation so its row shows. */thread?:boolean}
const [focusRequest,setFocusRequest]=createSignal<FocusRequest|null>(null);
export {focusRequest};
let nonce=0;
export const FOCUS_MAX_PAGES=40;
interface FocusApp {
 accountId:()=>string|null|undefined;selectedMailboxId:()=>string|null;searchActive:()=>boolean;
 selectMailbox:(id:string)=>Promise<void>;messages:()=>Email[];hasMore:()=>boolean;loadMore:()=>Promise<void>;
 mailboxes:()=>{id:string;name:string;parentId?:string|null}[];showToast:(tone:'success'|'error'|'info',message:string)=>void;openMessage?:(id:string)=>Promise<void>;
}
/** The folders above `folderId`, nearest first (a cycle in the data ends the walk). */
export function folderAncestors(boxes:readonly {id:string;parentId?:string|null}[],folderId:string):string[] {
 const byId=new Map(boxes.map(box=>[box.id,box])),out:string[]=[];
 for(let at=byId.get(folderId)?.parentId;at&&!out.includes(at)&&out.length<64;at=byId.get(at)?.parentId)out.push(at);
 return out;
}
/** Open every collapsed ancestor of `folderId` in the folder tree, so the folder being gone to is visible there. */
export function expandFolderTree(boxes:readonly {id:string;parentId?:string|null}[],folderId:string):void {
 try{
  const collapsed=accountUiPreferences().folderCollapsed??[];
  const open=folderAncestors(boxes,folderId).filter(id=>collapsed.includes(id));
  if(open.length)setAccountUiPreferences({folderCollapsed:collapsed.filter(id=>!open.includes(id))});
 }catch{/* no account preferences yet (signed out): nothing to expand */}
}
/** Go to `folderId`, page the list forward (capped) until `emailId` is loaded, then scroll it to the centre and flash it.
 * Returns false (with a toast) when the message is not in that folder any more, the account changed, or the page cap was hit. */
export async function focusMessageInFolder(app:FocusApp,folderId:string,emailId:string,options:{open?:boolean;maxPages?:number;thread?:boolean}={}):Promise<boolean> {
 const account=app.accountId(),name=app.mailboxes().find(box=>box.id===folderId)?.name??folderId;
 const fail=()=>{app.showToast('error',t('mail-focus-not-found',{folder:name}));return false;};
 if(!account)return false;
 expandFolderTree(app.mailboxes(),folderId);
 if(app.selectedMailboxId()!==folderId||app.searchActive())await app.selectMailbox(folderId);
 for(let pages=0;pages<=(options.maxPages??FOCUS_MAX_PAGES);pages++){
  if(app.accountId()!==account)return false;
  if(app.messages().some(message=>message.id===emailId))break;
  if(!app.hasMore())return fail();
  if(pages===(options.maxPages??FOCUS_MAX_PAGES))return fail();
  await app.loadMore();
 }
 if(app.accountId()!==account||app.selectedMailboxId()!==folderId)return false;
 setFocusRequest({emailId,nonce:++nonce,...(options.thread?{thread:true}:{})});
 if(options.open&&app.openMessage)await app.openMessage(emailId);
 return true;
}
export const clearFocusRequest=():void=>{setFocusRequest(null);};
