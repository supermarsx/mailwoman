import {createMemo,createSignal,createUniqueId,For,Show,type JSX} from 'solid-js';
import {loadCatalog,rawT,t} from '../../i18n';
import {newIdempotencyKey} from '../../api/recovery.ts';
import {matchingTemplates,TEMPLATE_BODY_LIMIT,TEMPLATE_LIMIT,TEMPLATE_TITLE_LIMIT,type ResponseTemplate} from './responseTemplateModel.ts';
import * as css from './responseTemplates.css.ts';

export function TemplateLibraryEditor(props:{value:readonly ResponseTemplate[];onChange:(next:ResponseTemplate[])=>void;disabled?:boolean;id?:string;describedBy?:string|undefined}):JSX.Element{
 void loadCatalog('response-templates');
 const fallbackId=createUniqueId(),[selected,setSelected]=createSignal<string|null>(null),[query,setQuery]=createSignal('');
 const item=createMemo(()=>props.value.find(row=>row.id===selected()));
 const rows=createMemo(()=>matchingTemplates(props.value,query()));
 const update=(patch:Partial<ResponseTemplate>):void=>{if(props.disabled)return;const id=selected();props.onChange(props.value.map(row=>row.id===id?{...row,...patch}:row));};
 function add():void{if(props.disabled||props.value.length>=TEMPLATE_LIMIT)return;const row={id:newIdempotencyKey(),title:rawT('response-templates-untitled'),body:''};props.onChange([...props.value,row]);setSelected(row.id);setQuery('');}
 return <div class={css.library} id={props.id??fallbackId} aria-describedby={props.describedBy}>
  <label class={css.field}>{t('response-templates-search')}<input class={css.control} type="search" value={query()} onInput={event=>setQuery(event.currentTarget.value)}/></label>
  <div class={css.actions}><button type="button" class="btn" disabled={props.disabled||props.value.length>=TEMPLATE_LIMIT} onClick={add}>{t('response-templates-add')}</button><span>{t('response-templates-count',{count:props.value.length,max:TEMPLATE_LIMIT})}</span></div>
  <div class={css.list} role="group" aria-label={t('response-templates-library')}><For each={rows()}>{row=><button type="button" class="btn btn--ghost" aria-pressed={selected()===row.id} onClick={()=>setSelected(row.id)}>{row.title||t('response-templates-untitled')}</button>}</For></div>
  <Show when={props.value.length===0}><p>{t('response-templates-empty')}</p></Show>
  <Show when={item()}>{row=><>
   <label class={css.field}>{t('response-templates-name')}<input class={css.control} disabled={props.disabled} value={row().title} maxLength={TEMPLATE_TITLE_LIMIT} onInput={event=>update({title:event.currentTarget.value})}/></label>
   <label class={css.field}>{t('response-templates-body')}<textarea class={css.body} disabled={props.disabled} value={row().body} maxLength={TEMPLATE_BODY_LIMIT} onInput={event=>update({body:event.currentTarget.value})}/></label>
   <button type="button" class="btn btn--ghost" disabled={props.disabled} onClick={()=>{if(!props.disabled){props.onChange(props.value.filter(value=>value.id!==row().id));setSelected(null);}}}>{t('response-templates-remove')}</button>
  </>}</Show>
 </div>;
}
