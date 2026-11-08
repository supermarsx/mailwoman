import {AdminSubtabPanel} from './shell/AdminSubtabPanel.tsx';
import { formatDateTime } from '../../i18n/format.ts';
import { AuditActor } from './shell/AuditActor.tsx';
import * as auditCss from './shell/audit.css.ts';
// Admin › Observability opens on live statistics. The audit viewer and telemetry
// configuration have separate tabs so neither displaces the overview's figures.
//
// Telemetry has a dedicated editor under /observability/telemetry. Its typed
// configuration is applied at server startup, with explicit environment pins.
//
import { createSignal, For, Show, onMount, useContext, lazy, type JSX } from 'solid-js';
import { ShellContext } from './shell/context.ts';
import { sectionPath } from './shell/router.ts';
import { useAdmin } from './context.ts';
import type { AuditLogEntry } from '../../state/slices/admin.ts';
import { t } from '../../i18n';
import { SectionTabs } from './shell/SectionTabs.tsx';
import { ObservabilityStats } from './ObservabilityStats.tsx';
import { InlineLoading } from '../../components/InlineLoading.tsx';
import { Feedback } from './shell/Feedback.tsx';
import { AuditRetention } from './Logs/AuditRetention.tsx';
import * as css from './admin.css.ts';

const Telemetry = lazy(() => import('./Telemetry.tsx').then(module => ({ default: module.Telemetry })));
const ClientErrors = lazy(() => import('./ClientErrors.tsx').then(module => ({ default: module.ClientErrors })));
export function Observability(): JSX.Element {
  const shell = useContext(ShellContext);
  const canStats=()=>shell?.can.can('observability.read')===true;
  const canAudit=()=>!shell || shell.can.can('audit.read');
  const selected=()=>{
    const tab=shell?.router.route().rest[0];
    // Preserve existing Statistics bookmarks while presenting one Overview.
    if((tab==='overview'||tab==='statistics')&&canStats())return 'overview';
    if(tab==='audit'&&canAudit())return 'audit';
    if(tab==='telemetry'&&canStats())return 'telemetry';
    if(tab==='client-errors'&&canStats())return 'client-errors';
    return canStats()?'overview':'audit';
  };
  const tabs=()=>[...(canStats()?[['overview','admin-obs-overview']]:[]),...(canAudit()?[['audit','admin-obs-audit']]:[]),...(canStats()?[['telemetry','admin-obs-telemetry-open']]:[]),...(canStats()?[['client-errors','admin-obs-client-errors']]:[])].map(([id,label])=>({id:id!,label:t(label!),path:sectionPath('observability',id==='overview'?[]:[id!])}));
  return <section aria-label={t('admin-obs-title')}><SectionTabs label={t('admin-obs-title')} items={tabs()} selected={selected()} navigate={path=>void shell?.router.navigate(path)}/><AdminSubtabPanel><Show when={selected()==='overview' && canStats()} fallback={<Show when={selected()==='client-errors' && canStats()} fallback={<Show when={selected()==='telemetry' && canStats()} fallback={<Show when={selected()==='audit' && canAudit()}><ObservabilityAudit/></Show>}><Telemetry/></Show>}><ClientErrors/></Show>}><ObservabilityStats title={t('admin-obs-overview')}/></Show></AdminSubtabPanel></section>;
}
function ObservabilityAudit(): JSX.Element {
  const shell = useContext(ShellContext);
  const { api } = useAdmin();
  const [audit, setAudit] = createSignal<AuditLogEntry[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  // An action's failure is a toast (t32 f39); a load failure stays on the page.
  const [failure, setFailure] = createSignal<string | null>(null);
  function fail(message: string): void {
    setFailure(null);
    setFailure(message);
  }

  async function reload(): Promise<void> {
    setLoading(true);
    try {
      setAudit(await api.listAudit(100));
      setError(null);
    } catch {
      setError(t('admin-obs-load-error'));
    } finally { setLoading(false); }
  }
  onMount(() => void reload());

  async function onExport(): Promise<void> {
    try {
      const jsonl = await api.exportAudit(1000);
      const blob = new Blob([jsonl], { type: 'application/x-ndjson' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'audit-log.jsonl';
      a.click();
      URL.revokeObjectURL(url);
    } catch {
      fail(t('admin-obs-export-error'));
    }
  }

  return (
    <section class={css.section} aria-label={t('admin-obs-audit')}>
      <h2 class={css.heading}>{t('admin-obs-audit')}</h2>
      <Show when={error()}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
      <Feedback kind="error" as="toast" message={failure()} />


      <div class={css.card}>
        <p class={css.note}>{t('admin-obs-telemetry-note')}</p>
        <Show when={shell?.can.can('observability.read')}><button class="btn" onClick={()=>void shell?.router.navigate(sectionPath('observability',['telemetry']))}>{t('admin-obs-telemetry-open')}</button></Show>
      </div>

      <div class={css.card}>
        <div class={css.headingRow}>
          <h3 class={css.heading}>{t('admin-obs-audit')}</h3>
          <button type="button" class="btn btn--ghost" onClick={() => void onExport()}>
            {t('admin-obs-export')}
          </button>
        </div>
        <Show when={audit().length > 0} fallback={<Show when={loading()} fallback={<Show when={!error()}><p class={css.note}>{t('admin-obs-audit-empty')}</p></Show>}><InlineLoading /></Show>}>
          <div class={css.tableWrap}>
            <table class={`${css.table} ${auditCss.auditTable}`}>
              <thead>
                <tr>
                  <th>{t('admin-obs-col-time')}</th>
                  <th>{t('admin-obs-col-actor')}</th>
                  <th>{t('admin-obs-col-action')}</th>
                  <th>{t('admin-obs-col-target')}</th>
                </tr>
              </thead>
              <tbody>
                <For each={audit()}>
                  {(a) => (
                    <tr>
                      <td data-label={t('admin-obs-col-time')}><time dateTime={a.ts} title={a.ts}>{formatDateTime(a.ts) || a.ts}</time></td>
                      <td data-label={t('admin-obs-col-actor')}>
                        <div><AuditActor {...a} /><span class={css.badge}>{a.actorKind}</span></div>
                      </td>
                      <td data-label={t('admin-obs-col-action')} dir="auto">{a.action}</td>
                      <td data-label={t('admin-obs-col-target')} dir="auto"><span class={auditCss.identifier} title={a.target ?? undefined}>{a.target ?? '—'}</span></td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </Show>
      </div>

      <div class={css.card}>
        <AuditRetention onChanged={() => void reload()} />
      </div>

    </section>
  );
}
