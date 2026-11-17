import {createEffect} from 'solid-js';
import {linkedAccountsRevision} from '../api/linkedMailboxSession.ts';
import {createIncomingMail,type IncomingMail} from './incomingMail.ts';
import {workspaceMailboxes,readWorkspaceJmap,type WorkspaceMailbox} from '../api/workspaceMail.ts';
import {observePreferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {CAP_MAIL,type Mailbox,type MailboxGetResponse} from '../api/jmap-types.ts';
import {responseFor} from '../api/jmap.ts';

/** Inactive owned accounts keep their own change cursor; unread-count increases
 * alone never trigger a notification. Requests use the verified target context. */
export function createFamilyIncomingMail(options:{activeLink:()=>string|null;deliver:(batch:IncomingMail)=>void}):{start:()=>void;stop:()=>void;changed:(link?:string|null,authority?:string)=>void}{
 let active=false,epoch=0,off:(()=>void)|undefined;
 let catalogLoaded=false,catalogLoading=false;
 const entries=new Map<string,{target:WorkspaceMailbox;monitor:ReturnType<typeof createIncomingMail>;abort:AbortController;refresh:()=>Promise<void>}>();
 function clear():void{epoch++;for(const entry of entries.values()){entry.abort.abort();entry.monitor.stop();}entries.clear();}
 async function load():Promise<void>{
  clear();catalogLoaded=false;catalogLoading=true;const generation=epoch;
  try{
   const targets=await workspaceMailboxes(options.activeLink());
   if(!active||generation!==epoch)return;
   for(const target of targets.filter(target=>!target.current)){
    const abort=new AbortController();let boxes:Mailbox[]=[];let busy=false,pending=false;
    const current=():boolean=>active&&generation===epoch&&preferenceAccountIsCurrent(target.owner)&&!abort.signal.aborted;
    const jmap=(body:Parameters<typeof readWorkspaceJmap>[1])=>readWorkspaceJmap(target,body,abort.signal);
    const monitor=createIncomingMail({owner:()=>target.owner,accountId:()=>target.accountId,client:{jmap},mailboxes:()=>boxes,deliver:batch=>{if(current())options.deliver({...batch,target,mailboxes:batch.mailboxes??boxes});}});
    const refresh=async():Promise<void>=>{
     if(!current())return;if(busy){pending=true;return;}busy=true;
     try{do{
      pending=false;
      const result=await jmap({using:[CAP_MAIL],methodCalls:[['Mailbox/get',{accountId:target.accountId},'family-boxes']]});
      if(!current())return;const got=responseFor<MailboxGetResponse>(result,'family-boxes');
      if(got.accountId!==target.accountId||!Array.isArray(got.list))return;boxes=got.list;monitor.changed();
     }while(pending&&current());}catch{/* A later frame or fallback tick retries without advancing the mail cursor. */}finally{busy=false;}
    };
    entries.set(target.linkId??'',{target,monitor,abort,refresh});
    // Seed the mail state immediately, before a slow folder metadata request.
    monitor.start();void refresh();
   }
   catalogLoaded=true;
  }catch{/* No unverified target or stale account is allowed to deliver. */}finally{if(generation===epoch)catalogLoading=false;}
 }
 createEffect(()=>{linkedAccountsRevision();if(active)void load();});
 return {
  start(){if(active)return;active=true;off=observePreferenceAccount(()=>{void load();});},
  stop(){active=false;off?.();off=undefined;clear();},
  changed(link,authority){if(active&&!catalogLoaded&&!catalogLoading&&entries.size===0){void load();return;}for(const entry of entries.values())if((link===undefined||entry.target.linkId===link)&&(!authority||entry.target.storageAuthority===authority))void entry.refresh();},
 };
}
