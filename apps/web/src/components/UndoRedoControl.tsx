import {createEffect,createMemo,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {observePreferenceAccount,preferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {formatTime,loadCatalog,t} from '../i18n/index.ts';
import {handleUndoKey,redoEntries,redoLast,redoThrough,undoBusy,undoAnnouncement,undoEnabled,undoEntries,undoLast,undoProgress,undoThrough,type UndoEntry} from '../state/undoHistory.ts';
import {Icon,type IconName} from './Icon.tsx';
import {createContextMenu} from './ContextMenu.tsx';
import {useCollapsibleBar} from './bars/CollapsibleBar.tsx';
import * as css from './undoRedo.css.ts';

type Kind='undo'|'redo';

function Half(props:{kind:Kind}):JSX.Element {
 let caret:HTMLButtonElement|undefined;
 const bar=useCollapsibleBar();
 const entries=():readonly UndoEntry[]=>props.kind==='undo'?undoEntries():redoEntries();
 const next=createMemo(()=>entries()[0]);
 const icon:IconName=props.kind;
 const label=()=>{const entry=next();return entry?t(`${props.kind}-button-next`,{label:entry.label}):t(`${props.kind}-button-none`);};
 const disabled=()=>next()===undefined||undoBusy();
 const menu=createContextMenu({
  ownerDocument:()=>caret?.ownerDocument??document,
  label:()=>t(`${props.kind}-menu-title`),
  scope:()=>`${props.kind}-history:${preferenceAccount()?.generation??'signed-out'}`,
  focusCurrent:false,
  actions:()=>entries().map(entry=>({id:String(entry.id),label:entry.label,icon,title:formatTime(entry.at),run:()=>{void (props.kind==='undo'?undoThrough(entry.id):redoThrough(entry.id));}})),
 });
 createEffect(()=>{if(disabled())menu.close(false);});
 createEffect(()=>{if(menu.isOpen()){const release=bar?.hold();onCleanup(()=>release?.());}});
 onCleanup(observePreferenceAccount(()=>menu.close(false)));
 return <span class={css.split} data-testid={`${props.kind}-split`}>
  <button type="button" class={`btn btn--ghost ${css.main}`} data-testid={`${props.kind}-button`} disabled={disabled()} aria-label={label()} title={label()} onClick={()=>void (props.kind==='undo'?undoLast():redoLast())}><Icon name={icon}/></button>
  <button ref={caret} type="button" class={`btn btn--ghost ${css.caret}`} data-testid={`${props.kind}-menu-button`} disabled={disabled()} aria-label={t(`${props.kind}-menu-button`)} title={t(`${props.kind}-menu-button`)} aria-haspopup="menu" aria-expanded={menu.isOpen()}
   on:click={event=>{if(menu.isOpen())menu.close();else menu.openBelow(event.currentTarget);}}
   on:keydown={event=>{if(event.key==='ArrowDown'||event.key==='ArrowUp'){event.preventDefault();menu.openBelow(event.currentTarget);}else if(event.key==='Escape'&&menu.isOpen()){event.preventDefault();menu.close();}}}><Icon name="chevronDown"/></button>
  {menu.menu()}
 </span>;
}

/**
 * Global Undo and Redo for mail actions (state/undoHistory.ts), each with a
 * caret listing the recent actions. Hidden when the history is off
 * (`ui.undoHistorySize` 0). Mounting it also binds Ctrl/Cmd+Z and
 * Ctrl/Cmd+Shift+Z / Ctrl+Y outside text fields.
 */
export function UndoRedoControl():JSX.Element {
 onMount(()=>{
  void loadCatalog('undo');
  const listener=(event:KeyboardEvent):void=>{handleUndoKey(event);};
  document.addEventListener('keydown',listener);
  onCleanup(()=>document.removeEventListener('keydown',listener));
 });
 const progress=()=>{const p=undoProgress();return p?t(`${p.kind}-progress`,{done:p.done+1,total:p.total}):'';};
 return <Show when={undoEnabled()}>
  <span class={css.group} role="group" aria-label={t('undo-group')} data-testid="undo-redo">
   <Half kind="undo"/><Half kind="redo"/>
   <Show when={progress()}><span class={css.progress} aria-hidden="true">{progress()}</span></Show>
   <span class={css.live} role="status" aria-live="polite" aria-atomic="true" data-testid="undo-status">{undoAnnouncement()}</span>
  </span>
 </Show>;
}
