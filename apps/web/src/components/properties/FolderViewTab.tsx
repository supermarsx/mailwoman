import {For,Show,createUniqueId,type JSX} from 'solid-js';
import {settings} from '../../api/settings.ts';
import {accountUiPreferences} from '../../api/accountPreferences.ts';
import {t} from '../../i18n';
import {Icon} from '../Icon.tsx';
import {VIEW_COLUMNS,VIEW_FIELDS,folderViewValue,folderViewLocked,folderColumnWidth,resetFolderColumnWidth,setFolderView,type ViewField,type ViewColumn} from '../folderViewPreferences.ts';
import {MESSAGE_LIST_ORDERS,MESSAGE_LIST_GROUPS} from '../messageOrder.ts';
import * as css from './folderDialog.css.ts';
const CHOICES:Partial<Record<ViewField,readonly string[]>>={style:['cards','table'],sort:MESSAGE_LIST_ORDERS,group:MESSAGE_LIST_GROUPS,filter:['all','unread','read','flagged','attachments','mentions'],font:['inherit','sans','serif','mono'],highlight:['default','unread','flagged','important','none'],readingPane:['right','bottom','off']};
const LIMITS:Partial<Record<ViewField,readonly [number,number]>>={fontSize:[11,24],rowSpacing:[0,16],listWidth:[180,1200],listHeight:[160,1200]};
export function FolderViewTab(props:{id:string}):JSX.Element {
 const uid=createUniqueId(),own=()=>accountUiPreferences().folderProps?.[props.id]?.view;
 const columns=()=>folderViewValue('columns',props.id).filter(column=>VIEW_COLUMNS.includes(column));
 function toggle(column:ViewColumn,checked:boolean){const next=checked?[...columns(),column]:columns().filter(c=>c!==column);if(next.length)setFolderView(props.id,{columns:next});}
 function move(column:ViewColumn,delta:number){const next=[...columns()],i=next.indexOf(column),j=i+delta;if(i<0||j<0||j>=next.length)return;[next[i],next[j]]=[next[j]!,next[i]!];setFolderView(props.id,{columns:next});}
 return <div class={css.group}>
  <p class={css.cardHelp}>{t('props-view-help')}</p>
  <section class={css.card}><h3 class={css.cardTitle}>{t('props-view-columns')}</h3><p class={css.cardHelp}>{t('props-view-columns-help')}</p>
   <For each={[...columns(),...VIEW_COLUMNS.filter(c=>!columns().includes(c))]}>{column=><div class={css.ovrHead}>
    <label class={css.row}><input type="checkbox" checked={columns().includes(column)} disabled={folderViewLocked('columns')||(columns().length===1&&columns()[0]===column)} onChange={e=>toggle(column,e.currentTarget.checked)}/>{t(`props-view-column-${column}`)}</label>
    <span class={css.row}>
     <button type="button" class="btn btn--ghost" aria-label={t('props-view-up')} disabled={folderViewLocked('columns')||columns().indexOf(column)<=0} onClick={()=>move(column,-1)}><Icon name="chevronUp"/></button>
     <button type="button" class="btn btn--ghost" aria-label={t('props-view-down')} disabled={folderViewLocked('columns')||columns().indexOf(column)<0||columns().indexOf(column)===columns().length-1} onClick={()=>move(column,1)}><Icon name="chevronDown"/></button>
     <input class={css.textField} type="number" min="64" max="800" step="1" aria-label={t('props-view-column-width',{column:t(`props-view-column-${column}`)})} value={folderColumnWidth(column,props.id)} disabled={settings()[`mail.messageListColumnWidth.${column}`]?.locked===true} onChange={e=>{if(e.currentTarget.checkValidity())setFolderView(props.id,{columnWidths:{[column]:e.currentTarget.valueAsNumber}});}}/>
     <Show when={own()?.columnWidths?.[column]!==undefined&&!settings()[`mail.messageListColumnWidth.${column}`]?.locked}><button type="button" class={css.linkButton} aria-label={`${t(`props-view-column-${column}`)}: ${t('props-view-inherit')}`} onClick={()=>resetFolderColumnWidth(props.id,column)}><Icon name="undo"/></button></Show>
    </span>
   </div>}</For>
   <button type="button" class={css.linkButton} disabled={folderViewLocked('columns')||!own()?.columns} onClick={()=>setFolderView(props.id,{columns:undefined})}>{t('props-view-inherit')}</button>
  </section>
  <section class={css.card}><For each={Object.keys(VIEW_FIELDS).filter(field=>field!=='columns') as ViewField[]}>{field=>{
   const id=`${uid}-${field}`,locked=()=>folderViewLocked(field),inherited=()=>own()?.[field]===undefined;
   return <div class={css.ovr}><div class={css.ovrHead}><label for={id}>{t(`props-view-${field}`)}</label><Show when={locked()}><Icon name="lock"/></Show><Show when={!locked()&&!inherited()}><button type="button" class={css.linkButton} onClick={()=>setFolderView(props.id,{[field]:undefined})}>{t('props-view-inherit')}</button></Show></div>
    <Show when={field==='filterQuery'} fallback={<Show when={CHOICES[field]} fallback={<input id={id} class={css.textField} type="number" min={LIMITS[field]?.[0]} max={LIMITS[field]?.[1]} value={Number(folderViewValue(field,props.id))} disabled={locked()} onChange={e=>{if(e.currentTarget.checkValidity())setFolderView(props.id,{[field]:e.currentTarget.valueAsNumber});}}/>}>{choices=><select id={id} class={css.textField} value={String(folderViewValue(field,props.id))} disabled={locked()} onChange={e=>setFolderView(props.id,{[field]:e.currentTarget.value})}><For each={choices()}>{choice=><option value={choice}>{field==='sort'?t(`mail-list-order-${choice}`):field==='group'?t(`mail-list-group-${choice}`):t(`props-view-option-${choice}`)}</option>}</For></select>}</Show>}><input id={id} class={css.textField} type="text" maxLength={2000} value={String(folderViewValue(field,props.id))} disabled={locked()} onChange={e=>setFolderView(props.id,{filterQuery:e.currentTarget.value})}/></Show>
   </div>;
  }}</For></section>
 </div>;
}
