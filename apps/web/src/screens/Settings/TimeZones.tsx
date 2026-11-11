import { createEffect, createMemo, createSignal, For, Show, type JSX } from 'solid-js';
import { validTimeZone } from '../../i18n/timezones.ts';
import {TimeZoneCombobox} from '../../components/TimeZoneCombobox.tsx';
import { t } from '../../i18n';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import { useLeaveConfirm } from '../../components/confirm.tsx';
import type { SettingsNavigation } from './navigation.tsx';
import { ControlRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
import * as kit from './layout/layout.css.ts';
import * as css from './styles.css.ts';
import {RegionalFormats} from './RegionalFormats.tsx';

function ZoneField(props:{kind:'displayZone'|'comparisonZone';navigation:SettingsNavigation}):JSX.Element {
  const setting=useSetting<string>(`time.${props.kind}`,props.kind==='displayZone'?'local':'');
  const [draft,setDraft]=createSignal(setting.value());const [error,setError]=createSignal('');
  const saved=createMemo(()=>setting.value());createEffect(()=>setDraft(saved()));
  const disabled=()=>!setting.entry()||setting.locked()||setting.saving();
  const extra=props.kind==='comparisonZone'?[{value:'',label:t('settings-time-none')}]:[];
  const LeaveDialog=useLeaveConfirm(props.navigation,()=>draft()!==setting.value(),()=>t('settings-time-discard'));
  async function save(event:SubmitEvent):Promise<void> {
    event.preventDefault();const value=draft().trim();
    if(!validTimeZone(value)&&!(props.kind==='displayZone'&&value==='local')&&!(props.kind==='comparisonZone'&&value==='')){setError(t('settings-time-invalid'));return;}
    setError('');if(await setting.set(value))setDraft(value);
  }
  return <form class={`${kit.row} ${kit.rowStacked}`} data-setting-row onSubmit={event=>void save(event)}>
    <TimeZoneCombobox id={`time-${props.kind}`} label={t(`settings-time-${props.kind}`)} value={draft()} browserValue={props.kind==='displayZone'?'local':undefined} extra={extra} freeText disabled={disabled()} onChange={setDraft} describedBy={`time-${props.kind}-help`}/>
    <p id={`time-${props.kind}-help`} class={css.meta}>{t(`settings-time-${props.kind}-help`)}</p><LockNote locked={setting.locked()}/>
    <Show when={error()||setting.error()}><p role="alert">{error()||setting.error()}</p></Show>
    <div class={css.actions}><button type="submit" class={css.button} disabled={disabled()||draft()===setting.value()}>{t('settings-time-save')}</button><button type="button" class={css.ghost} disabled={disabled()} onClick={()=>void setting.reset()}>{t('reader-inherit')}</button></div>
    <LeaveDialog/>
  </form>;
}
export function TimeZones(props:{navigation:SettingsNavigation}):JSX.Element {
  const helpers=useSetting<boolean>('time.showHelpers',true);
  return <><RegionalFormats navigation={props.navigation}/><SettingsCard id="time-card" wide title={t('settings-time-title')} description={t('settings-time-help')}>
    <ControlRow controlId="time-helpers" label={t('settings-time-helpers')} note={<><LockNote locked={helpers.locked()}/>
    <Show when={helpers.error()}><p role="alert">{helpers.error()}</p></Show></>}>{()=>
      <input id="time-helpers" type="checkbox" checked={helpers.value()} disabled={!helpers.entry()||helpers.locked()||helpers.saving()} onChange={event=>void helpers.set(event.currentTarget.checked)}/>
    }</ControlRow>
    <For each={['displayZone','comparisonZone'] as const}>{kind=><ZoneField kind={kind} navigation={props.navigation}/>}</For>
  </SettingsCard></>;
}
