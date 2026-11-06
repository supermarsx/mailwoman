import {createMemo,createSignal,For,onMount,type JSX} from 'solid-js';
import {ConfirmDialog} from '../screens/Admin/shell/ConfirmDialog.tsx';
import {isolate,loadCatalog,t} from '../i18n';
import * as settingsCss from '../screens/Settings/styles.css.ts';
import {treeRows} from './folderTree.ts';
import {validatePlacement,type DndFolder,type DndPolicy} from './folderDnd.ts';
import type {FolderArranger} from './folderArrange.ts';
/**
 * The keyboard and touch alternative to dragging: pick the new parent from an indented tree (or the top level).
 * Only places the move would be allowed to make are offered. The move then goes through the same arranger as a drop,
 * so the nest-mode question, admin policy, undo and rollback all apply.
 */
export function FolderMoveToDialog(props:{moving:readonly string[];arranger:FolderArranger;policy:()=>DndPolicy;onClose:()=>void;mount?:HTMLElement|undefined}):JSX.Element {
 onMount(()=>{void loadCatalog('folder-dnd');void loadCatalog('mail');});
 const folders=()=>props.arranger.visual() as DndFolder[];
 const options=createMemo(()=>{
  const all=folders();
  return treeRows(all,()=>true).filter(row=>validatePlacement(all,props.moving,{parentId:row.box.id,beforeId:null},props.policy()).ok).map(row=>({id:row.box.id,label:`${'   '.repeat(row.depth)}${row.depth>0?'└ ':''}${row.box.name}`}));
 });
 const topAllowed=()=>validatePlacement(folders(),props.moving,{parentId:null,beforeId:null},props.policy()).ok;
 const [target,setTarget]=createSignal<string>(topAllowed()?'':options()[0]?.id??'');
 const first=()=>folders().find(f=>f.id===props.moving[0]);
 return <ConfirmDialog open danger={false} title={t('folder-dnd-move-to',{name:isolate(first()?.name??'')})} confirmLabel={t('folder-dnd-nest-confirm')} cancelLabel={t('folder-dnd-cancel')} mount={props.mount} onCancel={props.onClose} onConfirm={()=>{props.arranger.place(props.moving,{parentId:target()||null,beforeId:null});props.onClose();}}>
  <label class={settingsCss.field}>{t('folder-parent')}
   <select class={settingsCss.select} size={Math.min(8,options().length+1)} value={target()} onChange={event=>setTarget(event.currentTarget.value)}>
    {topAllowed()&&<option value="">{t('folder-top-level')}</option>}
    <For each={options()}>{option=><option value={option.id}>{option.label}</option>}</For>
   </select>
  </label>
 </ConfirmDialog>;
}
