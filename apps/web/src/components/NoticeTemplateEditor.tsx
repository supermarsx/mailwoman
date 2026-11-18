import {createSignal,For,Show,type JSX} from 'solid-js';
import {loadCatalog,t} from '../i18n';
import {TemplateBodyEditor} from './compose/TemplateBodyEditor.tsx';
import {Icon} from './Icon.tsx';
import type {MessageNotice,NoticeKind} from './compose/messageNotices.ts';
import * as css from '../screens/Settings/styles.css.ts';
import * as table from '../screens/Settings/linkedAccounts.css.ts';
import * as entity from '../screens/Settings/entityTable.css.ts';

export function NoticeTemplateEditor(props:{value:readonly MessageNotice[];onChange:(rows:MessageNotice[])=>void;disabled?:boolean|undefined;id?:string|undefined;describedBy?:string|undefined}):JSX.Element {
 void loadCatalog('notices');
 const [selected,setSelected]=createSignal<string|null>(null);
 const current=()=>props.value.find(row=>row.id===selected());
 const change=(patch:Partial<MessageNotice>)=>props.onChange(props.value.map(row=>row.id===selected()?{...row,...patch}:row));
 const csv=(value:string)=>[...new Set(value.split(/[,;\n]+/).map(item=>item.trim()).filter(Boolean))];
 const add=()=>{const id=globalThis.crypto?.randomUUID?.()??`notice-${Date.now()}-${Math.random().toString(36).slice(2)}`;props.onChange([...props.value,{id,title:t('notices-new'),text:'',html:'',enabled:true,automatic:true,identities:[],kinds:['new','reply','forward'],audience:'all',domains:[],subjectContains:'',bodyContains:''}]);setSelected(id);};
 return <div id={props.id} aria-describedby={props.describedBy} class={css.stack}>
  <Show when={current()} fallback={<>
   <div class={table.header}><p class={css.meta}>{t('notices-help')}</p><button type="button" class={`${css.button} ${table.addButton}`} disabled={props.disabled||props.value.length>=20} onClick={add}><Icon name="plus"/>{t('notices-new')}</button></div>
   <ul class={table.table} aria-label={t('notices-title')}><For each={props.value}>{row=><li class={entity.row}>
    <span class={entity.avatar}><Icon name="file"/></span><div class={table.who}><b>{row.title}</b><span class={table.address}>{row.text.slice(0,120)}</span><span class={css.meta}>{row.automatic?t('notices-automatic'):t('notices-manual')}</span></div>
    <div class={table.actions}><button type="button" class={css.ghost} disabled={props.disabled} aria-label={`${t('common-edit')} ${row.title}`} onClick={()=>setSelected(row.id)}><Icon name="edit"/></button><button type="button" class={css.ghost} disabled={props.disabled} aria-label={`${t('common-delete')} ${row.title}`} onClick={()=>props.onChange(props.value.filter(item=>item.id!==row.id))}><Icon name="delete"/></button></div>
   </li>}</For></ul>
  </>}>{row=><>
   <div class={table.header}><div class={css.actions}><button type="button" class={css.ghost} onClick={()=>setSelected(null)}><Icon name="chevronLeft"/>{t('common-back')}</button><h3 class={css.heading}>{row().title}</h3></div></div>
   <fieldset disabled={props.disabled} class={css.section}>
    <label class={css.field}>{t('notices-name')}<input class={css.input} maxLength={80} value={row().title} onInput={event=>change({title:event.currentTarget.value})}/></label>
    <Show when={row().id} keyed>{id=><TemplateBodyEditor id={`notice-${id}`} label={t('notices-text')} text={row().text} html={row().html} disabled={props.disabled} onChange={(text,html)=>change({text,html})}/>}</Show>
    <label class={css.row}><input type="checkbox" checked={row().enabled} onChange={event=>change({enabled:event.currentTarget.checked})}/>{t('notices-enabled')}</label>
    <label class={css.row}><input type="checkbox" checked={row().automatic} onChange={event=>change({automatic:event.currentTarget.checked})}/>{t('notices-automatic')}</label>

   </fieldset>
   <fieldset disabled={props.disabled} class={css.section}><legend>{t('notices-rules')}</legend><p class={css.meta}>{t('notices-rule-help')}</p>
    <label class={css.field}>{t('notices-identities')}<input class={css.input} value={row().identities.join(', ')} onChange={event=>change({identities:csv(event.currentTarget.value)})}/></label>
    <div class={css.actions}><For each={['new','reply','forward'] as const}>{kind=><label class={css.row}><input type="checkbox" checked={row().kinds.includes(kind)} onChange={event=>change({kinds:event.currentTarget.checked?[...row().kinds,kind]:row().kinds.filter(value=>value!==kind) as NoticeKind[]})}/>{t(kind==='new'?'notices-new-message':`notices-${kind}`)}</label>}</For></div>
    <label class={css.field}>{t('notices-audience')}<select class={css.select} value={row().audience} onChange={event=>change({audience:event.currentTarget.value as MessageNotice['audience']})}><For each={['all','internal','external']}>{value=><option value={value}>{t(`notices-${value}`)}</option>}</For></select></label>
    <label class={css.field}>{t('notices-domains')}<input class={css.input} value={row().domains.join(', ')} onChange={event=>change({domains:csv(event.currentTarget.value).map(value=>value.toLowerCase())})}/></label>
    <label class={css.field}>{t('notices-subject')}<input class={css.input} maxLength={200} value={row().subjectContains} onInput={event=>change({subjectContains:event.currentTarget.value})}/></label>
    <label class={css.field}>{t('notices-body')}<input class={css.input} maxLength={500} value={row().bodyContains} onInput={event=>change({bodyContains:event.currentTarget.value})}/></label>
   </fieldset>
  </>}</Show>
 </div>;
}
