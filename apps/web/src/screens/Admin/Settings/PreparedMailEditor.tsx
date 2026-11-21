import {createMemo,For,type JSX} from 'solid-js';
import {t,useCatalog} from '../../../i18n';
import {FormField} from '../shell/FormField.tsx';
import {DEFAULT_PREPARED_MAIL,PREPARED_MAIL_BOUNDS,type PreparedMailConfig} from './preparedMail.ts';
import * as css from '../shell/ux.css.ts';
export function PreparedMailEditor(props:{id:string;describedBy?:string|undefined;invalid?:boolean|undefined;disabled?:boolean|undefined;draft:string;onDraft:(value:string)=>void}):JSX.Element{
 useCatalog('admin-cache');
 const value=createMemo<Record<string,unknown>>(()=>{try{const parsed:unknown=JSON.parse(props.draft);if(parsed&&typeof parsed==='object'&&!Array.isArray(parsed))return parsed as Record<string,unknown>;}catch{/* Do not silently replace the parent's invalid draft. */}return {...DEFAULT_PREPARED_MAIL};});
 const update=(key:keyof PreparedMailConfig,next:unknown)=>props.onDraft(JSON.stringify({...value(),[key]:next}));
 const keys=Object.keys(PREPARED_MAIL_BOUNDS) as (keyof typeof PREPARED_MAIL_BOUNDS)[];
 const labels={budgetMiB:()=>t('admin-cache-prepared-budgetMiB'),maxEntries:()=>t('admin-cache-prepared-maxEntries'),maxAccounts:()=>t('admin-cache-prepared-maxAccounts'),idleSeconds:()=>t('admin-cache-prepared-idleSeconds')};
 return <div id={props.id} role="group" aria-label={t('admin-cache-prepared-title')} aria-describedby={props.describedBy} aria-invalid={props.invalid}>
  <p>{t('admin-cache-prepared-help')}</p>
  <label><input type="checkbox" checked={value().enabled===true} disabled={props.disabled} onChange={event=>update('enabled',event.currentTarget.checked)}/>{t('admin-cache-prepared-enabled')}</label>
  <For each={keys}>{key=><FormField label={labels[key]()}>{w=><input id={w.id} class={css.control} type="number" min={PREPARED_MAIL_BOUNDS[key][0]} max={PREPARED_MAIL_BOUNDS[key][1]} step="1" aria-describedby={w.describedBy} disabled={props.disabled} value={typeof value()[key]==='number'||typeof value()[key]==='string'?String(value()[key]):''} onInput={event=>update(key,Number.isFinite(event.currentTarget.valueAsNumber)?event.currentTarget.valueAsNumber:event.currentTarget.value)}/>}</FormField>}</For>
  <p>{t('admin-cache-prepared-limits')}</p><p>{t('admin-cache-prepared-refresh')}</p>
 </div>;
}
