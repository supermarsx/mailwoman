import {For} from 'solid-js';
import type {RichTextApi} from './RichTextEditor.tsx';
import {Icon} from '../Icon.tsx';
import {t} from '../../i18n';
import * as css from './composerHistory.css.ts';

type Action='undo'|'redo';

/** Apple platforms show the Command chord; the editor binds Mod-z, Mod-y and Mod-Shift-z (RichTextEditor). */
export function isApplePlatform(nav:Pick<Navigator,'platform'|'userAgent'>&{userAgentData?:{platform?:string}}=navigator):boolean{
  return /Mac|iPhone|iPad|iPod/i.test(nav.userAgentData?.platform||nav.platform||nav.userAgent||'');
}
const SHORTCUT:Record<Action,{apple:string;other:string;aria:string}>={
  undo:{apple:'⌘Z',other:'Ctrl+Z',aria:'Control+Z Meta+Z'},
  redo:{apple:'⇧⌘Z',other:'Ctrl+Y',aria:'Control+Y Control+Shift+Z Meta+Shift+Z'},
};
export const historyShortcut=(action:Action,apple=isApplePlatform()):string=>SHORTCUT[action][apple?'apple':'other'];

/** Undo and Redo, each a split button: the action, and a native disclosure with the
 * history list that keeps keyboard access without stealing the editor selection.
 * Esc closes an open list; on a closed one it is left to the ribbon (an auto-hide bar closes). */
export function ComposerHistory(props:{api:RichTextApi|null}) {
  const depth=()=>props.api?.historyDepth?.()??{undo:0,redo:0};
  return <div class={`compose-rt__history ${css.history}`} data-detached-local>
    <For each={['undo','redo'] as const}>{action=>{
      let disclosure!:HTMLDetailsElement;
      const run=(count:number):void=>{const limit=Math.min(count,depth()[action],20);for(let index=0;index<limit;index++)props.api?.[action]?.();disclosure.open=false;};
      const label=():string=>t(`mail-compose-${action}`);
      const listLabel=():string=>t('mail-editor-action-history',{action:label()});
      return <div class={`compose-history-split ${css.split}`} data-history={action}>
        <button type="button" class={css.action} disabled={!depth()[action]} aria-label={label()} aria-keyshortcuts={SHORTCUT[action].aria} data-tooltip={t(`mail-compose-${action}-tip`,{shortcut:historyShortcut(action)})} onMouseDown={event=>event.preventDefault()} onClick={()=>run(1)}>
          <Icon name={action} class={css.icon}/><span class={css.label}>{label()}</span>
        </button>
        <details ref={disclosure} class={css.more} onKeyDown={event=>{if(event.key==='Escape'&&disclosure.open){disclosure.open=false;disclosure.querySelector('summary')?.focus();event.stopPropagation();}}}>
          <summary class={css.summary} aria-label={listLabel()} data-tooltip={listLabel()}><Icon name="chevronDown" class={css.chevron}/></summary>
          <div class="compose-history-split__list" role="group" aria-label={listLabel()}>
            <For each={Array.from({length:Math.min(20,depth()[action])},(_,index)=>index+1)} fallback={<span>{t('mail-editor-history-empty')}</span>}>{count=><button type="button" class="compose-rt__btn" onMouseDown={event=>event.preventDefault()} onClick={()=>run(count)}>{t(`mail-editor-${action}-groups`,{count})}</button>}</For>
          </div>
        </details>
      </div>;
    }}</For>
  </div>;
}
