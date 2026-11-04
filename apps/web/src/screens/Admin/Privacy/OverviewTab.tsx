import { Show, type JSX } from 'solid-js';
import { formatBytes, formatNumber, t } from '../../../i18n';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import { StatTile } from '../shell/StatTile.tsx';
import { StatsSection } from '../shell/ChartCard.tsx';
import { SourceBadge } from '../shell/SourceBadge.tsx';
import { sourceOf } from '../Settings/SettingDetail.tsx';
import { displayValue } from '../Settings/value.ts';
import { settingLabel } from '../Settings/labels.ts';
import * as css from './privacy.css.ts';
import { REMOTE_KEYS, type PrivacyTabContext } from './model.ts';
import { HistoryControls, ProxyCharts } from './ProxyCharts.tsx';

const last = (values: readonly (number | null)[] | undefined): number | null => {
  for (let i = (values?.length ?? 0) - 1; i >= 0; i--) { const v = values?.[i]; if (typeof v === 'number') return v; }
  return null;
};

export default function OverviewTab(props: { ctx: PrivacyTabContext; open: (tab: string) => void }): JSX.Element {
  const ctx = (): PrivacyTabContext => props.ctx;
  const stats = () => (ctx().statsError() ? undefined : ctx().stats());
  const series = (name: string) => last(stats()?.history?.series[name]);
  return (
    <>
      <Show when={ctx().statsError()}><p role="alert">{describeAdminError(ctx().statsError())}</p></Show>
      <AdminSubsection title={t('privacy-overview-status')} description={t('privacy-admin-help')}>
        <p role="status" data-testid="privacy-status">{ctx().statsError() ? t('privacy-status-unavailable') : stats() ? t('privacy-status-ok') : t('admin-shell-loading')}</p>
        <Show when={ctx().canReadSettings()}>
          <ul data-testid="privacy-key-settings">
            {REMOTE_KEYS.slice(0, 4).flatMap((key) => ctx().rows().filter((row) => row.key === key)).map((row) => (
              <li><a href="#" onClick={(e) => { e.preventDefault(); props.open('remote'); }}>{settingLabel(row.key)}</a>: <strong>{displayValue(row.kind, row.effective.value)}</strong> <SourceBadge source={sourceOf(row)} /></li>
            ))}
          </ul>
        </Show>
      </AdminSubsection>
      <StatsSection title={t('privacy-overview-figures')} kind="tiles">
        <div class={css.tiles}>
          <StatTile label={t('privacy-cache-entries')} value={stats()?.cacheEntries ?? null} unavailableReason={t('admin-shell-not-available')} testId="tile-entries" />
          <StatTile label={t('privacy-cache-bytes')} value={stats()?.cacheBytes ?? null} format={(n) => formatBytes(n)} unavailableReason={t('admin-shell-not-available')} testId="tile-bytes" />
          <StatTile label={t('privacy-replaced-trackers')} value={stats()?.trackersReplaced ?? null} unavailableReason={t('admin-shell-not-available')} testId="tile-trackers" />
          <StatTile label={t('privacy-chart-requests')} value={series('requestsPerSec')} format={(n) => `${formatNumber(n, { maximumFractionDigits: 2 })}/s`} unavailableReason={t('privacy-history-unavailable')} testId="tile-rps" />
          <StatTile label={t('privacy-chart-hit')} value={series('ramHitPct')} format={(n) => `${formatNumber(n, { maximumFractionDigits: 1 })} %`} unavailableReason={t('privacy-history-unavailable')} testId="tile-hit" />
          <StatTile label={t('privacy-chart-queued')} value={series('queueDepth')} unavailableReason={t('privacy-history-unavailable')} testId="tile-queue" />
        </div>
      </StatsSection>
      <AdminSubsection title={t('privacy-history-title')}>
        <HistoryControls range={ctx().range()} onRange={ctx().setRange} updatedAt={ctx().updatedAt()} loading={ctx().statsLoading()} onRefresh={ctx().refresh} history={stats()?.history} />
        <ProxyCharts stats={stats()} loading={ctx().statsLoading()} error={ctx().statsError()} view="overview" />
      </AdminSubsection>
    </>
  );
}
