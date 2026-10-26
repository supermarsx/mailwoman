import { For, Show, createSignal, onMount, useContext, type JSX } from 'solid-js';
import { analyticsApi, type AnalyticsApi, type AnalyticsConfig, type AnalyticsView } from '../../api/admin/analytics.ts';
import { loadCatalog, t } from '../../i18n';
import { AsyncPending } from '../../components/AsyncState.tsx';
import { AdminSubsection } from './AdminSubsection.tsx';
import { ShellContext } from './shell/context.ts';
import { useDirtyGuard } from './shell/useDirtyGuard.ts';
import { describeAdminError } from './shell/Feedback.tsx';
import * as css from './admin.css.ts';

export function Analytics(props:{api?:AnalyticsApi}):JSX.Element {
  const api=()=>props.api??analyticsApi;
  const shell=useContext(ShellContext);
  const writable=()=>shell?.can.can('integrations.write')===true;
  const egress=()=>shell?.can.can('egress.write')===true;
  const [view,setView]=createSignal<AnalyticsView>();
  const [config,setConfig]=createSignal<AnalyticsConfig>();
  const [etag,setEtag]=createSignal('');
  const [baseline,setBaseline]=createSignal('');
  const [busy,setBusy]=createSignal(false);
  const [error,setError]=createSignal('');
  const [saved,setSaved]=createSignal(false);
  const [destination,setDestination]=createSignal('');
  const dirty=()=>!!config()&&JSON.stringify(config())!==baseline();
  useDirtyGuard(dirty);
  function accept(result:{data:AnalyticsView;etag:string}):void {setView(result.data);setConfig(result.data.config);setBaseline(JSON.stringify(result.data.config));setEtag(result.etag);}
  async function reload():Promise<void> {setBusy(true);setError('');try{accept(await api().read());}catch(e){setError(describeAdminError(e));}finally{setBusy(false);}}
  async function save(event:SubmitEvent):Promise<void> {event.preventDefault();const value=config();if(!value||busy())return;setBusy(true);setError('');setSaved(false);try{accept(await api().save(value,etag()));setSaved(true);}catch(e){setError(describeAdminError(e));}finally{setBusy(false);}}
  async function permission(action:()=>Promise<void>):Promise<void> {setBusy(true);setError('');try{await action();setDestination('');const next=await api().read();setView(next.data);}catch(e){setError(describeAdminError(e));}finally{setBusy(false);}}
  const change=(patch:Partial<AnalyticsConfig>):void=>{setSaved(false);setConfig(value=>value&&({...value,...patch}));};
  onMount(()=>{void loadCatalog('admin-analytics');void reload();});
  return <div class={css.subsectionBody}><h2>{t('admin-analytics-title')}</h2><p>{t('admin-analytics-help')}</p>
    <Show when={error()}><p role="alert">{error()}</p></Show><Show when={saved()}><p role="status">{t('admin-analytics-saved')}</p></Show>
    <Show when={config()} fallback={<Show when={busy()} fallback={<button class="btn" onClick={()=>void reload()}>{t('admin-analytics-reload')}</button>}><AsyncPending /></Show>}>{value=><form class={css.subsectionBody} onSubmit={event=>void save(event)}>
      <AdminSubsection title={t('admin-analytics-collector')}>
        <label><input type="checkbox" checked={value().enabled} disabled={!writable()||busy()} onChange={event=>change({enabled:event.currentTarget.checked})}/> {t('admin-analytics-enabled')}</label>
        <label class="field"><span>{t('admin-analytics-provider')}</span><select value={value().provider} disabled={!writable()||busy()} onChange={event=>change({provider:event.currentTarget.value as AnalyticsConfig['provider']})}><option value="umami">Umami</option><option value="matomo">Matomo</option></select></label>
        <label class="field"><span>{t('admin-analytics-endpoint')}</span><input type="url" maxlength={2048} required={value().enabled} value={value().endpoint} disabled={!writable()||busy()} onInput={event=>change({endpoint:event.currentTarget.value})}/><small>{t('admin-analytics-endpoint-help')}</small></label>
        <label class="field"><span>{t('admin-analytics-site')}</span><input maxlength={64} required={value().enabled} value={value().siteId} disabled={!writable()||busy()} onInput={event=>change({siteId:event.currentTarget.value})}/><small>{t('admin-analytics-site-help')}</small></label>
        <label><input type="checkbox" checked={value().allowHttp} disabled={!writable()||busy()} onChange={event=>change({allowHttp:event.currentTarget.checked})}/> {t('admin-analytics-http')}</label>
        <div><button class="btn btn--primary" type="submit" disabled={!writable()||busy()||!dirty()}>{t('admin-analytics-save')}</button> <button class="btn" type="button" disabled={busy()} onClick={()=>void reload()}>{t('admin-analytics-reload')}</button></div>
      </AdminSubsection>
    </form>}</Show>
    <Show when={view()}>{value=><AdminSubsection title={t('admin-analytics-counts')} description={t('admin-analytics-counts-help')}><dl><dt>{t('admin-analytics-queued')}</dt><dd>{value().status.queued}</dd><dt>{t('admin-analytics-delivered')}</dt><dd>{value().status.delivered}</dd><dt>{t('admin-analytics-dropped')}</dt><dd>{value().status.dropped}</dd></dl></AdminSubsection>}</Show>
    <Show when={egress()}><AdminSubsection title={t('admin-analytics-private')} description={t('admin-analytics-private-help')}><For each={view()?.allowlist??[]}>{row=><div>{row.destination} <button class="btn" disabled={busy()} onClick={()=>void permission(()=>api().remove(row.id))}>{t('admin-analytics-remove')}</button></div>}</For><form onSubmit={event=>{event.preventDefault();void permission(()=>api().allow(destination()));}}><label class="field"><span>{t('admin-analytics-destination')}</span><input required maxlength={255} value={destination()} disabled={busy()} onInput={event=>setDestination(event.currentTarget.value)}/></label><button class="btn" disabled={busy()||!destination().trim()}>{t('admin-analytics-allow')}</button></form></AdminSubsection></Show>
  </div>;
}
