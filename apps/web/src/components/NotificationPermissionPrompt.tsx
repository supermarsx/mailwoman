import {createResource,createSignal,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {loadSettings,settings,settingsLoadStatus} from '../api/settings.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent,type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {SettingsService} from '../screens/Settings/service.ts';
import {loadCatalog,t} from '../i18n/index.ts';
import {Icon} from './Icon.tsx';
import {toast} from './toast/toast.ts';
import * as css from './notificationPermissionPrompt.css.ts';

type Service=Pick<SettingsService,'notifications'|'saveNotifications'>;
const permission=():NotificationPermission|'unsupported'=>typeof Notification==='undefined'?'unsupported':Notification.permission;
function dismissalKey(account:PreferenceAccount):string|null{return account.storageAuthority?`mw.notification-permission:v1:${account.storageAuthority}`:null;}
function dismissed(account:PreferenceAccount):boolean {const key=dismissalKey(account);try{return key!==null&&sessionStorage.getItem(key)==='later';}catch{return false;}}

/** The app asks by default; the browser prompt runs only from the Allow gesture.
 * The administrator's policy and verified owner are checked before every write. */
export function NotificationPermissionPrompt(props:{service?:Service}):JSX.Element {
 const service=props.service??new SettingsService();
 let disposed=false;
 onCleanup(()=>{disposed=true;});
 const [account,setAccount]=createSignal(preferenceAccount()),[policyOwner,setPolicyOwner]=createSignal<PreferenceAccount|null>(null);
 const [devicePermission,setDevicePermission]=createSignal(permission()),[later,setLater]=createSignal<PreferenceAccount|null>(null),[busy,setBusy]=createSignal(false);
 function allowed(owner:PreferenceAccount):boolean {
  const values=settings(),enabled=values['notifications.enabled'];
  return !disposed&&preferenceAccountIsCurrent(owner)&&policyOwner()===owner&&settingsLoadStatus()==='ready'&&values['notifications.askPermission']?.value===true&&enabled!==undefined&&!(enabled.locked&&enabled.value!==true);
 }
 const eligible=()=>{const owner=account();return owner&&allowed(owner)&&devicePermission()==='default'&&later()!==owner&&!dismissed(owner)?owner:false;};
 const [config]=createResource(eligible,async owner=>({owner,value:await service.notifications()}));
 const visible=()=>{const owner=eligible();return owner&&config.state==='ready'&&config.latest?.owner===owner;};
 function defer(owner:PreferenceAccount):void {setLater(owner);const key=dismissalKey(owner);try{if(key)sessionStorage.setItem(key,'later');}catch{/* Session-only memory still prevents repeated prompts. */}}
 async function allow():Promise<void> {
  const owner=account();if(!owner||!visible()||busy()||!allowed(owner)||typeof Notification==='undefined')return;
  setBusy(true);
  try{
   const result=await Notification.requestPermission();
   if(disposed||!preferenceAccountIsCurrent(owner))return;
   setDevicePermission(result);defer(owner);
   if(result!=='granted'||!allowed(owner))return;
   // Preserve rules/quiet hours changed while the browser prompt was open.
   const latest=await service.notifications();
   if(!allowed(owner))return;
   if(!latest.enabled)await service.saveNotifications({...latest,enabled:true});
   if(allowed(owner)){window.dispatchEvent(new Event('mw:notifications-changed'));toast.success(t('common-notifications-enabled'));}
  }catch{if(!disposed&&preferenceAccountIsCurrent(owner))toast.error(t('common-notification-permission-error'));}
  finally{if(!disposed&&preferenceAccountIsCurrent(owner))setBusy(false);}
 }
 onMount(()=>{
  void loadCatalog('common');
  onCleanup(observePreferenceAccount(owner=>{
   setAccount(owner);setPolicyOwner(null);setBusy(false);setDevicePermission(permission());
   if(owner&&permission()==='default')void loadSettings({refresh:true}).then(()=>{if(!disposed&&preferenceAccountIsCurrent(owner))setPolicyOwner(owner);});
  }));
  const refresh=()=>setDevicePermission(permission());window.addEventListener('focus',refresh);document.addEventListener('visibilitychange',refresh);
  onCleanup(()=>{window.removeEventListener('focus',refresh);document.removeEventListener('visibilitychange',refresh);});
 });
 return <Show when={visible()}><aside class={css.root} role="dialog" aria-label={t('common-notification-permission-title')}>
  <div class={css.message}><Icon name="bell"/><div><strong>{t('common-notification-permission-title')}</strong><p>{t('common-notification-permission-help')}</p></div></div>
  <div class={css.actions}><button type="button" class="btn btn--primary" disabled={busy()} onClick={()=>void allow()}>{t('common-notification-permission-allow')}</button><button type="button" class="btn btn--ghost" disabled={busy()} onClick={()=>{const owner=account();if(owner)defer(owner);}}>{t('common-notification-permission-later')}</button></div>
 </aside></Show>;
}
