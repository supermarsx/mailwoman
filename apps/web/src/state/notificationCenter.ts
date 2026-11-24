import {notifyEvent} from '../realtime/notificationEvents.ts';
import {t,loadCatalog} from '../i18n/index.ts';
import {useRealtime} from '../realtime/context.ts';
import {createSignal,createRoot,createEffect} from 'solid-js';
import {apiFetch} from '../api/http.ts';
import {withBase} from '../api/basePath.ts';
import {storageAuthorityHeaders} from '../api/storageAuthority.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent,type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import type {SessionNoticeContext} from '../screens/Settings/sessionAlertsModel.ts';
export interface CenterNotice {id:string;ts:number;kind:string;event:string;severity:string;state:'active'|'archived'|'dismissed';read:boolean;context?:SessionNoticeContext|null}
export type CenterAction='dismiss'|'archive'|'restore'|'read'|'publish-update';
const [notices,setNotices]=createSignal<CenterNotice[]>([]),[loading,setLoading]=createSignal(false),[failed,setFailed]=createSignal(false);
export const notificationNotices=notices,notificationLoading=loading,notificationFailed=failed;
export const notificationUnread=()=>notices().filter(n=>n.state==='active'&&!n.read).length;
let controller:AbortController|undefined,epoch=0,activeOwner:PreferenceAccount|null=null;
let seeded=false;const announced=new Set<string>();
// t52w: the page's load reads the list; the stream's first open and its first
// state frame (which names Notification only because the page had no state
// yet) used to read it twice more. Each of those is skipped once when a load
// for this owner finished less than LOAD_FRESH_MS ago; a reconnect, a moved
// Notification state, focus and wake still refresh.
const LOAD_FRESH_MS=10_000;let loadedAt=0,firstOnline=true;const notificationStates=new Map<string,string>();
const loadedRecently=():boolean=>loadedAt>0&&Date.now()-loadedAt<LOAD_FRESH_MS;
async function request(owner:PreferenceAccount,action?:{id:string;action:CenterAction},signal?:AbortSignal):Promise<unknown>{
 if(!preferenceAccountIsCurrent(owner))throw new DOMException('Account changed','AbortError');
 const response=await apiFetch(withBase('/api/account/notification-center'),{cache:'no-store',headers:{...storageAuthorityHeaders(owner.storageAuthority),'X-Mailwoman-Account':owner.id,...(action?{'content-type':'application/json'}:{})},...(signal?{signal}:{}),...(action?{method:'POST',body:JSON.stringify(action)}:{})});
 if(!response.ok)throw new Error('Notification centre unavailable');
 const value:unknown=await response.json();if(!preferenceAccountIsCurrent(owner))throw new DOMException('Account changed','AbortError');return value;
}
export async function refreshNotificationCenter():Promise<void>{
 const owner=preferenceAccount();if(!owner)return;const mine=++epoch;controller?.abort();const abort=new AbortController();controller=abort;setLoading(true);
 try{const value=await request(owner,undefined,abort.signal) as {notices?:CenterNotice[]};if(mine===epoch){const next=Array.isArray(value.notices)?value.notices:[];
 if(seeded)for(const row of next)if(row.state==='active'&&!row.read&&!announced.has(row.id)){
   const event=row.kind.includes('update')?'update':'security';
   notifyEvent(event,t(event==='update'?'settings-alert-new-update':'settings-alert-new-security'),row.severity==='critical'?'warning':'info',`notice:${row.id}`);
 }
 for(const row of next)announced.add(row.id);while(announced.size>2000)announced.delete(announced.values().next().value!);seeded=true;loadedAt=Date.now();
 setNotices(next);setFailed(false);}}
 catch{if(mine===epoch&&!abort.signal.aborted)setFailed(true);}
 finally{if(mine===epoch)setLoading(false);}
}
export async function changeCenterNotice(id:string,action:CenterAction):Promise<void>{
 const owner=preferenceAccount();if(!owner)throw new DOMException('No account','AbortError');
 const value=await request(owner,{id,action}) as {notice?:CenterNotice};if(!value.notice)throw new Error('Invalid notification response');
 controller?.abort();epoch++;setLoading(false);
 const notice=value.notice;if(action==='publish-update')announced.add(notice.id);setNotices(old=>[notice,...old.filter(row=>row.id!==notice.id)].sort((a,b)=>b.ts-a.ts));
 window.dispatchEvent(new Event('mw:notification-center-changed'));
}
export function publishAvailableUpdate(build:string):void{if(build.length<=240&&preferenceAccount())void changeCenterNotice(`update:${build}`,'publish-update').catch(()=>undefined);}
export function startNotificationCenter():()=>void{
 void loadCatalog('settings');const realtime=useRealtime();
 const realtimeOff=realtime.onStateChange(change=>{
  if(!activeOwner)return;let moved=false;
  for(const [account,states] of Object.entries(change.changed)){
   const next=states.Notification;if(next===undefined)continue;const previous=notificationStates.get(account);notificationStates.set(account,String(next));
   if(previous===undefined?!loadedRecently():previous!==String(next))moved=true;
  }
  if(moved)void refreshNotificationCenter();
 });
 const stopConnection=createRoot(dispose=>{let previous=realtime.connection.state();createEffect(()=>{const next=realtime.connection.state();if(next==='online'&&previous!=='online'&&activeOwner){const skip=firstOnline&&loadedRecently();firstOnline=false;if(!skip)void refreshNotificationCenter();}previous=next;});return dispose;});
 const off=observePreferenceAccount(owner=>{controller?.abort();epoch++;activeOwner=owner;seeded=false;loadedAt=0;firstOnline=true;notificationStates.clear();announced.clear();setNotices([]);setFailed(false);setLoading(false);if(owner)void refreshNotificationCenter();});
 const refresh=()=>{if(document.visibilityState==='visible'&&activeOwner)void refreshNotificationCenter();};
 const wake=(event:MessageEvent)=>{if(event.data?.type==='mw-push-wake')refresh();};
 window.addEventListener('focus',refresh);window.addEventListener('online',refresh);document.addEventListener('visibilitychange',refresh);navigator.serviceWorker?.addEventListener?.('message',wake);
 const timer=setInterval(()=>{if(realtime.connection.state()!=='online'||notificationFailed())refresh();},60_000);
 return ()=>{stopConnection();realtimeOff();off();clearInterval(timer);window.removeEventListener('focus',refresh);window.removeEventListener('online',refresh);document.removeEventListener('visibilitychange',refresh);navigator.serviceWorker?.removeEventListener?.('message',wake);controller?.abort();epoch++;activeOwner=null;setNotices([]);setLoading(false);};
}
