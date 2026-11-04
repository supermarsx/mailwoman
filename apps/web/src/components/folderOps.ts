import type {Client} from '../api/client.ts';
import {markFolderRead,markUnread,transferFolder,undoTransfer,type BulkAuthority,type BulkResult} from '../api/folderBulk.ts';
import {isolate,t} from '../i18n';
import {toast} from './toast/toast.ts';
/** The folder context menu's whole-folder jobs: each reports progress in one persistent toast, then its result, with Undo where it can be undone. */
export interface FolderOpsApp {
  accountId:()=>string|null|undefined;
  clearFolderUnread:(ids:readonly string[])=>void;
  refreshFolders:()=>Promise<void>;
  refreshCurrentMailbox:()=>Promise<void>;
}
export interface FolderJobHooks {app:FolderOpsApp;client:Pick<Client,'jmap'>;accountId:string;current:()=>boolean;signal?:AbortSignal}
let sequence=0;
function authority(hooks:FolderJobHooks,stop?:{requested:boolean}):BulkAuthority {
 return {current:hooks.current,...(hooks.signal?{signal:hooks.signal}:{}),stopRequested:()=>stop?.requested===true};
}
async function settle(hooks:FolderJobHooks):Promise<void> {
 if(!hooks.current())return;
 await Promise.allSettled([hooks.app.refreshFolders(),hooks.app.refreshCurrentMailbox()]);
}
function report(id:string,name:string,result:BulkResult,done:string,undo?:()=>void):void {
 const counts={name:isolate(name),count:result.processed,total:result.total,refused:result.refused};
 if(result.stopped)toast.warning(t('folder-ops-stopped',counts),{id,duration:Infinity});
 else if(result.refused>0)toast.warning(t('folder-ops-partial',counts),{id,duration:Infinity});
 else toast.success(t(done,counts),{id,duration:undo?10_000:6_000,...(undo&&result.changed.length>0?{action:{label:t('folder-ops-undo'),run:undo}}:{})});
}
function failed(id:string,name:string):void {toast.error(t('folder-ops-failed',{name:isolate(name)}),{id,duration:Infinity});}
/** Mark every unread message in `mailboxIds` read. The badges clear at once and the server's counts replace the guess. */
export async function runMarkAllRead(hooks:FolderJobHooks,name:string,mailboxIds:readonly string[]):Promise<BulkResult|null> {
 if(!hooks.current())return null;
 const id=`folder-job:${++sequence}`,stop={requested:false};
 toast.info(t('folder-ops-marking',{name:isolate(name)}),{id,persistent:true,busy:true,duration:Infinity,action:{label:t('folder-ops-stop'),run:()=>{stop.requested=true;}}});
 hooks.app.clearFolderUnread(mailboxIds);
 try{
  const result=await markFolderRead(hooks.client,hooks.accountId,mailboxIds,authority(hooks,stop),(done,total)=>{if(hooks.current())toast.info(t('folder-ops-marking-progress',{name:isolate(name),done,total}),{id,persistent:true,busy:true,duration:Infinity});});
  if(!hooks.current())return result;
  await settle(hooks);
  report(id,name,result,'folder-ops-marked',()=>{void markUnread(hooks.client,hooks.accountId,result.changed,authority(hooks)).then(()=>settle(hooks)).catch(()=>toast.error(t('folder-ops-failed',{name:isolate(name)})));});
  return result;
 }catch{if(hooks.current()){await settle(hooks);failed(id,name);}return null;}
}
/** Copy or move every message of `sourceId` to `targetId`. */
export async function runTransfer(hooks:FolderJobHooks,name:string,sourceId:string,targetId:string,mode:'copy'|'move'):Promise<BulkResult|null> {
 if(!hooks.current())return null;
 const id=`folder-job:${++sequence}`,stop={requested:false};
 toast.info(t(mode==='move'?'folder-ops-moving':'folder-ops-copying',{name:isolate(name)}),{id,persistent:true,busy:true,duration:Infinity,action:{label:t('folder-ops-stop'),run:()=>{stop.requested=true;}}});
 try{
  const result=await transferFolder(hooks.client,hooks.accountId,sourceId,targetId,mode,authority(hooks,stop),(done,total)=>{if(hooks.current())toast.info(t(mode==='move'?'folder-ops-moving-progress':'folder-ops-copying-progress',{name:isolate(name),done,total}),{id,persistent:true,busy:true,duration:Infinity});});
  if(!hooks.current())return result;
  await settle(hooks);
  report(id,name,result,mode==='move'?'folder-ops-moved':'folder-ops-copied',mode==='move'?()=>{void undoTransfer(hooks.client,hooks.accountId,sourceId,targetId,result.changed,authority(hooks)).then(()=>settle(hooks)).catch(()=>toast.error(t('folder-ops-failed',{name:isolate(name)})));}:undefined);
  return result;
 }catch{if(hooks.current()){await settle(hooks);failed(id,name);}return null;}
}
