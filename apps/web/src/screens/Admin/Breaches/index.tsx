// Admin > Compliance > Breach register: personal-data breach incidents with the
// notice clock, risk questionnaire, authority notice and data subject
// communication, automations and playbooks. These are record-keeping and
// workflow controls; whether an incident must be notified is for the operator
// to decide.
import { For, Show, createResource, createSignal, onCleanup, onMount, type JSX } from 'solid-js';
import { formatDateTime, formatNumber, loadCatalog, t } from '../../../i18n';
import {
  breachMeta, createBreach, listBreaches, registerFile, runBreachPass,
  type BreachFilter, type BreachList, type BreachMeta, type BreachSummary,
} from '../../../api/admin/breaches.ts';
import { listAdminSettings, type AdminSetting } from '../../../api/admin/settings.ts';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { InlineSettingEditor } from '../Settings/InlineSettingEditor.tsx';
import { BarChart } from '../shell/BarChart.tsx';
import { ChartCard, StatsSection } from '../shell/ChartCard.tsx';
import { LineChart } from '../shell/LineChart.tsx';
import { StatTile } from '../shell/StatTile.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import * as css from '../admin.css.ts';
import { Countdown, Detail } from './Detail.tsx';
import { SETTING_GROUPS, saveFile, splitList, urgent, yearFileName } from './model.ts';

export interface BreachApi {
  list: (f: BreachFilter) => Promise<BreachList>;
  meta: () => Promise<BreachMeta>;
  settings: () => Promise<AdminSetting[]>;
}

const liveApi: BreachApi = {
  list: (f) => listBreaches(f),
  meta: breachMeta,
  settings: async () => {
    const r = await listAdminSettings();
    return r.kind === 'ok' ? r.data : [];
  },
};

const STATUS_FILTERS = ['open', 'draft', 'detected', 'assessing', 'contained', 'notified-authority', 'notified-subjects', 'closed', 'not-notifiable'] as const;

export default function Breaches(props: { api?: BreachApi }): JSX.Element {
  const api = props.api ?? liveApi;
  onMount(() => void loadCatalog('admin-breaches'));
  const [now, setNow] = createSignal(Math.floor(Date.now() / 1000));
  const tick = setInterval(() => setNow(Math.floor(Date.now() / 1000)), 30_000);
  onCleanup(() => clearInterval(tick));
  const [filter, setFilter] = createSignal<BreachFilter>({ status: 'open' });
  const [list, { refetch }] = createResource(filter, (f) => api.list(f));
  const [meta] = createResource(() => api.meta().catch(() => undefined));
  const [rows] = createResource(() => api.settings().catch(() => [] as AdminSetting[]));
  const [selected, setSelected] = createSignal<string | null>(null);
  const [error, setError] = createSignal('');
  const [title, setTitle] = createSignal('');
  const [description, setDescription] = createSignal('');
  const [type, setType] = createSignal('other');
  const [draft, setDraft] = createSignal(false);
  const [affected, setAffected] = createSignal('');
  const [year, setYear] = createSignal(String(new Date().getFullYear()));
  const patch = (p: Partial<BreachFilter>): void => { const next: BreachFilter = { ...filter(), ...p }; setFilter(() => next); };
  const settingRows = (keys: readonly string[]): AdminSetting[] => {
    const all = rows.error ? [] : rows.latest ?? [];
    return keys.flatMap((k) => all.filter((r) => r.key === k));
  };
  async function create(): Promise<void> {
    setError('');
    try {
      const made = await createBreach({ title: title().trim(), description: description(), incidentType: type(), draft: draft(), affected: splitList(affected()) });
      setTitle(''); setDescription(''); setAffected('');
      setSelected(made.id);
      await refetch();
    } catch (e) { setError(describeAdminError(e)); }
  }
  async function checkNow(): Promise<void> {
    setError('');
    try { await runBreachPass(); await refetch(); } catch (e) { setError(describeAdminError(e)); }
  }
  async function exportRegister(format: 'csv' | 'json' | 'html'): Promise<void> {
    setError('');
    try {
      const text = await registerFile(year(), format);
      saveFile(yearFileName(year(), format), text, format === 'csv' ? 'text/csv' : format === 'json' ? 'application/json' : 'text/html');
    } catch (e) { setError(describeAdminError(e)); }
  }
  const counts = (l: BreachList): { overdue: number; soon: number } => urgent(l.incidents, now());
  const hit = (r: BreachSummary): string => `breach-row-${r.id}`;
  return (
    <section class={css.section} aria-labelledby="admin-breach-heading" data-testid="admin-breaches">
      <h2 id="admin-breach-heading" class={css.heading}>{t('admin-breach-title')}</h2>
      <p class={css.note}>{t('admin-breach-intro')}</p>
      <Show when={list.error}><p role="alert">{describeAdminError(list.error)}</p></Show>
      <Show when={list()}>
        {(l) => (
          <>
            <Show when={counts(l()).overdue > 0}>
              <p role="alert" data-testid="breach-overdue-banner">{t('admin-breach-banner-overdue', { count: counts(l()).overdue })}</p>
            </Show>
            <StatsSection title={t('admin-breach-stats')} kind="tiles" testId="breach-stats">
              <StatTile label={t('admin-breach-tile-open')} value={l().stats.open} testId="breach-tile-open" />
              <StatTile label={t('admin-breach-tile-drafts')} value={l().stats.drafts} testId="breach-tile-drafts" />
              <StatTile label={t('admin-breach-tile-overdue')} value={l().stats.overdue} testId="breach-tile-overdue" />
              <StatTile label={t('admin-breach-tile-soon')} value={l().stats.dueSoon} testId="breach-tile-soon" />
              <StatTile label={t('admin-breach-tile-late')} value={l().stats.notifiedLate} testId="breach-tile-late" />
            </StatsSection>
            <StatsSection title={t('admin-breach-charts')} kind="charts">
              <ChartCard testId="breach-chart-created" title={t('admin-breach-chart-created')} unit={t('admin-breach-chart-unit')}
                summary={<>{t('admin-breach-chart-summary', { total: l().stats.created30d.reduce((n, p) => n + p.count, 0) })}</>}
                table={{ columns: [t('admin-breach-col-day'), t('admin-breach-col-count')], numeric: [false, true], rows: l().stats.created30d.map((p) => [formatDateTime(p.t * 1000, { dateStyle: 'short' }), String(p.count)]) }}
                empty={l().stats.created30d.every((p) => p.count === 0) ? t('admin-breach-chart-empty') : undefined}>
                <LineChart times={l().stats.created30d.map((p) => p.t)} series={[{ label: t('admin-breach-chart-created'), slot: 1, values: l().stats.created30d.map((p) => p.count) }]} format={(n) => formatNumber(n)} formatTime={(s) => formatDateTime(s * 1000, { dateStyle: 'short' })} whole />
              </ChartCard>
              <BarChart testId="breach-chart-status" title={t('admin-breach-chart-status')} bars={Object.entries(l().stats.byStatus).map(([k, v]) => ({ label: t(`admin-breach-status-${k}`), value: v }))} emptyText={t('admin-breach-chart-empty')} />
            </StatsSection>
          </>
        )}
      </Show>

      <div class={css.grid} role="search" aria-label={t('admin-breach-filters')}>
        <label>{t('admin-breach-col-status')}
          <select value={filter().status ?? ''} onChange={(e) => patch({ status: e.currentTarget.value })} data-testid="breach-filter-status">
            <option value="">{t('admin-breach-any')}</option>
            <For each={STATUS_FILTERS}>{(s) => <option value={s}>{s === 'open' ? t('admin-breach-open') : s === 'draft' ? t('admin-breach-draft') : t(`admin-breach-status-${s}`)}</option>}</For>
          </select>
        </label>
        <label>{t('admin-breach-col-risk')}
          <select value={filter().severity ?? ''} onChange={(e) => patch({ severity: e.currentTarget.value })}>
            <option value="">{t('admin-breach-any')}</option>
            <For each={['unassessed', 'none', 'risk', 'high']}>{(s) => <option value={s}>{t(`admin-breach-sev-${s}`)}</option>}</For>
          </select>
        </label>
        <label>{t('admin-breach-col-due')}
          <select value={filter().due ?? ''} onChange={(e) => patch({ due: e.currentTarget.value })}>
            <option value="">{t('admin-breach-any')}</option><option value="overdue">{t('admin-breach-due-overdue')}</option><option value="soon">{t('admin-breach-due-soon')}</option>
          </select>
        </label>
        <label>{t('admin-breach-search')}<input type="search" value={filter().q ?? ''} onChange={(e) => patch({ q: e.currentTarget.value })} /></label>
        <button type="button" class="btn btn--ghost" onClick={() => void checkNow()} data-testid="breach-check-now">{t('admin-breach-check-now')}</button>
      </div>

      <Show when={list()?.incidents.length} fallback={<p class={css.note} role="status">{list.loading ? t('admin-breach-loading') : t('admin-breach-empty')}</p>}>
        <div class={css.tableWrap}>
          <table class={css.table} data-testid="breach-table">
            <caption>{t('admin-breach-caption', { shown: list()?.incidents.length ?? 0, total: list()?.matched ?? 0 })}</caption>
            <thead><tr>
              <th scope="col">{t('admin-breach-col-ref')}</th><th scope="col">{t('admin-breach-col-title')}</th><th scope="col">{t('admin-breach-col-status')}</th>
              <th scope="col">{t('admin-breach-col-risk')}</th><th scope="col">{t('admin-breach-col-clock')}</th><th scope="col">{t('admin-breach-assignee')}</th>
              <th scope="col"><span class="sr-only">{t('admin-breach-col-actions')}</span></th>
            </tr></thead>
            <tbody>
              <For each={list()?.incidents ?? []}>{(r) => (
                <tr data-testid={hit(r)}>
                  <td>{r.reference}</td><td>{r.title}</td>
                  <td>{t(`admin-breach-status-${r.status}`)}<Show when={r.draft}> <span class={css.badge}>{t('admin-breach-draft')}</span></Show></td>
                  <td>{t(`admin-breach-sev-${r.severity}`)}</td>
                  <td><Countdown clock={r.clock} now={now()} /></td>
                  <td>{r.assignee ?? t('admin-breach-unassigned')}</td>
                  <td><button type="button" class="btn btn--ghost" onClick={() => setSelected(r.id)} data-testid={`breach-open-${r.id}`}>{t('admin-breach-open-incident')}</button></td>
                </tr>
              )}</For>
            </tbody>
          </table>
        </div>
      </Show>
      <Show when={selected()}>{(id) => <Detail id={id()} meta={meta()} now={now} onChange={() => void refetch()} onClose={() => setSelected(null)} />}</Show>
      <Show when={error()}><p role="alert" class={css.error}>{error()}</p></Show>

      <AdminSubsection title={t('admin-breach-h-new')} description={t('admin-breach-new-note')}>
        <label>{t('admin-breach-col-title')}<input value={title()} onInput={(e) => setTitle(e.currentTarget.value)} data-testid="breach-new-title" /></label>
        <label>{t('admin-breach-description')}<textarea rows="2" value={description()} onInput={(e) => setDescription(e.currentTarget.value)} /></label>
        <label>{t('admin-breach-type')}
          <select value={type()} onChange={(e) => setType(e.currentTarget.value)}>
            <For each={meta()?.types ?? ['other']}>{(k) => <option value={k}>{t(`admin-breach-type-${k}`)}</option>}</For>
          </select>
        </label>
        <label>{t('admin-breach-affected')}<textarea rows="2" value={affected()} onInput={(e) => setAffected(e.currentTarget.value)} /></label>
        <label><input type="checkbox" checked={draft()} onChange={(e) => setDraft(e.currentTarget.checked)} data-testid="breach-new-draft" /> {t('admin-breach-new-draft')}</label>
        <button type="button" class="btn" disabled={!title().trim()} onClick={() => void create()} data-testid="breach-new-submit">{t('admin-breach-new-submit')}</button>
      </AdminSubsection>

      <AdminSubsection title={t('admin-breach-h-export')} description={t('admin-breach-export-note')}>
        <label>{t('admin-breach-year')}<input type="number" min="2000" max="2200" value={year()} onInput={(e) => setYear(e.currentTarget.value)} data-testid="breach-year" /></label>
        <div class={css.headingRow}>
          <button type="button" class="btn" onClick={() => void exportRegister('csv')} data-testid="breach-export-csv">{t('admin-breach-fmt-csv')}</button>
          <button type="button" class="btn" onClick={() => void exportRegister('json')} data-testid="breach-export-json">{t('admin-breach-fmt-json')}</button>
          <button type="button" class="btn" onClick={() => void exportRegister('html')} data-testid="breach-export-html">{t('admin-breach-fmt-html')}</button>
        </div>
      </AdminSubsection>

      <For each={SETTING_GROUPS}>{(group) => (
        <AdminSubsection title={t(`admin-breach-group-${group.id}`)} description={t(`admin-breach-group-${group.id}-note`)}>
          <For each={settingRows(group.keys)}>{(row) => <InlineSettingEditor settingKey={row.key} setting={row} />}</For>
        </AdminSubsection>
      )}</For>
    </section>
  );
}
