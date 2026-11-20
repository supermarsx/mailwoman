import {createMemo,Show,type JSX} from 'solid-js';
import {loadCatalog,t,formatNumber} from '../../../i18n';
import {FormField} from '../shell/FormField.tsx';
import {DEFAULT_DATABASE_POOL,type DatabasePoolPolicy} from './databasePool.ts';
import * as css from '../shell/ux.css.ts';

export function DatabasePoolEditor(props:{id:string;describedBy?:string|undefined;invalid?:boolean|undefined;disabled?:boolean|undefined;draft:string;onDraft:(value:string)=>void}):JSX.Element{
 void loadCatalog('admin-settings');
 const value=createMemo<Record<string,unknown>>(()=>{try{const parsed:unknown=JSON.parse(props.draft);return parsed&&typeof parsed==='object'&&!Array.isArray(parsed)?{...DEFAULT_DATABASE_POOL,...parsed}:{...DEFAULT_DATABASE_POOL};}catch{return {...DEFAULT_DATABASE_POOL};}});
 const update=(key:keyof DatabasePoolPolicy,next:number|string|null)=>props.onDraft(JSON.stringify({...value(),[key]:next}));
 const inputValue=(key:keyof DatabasePoolPolicy)=>typeof value()[key]==='number'?Number(value()[key]):'';
 const numeric=(key:keyof DatabasePoolPolicy,label:string,min:number,max:number)=><FormField label={label} help={t('admin-pool-range',{min:formatNumber(min),max:formatNumber(max)})}>{w=><input id={w.id} class={css.control} type="number" min={min} max={max} step="1" aria-describedby={w.describedBy} disabled={props.disabled} value={inputValue(key)} onInput={event=>update(key,Number.isFinite(event.currentTarget.valueAsNumber)?event.currentTarget.valueAsNumber:event.currentTarget.value)}/>}</FormField>;
 const maximum=(key:'sqliteMaxConnections'|'postgresMaxConnections',label:string,max:number,manual:number)=><FormField label={label} help={t('admin-pool-range',{min:'1',max:formatNumber(max)})}>{w=><div class={css.fieldBlock}><select id={w.id} class={css.control} disabled={props.disabled} aria-describedby={w.describedBy} value={value()[key]===null?'auto':'custom'} onChange={event=>update(key,event.currentTarget.value==='auto'?null:manual)}><option value="auto">{t('admin-pool-auto')}</option><option value="custom">{t('admin-pool-custom')}</option></select><Show when={value()[key]!==null}><input class={css.control} type="number" aria-label={label} aria-describedby={w.describedBy} disabled={props.disabled} min="1" max={max} step="1" value={inputValue(key)} onInput={event=>update(key,Number.isFinite(event.currentTarget.valueAsNumber)?event.currentTarget.valueAsNumber:event.currentTarget.value)}/></Show></div>}</FormField>;
 const budget=()=>{const budget=Number(value().postgresConnectionBudget),reserved=Number(value().postgresReservedConnections),replicas=Number(value().replicaCount);return Number.isSafeInteger(budget)&&Number.isSafeInteger(reserved)&&Number.isSafeInteger(replicas)&&replicas>0&&budget>reserved?Math.floor((budget-reserved)/replicas):null;};
 return <div id={props.id} role="group" aria-label={t('admin-pool-title')} aria-describedby={props.describedBy} aria-invalid={props.invalid} class={css.form}>
  <p class={css.fieldsetHelp}>{t('admin-pool-help')}</p>
  <fieldset class={css.fieldset} disabled={props.disabled}><legend class={css.legend}>{t('admin-pool-connections')}</legend><p class={css.fieldsetHelp}>{t('admin-pool-connections-help')}</p>
   {numeric('minimumConnections',t('admin-pool-minimum'),0,1024)}
   {maximum('sqliteMaxConnections',t('admin-pool-sqlite-maximum'),32,8)}
   {maximum('postgresMaxConnections',t('admin-pool-postgres-maximum'),1024,32)}
  </fieldset>
  <fieldset class={css.fieldset} disabled={props.disabled}><legend class={css.legend}>{t('admin-pool-timing')}</legend>
   {numeric('acquireTimeoutMs',t('admin-pool-acquire'),100,120000)}
   {numeric('idleTimeoutSeconds',t('admin-pool-idle'),1,3600)}
   {numeric('maxLifetimeSeconds',t('admin-pool-lifetime'),30,86400)}
  </fieldset>
  <fieldset class={css.fieldset} disabled={props.disabled}><legend class={css.legend}>{t('admin-pool-budget')}</legend><p class={css.fieldsetHelp}>{t('admin-pool-budget-help')}</p>
   {numeric('postgresConnectionBudget',t('admin-pool-total'),1,1000000)}
   {numeric('postgresReservedConnections',t('admin-pool-reserved'),0,999999)}
   {numeric('replicaCount',t('admin-pool-replicas'),1,10000)}
   <Show when={budget()!==null}><p class={css.fieldsetHelp}>{t('admin-pool-per-replica',{count:formatNumber(budget()!)})}</p></Show>
  </fieldset>
 </div>;
}
