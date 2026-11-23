import {AppContext} from '../../state/context.ts';
import {OfflineQueueNotice,describeQueued} from '../../components/OfflineQueueNotice.tsx';
import {createRangeSelection} from '../../components/rangeSelection.ts';
import {RangeSelectionBar,RangeSelectionToggle} from '../../components/RangeSelectionBar.tsx';
import {SearchInput} from '../../components/SearchInput.tsx';
import {iconLabel,filters,filterField,center} from './style.css.ts';
import {createSignal,createEffect,useContext,For,Show,onMount,type JSX} from 'solid-js';
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
 const app=useContext(AppContext);
 onMount(()=>{void loadCatalog('security');void refreshNotificationCenter();});
 const [state,setState]=createSignal('active'),[query,setQuery]=createSignal(''),[kind,setKind]=createSignal('all'),[busy,setBusy]=createSignal<string>();
 const queueFailures=()=>state()==='active'&&(kind()==='all'||kind()==='queue')?(app?.offlineFailed()??[]).filter(item=>`${describeQueued(item)} ${item.lastError??''}`.toLowerCase().includes(query().trim().toLowerCase())):[];
 createEffect(()=>{if(notificationFailed())toast.error(t('notification-center-error'));});
 const shown=()=>notificationNotices().filter(n=>n.state===state()&&(kind()==='all'||n.kind===kind())&&`${notificationTitle(n)} ${n.context?.ip??''} ${n.context?.browser??''}`.toLowerCase().includes(query().trim().toLowerCase()));
 const bulk=createRangeSelection({ids:()=>shown().map(notice=>notice.id),scope:()=>JSON.stringify([state(),query(),kind()]),onError:()=>toast.error(t('notification-center-error'))});
 function success(action:CenterAction,count:number):void{toast.success(t('notification-center-action-success',{action:t(action==='dismiss'?'notification-center-dismissed':action==='archive'?'notification-center-archived':'notification-center-active'),count}));}
 function bulkAction(action:CenterAction):Promise<void>{return bulk.runAll(async(ids,isCurrent)=>{let completed=0;try{for(const id of ids){if(!isCurrent())break;await changeCenterNotice(id,action);completed++;}}finally{if(completed&&isCurrent())success(action,completed);}});}
 async function action(notice:CenterNotice,action:CenterAction):Promise<void>{setBusy(notice.id);try{await changeCenterNotice(notice.id,action);if(['dismiss','archive','restore'].includes(action))success(action,1);}catch{toast.error(t('notification-center-error'));}finally{setBusy(undefined);}}
 async function open(notice:CenterNotice):Promise<void>{
  if(notice.kind==='update'){reloadNow();return;}
  void action(notice,'read');
  if(notice.event==='apiKeyCreated')requestSettings('access');
  else if(notice.event==='forwardingAdded')requestSettings('rules');
  else requestSettings('security',notice.kind==='login'?'security-history-card':notice.kind==='session'?'session-alerts-card':['newDevice','newCountry','delegateAdded'].includes(notice.event)?'sessions-card':notice.event==='twoFactorChanged'?'twofa-card-overview':'security-activity-card');
 }
 return <section class={`${css.section} ${center}`} aria-label={t('common-module-notifications')} data-testid="notification-center" data-range-selection-root>
  <h2 class={`${css.heading} ${iconLabel}`}><Icon name="bell"/>{t('common-module-notifications')}</h2><p class={css.prose}>{t('notification-center-help')}</p>
  <div class={css.actions}><RangeSelectionToggle selection={bulk}/><For each={['active','archived','dismissed']}>{value=><button type="button" class={`${css.ghost} ${iconLabel}`} aria-pressed={state()===value} onClick={()=>setState(value)}>{t(`notification-center-${value}`)}</button>}</For></div>
  <div class={filters}><label class={filterField}><span class={iconLabel}><Icon name="search"/>{t('common-search')}</span><SearchInput class={css.input} aria-label={t('common-search')} value={query()} onInput={event=>setQuery(event.currentTarget.value)}/></label><label class={filterField}><span>{t('notification-center-type')}</span><select class={css.input} value={kind()} onChange={event=>setKind(event.currentTarget.value)}><For each={['all','security','session','login','update','queue']}>{value=><option value={value}>{t(value==='queue'?'mail-queue-notice-label':`notification-center-type-${value}`)}</option>}</For></select></label></div>
  <Show when={notificationFailed()}><button type="button" class={css.ghost} onClick={()=>void refreshNotificationCenter()}>{t('common-retry')}</button></Show>
  <Show when={queueFailures().length}><OfflineQueueNotice items={queueFailures()}/></Show>
  <Show when={shown().length} fallback={<Show when={!queueFailures().length&&!notificationFailed()}><p class={css.meta} role="status">{t(notificationLoading()?'common-loading':'notification-center-empty')}</p></Show>}>
   <ul class={css.list}><For each={shown()}>{notice=><li class={card.notice} tabIndex={0} data-selection-id={notice.id} data-bulk-selected={bulk.selected().has(notice.id)} onClick={event=>{if(!(event.target as Element).closest('button,input,a,summary,label,select,textarea'))bulk.click(notice.id,event);}} onKeyDown={event=>{if(event.target===event.currentTarget)bulk.keydown(notice.id,event);}} data-unread={notice.read?'false':'true'}>
    <div class={card.heading}><strong class={iconLabel}><Icon name={notice.kind==='update'?'download':notice.kind==='login'?'sessions':'shield'}/>{notificationTitle(notice)}</strong><time datetime={new Date(notice.ts*1000).toISOString()} class={css.meta}>{formatDateTime(notice.ts*1000)}</time></div>
    <Show when={notice.context}><SessionNoticeDetails context={notice.context}/></Show>
    <div class={css.actions}><button type="button" class={`${css.ghost} ${iconLabel}`} disabled={bulk.busy()||!!busy()} onClick={()=>void open(notice)}><Icon name="openSide"/>{t(notice.kind==='update'?'common-new-version-reload':'notification-center-open')}</button>
     <button type="button" class={`${css.ghost} ${iconLabel}`} disabled={bulk.busy()||!!busy()||notice.state==='dismissed'} onClick={()=>void action(notice,'dismiss')}><Icon name="close"/>{t('notification-center-dismiss')}</button>
     <button type="button" class={`${css.ghost} ${iconLabel}`} disabled={bulk.busy()||!!busy()||notice.state==='archived'} onClick={()=>void action(notice,'archive')}><Icon name="archive"/>{t('notification-center-archive')}</button>
     <Show when={notice.state!=='active'}><button type="button" class={`${css.ghost} ${iconLabel}`} disabled={bulk.busy()||!!busy()} onClick={()=>void action(notice,'restore')}><Icon name="restore"/>{t('notification-center-restore')}</button></Show>
    </div>
   </li>}</For></ul>
  </Show>
 <RangeSelectionBar selection={bulk} total={shown().length} actions={[
  {id:'dismiss',label:t('notification-center-dismiss'),icon:'close',run:()=>bulkAction('dismiss')},
  {id:'archive',label:t('notification-center-archive'),icon:'archive',run:()=>bulkAction('archive')},
  {id:'restore',label:t('notification-center-restore'),icon:'restore',run:()=>bulkAction('restore')},
 ]}/>
 </section>;
}
