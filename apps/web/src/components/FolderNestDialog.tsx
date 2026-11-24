import {Icon} from './Icon.tsx';
import {createSignal,createUniqueId,Show,type JSX} from 'solid-js';
import {ConfirmDialog} from '../screens/Admin/shell/ConfirmDialog.tsx';
import {isolate,t} from '../i18n';
import * as css from './folderDnd.css.ts';
import type {FolderArranger} from './folderArrange.ts';
import { loadCatalog } from '../i18n/catalog.ts';
// i18n: the ids this file shows live in these catalogs, which load with it.
void loadCatalog('folder-dnd');
/**
 * Asked once when a drop would change a folder's parent: change it on the server (everywhere), or only arrange it in this
 * app (a view stored in the user's preferences). "Remember my choice" saves `folderNestMode`.
 */
export function FolderNestDialog(props:{arranger:FolderArranger;nameOf:(id:string)=>string;mount?:HTMLElement|undefined}):JSX.Element {
 const id=createUniqueId();
 const [mode,setMode]=createSignal<'server'|'local'>('local'),[remember,setRemember]=createSignal(false);
 return <Show when={props.arranger.pending()}>{pending=>{
  const many=()=>pending().moving.length>1,name=()=>isolate(props.nameOf(pending().moving[0]!));
  return <ConfirmDialog open danger={false} title={many()?t('folder-dnd-nest-title-many',{count:pending().moving.length}):t('folder-dnd-nest-title',{name:name()})} body={t('folder-dnd-nest-body')} confirmLabel={t('folder-dnd-nest-confirm')} cancelLabel={t('folder-dnd-cancel')} mount={props.mount} onCancel={()=>props.arranger.cancel()} onConfirm={()=>{props.arranger.choose(mode(),remember());}}>
   <fieldset class={css.choices}>
    <legend class={css.legend}>{t('folder-dnd-nest-legend')}</legend>
    <label class={css.nestOption}><input type="radio" name={id} aria-labelledby={`${id}-local-title`} aria-describedby={`${id}-local-help`} checked={mode()==='local'} onChange={()=>setMode('local')}/><span class={css.nestText}><span class={css.nestTitle} id={`${id}-local-title`}><Icon name="eye"/><strong>{t('folder-dnd-nest-local')}</strong></span><span class={css.hint} id={`${id}-local-help`}>{t('folder-dnd-nest-local-help')}</span></span></label>
    <label class={css.nestOption}><input type="radio" name={id} aria-labelledby={`${id}-server-title`} aria-describedby={`${id}-server-help`} checked={mode()==='server'} onChange={()=>setMode('server')}/><span class={css.nestText}><span class={css.nestTitle} id={`${id}-server-title`}><Icon name="globe"/><strong>{t('folder-dnd-nest-server')}</strong></span><span class={css.hint} id={`${id}-server-help`}>{t('folder-dnd-nest-server-help')}</span></span></label>
    <label class={css.rememberChoice}><input type="checkbox" checked={remember()} onChange={event=>setRemember(event.currentTarget.checked)}/><span>{t('folder-dnd-nest-remember')}</span></label>
   </fieldset>
  </ConfirmDialog>;
 }}</Show>;
}
