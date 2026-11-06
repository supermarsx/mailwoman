// Admin > Compliance > Consents: per purpose, how many people have a current yes
// or a withdrawal, the withdrawal rate, a daily graph of grants and withdrawals,
// the consent.* settings (legal basis, text version, own wording) and a CSV
// export of the ledger. These are configurable controls; whether a configuration
// meets a given law is for the operator to decide.

import { For, Show, createResource, createSignal, onMount, type JSX } from 'solid-js';
import { formatDateTime, formatNumber, formatPercent, loadCatalog, t } from '../../../i18n';
import { apiFetch } from '../../../api/http.ts';
import { adminUrl } from '../../../api/adminPath.ts';
import { basePath } from '../../../api/basePath.ts';
import { listAdminSettings, type AdminSetting } from '../../../api/admin/settings.ts';
import { adminRequest } from '../shell/adminApi.ts';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { InlineSettingEditor } from '../Settings/InlineSettingEditor.tsx';
import { BarChart } from '../shell/BarChart.tsx';
import { ChartCard, StatsSection } from '../shell/ChartCard.tsx';
import { LineChart } from '../shell/LineChart.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import * as css from '../admin.css.ts';

const DAY = 86_400;
const RANGES = [7, 30, 90] as const;

export interface PurposeStat { purpose: string; granted: number; withdrawn: number; withdrawalRate: number; grantEvents: number; withdrawEvents: number }
export interface ConsentOverview { from: number; to: number; total: number; purposes: PurposeStat[]; series: { t: number; granted: number; withdrawn: number }[] }

export interface ConsentsApi {
  stats: (from: number, to: number) => Promise<ConsentOverview>;
  exportCsv: (from: number, to: number) => Promise<string>;
  settings: () => Promise<AdminSetting[]>;
}

export const consentsApi: ConsentsApi = {
  stats: async (from, to) => (await adminRequest<ConsentOverview>(`/consents/stats?from=${from}&to=${to}`)).data,
  exportCsv: async (from, to) => {
    const response = await apiFetch(adminUrl(`/consents/ledger.csv?from=${from}&to=${to}`, basePath()), { credentials: 'same-origin', headers: { Accept: 'text/csv' } });
    if (!response.ok) throw new Error(`export failed (${response.status})`);
    return response.text();
  },
  settings: async () => { const r = await listAdminSettings(); return r.kind === 'ok' ? r.data : []; },
};

function download(text: string): void {
  const url = URL.createObjectURL(new Blob([text], { type: 'text/csv;charset=utf-8' }));
  const a = document.createElement('a');
  a.href = url; a.download = 'consent-ledger.csv'; document.body.appendChild(a); a.click(); a.remove();
  URL.revokeObjectURL(url);
}

const purposeName = (id: string): string => (id.startsWith('assistant.') ? `${t('admin-consents-purpose-assistant')} / ${id.slice(10)}` : t(`admin-consents-purpose-${id}`));

export default function Consents(props: { api?: ConsentsApi }): JSX.Element {
  const api = props.api ?? consentsApi;
  onMount(() => void loadCatalog('admin-consents'));
  const [days, setDays] = createSignal<(typeof RANGES)[number]>(30);
  const range = (): [number, number] => { const to = Math.floor(Date.now() / 1000) + 1; return [to - days() * DAY, to]; };
  const [stats, { refetch }] = createResource(days, () => api.stats(...range()));
  const [rows] = createResource(() => api.settings().catch(() => [] as AdminSetting[]));
  const [failure, setFailure] = createSignal<unknown>(null);
  const settingRows = (): AdminSetting[] => (rows.error ? [] : (rows.latest ?? []).filter((r) => r.key.startsWith('consent.')));
  const day = (secs: number): string => formatDateTime(secs * 1000, { dateStyle: 'medium' } as Intl.DateTimeFormatOptions);

  return (
    <section class={css.section} data-testid="admin-consents">
      <h2 class={css.heading}>{t('admin-consents-title')}</h2>
      <p class={css.note}>{t('admin-consents-intro')}</p>
      <Show when={stats.error}><p role="alert">{describeAdminError(stats.error)}</p></Show>
      <AdminSubsection title={t('admin-consents-overview')}
        actions={<>
          <label>{t('admin-consents-range')} <select value={String(days())} onChange={(e) => setDays(Number(e.currentTarget.value) as (typeof RANGES)[number])}>
            <For each={RANGES}>{(d) => <option value={String(d)}>{t('admin-consents-range-days', { days: d })}</option>}</For></select></label>{' '}
          <button type="button" class="btn" onClick={() => void refetch()}>{t('admin-consents-refresh')}</button>{' '}
          <button type="button" class="btn" data-testid="consent-export" onClick={() => { setFailure(null); void api.exportCsv(...range()).then(download).catch(setFailure); }}>{t('admin-consents-export')}</button>
        </>}>
        <Show when={failure()}><p role="alert">{describeAdminError(failure())}</p></Show>
        <Show when={stats.latest}>{(s) => (
          <>
            <table class={css.table} data-testid="consent-table">
              <thead><tr>
                <th scope="col">{t('admin-consents-col-purpose')}</th><th scope="col">{t('admin-consents-col-granted')}</th>
                <th scope="col">{t('admin-consents-col-withdrawn')}</th><th scope="col">{t('admin-consents-col-rate')}</th>
              </tr></thead>
              <tbody><For each={s().purposes} fallback={<tr><td colspan="4">{t('admin-consents-none')}</td></tr>}>{(p) => (
                <tr data-testid={`consent-row-${p.purpose}`}>
                  <th scope="row">{purposeName(p.purpose)}</th>
                  <td>{formatNumber(p.granted)}</td><td>{formatNumber(p.withdrawn)}</td><td>{formatPercent(p.withdrawalRate)}</td>
                </tr>
              )}</For></tbody>
            </table>
            <StatsSection title={t('admin-consents-charts')} kind="charts">
              <BarChart testId="consent-chart-rate" title={t('admin-consents-chart-rate')} bars={s().purposes.map((p) => ({ label: purposeName(p.purpose), value: Math.round(p.withdrawalRate * 100) }))} format={(n) => `${n} %`} emptyText={t('admin-consents-none')} />
              <ChartCard
                testId="consent-chart-daily"
                title={t('admin-consents-chart-daily')}
                unit={t('admin-consents-chart-daily-unit')}
                legend={[{ label: t('admin-consents-col-granted'), slot: 1 }, { label: t('admin-consents-col-withdrawn'), slot: 2 }]}
                summary={<>{t('admin-consents-chart-daily-summary', { granted: s().series.reduce((a, p) => a + p.granted, 0), withdrawn: s().series.reduce((a, p) => a + p.withdrawn, 0) })}</>}
                table={{ columns: [t('admin-consents-col-day'), t('admin-consents-col-granted'), t('admin-consents-col-withdrawn')], numeric: [false, true, true], rows: s().series.map((p) => [day(p.t), String(p.granted), String(p.withdrawn)]) }}
                empty={s().total === 0 ? t('admin-consents-none') : undefined}
              >
                <LineChart times={s().series.map((p) => p.t)} series={[{ label: t('admin-consents-col-granted'), slot: 1, values: s().series.map((p) => p.granted) }, { label: t('admin-consents-col-withdrawn'), slot: 2, values: s().series.map((p) => p.withdrawn) }]} format={(n) => formatNumber(n, { maximumFractionDigits: 0 })} formatTime={day} whole />
              </ChartCard>
            </StatsSection>
          </>
        )}</Show>
      </AdminSubsection>
      <AdminSubsection title={t('admin-consents-settings')} description={t('admin-consents-settings-help')}>
        <For each={settingRows()}>{(row) => <InlineSettingEditor settingKey={row.key} setting={row} noScoped />}</For>
      </AdminSubsection>
    </section>
  );
}
