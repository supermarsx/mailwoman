// Request protection > Overview: a compact Admin probe summary (state, blocks,
// refused requests, last seen) with an "Edit policies" link to the Admin probe
// subtab, where every policy and the full counters live.

import { createSignal, onMount, Show, useContext, type JSX } from 'solid-js';
import { adminProbeApi, type AdminProbeApi, type AdminProbeOverview } from '../../api/admin/adminProbe.ts';
import { formatDateTime, formatNumber, formatRelative, t } from '../../i18n';
import { ShellContext } from './shell/context.ts';
import { sectionPath } from './shell/router.ts';
import { ProbeSummary } from './AdminProbePanel.tsx';
import * as pg from './protocolGuard.css.ts';
import * as admin from './admin.css.ts';

export function AdminProbeCard(props: { api?: AdminProbeApi | undefined; onOpenEditor?: (() => void) | undefined }): JSX.Element {
  const shell = useContext(ShellContext);
  const [data, setData] = createSignal<AdminProbeOverview>();
  const [failed, setFailed] = createSignal(false);
  onMount(() => {
    void (async () => {
      try { setData((await (props.api ?? adminProbeApi).overview()).data); } catch { setFailed(true); }
    })();
  });
  const open = (): void => (props.onOpenEditor ? props.onOpenEditor() : void shell?.router.navigate(sectionPath('security', ['protection', 'admin-probe'])));
  const last = (d: AdminProbeOverview): string | null => d.topOffenders.map((o) => o.lastSeen).filter((x): x is string => !!x).sort().pop() ?? null;
  const tile = (value: string, label: string): JSX.Element => <div class={pg.tile}><span class={pg.tileValue}>{value}</span><span class={pg.tileLabel}>{label}</span></div>;
  return (
    <section class={admin.card} aria-labelledby="admin-probe-h" data-testid="admin-probe-summary-card">
      <h3 id="admin-probe-h" class={admin.heading}>{t('admin-probe-title')}</h3>
      <Show when={data()} fallback={<p class={admin.note}>{failed() ? t('admin-probe-summary-unavailable') : t('admin-probe-summary-loading')}</p>}>
        {(d) => <>
          <ProbeSummary policy={d().policy} />
          <div class={pg.tiles}>
            {tile(formatNumber(d().stats.activeBlocks), t('admin-probe-stat-active'))}
            {tile(formatNumber(d().stats.refusedRequests), t('admin-probe-stat-refused'))}
            {tile(formatNumber(d().stats.blocksLast24h), t('admin-probe-stat-24h'))}
          </div>
          <p class={admin.note}>
            {t('admin-probe-last-seen')}{' '}
            <Show when={last(d())} fallback={t('admin-protocol-never')}>{(iso) => <time datetime={iso()} title={formatDateTime(iso())}>{formatRelative(iso())}</time>}</Show>
          </p>
          <button class="btn btn--ghost" onClick={open}>{t('admin-probe-edit-policies')}</button>
        </>}
      </Show>
    </section>
  );
}
