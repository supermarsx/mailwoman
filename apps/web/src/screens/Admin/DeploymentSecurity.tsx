import { createSignal, onMount, Show, For, useContext, type JSX } from 'solid-js';
import { adminRequest } from './shell/adminApi.ts';
import { describeAdminError } from './shell/Feedback.tsx';
import { DataTable } from './shell/DataTable.tsx';
import { t } from '../../i18n';
import { ShellContext } from './shell/context.ts';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';

export interface DeploymentCheck { id: string; status: 'pass'|'warning'|'info'; value: unknown; settings: string[] }
export interface DeploymentAssessment { generatedAt: string; scope: string; checks: DeploymentCheck[]; limitations: string[] }
export function DeploymentSecurity(props: { load?: () => Promise<DeploymentAssessment> }): JSX.Element {
  const shell=useContext(ShellContext);
  const [report,setReport]=createSignal<DeploymentAssessment>();
  const [busy,setBusy]=createSignal(true);const [error,setError]=createSignal('');
  const [query,setQuery]=createSignal('');const [status,setStatus]=createSignal('all');
  async function reload():Promise<void>{setBusy(true);try{setReport(await(props.load??(async()=>(await adminRequest<DeploymentAssessment>('/security/deployment')).data))());setError('');}catch(e){setError(describeAdminError(e));}finally{setBusy(false);}}
  onMount(()=>void reload());
  const title=(id:string):string=>t(`admin-deployment-check-${id}`);
  const rows=():DeploymentCheck[] => report()?.checks.filter(row=>(status()==='all'||row.status===status())&&title(row.id).toLowerCase().includes(query().toLowerCase()))??[];
  const details=(row:DeploymentCheck):string=>{
    const v=row.value as Record<string, string|number|boolean>;
    if(typeof row.value==='boolean')return t(row.value?'admin-deployment-yes':'admin-deployment-no');
    if(typeof row.value==='number')return String(row.value);
    return t(`admin-deployment-value-${row.id}`,Object.fromEntries(Object.entries(v).map(([key,value])=>[key,typeof value==='boolean'?t(value?'admin-deployment-yes':'admin-deployment-no'):value])));
  };
  return <section class={css.section} aria-label={t('admin-deployment-title')}>
    <div class={ux.pageHead}><h2 class={css.heading}>{t('admin-deployment-title')}</h2><div class={ux.pageActions}><a class="btn btn--ghost" href="#/diagnostics">{t('admin-deployment-back')}</a><button class="btn" disabled={busy()} onClick={()=>void reload()}>{t('admin-deployment-refresh')}</button></div></div>
    <p class={css.note}>{t('admin-deployment-help')}</p>
    <div class={css.card}><div class={css.grid}>
      <label class="field">{t('admin-deployment-search')}<input type="search" value={query()} onInput={e=>setQuery(e.currentTarget.value)}/></label>
      <label class="field">{t('admin-deployment-status')}<select value={status()} onChange={e=>setStatus(e.currentTarget.value)}><For each={['all','pass','warning','info']}>{value=><option value={value}>{t(`admin-deployment-status-${value}`)}</option>}</For></select></label>
    </div>
    <DataTable caption={t('admin-deployment-title')} rows={rows()} rowKey={r=>r.id} state={busy()?'loading':error()?'error':'ready'} errorText={error()} onRetry={()=>void reload()} emptyText={t('admin-deployment-empty')} columns={[
      {id:'check',header:t('admin-deployment-check'),cell:r=>title(r.id)},
      {id:'status',header:t('admin-deployment-status'),cell:r=>t(`admin-deployment-status-${r.status}`)},
      {id:'value',header:t('admin-deployment-observed'),cell:details},
      {id:'settings',header:t('admin-deployment-settings'),cell:r=><Show when={shell?.can.can('settings.read')}><For each={r.settings}>{key=><p><a href={`#/settings/${encodeURIComponent(key)}`}>{key}</a></p>}</For></Show>},
    ]}/></div>
    <Show when={report()}>{r=><section class={css.card}><h3>{t('admin-deployment-limits')}</h3><p>{t('admin-deployment-snapshot',{time:r().generatedAt})}</p><ul><For each={r().limitations}>{id=><li>{t(`admin-deployment-limit-${id}`)}</li>}</For></ul></section>}</Show>
  </section>;
}
