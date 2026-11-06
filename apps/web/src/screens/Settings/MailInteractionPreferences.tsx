import {For,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {SettingRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import {t} from '../../i18n';
export function MailInteractionPreferences():JSX.Element {
 const choices=[
  {key:'mail.resumeTabs',label:'settings-mail-resume-tabs',help:'settings-mail-resume-tabs-help',handle:useSetting<boolean>('mail.resumeTabs',true)},
  {key:'mail.modifierSelection',label:'settings-mail-modifier-selection',help:'settings-mail-modifier-selection-help',handle:useSetting<boolean>('mail.modifierSelection',true)},
  {key:'mail.dragToFolder',label:'settings-mail-drag-folder',help:'settings-mail-drag-folder-help',handle:useSetting<boolean>('mail.dragToFolder',true)},
 ];
 return <SettingsCard id="settings-card-mail-interactions" title={t('settings-mail-interactions')}><For each={choices}>{choice=>{
  const disabled=()=>!choice.handle.entry()||choice.handle.locked()||choice.handle.saving();
  return <SettingRow controlId={choice.key} label={t(choice.label)} help={t(choice.help)} handle={choice.handle}>{describedBy=>
   <input id={choice.key} type="checkbox" checked={choice.handle.value()} disabled={disabled()} onChange={event=>void choice.handle.set(event.currentTarget.checked)} aria-describedby={describedBy()}/>}</SettingRow>;
 }}</For></SettingsCard>;
}
