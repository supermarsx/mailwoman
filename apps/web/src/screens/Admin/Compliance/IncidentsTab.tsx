import {For,Show,createResource,createSignal,useContext,type JSX} from 'solid-js';
import {formatDateTime,loadCatalog,t} from '../../../i18n';
import {adminRequest} from '../shell/adminApi.ts';
import {ShellContext} from '../shell/context.ts';
import {describeAdminError} from '../shell/Feedback.tsx';
import {InsertDialog} from '../../../components/compose/insert/InsertDialog.tsx';
import {InlineSettingEditor} from '../Settings/InlineSettingEditor.tsx';
import {useSettingRows} from './useSettingRows.ts';
import {saveText} from '../../../api/admin/compliance.ts';
import * as css from '../admin.css.ts';
import * as ccss from './compliance.css.ts';
interface Incident {title:string;category:string;source:string;component:string;status:string;significance:string;detectedAt:number;awareAt:number|null;firstHealthyAt:number|null;verifiedAt:number|null;resolvedAt:number|null;rootCause:string;impact:string;owner:string;measures:string;recoveryPlan:string;recoveryEvidence:string;recoveryTimeMinutes:number|null;recoveryPointMinutes:number|null;auditReference:string;events:{at:number;kind:string;note:string;actor:string}[];reports:{phase:string;submittedAt:number;reference:string}[]}
interface Row {id:string;updatedAt:number;version:string;incident:Incident;deadlines:Record<string,number>}
const CATEGORIES=['outage','account-compromise','service-disruption','vulnerability'];
const STATES=['detected','active','recovering','resolved','closed'];
const PHASES=['early-warning','notification','progress','final'];
const KEYS=['incidents.auto.enabled','incidents.auto.failureSamples','incidents.nis2.trustService','incidents.notify.email','incidents.notify.operatorAddresses','incidents.retention.days'];
const time=(at:number|null|undefined):string=>at?formatDateTime(at*1000):'—';
const localInput=(at:number|null):string=>at?new Date(at*1000-new Date(at*1000).getTimezoneOffset()*60000).toISOString().slice(0,16):'';
const dateValue=(value:string):number|null=>value?Math.floor(new Date(value).getTime()/1000):null;
function Editor(props:{row:Row|null;onSave:()=>Promise<void>;onClose:()=>void}):JSX.Element{
 const initial=props.row?.incident;
 const [fields,setFields]=createSignal({title:initial?.title??'',category:initial?.category??'service-disruption',status:initial?.status??'detected',significance:initial?.significance??'unassessed',awareAt:initial?.awareAt??null,handledAt:initial?.resolvedAt??null,owner:initial?.owner??'',impact:initial?.impact??'',rootCause:initial?.rootCause??'',measures:initial?.measures??'',recoveryPlan:initial?.recoveryPlan??'',recoveryEvidence:initial?.recoveryEvidence??'',recoveryTimeMinutes:initial?.recoveryTimeMinutes??null,recoveryPointMinutes:initial?.recoveryPointMinutes??null});
 const [busy,setBusy]=createSignal(false),[error,setError]=createSignal<unknown>();
 const change=(key:string,value:unknown)=>setFields(previous=>({...previous,[key]:value}));
 return <InsertDialog title={t('res-incidents')} onClose={props.onClose} busy={busy()} testId="incident-editor"><form class={ccss.sectionBox} onSubmit={async e=>{e.preventDefault();setBusy(true);setError(undefined);try{await adminRequest(`/incidents${props.row?`/${encodeURIComponent(props.row.id)}`:''}`,{method:props.row?'PUT':'POST',body:{...fields(),...(props.row?{updatedAt:props.row.updatedAt,version:props.row.version}:{})}});await props.onSave();}catch(error){setError(error);}finally{setBusy(false);}}}>
 <label>{t('res-title')}<input class={ccss.field} required maxLength={200} value={fields().title} onInput={e=>change('title',e.currentTarget.value)}/></label>
 <For each={['category','status','significance'] as const}>{key=><label>{t(`res-${key}`)}<select class={ccss.field} value={fields()[key]} onChange={e=>change(key,e.currentTarget.value)}><For each={key==='category'?CATEGORIES:key==='status'?STATES:['unassessed','significant','not-significant']}>{option=><option value={option}>{t(`res-${option}`)}</option>}</For></select></label>}</For>
 <label>{t('res-awareness')}<input class={ccss.field} type="datetime-local" value={localInput(fields().awareAt)} onInput={e=>change('awareAt',dateValue(e.currentTarget.value))}/></label>
 <Show when={['resolved','closed'].includes(fields().status)}><label>{t('res-handled')}<input class={ccss.field} type="datetime-local" value={localInput(fields().handledAt)} onInput={e=>change('handledAt',dateValue(e.currentTarget.value))}/></label></Show>
 <For each={['owner','impact','rootCause','measures','recoveryPlan','recoveryEvidence'] as const}>{key=><label>{t(`res-${key}`)}<textarea class={ccss.textarea} maxLength={4000} value={fields()[key]} onInput={e=>change(key,e.currentTarget.value)}/></label>}</For>
 <For each={['recoveryTimeMinutes','recoveryPointMinutes'] as const}>{key=><label>{t(`res-${key}`)}<input class={ccss.field} type="number" min="0" step="1" value={fields()[key]??''} onInput={e=>change(key,e.currentTarget.value===''?null:Number(e.currentTarget.value))}/></label>}</For>
 <p class={css.note}>{t('res-recovery-help')}</p><Show when={error()}><p role="alert">{describeAdminError(error())}</p></Show><div class={ccss.row}><button class="btn" type="submit" disabled={busy()||!fields().title.trim()}>{t('common-save')}</button><button class="btn btn--ghost" type="button" disabled={busy()} onClick={props.onClose}>{t('common-cancel')}</button></div>
 </form></InsertDialog>;
}
export default function IncidentsTab():JSX.Element{
 void loadCatalog('resilience');const shell=useContext(ShellContext);const write=()=>shell?.can.can('privacy.write')??false;
 const settings=useSettingRows(KEYS);const [data,{refetch}]=createResource(async()=>(await adminRequest<{items:Row[];automatic:boolean}>('/incidents')).data);
 const [query,setQuery]=createSignal(''),[editing,setEditing]=createSignal<{row:Row|null}>(),[failure,setFailure]=createSignal<unknown>(),[busy,setBusy]=createSignal(false),[report,setReport]=createSignal<Row>();
 const [phase,setPhase]=createSignal('early-warning'),[submitted,setSubmitted]=createSignal(localInput(Math.floor(Date.now()/1000))),[reference,setReference]=createSignal('');
 const rows=()=>data.error?[]:data()?.items.filter(row=>`${row.incident.title} ${row.incident.category} ${row.incident.status}`.toLocaleLowerCase().includes(query().trim().toLocaleLowerCase()))??[];
 async function run(action:()=>Promise<unknown>):Promise<void>{if(busy())return;setBusy(true);setFailure(undefined);try{await action();await refetch();}catch(error){setFailure(error);}finally{setBusy(false);}}
 return <div class={css.section} data-testid="operational-incidents"><p class={css.note}>{t('res-help')}</p>
 <details><summary>{t('res-automation')}</summary><For each={settings.rows()}>{row=><InlineSettingEditor settingKey={row.key} setting={row} label={t(`res-${row.key.replace(/\./g,'-')}`)} onChanged={()=>settings.reload()}/>}</For></details>
 <div class={ccss.row}><label>{t('res-search')}<input class={ccss.field} type="search" value={query()} onInput={e=>setQuery(e.currentTarget.value)}/></label><button class="btn" disabled={!write()||data.loading} onClick={()=>setEditing({row:null})}>{t('res-add')}</button><button class="btn btn--ghost" disabled={!write()||busy()} onClick={()=>void run(()=>adminRequest('/incidents/scan',{method:'POST',body:{}}))}>{t('res-scan')}</button><button class="btn btn--ghost" disabled={busy()} onClick={()=>void refetch()}>{t('res-refresh')}</button></div>
 <Show when={failure()||data.error}><p role="alert">{describeAdminError(failure()||data.error)}</p></Show>
 <For each={rows()}>{row=><details class={ccss.sectionBox}><summary><strong>{row.incident.title}</strong> · {t(`res-${row.incident.status}`)} · {time(row.incident.detectedAt)}</summary>
 <dl><dt>{t('res-category')}</dt><dd>{t(`res-${row.incident.category}`)}</dd><dt>{t('res-significance')}</dt><dd>{t(`res-${row.incident.significance}`)}</dd><dt>{t('res-awareness')}</dt><dd>{time(row.incident.awareAt)}</dd><dt>{t('res-recovery-observed')}</dt><dd>{time(row.incident.firstHealthyAt)}</dd><dt>{t('res-recovery-verified')}</dt><dd>{time(row.incident.verifiedAt)}</dd><dt>{t('res-owner')}</dt><dd>{row.incident.owner||'—'}</dd><dt>{t('res-impact')}</dt><dd>{row.incident.impact||'—'}</dd></dl>
 <Show when={row.incident.auditReference}><p>{t('res-audit')}: <code>{row.incident.auditReference}</code></p></Show>
 <h3>{t('res-reporting')}</h3><p class={css.note}>{t('res-reporting-help')}</p><ul><For each={PHASES.filter(p=>row.deadlines[p])}>{p=><li>{t(`res-${p}`)}: {time(row.deadlines[p])}<Show when={row.deadlines[p]!<Date.now()/1000&&!row.incident.reports.some(r=>r.phase===p)}><span class={css.badge}>{t('res-overdue')}</span></Show></li>}</For></ul>
 <For each={row.incident.reports}>{r=><p>{t(`res-${r.phase}`)} · {time(r.submittedAt)} · {r.reference}</p>}</For>
 <div class={ccss.row}><button class="btn" disabled={!write()} onClick={()=>setEditing({row})}>{t('common-edit')}</button><button class="btn btn--ghost" disabled={!write()||row.incident.significance!=='significant'} onClick={()=>{setReport(row);setReference('');setPhase('early-warning');}}>{t('res-record-report')}</button><button class="btn btn--ghost" onClick={()=>void run(async()=>{const value=await adminRequest(`/incidents/${encodeURIComponent(row.id)}/export`);saveText(JSON.stringify(value.data,null,2),'incident-report.json','application/json');})}>{t('res-export')}</button></div>
 <h3>{t('res-timeline')}</h3><For each={row.incident.events}>{event=><p><time>{time(event.at)}</time> · {event.note}</p>}</For>
 </details>}</For>
 <Show when={editing()} keyed>{value=><Editor row={value.row} onSave={async()=>{setEditing(undefined);await refetch();}} onClose={()=>setEditing(undefined)}/>}</Show>
 <Show when={report()} keyed>{row=><InsertDialog testId="incident-report" title={t('res-record-report')} busy={busy()} onClose={()=>setReport(undefined)}><form class={ccss.sectionBox} onSubmit={e=>{e.preventDefault();void run(async()=>{await adminRequest(`/incidents/${encodeURIComponent(row.id)}/reports`,{method:'POST',body:{phase:phase(),submittedAt:dateValue(submitted()),reference:reference()}});setReport(undefined);});}}><p>{t('res-reporting-help')}</p><Show when={failure()}><p role="alert">{describeAdminError(failure())}</p></Show><label>{t('res-reporting')}<select class={ccss.field} value={phase()} onChange={e=>setPhase(e.currentTarget.value)}><For each={PHASES}>{p=><option value={p}>{t(`res-${p}`)}</option>}</For></select></label><label>{t('res-submitted')}<input class={ccss.field} required type="datetime-local" value={submitted()} onInput={e=>setSubmitted(e.currentTarget.value)}/></label><label>{t('res-reference')}<input class={ccss.field} required maxLength={500} value={reference()} onInput={e=>setReference(e.currentTarget.value)}/></label><button class="btn" type="submit" disabled={busy()||!reference().trim()}>{t('common-save')}</button></form></InsertDialog>}</Show>
 </div>;
}
