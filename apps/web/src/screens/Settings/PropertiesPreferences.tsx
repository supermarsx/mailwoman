import {Show,onMount,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {SettingRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import {SelectRow,ToggleRow,type CardSpec} from './layout/rows.tsx';
import {loadCatalog,t} from '../../i18n';
import * as css from './styles.css.ts';
void loadCatalog('properties');
/** Properties dialogs, priority marks and folder export: the user-level choices. An administrator can set a default and lock any of them, or switch a feature off for everyone. */
export const PROPERTIES_CARD:CardSpec={id:'folders-card-properties',section:'folders',title:'properties-card',help:'properties-card-help',rows:[
 {id:'priority-marks',label:'properties-priority-marks',keywords:'importance urgent high low'},{id:'priority-marks-style',label:'properties-priority-style'},
 {id:'compose-priority',label:'properties-compose-priority'},{id:'compose-priority-default',label:'properties-compose-priority-default'},
 {id:'properties-folder',label:'properties-folder-enabled'},{id:'properties-message',label:'properties-message-enabled'},
 {id:'properties-notes',label:'properties-notes-enabled'},{id:'properties-subfolders',label:'properties-export-subfolders',keywords:'zip download export'},
 {id:'properties-shortcut',label:'properties-shortcut-enabled'},{id:'properties-shortcut-key',label:'properties-shortcut-key'}]};
const is=<T extends string>(values:readonly T[])=>(v:string):v is T=>(values as readonly string[]).includes(v);
const MARKS=['both','high','off'] as const,STYLES=['icon','iconText','bar'] as const,LEVELS=['normal','high','low'] as const;
export function PropertiesPreferences():JSX.Element {
 onMount(()=>{void loadCatalog('properties');});
 const marks=useSetting<(typeof MARKS)[number]>('mail.priorityMarks','both'),style=useSetting<(typeof STYLES)[number]>('mail.priorityMarks.style','icon');
 const composeOn=useSetting<boolean>('mail.compose.priority.enabled',true),composeDefault=useSetting<(typeof LEVELS)[number]>('mail.compose.priority.default','normal');
 const folderOn=useSetting<boolean>('properties.folder.enabled',true),messageOn=useSetting<boolean>('properties.message.enabled',true),notes=useSetting<boolean>('properties.folder.notes.enabled',true);
 const subfolders=useSetting<boolean>('export.folder.includeSubfolders',true),shortcut=useSetting<boolean>('properties.shortcut.enabled',true),key=useSetting<string>('properties.shortcut.key','Alt+Enter');
 const err=()=>marks.error()||style.error()||composeOn.error()||composeDefault.error()||folderOn.error()||messageOn.error()||notes.error()||subfolders.error()||shortcut.error()||key.error();
 return <SettingsCard id="folders-card-properties" title={t('properties-card')} description={t('properties-card-help')}>
  <SelectRow id="priority-marks" label={t('properties-priority-marks')} handle={marks} accept={is(MARKS)} options={MARKS.map(v=>({value:v,label:t(`properties-marks-${v}`)}))}/>
  <SelectRow id="priority-marks-style" label={t('properties-priority-style')} handle={style} accept={is(STYLES)} options={STYLES.map(v=>({value:v,label:t(`properties-style-${v}`)}))}/>
  <ToggleRow id="compose-priority" label={t('properties-compose-priority')} handle={composeOn}/>
  <SelectRow id="compose-priority-default" label={t('properties-compose-priority-default')} handle={composeDefault} accept={is(LEVELS)} options={LEVELS.map(v=>({value:v,label:t(`properties-level-${v}`)}))}/>
  <ToggleRow id="properties-folder" label={t('properties-folder-enabled')} handle={folderOn}/>
  <ToggleRow id="properties-message" label={t('properties-message-enabled')} handle={messageOn}/>
  <ToggleRow id="properties-notes" label={t('properties-notes-enabled')} handle={notes}/>
  <ToggleRow id="properties-subfolders" label={t('properties-export-subfolders')} handle={subfolders}/>
  <ToggleRow id="properties-shortcut" label={t('properties-shortcut-enabled')} handle={shortcut}/>
  <SettingRow controlId="properties-shortcut-key" label={t('properties-shortcut-key')} handle={key}>{d=><input id="properties-shortcut-key" class={css.input} value={key.value()} disabled={!key.entry()||key.locked()||key.saving()||!shortcut.value()} aria-describedby={d()} onChange={e=>void key.set(e.currentTarget.value.trim()||'Alt+Enter')}/>}</SettingRow>
  <Show when={err()}><p role="alert">{err()}</p></Show>
 </SettingsCard>;
}
