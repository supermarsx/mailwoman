import * as css from './styles.css.ts';
import {SettingRow} from './layout/SettingRow.tsx';
import {Language} from './Language.tsx';
import { For, Show, type JSX } from 'solid-js';
import {TimeZoneCombobox} from '../../components/TimeZoneCombobox.tsx';
import { t } from '../../i18n';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import type { SettingsNavigation } from './navigation.tsx';
import { ControlRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
import {RegionalFormats} from './RegionalFormats.tsx';

function ZoneField(props:{kind:'displayZone'|'comparisonZone';navigation:SettingsNavigation}):JSX.Element {
 const setting=useSetting<string>(`time.${props.kind}`,props.kind==='displayZone'?'local':'');
 const disabled=()=>!setting.entry()||setting.locked()||setting.saving();
 const extra=props.kind==='comparisonZone'?[{value:'',label:t('settings-time-none')}]:[];
 return <SettingRow controlId={`time-${props.kind}`} label={t(`settings-time-${props.kind}`)} help={t(`settings-time-${props.kind}-help`)} handle={setting}>{description=><TimeZoneCombobox id={`time-${props.kind}`} hideLabel label={t(`settings-time-${props.kind}`)} value={setting.value()} browserValue={props.kind==='displayZone'?'local':undefined} extra={extra} disabled={disabled()} onChange={value=>void setting.set(value)} describedBy={description()}/>}</SettingRow>;
}
export function TimeZones(props:{navigation:SettingsNavigation}):JSX.Element {
  const helpers=useSetting<boolean>('time.showHelpers',true);
  return <div class={css.stack}><Language/><RegionalFormats navigation={props.navigation}/><SettingsCard id="time-card" wide title={t('settings-time-title')} description={t('settings-time-help')}>
    <ControlRow controlId="time-helpers" label={t('settings-time-helpers')} note={<><LockNote locked={helpers.locked()}/>
    <Show when={helpers.error()}><p role="alert">{helpers.error()}</p></Show></>}>{()=>
      <input id="time-helpers" type="checkbox" checked={helpers.value()} disabled={!helpers.entry()||helpers.locked()||helpers.saving()} onChange={event=>void helpers.set(event.currentTarget.checked)}/>
    }</ControlRow>
    <For each={['displayZone','comparisonZone'] as const}>{kind=><ZoneField kind={kind} navigation={props.navigation}/>}</For>
  </SettingsCard></div>;
}
