import {createSignal,For,Show,onMount,type JSX} from 'solid-js';
import {notificationNotices,notificationLoading,notificationFailed,refreshNotificationCenter,changeCenterNotice,type CenterNotice,type CenterAction} from '../../state/notificationCenter.ts';
import {t,loadCatalog,formatDateTime} from '../../i18n';
import {requestSettings} from '../../state/settingsRequest.ts';
import {reloadNow} from '../../state/NewVersionToast.tsx';
import {toast} from '../../components/toast/toast.ts';
import {Icon} from '../../components/Icon.tsx';
import {SessionNoticeDetails} from '../../screens/Settings/SessionNoticeDetails.tsx';
import {knownKind} from '../../screens/Settings/sessionAlertsModel.ts';
import * as card from '../../screens/Settings/sessionNotice.css.ts';
import * as css from '../../screens/Settings/styles.css.ts';
const events=new Set(['newDevice','newCountry','twoFactorChanged','delegateAdded','forwardingAdded','apiKeyCreated']);
export function notificationTitle(notice:CenterNotice):string{
 if(notice.kind==='update')return t('common-new-version');
 if(notice.kind==='login')return t('notification-center-login');
 if(notice.kind==='session')return t(`session-alerts-kind-${knownKind(notice.event)}`,{from:t('session-alerts-unknown-country'),to:notice.context?.country??t('session-alerts-unknown-country')});
 return t(events.has(notice.event)?`notification-center-${notice.event}`:'notification-center-security');
}
export default function Notifications():JSX.Element{
 onMount(()=>{void loadCatalog('security');void refreshNotificationCenter();});
 const [state,setState]=createSignal('active'),[query,setQuery]=createSignal(''),[kind,setKind]=createSignal('all'),[busy,setBusy]=createSignal<string>();
 const shown=()=>notificationNotices().filter(n=>n.state===state()&&(kind()==='all'||n.kind===kind())&&`${notificationTitle(n)} ${n.context?.ip??''} ${n.context?.browser??''}`.toLowerCase().includes(query().trim().toLowerCase()));
 async function action(notice:CenterNotice,action:CenterAction):Promise<void>{setBusy(notice.id);try{await changeCenterNotice(notice.id,action);}catch{toast.error(t('notification-center-error'));}finally{setBusy(undefined);}}
 async function open(notice:CenterNotice):Promise<void>{
  if(notice.kind==='update'){reloadNow();return;}
  void action(notice,'read');
  if(notice.event==='apiKeyCreated')requestSettings('access');
  else if(notice.event==='forwardingAdded')requestSettings('rules');
  else requestSettings('security',notice.kind==='login'?'security-history-card':notice.kind==='session'?'session-alerts-card':['newDevice','newCountry','delegateAdded'].includes(notice.event)?'sessions-card':notice.event==='twoFactorChanged'?'twofa-card-overview':'security-activity-card');
 }
 return <section class={css.section} style={{'max-inline-size':'64rem','inline-size':'100%','box-sizing':'border-box'}} aria-label={t('common-module-notifications')} data-testid="notification-center">
  <h2 class={css.heading}>{t('common-module-notifications')}</h2><p class={css.prose}>{t('notification-center-help')}</p>
  <div class={css.actions}><For each={['active','archived','dismissed']}>{value=><button type="button" class={css.ghost} aria-pressed={state()===value} onClick={()=>setState(value)}>{t(`notification-center-${value}`)}</button>}</For><button type="button" class={css.ghost} disabled={notificationLoading()} onClick={()=>void refreshNotificationCenter()}><Icon name="refresh"/>{t('common-refresh')}</button></div>
  <div class={css.actions}><label>{t('common-search')}<input class={css.input} type="search" value={query()} onInput={event=>setQuery(event.currentTarget.value)}/></label><label>{t('notification-center-type')}<select class={css.input} value={kind()} onChange={event=>setKind(event.currentTarget.value)}><For each={['all','security','session','login','update']}>{value=><option value={value}>{t(`notification-center-type-${value}`)}</option>}</For></select></label></div>
  <Show when={notificationFailed()}><p role="alert">{t('notification-center-error')}</p></Show>
  <Show when={shown().length} fallback={<p class={css.meta} role="status">{t(notificationLoading()?'common-loading':'notification-center-empty')}</p>}>
   <ul class={css.list}><For each={shown()}>{notice=><li class={card.notice} data-unread={notice.read?'false':'true'}>
    <strong><Icon name={notice.kind==='update'?'download':notice.kind==='login'?'sessions':'shield'}/>{notificationTitle(notice)}</strong><time datetime={new Date(notice.ts*1000).toISOString()} class={css.meta}>{formatDateTime(notice.ts*1000)}</time>
    <Show when={notice.context}><SessionNoticeDetails context={notice.context}/></Show>
    <div class={css.actions}><button type="button" class={css.ghost} disabled={!!busy()} onClick={()=>void open(notice)}><Icon name="openSide"/>{t(notice.kind==='update'?'common-new-version-reload':'notification-center-open')}</button>
     <button type="button" class={css.ghost} disabled={!!busy()||notice.state==='dismissed'} onClick={()=>void action(notice,'dismiss')}><Icon name="close"/>{t('notification-center-dismiss')}</button>
     <button type="button" class={css.ghost} disabled={!!busy()||notice.state==='archived'} onClick={()=>void action(notice,'archive')}><Icon name="archive"/>{t('notification-center-archive')}</button>
     <Show when={notice.state!=='active'}><button type="button" class={css.ghost} disabled={!!busy()} onClick={()=>void action(notice,'restore')}><Icon name="restore"/>{t('notification-center-restore')}</button></Show>
    </div>
   </li>}</For></ul>
  </Show>
 </section>;
}
