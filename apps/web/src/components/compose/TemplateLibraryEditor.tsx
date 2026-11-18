import {TemplateBodyEditor} from './TemplateBodyEditor.tsx';
import {createRangeSelection} from '../rangeSelection.ts';
import {RangeSelectionBar,RangeSelectionToggle} from '../RangeSelectionBar.tsx';
import {SearchInput} from '../SearchInput.tsx';
import {createMemo,createSignal,createUniqueId,For,Show,type JSX} from 'solid-js';
import {loadCatalog,rawT,t} from '../../i18n';
import {newIdempotencyKey} from '../../api/recovery.ts';
import {matchingTemplates,TEMPLATE_LIMIT,TEMPLATE_TITLE_LIMIT,TEMPLATE_HEADERS,TEMPLATE_HEADER_LIMITS,validResponseTemplates,type ResponseTemplate} from './responseTemplateModel.ts';
import * as css from './responseTemplates.css.ts';
import * as table from '../../screens/Settings/managementTable.css.ts';
import * as ux from '../../screens/Admin/shell/ux.css.ts';
import {Icon} from '../Icon.tsx';

export function TemplateLibraryEditor(props:{value:readonly ResponseTemplate[];onChange:(next:ResponseTemplate[])=>void;disabled?:boolean;id?:string;describedBy?:string|undefined;onUse?:(row:ResponseTemplate)=>void}):JSX.Element{
 void loadCatalog('response-templates');
 const fallbackId=createUniqueId(),[selected,setSelected]=createSignal<string|null>(null),[query,setQuery]=createSignal('');
 const item=createMemo(()=>props.value.find(row=>row.id===selected()));
 const rows=createMemo(()=>matchingTemplates(props.value,query()));
 const bulk=createRangeSelection({ids:()=>rows().map(row=>row.id),scope:()=>JSON.stringify([props.id??fallbackId,query()])});
 const update=(patch:Partial<ResponseTemplate>):void=>{if(props.disabled)return;const id=selected();props.onChange(props.value.map(row=>row.id===id?{...row,...patch}:row));};
 function add():void{if(props.disabled||props.value.length>=TEMPLATE_LIMIT)return;const row={id:newIdempotencyKey(),title:rawT('response-templates-untitled'),body:''};props.onChange([...props.value,row]);setSelected(row.id);setQuery('');}
 return <div data-range-selection-root class={css.library} id={props.id??fallbackId} aria-describedby={props.describedBy}>
  <Show when={!item()}>
  <div class={table.bar}><label class={table.search}>{t('response-templates-search')}<SearchInput class={css.control}  value={query()} onInput={event=>setQuery(event.currentTarget.value)}/></label>
  <div class={css.actions}><RangeSelectionToggle selection={bulk}/><span>{t('response-templates-count',{count:props.value.length,max:TEMPLATE_LIMIT})}</span><button type="button" class="btn" disabled={props.disabled||props.value.length>=TEMPLATE_LIMIT} onClick={add}><Icon name="plus"/>{t('response-templates-add')}</button></div></div>
  <Show when={rows().length>0}><div class={table.scroll} role="region" aria-label={t('response-templates-library')} tabindex="0"><table class={table.table}><thead><tr><th class={table.heading}>{t('response-templates-name')}</th><th class={`${table.heading} ${table.previewColumn}`}>{t('response-templates-body')}</th><th class={`${table.heading} ${table.actionColumn}`}><span>{t('common-actions')}</span></th></tr></thead><tbody><For each={rows()}>{row=><tr class={table.row} data-current={selected()===row.id?'true':undefined} tabIndex={0} data-selection-id={row.id} data-bulk-selected={bulk.selected().has(row.id)} onClick={event=>{if(!(event.target as Element).closest('button,input,a'))bulk.click(row.id,event);}} onKeyDown={event=>{if(event.target===event.currentTarget)bulk.keydown(row.id,event);}}>
   <td class={table.cell}><div class={table.name}><Icon name={row.subject||row.to?'mail':'file'}/><button type="button" class={table.title} aria-pressed={selected()===row.id} title={row.title} onClick={event=>bulk.click(row.id,event,()=>setSelected(row.id))}>{row.title||t('response-templates-untitled')}</button></div><Show when={row.subject}><span class={table.text}>{row.subject}</span></Show></td>
   <td class={`${table.cell} ${table.previewColumn}`}><span class={table.text} title={row.body}>{row.body.slice(0,160)}</span><Show when={row.to}><span class={table.text}>{row.to}</span></Show></td>
   <td class={table.cell}><div class={table.actions}><Show when={props.onUse}><button type="button" class={`btn btn--ghost ${ux.iconAction}`} aria-label={t('response-templates-compose')} title={t('response-templates-compose')} disabled={!validResponseTemplates([row])} onClick={()=>props.onUse?.(row)}><Icon name="compose"/></button></Show><button type="button" class={`btn btn--ghost ${ux.iconAction}`} aria-label={t('common-edit')} title={t('common-edit')} onClick={()=>setSelected(row.id)}><Icon name={props.disabled?'eye':'edit'}/></button><button type="button" class={`btn btn--danger ${ux.iconAction}`} aria-label={t('response-templates-remove')} title={t('response-templates-remove')} disabled={props.disabled} onClick={()=>{if(!props.disabled){props.onChange(props.value.filter(value=>value.id!==row.id));if(selected()===row.id)setSelected(null);}}}><Icon name="delete"/></button></div></td>
  </tr>}</For></tbody></table></div></Show>
  <Show when={props.value.length===0}><p>{t('response-templates-empty')}</p></Show>
  <RangeSelectionBar selection={bulk} total={rows().length} actions={[
   {id:'copy',label:t('common-copy'),icon:'copy',run:()=>navigator.clipboard.writeText(rows().filter(row=>bulk.selected().has(row.id)).map(row=>row.body).join('\n\n'))},
   {id:'delete',label:t('response-templates-remove'),icon:'delete',disabled:props.disabled===true,run:()=>{if(!props.disabled){props.onChange(props.value.filter(row=>!bulk.selected().has(row.id)));bulk.clear();}}},
  ]}/>
  </Show>
  <Show when={item()}>{row=><div class={table.editor}>
   <div class={table.bar}><div class={css.actions}><button type="button" class="btn btn--ghost" onClick={()=>setSelected(null)}><Icon name="chevronLeft"/>{t('common-back')}</button><h3>{row().title||t('response-templates-untitled')}</h3></div><Show when={props.onUse}><button type="button" class="btn" disabled={!validResponseTemplates([row()])} onClick={()=>props.onUse?.(row())}><Icon name="compose"/>{t('response-templates-compose')}</button></Show></div>

   <label class={css.field}>{t('response-templates-name')}<input class={css.control} disabled={props.disabled} value={row().title} maxLength={TEMPLATE_TITLE_LIMIT} onInput={event=>update({title:event.currentTarget.value})}/></label>
   <div class={table.headerFields}><For each={TEMPLATE_HEADERS}>{field => (
    <label class={css.field}>{t(`response-templates-header-${field}`)}
     <input class={css.control} disabled={props.disabled} value={row()[field] ?? ''}
       maxLength={TEMPLATE_HEADER_LIMITS[field]} onInput={event => update({[field]: event.currentTarget.value})}/>
    </label>
   )}</For></div>
   <Show when={row().id} keyed>{id=><TemplateBodyEditor id={`template-${id}`} label={t('response-templates-body')} text={row().body} html={row().html??''} disabled={props.disabled} onChange={(body,html)=>update({body,html})}/>}</Show>
  </div>}</Show>
 </div>;
}
