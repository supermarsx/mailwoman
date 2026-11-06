import {createSignal,Show,type JSX} from 'solid-js';
import {ConfirmDialog} from '../screens/Admin/shell/ConfirmDialog.tsx';
import {isolate,t} from '../i18n';
import * as css from './folderDnd.css.ts';
import type {FolderArranger} from './folderArrange.ts';
/**
 * Asked once when a drop would change a folder's parent: change it on the server (everywhere), or only arrange it in this
 * app (a view stored in the user's preferences). "Remember my choice" saves `folderNestMode`.
 */
export function FolderNestDialog(props:{arranger:FolderArranger;nameOf:(id:string)=>string;mount?:HTMLElement|undefined}):JSX.Element {
 const [mode,setMode]=createSignal<'server'|'local'>('local'),[remember,setRemember]=createSignal(false);
 return <Show when={props.arranger.pending()}>{pending=>{
  const many=()=>pending().moving.length>1,name=()=>isolate(props.nameOf(pending().moving[0]!));
  return <ConfirmDialog open danger={false} title={many()?t('folder-dnd-nest-title-many',{count:pending().moving.length}):t('folder-dnd-nest-title',{name:name()})} body={t('folder-dnd-nest-body')} confirmLabel={t('folder-dnd-nest-confirm')} cancelLabel={t('folder-dnd-cancel')} mount={props.mount} onCancel={()=>props.arranger.cancel()} onConfirm={()=>{props.arranger.choose(mode(),remember());}}>
   <fieldset class={css.choices}>
    <legend class={css.legend}>{t('folder-dnd-nest-legend')}</legend>
    <label class={css.choice}><input type="radio" name="folder-nest-mode" checked={mode()==='local'} onChange={()=>setMode('local')}/><span><strong>{t('folder-dnd-nest-local')}</strong><span class={css.hint}>{t('folder-dnd-nest-local-help')}</span></span></label>
    <label class={css.choice}><input type="radio" name="folder-nest-mode" checked={mode()==='server'} onChange={()=>setMode('server')}/><span><strong>{t('folder-dnd-nest-server')}</strong><span class={css.hint}>{t('folder-dnd-nest-server-help')}</span></span></label>
    <label class={css.choice}><input type="checkbox" checked={remember()} onChange={event=>setRemember(event.currentTarget.checked)}/><span>{t('folder-dnd-nest-remember')}</span></label>
   </fieldset>
  </ConfirmDialog>;
 }}</Show>;
}
