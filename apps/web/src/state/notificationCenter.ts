import {createSignal} from 'solid-js';
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
async function request(owner:PreferenceAccount,action?:{id:string;action:CenterAction},signal?:AbortSignal):Promise<unknown>{
 if(!preferenceAccountIsCurrent(owner))throw new DOMException('Account changed','AbortError');
 const response=await apiFetch(withBase('/api/account/notification-center'),{cache:'no-store',headers:{...storageAuthorityHeaders(owner.storageAuthority),'X-Mailwoman-Account':owner.id,...(action?{'content-type':'application/json'}:{})},...(signal?{signal}:{}),...(action?{method:'POST',body:JSON.stringify(action)}:{})});
 if(!response.ok)throw new Error('Notification centre unavailable');
 const value:unknown=await response.json();if(!preferenceAccountIsCurrent(owner))throw new DOMException('Account changed','AbortError');return value;
}
export async function refreshNotificationCenter():Promise<void>{
 const owner=preferenceAccount();if(!owner)return;const mine=++epoch;controller?.abort();const abort=new AbortController();controller=abort;setLoading(true);
 try{const value=await request(owner,undefined,abort.signal) as {notices?:CenterNotice[]};if(mine===epoch){setNotices(Array.isArray(value.notices)?value.notices:[]);setFailed(false);}}
 catch{if(mine===epoch&&!abort.signal.aborted)setFailed(true);}
 finally{if(mine===epoch)setLoading(false);}
}
export async function changeCenterNotice(id:string,action:CenterAction):Promise<void>{
 const owner=preferenceAccount();if(!owner)throw new DOMException('No account','AbortError');
 const value=await request(owner,{id,action}) as {notice?:CenterNotice};if(!value.notice)throw new Error('Invalid notification response');
 controller?.abort();epoch++;setLoading(false);
 const notice=value.notice;setNotices(old=>[notice,...old.filter(row=>row.id!==notice.id)].sort((a,b)=>b.ts-a.ts));
 window.dispatchEvent(new Event('mw:notification-center-changed'));
}
export function publishAvailableUpdate(build:string):void{if(build.length<=240&&preferenceAccount())void changeCenterNotice(`update:${build}`,'publish-update').catch(()=>undefined);}
export function startNotificationCenter():()=>void{
 const off=observePreferenceAccount(owner=>{controller?.abort();epoch++;activeOwner=owner;setNotices([]);setFailed(false);setLoading(false);if(owner)void refreshNotificationCenter();});
 const refresh=()=>{if(document.visibilityState==='visible'&&activeOwner)void refreshNotificationCenter();};
 const wake=(event:MessageEvent)=>{if(event.data?.type==='mw-push-wake')refresh();};
 window.addEventListener('focus',refresh);window.addEventListener('online',refresh);document.addEventListener('visibilitychange',refresh);navigator.serviceWorker?.addEventListener?.('message',wake);
 const timer=setInterval(refresh,60_000);
 return ()=>{off();clearInterval(timer);window.removeEventListener('focus',refresh);window.removeEventListener('online',refresh);document.removeEventListener('visibilitychange',refresh);navigator.serviceWorker?.removeEventListener?.('message',wake);controller?.abort();epoch++;activeOwner=null;setNotices([]);setLoading(false);};
}
