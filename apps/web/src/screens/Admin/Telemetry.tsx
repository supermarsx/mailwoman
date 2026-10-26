import { createSignal, For, onMount, Show, useContext, type JSX } from 'solid-js';
import { telemetryApi, type TelemetryApi, type TelemetryConfig, type TelemetryView } from '../../api/admin/telemetry.ts';
import { loadCatalog, t } from '../../i18n';
import { AsyncPending } from '../../components/AsyncState.tsx';
import { AdminSubsection } from './AdminSubsection.tsx';
import { ShellContext } from './shell/context.ts';
import { useDirtyGuard } from './shell/useDirtyGuard.ts';
import { describeAdminError } from './shell/Feedback.tsx';
import { sectionPath } from './shell/router.ts';
import * as css from './admin.css.ts';
import './telemetry.css';

const numbers = [
  ['sampleRatio',0,1,0.01], ['exportTimeoutMs',100,30000,100], ['metricsIntervalSecs',10,3600,1],
  ['spanQueueSize',128,32768,1], ['spanBatchSize',1,4096,1], ['spanExportDelayMs',100,60000,100],
] as const;
export function Telemetry(props:{api?:TelemetryApi}):JSX.Element {
  const shell=useContext(ShellContext);
  const api=()=>props.api??telemetryApi;
  const [view,setView]=createSignal<TelemetryView>();
  const [config,setConfig]=createSignal<TelemetryConfig>();
  const [baseline,setBaseline]=createSignal('');
  const [etag,setEtag]=createSignal('');
  const [busy,setBusy]=createSignal(false);
  const [error,setError]=createSignal('');
  const [saved,setSaved]=createSignal(false);
  const dirty=()=>!!config()&&JSON.stringify(config())!==baseline();
  useDirtyGuard(dirty);
  const disabled=(key:string)=>busy()||shell?.can.can('integrations.write')!==true||view()?.pinned.includes(key)===true;
  function accept(result:{data:TelemetryView;etag:string}):void {setView(result.data);setConfig(result.data.config);setBaseline(JSON.stringify(result.data.config));setEtag(result.etag);}
  async function reload():Promise<void> {setBusy(true);setError('');try{accept(await api().read());}catch(e){setError(describeAdminError(e));}finally{setBusy(false);}}
  async function save(event:SubmitEvent):Promise<void> {event.preventDefault();const value=config();if(!value||busy())return;setBusy(true);setSaved(false);setError('');try{accept(await api().save(value,etag()));setSaved(true);}catch(e){setError(describeAdminError(e));}finally{setBusy(false);}}
  function change(patch:Partial<TelemetryConfig>):void {setSaved(false);setConfig(value=>value&&({...value,...patch}));}
  onMount(()=>{void loadCatalog('admin-telemetry');void reload();});
  return <section class={css.subsectionBody} aria-label={t('admin-telemetry-title')}>
    <h2>{t('admin-telemetry-title')}</h2><p>{t('admin-telemetry-help')}</p>
    <button class="btn btn--ghost" onClick={()=>void shell?.router.navigate(sectionPath('observability'))}>{t('admin-telemetry-back')}</button>
    <Show when={error()}><p role="alert">{error()}</p></Show><Show when={saved()}><p role="status">{t('admin-telemetry-saved')}</p></Show>
    <Show when={view()?.restartRequired}><p role="status">{t('admin-telemetry-restart')}</p></Show>
    <Show when={config()} fallback={<Show when={busy()} fallback={<button class="btn" onClick={()=>void reload()}>{t('admin-telemetry-reload')}</button>}><AsyncPending/></Show>}>{value=><form class={css.subsectionBody} onSubmit={event=>void save(event)}>
      <AdminSubsection title={t('admin-telemetry-collector')} description={t('admin-telemetry-collector-help')}>
        <For each={['enabled','traces','metrics'] as const}>{key=><label><input type="checkbox" checked={view()?.pinned.includes(key)?Boolean(view()?.effective[key]):value()[key]} disabled={disabled(key)} onChange={event=>change({[key]:event.currentTarget.checked})}/>{t(`admin-telemetry-${key}`)}</label>}</For>
        <For each={['endpoint','serviceName','environment'] as const}>{key=><label class="field"><span>{t(`admin-telemetry-${key}`)}</span><input type={key==='endpoint'?'url':'text'} maxlength={key==='endpoint'?2048:key==='serviceName'?128:64} value={view()?.pinned.includes(key)?String(view()?.effective[key]):value()[key]} disabled={disabled(key)} onInput={event=>change({[key]:event.currentTarget.value})}/><small>{t(`admin-telemetry-${key}-help`)}</small></label>}</For>
        <Show when={view()?.pinned.length}><p>{t('admin-telemetry-pinned')}</p></Show>
      </AdminSubsection>
      <AdminSubsection title={t('admin-telemetry-limits')} description={t('admin-telemetry-limits-help')}>
        <div class="telemetry-grid">
        <For each={numbers}>{([key,min,max,step])=><label class="field"><span>{t(`admin-telemetry-${key}`)}</span><input type="number" min={min} max={max} step={step} required value={value()[key]} disabled={disabled(key)} onInput={event=>change({[key]:event.currentTarget.valueAsNumber})}/><small>{t(`admin-telemetry-${key}-help`)}</small></label>}</For>
        </div>
      </AdminSubsection>
      <AdminSubsection title="Prometheus" description={t('admin-telemetry-prometheus-help')}>
        <label><input type="checkbox" checked={view()?.pinned.includes('prometheusEnabled')?view()?.effective.prometheusEnabled:value().prometheusEnabled} disabled={disabled('prometheusEnabled')} onChange={event=>change({prometheusEnabled:event.currentTarget.checked})}/>{t('admin-telemetry-prometheusEnabled')}</label>
        <Show when={!view()?.metricsTokenConfigured}><p>{t('admin-telemetry-token-missing')}</p></Show>
      </AdminSubsection>
      <div><button type="submit" class="btn btn--primary" disabled={busy()||!dirty()||shell?.can.can('integrations.write')!==true}>{t('admin-telemetry-save')}</button> <button type="button" class="btn" disabled={busy()} onClick={()=>void reload()}>{t('admin-telemetry-reload')}</button></div>
    </form>}</Show>
  </section>;
}
