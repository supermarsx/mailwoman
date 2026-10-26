import {For} from 'solid-js';
import type {RichTextApi} from './RichTextEditor.tsx';
import {Icon} from '../Icon.tsx';
import {t} from '../../i18n';

export function ComposerHistory(props:{api:RichTextApi|null}) {
  const depth=()=>props.api?.historyDepth?.()??{undo:0,redo:0};
  return <div class="compose-rt__history" data-detached-local>
    <button type="button" class="compose-rt__btn" disabled={!depth().undo} aria-label={t('mail-compose-undo')} onMouseDown={event=>event.preventDefault()} onClick={()=>props.api?.undo?.()}><Icon name="back"/>{t('mail-compose-undo')}</button>
    <button type="button" class="compose-rt__btn" disabled={!depth().redo} aria-label={t('mail-compose-redo')} onMouseDown={event=>event.preventDefault()} onClick={()=>props.api?.redo?.()}><Icon name="forward"/>{t('mail-compose-redo')}</button>
    <label>{t('mail-editor-history')}<select aria-label={t('mail-editor-history')} value="0" disabled={!depth().undo} onChange={event=>{const count=Math.min(Number(event.currentTarget.value),depth().undo,20);for(let index=0;index<count;index++)props.api?.undo?.();event.currentTarget.value='0';}}><option value="0">{t('mail-editor-history')}</option><For each={Array.from({length:Math.min(20,depth().undo)},(_,index)=>index+1)}>{count=><option value={count}>{t('mail-editor-undo-groups',{count})}</option>}</For></select></label>
  </div>;
}
