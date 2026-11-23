import {apiFetch} from '../api/http.ts';
import {withBase} from '../api/basePath.ts';
import {alertValue,type AlertValues} from '../api/notificationControls.ts';
import {workspaceMailboxes,workspaceFolders,readWorkspaceJmap,readWorkspaceNotifications,type WorkspaceMailbox} from '../api/workspaceMail.ts';
import {CAP_CORE,CAP_MAIL} from '../api/jmap-types.ts';
import {responseFor} from '../api/jmap.ts';
import {activeLinkedMailbox} from '../api/linkedMailboxSession.ts';
import {preferenceAccountIsCurrent,type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {shouldNotify} from '../api/notificationPolicy.ts';
import {toast} from '../components/toast/toast.ts';
import {t,loadCatalog} from '../i18n/index.ts';
import type {NotificationConfig} from '../screens/Settings/types.ts';

type Checkpoint={checkedAt:number|null;now:number};
export type SummaryCounts={inbox:number;spam:number};
const key=(name:string)=>`notifications.loginSummary.${name}`;
export function summaryWindow(check:Checkpoint,values:AlertValues):{after:string;before:string}|null{
 if(check.checkedAt===null)return null;
 if(!Number.isSafeInteger(check.now)||!Number.isSafeInteger(check.checkedAt)||check.checkedAt>check.now)throw new Error('Invalid mail checkpoint');
 const start=Math.max(check.checkedAt,check.now-alertValue('notifications.loginSummary.lookbackDays',60,values)*86400);
 return {after:new Date(start*1000).toISOString(),before:new Date(check.now*1000).toISOString()};
}
export function summaryMayShow(check:Checkpoint,values:AlertValues,counts:SummaryCounts,config:NotificationConfig):boolean{
 if(check.checkedAt===null||!alertValue(key('enabled'),true,values)||check.now-check.checkedAt<alertValue(key('minimumAwayMinutes'),1,values)*60)return false;
 if(!alertValue(key('showEmpty'),false,values)&&counts.inbox+counts.spam===0)return false;
 // Content rules apply to individual live alerts, not aggregate counts. Only
 // the independently configured quiet-hours window applies to this summary.
 return !alertValue(key('quietHours'),true,values)||shouldNotify({...config,enabled:true,rules:[],presentation:{}},{});
}
async function checkpoint(target:WorkspaceMailbox,signal:AbortSignal,body?:unknown):Promise<Checkpoint&{accepted?:boolean}>{
 const response=await apiFetch(withBase('/api/account/mail-check'),{mailboxTarget:target,signal,credentials:'same-origin',cache:'no-store',pace:'background',...(body?{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(body)}:{})});
 if(!response.ok)throw new Error('Mail checkpoint unavailable');
 return response.json();
}
/** Two count queries at most; no message bodies or full message listings. */
export async function countSummary(target:WorkspaceMailbox,signal:AbortSignal,check:Checkpoint,values:AlertValues):Promise<SummaryCounts>{
 const window=summaryWindow(check,values),counts:SummaryCounts={inbox:0,spam:0};
 // First visit still verifies the folder list before acknowledging the baseline.
 const boxes=await workspaceFolders(target,signal);
 if(!window)return counts;
 const calls=(['inbox','spam'] as const).flatMap(role=>{
  if(!alertValue(key(role),true,values))return [];
  const folders=boxes.filter(box=>box.role===(role==='spam'?'junk':'inbox'));
  return folders.map((box,index)=>['Email/query',{accountId:target.accountId,filter:{inMailbox:box.id,...window,...(alertValue(key('unreadOnly'),false,values)?{notKeyword:'$seen'}:{})},limit:1,calculateTotal:true},`${role}-${index}`] as [string,Record<string,unknown>,string]);
 });
 if(!calls.length)return counts;
 const result=await readWorkspaceJmap(target,{using:[CAP_CORE,CAP_MAIL],methodCalls:calls},signal);
 for(const call of calls){
  const query=responseFor<{total?:number}>(result,call[2]);
  if(!Number.isSafeInteger(query.total)||query.total!<0)throw new Error('Mail count unavailable');
  counts[call[2].startsWith('spam')?'spam':'inbox']+=query.total!;
 }
 return counts;
}

/** Lifetime is one authenticated account generation. No background polling:
 * successful network folder checks renew the checkpoint at most every minute.
 * An offline/cache read never advances it. Claims are atomic across tabs. */
export function startLoginMailSummary(owner:PreferenceAccount):()=>void{
 const controller=new AbortController(),{signal}=controller;
 const completed=new Set<string>();
 let busy=false,initial=true,lastAttempt=0,retries=0,timer:ReturnType<typeof setTimeout>|undefined;
 const current=()=>!signal.aborted&&preferenceAccountIsCurrent(owner);
 const run=async()=>{
  if(busy||!current()||document.hidden||navigator.onLine===false)return;
  busy=true;lastAttempt=Date.now();clearTimeout(timer);timer=undefined;
  try{
   await loadCatalog('login-summary');
   const summaries:string[]=[],durations:number[]=[];let failed=false;
   const targets=await workspaceMailboxes(activeLinkedMailbox()?.linkId??null);
   if(!current())return;
   const active=targets.find(target=>target.current);if(!active)return;
   const activeConfig=await readWorkspaceNotifications(active);
   const linked=alertValue(key('linkedAccounts'),true,activeConfig.presentation);
   for(const target of targets.filter(target=>target.current||(initial&&linked))){
    if(!current())return;
    if(initial&&!target.current&&completed.has(target.storageAuthority))continue;
    try{
     const config=target.current?activeConfig:await readWorkspaceNotifications(target),values=config.presentation??{};
     const check=await checkpoint(target,signal);
     const counts=await countSummary(target,signal,check,values);
     if(!current())return;
     const result=await checkpoint(target,signal,{expected:check.checkedAt,checkedAt:check.now});
     if(!current())return;
     if(initial&&!completed.has(target.storageAuthority)&&result.accepted&&summaryMayShow(check,values,counts,config)){
      const parts:string[]=[];
      if(alertValue(key('inbox'),true,values))parts.push(t('mail-login-summary-inbox',{count:counts.inbox}));
      if(alertValue(key('spam'),true,values))parts.push(t('mail-login-summary-spam',{count:counts.spam}));
      const days=alertValue(key('lookbackDays'),60,values),limited=check.checkedAt!==null&&check.now-check.checkedAt>days*86400;
      if(parts.length){summaries.push(t(limited?'mail-login-summary-limited':'mail-login-summary',{account:target.label,counts:parts.join(' · '),days}));durations.push(alertValue(key('durationSeconds'),15,values));}
     }
     completed.add(target.storageAuthority);
    }catch{if(signal.aborted)return;failed=true;}
   }
   if(summaries.length)toast.info(summaries.join(' · '),{duration:Math.max(...durations)*1000,respectPreferences:true,testId:'login-mail-summary'});
   if(failed)throw new Error('Incomplete mail checks');
   initial=false;retries=0;
  }catch{
   // No baseline advance after a failed count. Bounded recovery also handles
   // login boot racing the linked-account catalog or initial connection.
   if(current()&&retries++<3)timer=setTimeout(()=>void run(),15_000);
  }finally{busy=false;}
 };
 const checked=(event:Event)=>{if((event as CustomEvent).detail?.accountId===owner.id&&Date.now()-lastAttempt>=60_000)void run();};
 const returned=()=>{if(!initial||!current())return;clearTimeout(timer);timer=setTimeout(()=>void run(),Math.max(0,Math.min(15_000,15_000-(Date.now()-lastAttempt))));};
 window.addEventListener('mw:mail-network-checked',checked);
 window.addEventListener('online',returned);document.addEventListener('visibilitychange',returned);
 void run();
 return()=>{controller.abort();clearTimeout(timer);window.removeEventListener('mw:mail-network-checked',checked);window.removeEventListener('online',returned);document.removeEventListener('visibilitychange',returned);};
}
