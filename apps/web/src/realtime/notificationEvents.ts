import {alertValue} from '../api/notificationControls.ts';
import {connectionToastAllowed} from '../api/notificationRuntime.ts';
import {capabilityEnabled} from '../platform/capabilities.ts';
import {getPlatform} from '../platform/index.ts';
import {toast,type ToastTone} from '../components/toast/toast.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
export type AlertEvent='queued'|'sent'|'failed'|'security'|'update';
/** Submission success means acceptance by the sending service, never recipient delivery. */
export function notifyEvent(event:AlertEvent,message:string,tone:ToastTone='info',id?:string,presentToast?:()=>void):void{
 const mode=alertValue<string>(`notifications.events.${event}`,'toast');if(mode==='none'||!connectionToastAllowed())return;
 if((mode==='toast'||mode==='both')&&presentToast)presentToast();
 else if(mode==='toast'||mode==='both')toast.show(tone,message,{...(id?{id}:{}),testId:`notification-${event}`});
 if(mode==='native'||mode==='both'){
  const owner=preferenceAccount();if(!owner)return;
  if(capabilityEnabled('notifications')||(typeof Notification!=='undefined'&&Notification.permission==='granted'))void Promise.resolve().then(()=>{if(preferenceAccountIsCurrent(owner))return getPlatform().notify({id:id??`mw-${event}-${Date.now()}`,title:'Mailwoman',body:message});}).catch(()=>undefined);
 }
}
