import {For} from 'solid-js';
import type {RichTextApi} from './RichTextEditor.tsx';
import {Icon} from '../Icon.tsx';
import {t} from '../../i18n';

/** Native disclosure keeps history keyboard accessible without stealing editor selection.
 * Esc closes an open list; on a closed one it is left to the ribbon (an auto-hide bar closes). */
export function ComposerHistory(props:{api:RichTextApi|null}) {
  const depth=()=>props.api?.historyDepth?.()??{undo:0,redo:0};
  return <div class="compose-rt__history" data-detached-local>
    <For each={['undo','redo'] as const}>{action=>{
      let disclosure!:HTMLDetailsElement;
      const run=(count:number):void=>{const limit=Math.min(count,depth()[action],20);for(let index=0;index<limit;index++)props.api?.[action]?.();disclosure.open=false;};
      return <div class="compose-history-split">
        <button type="button" class="compose-rt__btn" disabled={!depth()[action]} aria-label={t(`mail-compose-${action}`)} onMouseDown={event=>event.preventDefault()} onClick={()=>run(1)}><Icon name={action==='undo'?'back':'forward'}/>{t(`mail-compose-${action}`)}</button>
        <details ref={disclosure} onKeyDown={event=>{if(event.key==='Escape'&&disclosure.open){disclosure.open=false;disclosure.querySelector('summary')?.focus();event.stopPropagation();}}}>
          <summary aria-label={t('mail-editor-action-history',{action:t(`mail-compose-${action}`)})} data-tooltip={t('mail-editor-action-history',{action:t(`mail-compose-${action}`)})}><Icon name="chevronDown"/></summary>
          <div class="compose-history-split__list" role="group" aria-label={t('mail-editor-action-history',{action:t(`mail-compose-${action}`)})}>
            <For each={Array.from({length:Math.min(20,depth()[action])},(_,index)=>index+1)} fallback={<span>{t('mail-editor-history-empty')}</span>}>{count=><button type="button" class="compose-rt__btn" onMouseDown={event=>event.preventDefault()} onClick={()=>run(count)}>{t(`mail-editor-${action}-groups`,{count})}</button>}</For>
          </div>
        </details>
      </div>;
    }}</For>
  </div>;
}
