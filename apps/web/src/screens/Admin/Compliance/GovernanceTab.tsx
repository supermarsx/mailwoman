import {For,Show,createMemo,createResource,createSignal,useContext,type JSX} from 'solid-js';
import {formatDateWith} from '../../../i18n/format.ts';
import {loadCatalog,t} from '../../../i18n';
import {adminRequest} from '../shell/adminApi.ts';
import {ShellContext} from '../shell/context.ts';
import {describeAdminError} from '../shell/Feedback.tsx';
import {InsertDialog} from '../../../components/compose/insert/InsertDialog.tsx';
import {complianceApi,saveText} from '../../../api/admin/compliance.ts';
import * as css from '../admin.css.ts';
import * as ccss from './compliance.css.ts';

export type GovernanceKind='contract'|'transfer'|'control';
interface Row {id:string;name:string;builtin:boolean;enabled:boolean;[key:string]:unknown}
interface Register {contract:Row[];transfer:Row[];control:Row[];catalog:Row[];processor:Row[]}
export const FIELDS:Record<GovernanceKind,string[]>={
 contract:['owner','status','contractType','counterparty','processorId','purpose','signedAt','expiresAt','reviewAt','jurisdiction','documentUrl','clauses','notes'],
 transfer:['owner','status','direction','exporter','importer','sourceCountry','destinationCountry','processorId','contractId','categories','purpose','legalBasis','mechanism','safeguards','assessmentUrl','reviewAt','notes'],
 control:['owner','status','controlId','mappings','implementation','evidence','testedAt','reviewAt','notes'],
};
const OPTIONS:Record<string,string[]>={contractType:['dpa','msa','scc','other'],direction:['internal','international'],mechanism:['internal','adequacy','scc','bcr','derogation','other']};
const STATUSES:Record<GovernanceKind,string[]>={contract:['draft','active','expired','terminated'],transfer:['draft','active','suspended','closed'],control:['unassessed','planned','implemented','verified','exception']};
const text=(row:Row,key:string):string=>String(row[key]??'');
const today=():string=>new Date().toISOString().slice(0,10);

function Editor(props:{kind:GovernanceKind;row:Row|null;processors:Row[];contracts:Row[];onSave:(name:string,fields:Record<string,string>)=>Promise<void>;onClose:()=>void}):JSX.Element{
 const [name,setName]=createSignal(props.row?.name??'');
 const [fields,setFields]=createSignal<Record<string,string>>(Object.fromEntries(FIELDS[props.kind].map(key=>[key,props.row?text(props.row,key):key==='status'?STATUSES[props.kind][0]!:key==='direction'||key==='mechanism'?'internal':key==='contractType'?'dpa':''])));
 const [step,setStep]=createSignal(0),[busy,setBusy]=createSignal(false),[error,setError]=createSignal<unknown>();
 const shown=()=>step()===0?FIELDS[props.kind].slice(0,4):FIELDS[props.kind].slice(4);
 const change=(key:string,value:string)=>setFields(previous=>({...previous,[key]:value}));
 const label=(key:string)=>t(`governance-field-${key}`);
 async function save():Promise<void>{if(busy())return;setBusy(true);setError(undefined);try{await props.onSave(name().trim(),fields());}catch(failure){setError(failure);}finally{setBusy(false);}}
 return <InsertDialog title={t(`governance-${props.kind}`)} busy={busy()} onClose={props.onClose} testId="governance-editor">
  <form class={ccss.sectionBox} onSubmit={event=>{event.preventDefault();if(step()<2)setStep(step()+1);else void save();}}>
   <p>{t(`governance-guide-${step()}`)}</p>
   <Show when={step()===0}><label>{t('compliance-records-name')}<input class={ccss.field} required maxLength={200} value={name()} onInput={event=>setName(event.currentTarget.value)}/></label></Show>
   <Show when={step()<2} fallback={<dl><dt>{t('compliance-records-name')}</dt><dd>{name()}</dd><For each={FIELDS[props.kind]}>{key=><><dt>{label(key)}</dt><dd>{fields()[key]||'—'}</dd></>}</For></dl>}>
    <For each={shown()}>{key=>{
     const options=()=>key==='status'?STATUSES[props.kind]:OPTIONS[key];
     const linked=()=>key==='processorId'?props.processors:key==='contractId'?props.contracts:undefined;
     return <label>{label(key)}
      <Show when={options()} fallback={<Show when={linked()} fallback={
       <Show when={key.endsWith('At')} fallback={<textarea class={ccss.textarea} maxLength={2000} value={fields()[key]??''} onInput={event=>change(key,event.currentTarget.value)}/>}>
        <input class={ccss.field} type="date" value={fields()[key]??''} onInput={event=>change(key,event.currentTarget.value)}/>
       </Show>
      }>{rows=><select class={ccss.field} value={fields()[key]??''} onChange={event=>change(key,event.currentTarget.value)}><option value="">—</option><For each={rows()}>{row=><option value={row.id}>{row.name}</option>}</For></select>}</Show>}>
       {values=><select class={ccss.field} value={fields()[key]??''} onChange={event=>change(key,event.currentTarget.value)}><For each={values()}>{value=><option value={value}>{t(`governance-option-${value}`)}</option>}</For></select>}
      </Show>
     </label>;
    }}</For>
   </Show>
   <Show when={error()}><p role="alert">{describeAdminError(error())}</p></Show>
   <div class={ccss.row}><button class="btn btn--ghost" type="button" disabled={busy()} onClick={()=>step()>0?setStep(step()-1):props.onClose()}>{t(step()>0?'common-back':'common-cancel')}</button><button class="btn" type="submit" disabled={busy()||!name().trim()}>{t(step()<2?'common-next':'common-save')}</button></div>
  </form>
 </InsertDialog>;
}

function GovernanceTab(props:{kind:GovernanceKind}):JSX.Element{
 void loadCatalog('admin-governance');
 const shell=useContext(ShellContext);const canWrite=()=>shell?.can.can('settings.write')??false;
 const [data,{refetch}]=createResource(async()=>(await adminRequest<Register>('/compliance/governance')).data);
 const [query,setQuery]=createSignal(''),[filter,setFilter]=createSignal(''),[editing,setEditing]=createSignal<{row:Row|null}>(),[failure,setFailure]=createSignal<unknown>();
 const rows=createMemo(()=>{
  const saved=data()?.[props.kind]??[];
  const all=props.kind==='control'?[...saved,...(data()?.catalog??[]).filter(row=>!saved.some(value=>value.controlId===row.controlId))]:saved;
  return all.filter(row=>(!filter()||row.status===filter())&&JSON.stringify(row).toLocaleLowerCase().includes(query().trim().toLocaleLowerCase()));
 });
 const columns=()=>props.kind==='contract'?['counterparty','contractType','status','owner','expiresAt','reviewAt','documentUrl']:props.kind==='transfer'?['direction','exporter','importer','sourceCountry','destinationCountry','mechanism','status','owner','reviewAt']:['status','owner','reviewAt','mappings'];
 const cell=(row:Row,key:string):string=>key.endsWith('At')&&text(row,key)?formatDateWith(new Date(`${text(row,key)}T12:00:00Z`),{dateStyle:'medium',timeZone:'UTC'}):['status','contractType','direction','mechanism'].includes(key)?t(`governance-option-${text(row,key)}`):text(row,key);
 async function save(name:string,fields:Record<string,string>):Promise<void>{
  const row=editing()?.row;
  const id=row&&!row.builtin?row.id:null;
  await adminRequest(`/compliance/records${id?`/${encodeURIComponent(id)}`:''}`,{method:id?'PUT':'POST',body:{kind:props.kind,name,fields,enabled:true}});
  setEditing(undefined);await refetch();
 }
 async function exportRows(format:'json'|'csv'|'html'):Promise<void>{
  try{setFailure(undefined);const value=await complianceApi.exportRecords(props.kind,format,'en');saveText(value,`${props.kind}.${format}`,format==='json'?'application/json':format==='html'?'text/html':'text/csv');}catch(error){setFailure(error);}
 }
 return <div class={css.section} data-testid={`governance-${props.kind}`}>
  <p class={css.note}>{t('governance-intro')}</p>
  <div class={ccss.row}><label>{t('governance-search')}<input type="search" class={ccss.field} value={query()} onInput={event=>setQuery(event.currentTarget.value)}/></label>
   <label>{t('governance-field-status')}<select class={ccss.field} value={filter()} onChange={event=>setFilter(event.currentTarget.value)}><option value="">{t('governance-all')}</option><For each={STATUSES[props.kind]}>{status=><option value={status}>{t(`governance-option-${status}`)}</option>}</For></select></label>
   <button class="btn" disabled={!canWrite()||data.loading||!!data.error} onClick={()=>setEditing({row:null})}>{t('governance-add')}</button>
   <For each={['json','csv','html'] as const}>{format=><button class="btn btn--ghost" onClick={()=>void exportRows(format)}>{t(`compliance-records-export-${format}`)}</button>}</For>
  </div>
  <Show when={data.error||failure()}><p role="alert">{describeAdminError(data.error||failure())}</p></Show>
  <Show when={data.loading}><p role="status">{t('admin-shell-loading')}</p></Show>
  <div class={css.tableWrap}><table class={css.table}><thead><tr><th>{t('compliance-records-name')}</th><For each={columns()}>{key=><th>{t(`governance-field-${key}`)}</th>}</For><th>{t('compliance-records-actions')}</th></tr></thead><tbody>
   <For each={rows()}>{row=><tr><th scope="row">{row.name}</th><For each={columns()}>{key=><td>
    <Show when={key==='documentUrl'&&text(row,key).startsWith('https://')} fallback={cell(row,key)}><a href={text(row,key)} target="_blank" rel="noopener noreferrer">{t('governance-field-documentUrl')}</a></Show>
    <Show when={key==='reviewAt'&&text(row,key)&&text(row,key)<today()}><span class={css.badge}>{t('governance-overdue')}</span></Show>
   </td>}</For><td><button class="btn btn--ghost" disabled={!canWrite()} onClick={()=>setEditing({row})}>{t('compliance-records-edit')}</button></td></tr>}</For>
  </tbody></table></div>
  <Show when={editing()} keyed>{edit=><Editor kind={props.kind} row={edit.row} processors={data()?.processor??[]} contracts={data()?.contract??[]} onSave={save} onClose={()=>setEditing(undefined)}/>}</Show>
 </div>;
}
export function ContractsTab():JSX.Element{return <GovernanceTab kind="contract"/>;}
export function TransfersTab():JSX.Element{return <GovernanceTab kind="transfer"/>;}
export function ControlsTab():JSX.Element{return <GovernanceTab kind="control"/>;}
