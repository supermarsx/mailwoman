import {For,Show,onMount,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {LockNote} from './LockNote.tsx';
import {loadCatalog,t} from '../../i18n';
import {UNDO_HISTORY_CHOICES,UNDO_HISTORY_DEFAULT,UNDO_HISTORY_LIMIT,UNDO_HISTORY_SIZE_KEY,undoHistoryCap} from '../../state/undoHistory.ts';
import * as css from './styles.css.ts';

const isSize=(value:unknown):value is number=>typeof value==='number'&&Number.isInteger(value)&&value>=0&&value<=UNDO_HISTORY_LIMIT;

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
 return <section class={css.section} aria-label={t('undo-setting-section')} data-testid="undo-setting"><h3 class={css.heading}>{t('undo-setting-section')}</h3>
  <div class={css.field}><label class={css.check}><input class={css.checkbox} type="checkbox" data-testid="undo-enabled" checked={shown()>0} disabled={disabled()||cap()===0} onChange={event=>void size.set(event.currentTarget.checked?turnOn():0)} aria-describedby="undo-setting-help undo-setting-lock"/>{t('undo-setting-enabled')}</label>
   <p id="undo-setting-help" class={css.meta}>{t('undo-setting-help')}</p><LockNote id="undo-setting-lock" locked={size.locked()}/>
   <Show when={cap()<UNDO_HISTORY_LIMIT}><p class={css.meta} data-testid="undo-capped">{t('undo-setting-capped',{max:cap()})}</p></Show></div>
  <div class={css.field}><label for="undo-history-size">{t('undo-setting-size')}</label>
   <select id="undo-history-size" class={css.select} data-testid="undo-size" value={String(shown())} disabled={disabled()||cap()===0} onChange={event=>void size.set(Number(event.currentTarget.value))}>
    <For each={choices()}>{choice=><option value={String(choice)} selected={choice===shown()}>{choice===0?t('undo-setting-off'):t('undo-setting-size-count',{count:choice})}</option>}</For>
   </select>
   <Show when={size.error()}><p role="alert">{size.error()}</p></Show>
   <button type="button" class={css.ghost} disabled={disabled()} onClick={()=>void size.reset()}>{t('reader-inherit')}</button></div>
 </section>;
}
