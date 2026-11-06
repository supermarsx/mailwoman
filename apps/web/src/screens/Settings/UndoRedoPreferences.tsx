import {For,Show,onMount,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {SettingRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import type {CardSpec} from './layout/rows.tsx';
import {loadCatalog,t} from '../../i18n';
import {UNDO_HISTORY_CHOICES,UNDO_HISTORY_DEFAULT,UNDO_HISTORY_LIMIT,UNDO_HISTORY_SIZE_KEY,undoHistoryCap} from '../../state/undoHistory.ts';
import * as css from './styles.css.ts';

const isSize=(value:unknown):value is number=>typeof value==='number'&&Number.isInteger(value)&&value>=0&&value<=UNDO_HISTORY_LIMIT;
export const UNDO_CARD:CardSpec={id:'undo-card',section:'shortcuts',title:'undo-setting-section',rows:[{id:'undo-enabled',label:'undo-setting-enabled',help:'undo-setting-help'},{id:'undo-history-size',label:'undo-setting-size'}]};

/** `ui.undoHistorySize`: the Undo/Redo history on or off and how many actions it keeps, within the administrator's `ui.undoHistoryMax`. */
export function UndoRedoPreferences():JSX.Element {
 onMount(()=>void loadCatalog('undo'));
 const size=useSetting<number>(UNDO_HISTORY_SIZE_KEY,UNDO_HISTORY_DEFAULT,isSize);
 const cap=()=>undoHistoryCap();
 const shown=()=>Math.min(size.value(),cap());
 const disabled=()=>!size.entry()||size.locked()||size.saving();
 const choices=()=>{
  const list:number[]=UNDO_HISTORY_CHOICES.filter(choice=>choice===0||choice<=cap());
  if(cap()>0&&cap()<UNDO_HISTORY_LIMIT&&!list.includes(cap()))list.push(cap());
  if(!list.includes(shown()))list.push(shown());
  return list.sort((a,b)=>a-b);
 };
 const turnOn=():number=>Math.min(UNDO_HISTORY_DEFAULT,cap());
 return <SettingsCard id="undo-card" testId="undo-setting" title={t('undo-setting-section')}>
  <SettingRow controlId="undo-enabled" label={t('undo-setting-enabled')} help={t('undo-setting-help')} handle={size}>{d=>
   <input id="undo-enabled" type="checkbox" data-testid="undo-enabled" checked={shown()>0} disabled={disabled()||cap()===0} onChange={event=>void size.set(event.currentTarget.checked?turnOn():0)} aria-describedby={d()}/>}</SettingRow>
  <Show when={cap()<UNDO_HISTORY_LIMIT}><p class={css.meta} data-testid="undo-capped">{t('undo-setting-capped',{max:cap()})}</p></Show>
  <SettingRow controlId="undo-history-size" label={t('undo-setting-size')} handle={size} lockNote={false}>{d=>
   <select id="undo-history-size" class={css.select} data-testid="undo-size" aria-describedby={d()} value={String(shown())} disabled={disabled()||cap()===0} onChange={event=>void size.set(Number(event.currentTarget.value))}>
    <For each={choices()}>{choice=><option value={String(choice)} selected={choice===shown()}>{choice===0?t('undo-setting-off'):t('undo-setting-size-count',{count:choice})}</option>}</For>
   </select>}</SettingRow>
  <Show when={size.error()}><p role="alert">{size.error()}</p></Show>
 </SettingsCard>;
}
