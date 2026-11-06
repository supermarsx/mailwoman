import {For,Show,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {SettingRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import {ToggleRow,type CardSpec} from './layout/rows.tsx';
import {t} from '../../i18n';
import {FOLDER_COUNT_MODES,FOLDER_COUNT_SCOPES} from '../../api/countBadges.ts';
import * as css from './styles.css.ts';
// Counter bubbles (t39): what a folder count shows, which folders show one, the
// smaller secondary size, and the opt-in bubble beside the mailbox name.
export const FOLDER_COUNT_CARD:CardSpec={id:'folders-card-counts',section:'folders',title:'settings-folder-counts-card',help:'settings-folder-counts-card-help',rows:[
 {id:'folder-counts',label:'settings-folder-counts'},{id:'folder-count-scope',label:'settings-folder-count-scope',help:'settings-folder-count-scope-help'},
 {id:'folder-counts-compact',label:'settings-folder-counts-compact'},{id:'account-unread-badge',label:'settings-account-unread-badge'}]};
export function FolderCountPreferences():JSX.Element {
 const counts=useSetting<string>('mail.folderCounts','both'),scope=useSetting<string>('mail.folderCountScope','all'),compact=useSetting<boolean>('mail.compactFolderBadges',true),head=useSetting<boolean>('mail.accountUnreadBadge',false);
 const off=(s:{entry:()=>unknown;locked:()=>boolean;saving:()=>boolean})=>!s.entry()||s.locked()||s.saving();
 return <SettingsCard id="folders-card-counts" title={t('settings-folder-counts-card')} description={t('settings-folder-counts-card-help')}>
  <SettingRow controlId="folder-counts" label={t('settings-folder-counts')} handle={counts} resetLabel={t('settings-folder-counts-reset')}>{d=><select id="folder-counts" class={css.select} value={counts.value()} disabled={off(counts)} aria-describedby={d()} onChange={event=>void counts.set(event.currentTarget.value)}><For each={FOLDER_COUNT_MODES}>{value=><option value={value}>{t(`settings-folder-counts-${value}`)}</option>}</For></select>}</SettingRow>
  <SettingRow controlId="folder-count-scope" label={t('settings-folder-count-scope')} help={t('settings-folder-count-scope-help')} handle={scope}>{d=><select id="folder-count-scope" class={css.select} value={scope.value()} disabled={off(scope)} aria-describedby={d()} onChange={event=>void scope.set(event.currentTarget.value)}><For each={FOLDER_COUNT_SCOPES}>{value=><option value={value}>{t(`settings-folder-count-scope-${value}`)}</option>}</For></select>}</SettingRow>
  <SettingRow controlId="folder-counts-compact" label={t('settings-folder-counts-compact')} handle={compact} resetLabel={t('settings-folder-counts-reset-size')}>{d=><input id="folder-counts-compact" type="checkbox" checked={compact.value()} disabled={off(compact)} aria-describedby={d()} onChange={event=>void compact.set(event.currentTarget.checked)}/>}</SettingRow>
  <ToggleRow id="account-unread-badge" label={t('settings-account-unread-badge')} handle={head}/>
  <Show when={counts.error()||scope.error()||compact.error()||head.error()}><p role="alert">{counts.error()||scope.error()||compact.error()||head.error()}</p></Show>
 </SettingsCard>;
}
