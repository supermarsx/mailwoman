import {toast} from '../../components/toast/toast.ts';
import type {SettingHandle} from './useSetting.ts';
import {For,Show,useContext,type JSX} from 'solid-js';
import {ALERT_CONTROLS,alertSlug} from '../../api/notificationControls.ts';
import {AppContext} from '../../state/context.ts';
import {useSetting} from './useSetting.ts';
import {SettingRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import {t} from '../../i18n/index.ts';
import * as css from './styles.css.ts';
type Control=typeof ALERT_CONTROLS[number];
async function saveControl<T>(handle:SettingHandle<T>,value:T):Promise<void>{if(!await handle.set(value))toast.error(handle.error()||t('settings-notif-error'),{action:{label:t('common-retry'),run:()=>saveControl(handle,value)}});}

function Control(props:{row:Control}):JSX.Element{
 const [key,fallback]=props.row,handle=useSetting<unknown>(key,fallback),id=`alert-${alertSlug(key)}`;
 const ranges:Record<string,readonly [number,number]>={'notifications.loginSummary.minimumAwayMinutes':[0,1440],'notifications.loginSummary.lookbackDays':[1,365],'notifications.loginSummary.durationSeconds':[2,120],'notifications.incoming.durationSeconds':[2,120],'notifications.toasts.limit':[1,8]};
 const [min,max]=ranges[key]??[1,60];
 const options=key.endsWith('.scope')?['received','inbox','selected']:['none','toast','native','both'];
 return <SettingRow controlId={id} label={t(`settings-alert-${alertSlug(key)}`)} help={t(`settings-alert-help-${alertSlug(key)}`)} handle={{...handle,error:()=>''}}>{description=>
  <Show when={typeof fallback!=='boolean'} fallback={<input id={id} type="checkbox" checked={handle.value()===true} disabled={!handle.entry()||handle.locked()||handle.saving()} aria-describedby={description()} onChange={e=>void saveControl(handle,e.currentTarget.checked)}/>}>
   <Show when={typeof fallback==='string'} fallback={<input id={id} type="number" min={min} max={max} class={css.input} value={Number(handle.value())} disabled={!handle.entry()||handle.locked()||handle.saving()} aria-describedby={description()} onChange={e=>void saveControl(handle,Number(e.currentTarget.value))}/>}>
    <select id={id} class={css.select} value={String(handle.value())} disabled={!handle.entry()||handle.locked()||handle.saving()} aria-describedby={description()} onChange={e=>void saveControl(handle,e.currentTarget.value)}><For each={options}>{option=><option value={option}>{t(`settings-alert-${key.endsWith('.scope')?'scope':'mode'}-${option}`)}</option>}</For></select>
   </Show>
  </Show>
 }</SettingRow>;
}
function FolderChoices(props:{muted?:boolean}):JSX.Element{
 const app=useContext(AppContext),key=props.muted?'notifications.incoming.mutedFolders':'notifications.incoming.folders',handle=useSetting<string[]>(key,[],(v):v is string[]=>Array.isArray(v)&&v.every(item=>typeof item==='string'));
 return <SettingRow controlId={`alert-${alertSlug(key)}`} label={t(`settings-alert-${alertSlug(key)}`)} handle={{...handle,error:()=>''}} stacked>{()=>
  <fieldset id={`alert-${alertSlug(key)}`} class={`${css.bareFieldset} ${css.stack}`} disabled={!handle.entry()||handle.locked()||handle.saving()}><For each={(app?.mailboxes()??[]).filter(box=>!['sent','drafts','trash'].includes(box.role??''))}>{box=><label class={css.actions}><input type="checkbox" checked={handle.value().includes(box.id)} onChange={e=>void saveControl(handle,e.currentTarget.checked?[...handle.value(),box.id]:handle.value().filter(id=>id!==box.id))}/>{box.name}</label>}</For></fieldset>
 }</SettingRow>;
}
export function NotificationControls(props:{group?:'incoming'|'events'|'toasts'|'loginSummary'}):JSX.Element{
 return <>
  <For each={props.group?[props.group]:['incoming','events','toasts'] as const}>{group=><SettingsCard id={`notif-card-${group}`} title={t(`settings-alert-${group==='incoming'?'channels':group}-title`)}><For each={ALERT_CONTROLS.filter(row=>row[0].startsWith(`notifications.${group}.`))}>{row=><Control row={row}/>}</For></SettingsCard>}</For>
  <Show when={!props.group||props.group==='incoming'}><SettingsCard id="notif-card-folders" title={t('settings-alert-folders-title')} description={t('settings-alert-folder-help')}><FolderChoices/><FolderChoices muted/><p class={css.meta}>{t('settings-alert-background-scope')}</p></SettingsCard></Show>
 </>;
}
export function StorageRequestControl():JSX.Element{return <Control row={ALERT_CONTROLS.find(row=>row[0]==='offline.requestPersistence')!}/>;}
