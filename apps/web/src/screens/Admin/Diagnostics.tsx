import { For, Show, createSignal, onMount, onCleanup, type JSX } from 'solid-js';
import { t } from '../../i18n';
import { adminRead, adminRequest, type AdminReadResult } from './shell/adminApi.ts';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import { useShell } from './shell/context.ts';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';
import { AsyncPending } from '../../components/AsyncState.tsx';
import { DeploymentSecurity } from './DeploymentSecurity.tsx';

interface Check { id: string; status: string; durationMs: number; summary: string; domain?: string; backendId?: string; protocol?: string; fixId?: string }
interface Run { id: string; suite: string; startedAt: number; status: string; results: Check[] }
interface History { runs: Run[]; nextCursor: string | null }
interface Job { run: Run; total: number; completed: number; limited: boolean; historySaved: boolean; auditSaved?: boolean; termination?: string }
interface Catalogue { canRunBackends?: boolean; limits?: {concurrency:number;checks:number;runSeconds:number;checkSeconds:number}; suites: {id:string; supported:boolean; description?:string; reason?:string}[] }
export interface DiagnosticsApi {
  history: (cursor?: string) => Promise<AdminReadResult<History>>;
  run: () => Promise<{ run: Run; historySaved: boolean; auditSaved: boolean; retentionApplied?: boolean }>;
  start?: () => Promise<Job>;
  job?: (id: string) => Promise<Job>;
  cancel?: (id: string) => Promise<unknown>;
  report?: () => Promise<unknown>;
  catalogue?: () => Promise<Catalogue>;
}
const httpApi: DiagnosticsApi = {
  history: (cursor) => adminRead<History>(`/diag/runs${cursor ? `?cursor=${encodeURIComponent(cursor)}` : ''}`),
  run: async () => (await adminRequest<{ run: Run; historySaved: boolean; auditSaved: boolean; retentionApplied?: boolean }>('/diag/run', { method: 'POST', body: { suite: 'server' } })).data,
  start: async () => (await adminRequest<Job>('/diag/jobs', {method:'POST',body:{suite:'backends'}})).data,
  job: async id => (await adminRequest<Job>(`/diag/jobs/${encodeURIComponent(id)}`)).data,
  cancel: async id => adminRequest(`/diag/jobs/${encodeURIComponent(id)}/cancel`,{method:'POST',body:{}}),
  report: async () => (await adminRequest<unknown>('/diag/report')).data,
  catalogue: async () => (await adminRequest<Catalogue>('/diag/catalogue')).data,
};

export function Diagnostics(props: { api?: DiagnosticsApi }): JSX.Element {
  const shell=useShell();
  return <Show when={shell.router.route().rest[0]==='security'} fallback={<DiagnosticRuns {...props}/>}><DeploymentSecurity/></Show>;
}
function DiagnosticRuns(props: { api?: DiagnosticsApi }): JSX.Element {
  const shell = useShell();
  const api = (): DiagnosticsApi => props.api ?? httpApi;
  const [runs, setRuns] = createSignal<Run[]>([]);
  const [selected, setSelected] = createSignal<Run>();
  const [cursor, setCursor] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string>();
  const [notice, setNotice] = createSignal<string>();
  const [job,setJob]=createSignal<Job>();
  const [status,setStatus]=createSignal('');
  const [query,setQuery]=createSignal('');
  const [catalogue,setCatalogue]=createSignal<Catalogue>();
  let timer:ReturnType<typeof setTimeout>|undefined;
  let disposed=false;
  onCleanup(()=>{disposed=true;clearTimeout(timer);});
  const filtered=()=>selected()?.results.filter(check=>(!status() || check.status===status()) && `${check.summary} ${check.domain??''} ${check.backendId??''} ${check.protocol??''}`.toLowerCase().includes(query().toLowerCase())) ?? [];
  async function receive(value:Job):Promise<void>{
    if(disposed)return;
    setJob(value);setSelected(value.run);
    if(value.run.status==='running') timer=setTimeout(()=>{void api().job?.(value.run.id).then(receive).catch(e=>{if(!disposed){setError(describeAdminError(e));setBusy(false);}});},1000);
    else {setBusy(false);if(value.historySaved)setRuns([value.run,...runs().filter(r=>r.id!==value.run.id)]);setNotice(value.termination==='cancelled'?t('admin-diag-cancelled'):value.limited?t('admin-diag-limited'):!value.historySaved?t('admin-diag-not-saved'):undefined);}
  }
  async function startBackends():Promise<void>{setBusy(true);setError(undefined);setNotice(undefined);try{const value=await api().start?.();if(value)await receive(value);}catch(e){setError(describeAdminError(e));setBusy(false);}}
  async function cancelJob():Promise<void>{const current=job();if(!current)return;try{await api().cancel?.(current.run.id);setNotice(t('admin-diag-cancelling'));}catch(e){setError(describeAdminError(e));}}
  async function supportReport(copy:boolean):Promise<void>{try{const report=await api().report?.();const text=JSON.stringify({...report as object,selectedRun:selected()},null,2);if(copy){await navigator.clipboard.writeText(text);setNotice(t('admin-diag-copied'));}else{const url=URL.createObjectURL(new Blob([text],{type:'application/json'}));const link=document.createElement('a');link.href=url;link.download='mailwoman-diagnostics.json';link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}}catch(e){setError(describeAdminError(e));}}
  async function history(more = false): Promise<void> {
    setBusy(true); setError(undefined); setNotice(undefined);
    try {
      const result = await api().history(more ? cursor() ?? undefined : undefined);
      if (result.kind === 'unavailable') { setNotice(t('admin-diag-unavailable')); return; }
      setRuns(more ? [...runs(), ...result.data.runs] : result.data.runs);
      setCursor(result.data.nextCursor);
      if (!selected()) setSelected(result.data.runs[0]);
    } catch (e) { setError(describeAdminError(e)); }
    finally { setBusy(false); }
  }
  async function run(): Promise<void> {
    setBusy(true); setError(undefined); setNotice(undefined);
    try {
      const result = await api().run();
      setSelected(result.run);
      if (result.historySaved) setRuns([result.run, ...runs().filter((r) => r.id !== result.run.id)]);
      if (!result.historySaved || !result.auditSaved) setNotice(t('admin-diag-not-saved'));
      else if (result.retentionApplied === false) setNotice(t('admin-diag-retention-failed'));
    } catch (e) { setError(describeAdminError(e)); }
    finally { setBusy(false); }
  }
  onMount(() => {void history();void api().catalogue?.().then(value=>{if(!disposed)setCatalogue(value);}).catch(e=>{if(!disposed)setError(describeAdminError(e));});});
  return <section class={css.section} aria-label={t('admin-diag-title')}>
    <h2 class={css.heading}>{t('admin-diag-title')}</h2>
    <p class={css.note}>{t('admin-diag-description')}</p>
    <div class={ux.pageActions}>
      <a class="btn" href="#/diagnostics/security">{t('admin-deployment-title')}</a>
      <button class="btn btn--primary" type="button" disabled={busy() || !shell.can.can('diagnostics.run')} onClick={() => void run()}>{t('admin-diag-run')}</button>
      <button class="btn" type="button" disabled={busy()} onClick={() => void history()}>{t('admin-diag-refresh')}</button>
      <Show when={api().start}><button class="btn" type="button" disabled={busy() || !shell.can.can('diagnostics.run') || (!!api().catalogue && catalogue()?.canRunBackends!==true)} title={t('admin-diag-backends-permission')} onClick={()=>void startBackends()}>{t('admin-diag-backends')}</button></Show>
      <Show when={job()?.run.status==='running'}><button class="btn" type="button" onClick={()=>void cancelJob()}>{t('admin-diag-cancel')}</button><span role="status">{job()?.completed} / {job()?.total}</span></Show>
      <Show when={api().report}><button class="btn" type="button" onClick={()=>void supportReport(true)}>{t('admin-diag-copy')}</button><button class="btn" type="button" onClick={()=>void supportReport(false)}>{t('admin-diag-export')}</button></Show>
    </div>
    <Show when={busy()}><AsyncPending timeoutMs={70_000} /></Show>
    <p class={css.note}>{t('admin-diag-redaction')}</p>
    <Show when={catalogue()}><details><summary>{t('admin-diag-catalogue')}</summary><ul><For each={catalogue()?.suites}>{suite=><li>{suite.id}: {suite.description ?? suite.reason}</li>}</For></ul><Show when={catalogue()?.limits}>{limits=><p>{t('admin-diag-limits',{concurrency:limits().concurrency,checks:limits().checks,run:limits().runSeconds,check:limits().checkSeconds})}</p>}</Show></details></Show>
    <Feedback kind="error" message={error()} /><Feedback kind="info" message={notice()} />
    <Show when={selected()}>{(current) => <div class={css.card}>
      <h3>{t('admin-diag-result')} — {current().status}</h3>
      <div class={ux.pageActions}><label>{t('admin-diag-filter')}<input value={query()} onInput={e=>setQuery(e.currentTarget.value)}/></label><label>{t('admin-diag-status')}<select value={status()} onChange={e=>setStatus(e.currentTarget.value)}><option value="">{t('admin-diag-all')}</option><For each={['pass','fail','skipped']}>{value=><option value={value}>{value}</option>}</For></select></label></div>
      <div class={css.tableWrap}><table class={css.table}>
        <caption>{t('admin-diag-checks')}</caption>
        <thead><tr><th scope="col">{t('admin-diag-check')}</th><th scope="col">{t('admin-diag-status')}</th><th scope="col">{t('admin-diag-duration')}</th></tr></thead>
        <tbody><For each={filtered()} fallback={<tr><td colSpan={3}>{t(current().results.length ? 'admin-diag-no-match' : 'admin-diag-no-checks')}</td></tr>}>{(check) => <tr><th scope="row">{check.summary}<Show when={check.domain}><div><a href={`#/domains/${encodeURIComponent(check.domain!)}`}>{check.domain}</a> · {check.backendId} · {check.protocol}</div></Show></th><td>{check.status}</td><td>{check.durationMs} ms</td></tr>}</For></tbody>
      </table></div>
    </div>}</Show>
    <h3>{t('admin-diag-history')}</h3>
    <Show when={runs().length > 0} fallback={<p>{t('admin-diag-empty')}</p>}>
      <ul><For each={runs()}>{(entry) => <li><button type="button" class="btn" onClick={() => setSelected(entry)}>{new Date(entry.startedAt * 1000).toLocaleString()} · {entry.suite} · {entry.status}</button></li>}</For></ul>
    </Show>
    <Show when={cursor()}><button class="btn" type="button" disabled={busy()} onClick={() => void history(true)}>{t('admin-diag-more')}</button></Show>
  </section>;
}
