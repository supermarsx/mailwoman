// Admin > Privacy > Cookies. The consent control's settings (edited in place), the
// cookie and storage inventory with the administrator's own rules, the storage the
// browsers reported that nothing classifies yet, consent statistics, a preview of
// the control and the consent log export. These are configurable controls; whether
// a configuration meets a given law is for the operator to decide.

import { For, Show, createResource, createSignal, onMount, type JSX } from 'solid-js';
import { formatDateTime, formatNumber, loadCatalog, t } from '../../../i18n';
import { InlineLoading } from '../../../components/InlineLoading.tsx';
import { InlineGroup, SettingsGate, rowsFor } from '../Privacy/InlineGroup.tsx';
import type { PrivacyTabContext } from '../Privacy/model.ts';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import { BarChart } from '../shell/BarChart.tsx';
import { ChartCard, StatsSection } from '../shell/ChartCard.tsx';
import { LineChart } from '../shell/LineChart.tsx';
import { previewConsent } from '../../../cookies/runtime.ts';
import { scanStorage, unclassified as unclassifiedOf } from '../../../cookies/enforce.ts';
import type { Category, StorageKind } from '../../../cookies/policy.ts';
import * as css from '../admin.css.ts';
import * as shell from '../shell/shell.css.ts';
import { cookiesApi, type CookiesApi } from './api.ts';
import { CATEGORY_IDS, COOKIE_GROUPS, KIND_IDS, configFromRows, share, type ConsentStats, type DetectedRow, type InventoryView } from './model.ts';

const DAY = 86_400;
const RANGES = [7, 30, 90] as const;

function categoryOptions(): JSX.Element {
  return <For each={CATEGORY_IDS}>{(c) => <option value={c}>{t(`cookies-category-${c}`)}</option>}</For>;
}

export default function CookiesTab(props: { ctx: PrivacyTabContext; api?: CookiesApi }): JSX.Element {
  const api = props.api ?? cookiesApi;
  onMount(() => { void loadCatalog('admin-cookies'); void loadCatalog('cookies'); });
  const [inventory, { refetch: reloadInventory }] = createResource(() => api.inventory());
  const [detected, { refetch: reloadDetected }] = createResource(() => api.detected());
  const [days, setDays] = createSignal<(typeof RANGES)[number]>(30);
  const [stats, { refetch: reloadStats }] = createResource(days, (d) => { const to = Math.floor(Date.now() / 1000) + 1; return api.stats(to - d * DAY, to); });
  const [message, setMessage] = createSignal<string | null>(null);
  const [failure, setFailure] = createSignal<unknown>(null);
  const run = async (work: () => Promise<unknown>, done?: string): Promise<void> => {
    setFailure(null);
    try { await work(); if (done) setMessage(done); } catch (error) { setFailure(error); }
  };
  const inv = (): InventoryView | undefined => (inventory.error ? undefined : inventory.latest);
  const list = (): DetectedRow[] => (detected.error ? [] : detected.latest ?? []);

  return (
    <SettingsGate ctx={props.ctx}>
      <p>{t('admin-cookies-intro')}</p>
      <Show when={failure()}><p role="alert">{describeAdminError(failure())}</p></Show>
      <Show when={message()}><p role="status">{message()}</p></Show>
      <For each={COOKIE_GROUPS}>{(group) => (
        <InlineGroup ctx={props.ctx} title={t(`admin-cookies-group-${group.id}`)} rows={rowsFor(props.ctx.rows(), group.keys)} />
      )}</For>

      <AdminSubsection title={t('admin-cookies-preview')} description={t('admin-cookies-preview-help')}>
        <button type="button" class="btn" onClick={() => previewConsent(configFromRows(props.ctx.rows(), inv()))}>{t('admin-cookies-preview-open')}</button>
      </AdminSubsection>

      <AdminSubsection title={t('admin-cookies-inventory')} description={t('admin-cookies-inventory-help')}>
        <Show when={inventory.error}><p role="alert">{describeAdminError(inventory.error)}</p></Show>
        <Show when={inventory.loading && !inv()}><InlineLoading /></Show>
        <Show when={inv()}>{(view) => (
          <>
            <div class={css.tableWrap} tabindex="0" role="region" aria-label={t('admin-cookies-inventory')}>
              <table class={css.table} data-testid="cookie-inventory">
                <thead><tr>
                  <th scope="col">{t('admin-cookies-col-name')}</th><th scope="col">{t('admin-cookies-col-kind')}</th><th scope="col">{t('admin-cookies-col-category')}</th>
                  <th scope="col">{t('admin-cookies-col-purpose')}</th><th scope="col">{t('admin-cookies-col-duration')}</th><th scope="col">{t('admin-cookies-col-provider')}</th><th scope="col">{t('admin-cookies-col-source')}</th>
                </tr></thead>
                <tbody>
                  <For each={view().rules}>{(rule) => (
                    <tr data-rule={rule.id}>
                      <th scope="row"><code>{rule.pattern}</code></th><td>{t(`admin-cookies-kind-${rule.kind}`)}</td>
                      <td>
                        <select aria-label={t('admin-cookies-category-of', { name: rule.pattern })} value={rule.category}
                          onChange={(e) => void run(async () => { await api.saveRule({ kind: rule.kind, pattern: rule.pattern, category: e.currentTarget.value as Category, note: rule.note }); await reloadInventory(); }, t('admin-cookies-saved'))}>
                          {categoryOptions()}
                        </select>
                      </td>
                      <td>{rule.note || t('admin-cookies-own-rule')}</td><td>-</td><td>-</td>
                      <td><button type="button" class="btn btn--ghost" onClick={() => void run(async () => { await api.deleteRule(rule.id); await reloadInventory(); }, t('admin-cookies-removed'))}>{t('admin-cookies-remove')}<span class={shell.srOnly}> {rule.pattern}</span></button></td>
                    </tr>
                  )}</For>
                  <For each={view().builtin}>{(entry) => (
                    <tr>
                      <th scope="row"><code>{entry.pattern}</code></th><td>{t(`admin-cookies-kind-${entry.kind}`)}</td><td>{t(`cookies-category-${entry.category}`)}</td>
                      <td>{entry.purpose}</td><td>{entry.duration}</td><td>{entry.provider} ({entry.party === 'first' ? t('admin-cookies-first-party') : t('admin-cookies-third-party')})</td><td>{t('admin-cookies-builtin')}</td>
                    </tr>
                  )}</For>
                </tbody>
              </table>
            </div>
            <AddRule api={api} onSaved={() => { setMessage(t('admin-cookies-saved')); void reloadInventory(); }} onError={setFailure} />
          </>
        )}</Show>
      </AdminSubsection>

      <AdminSubsection title={t('admin-cookies-detected')} description={t('admin-cookies-detected-help')}
        actions={<button type="button" class="btn" onClick={() => void run(async () => {
          const found = await scanStorage();
          const unknown = unclassifiedOf(found, { enabled: true, inventory: inv()?.builtin ?? [], rules: inv()?.rules ?? [] });
          const recorded = unknown.length ? await api.scan(unknown) : 0;
          await reloadDetected();
          setMessage(t('admin-cookies-scanned', { total: found.length, unknown: recorded }));
        })}>{t('admin-cookies-scan')}</button>}>
        <Show when={detected.error}><p role="alert">{describeAdminError(detected.error)}</p></Show>
        <Show when={list().length === 0 && !detected.loading}><p>{t('admin-cookies-detected-none')}</p></Show>
        <Show when={list().length > 0}>
          <div class={css.tableWrap} tabindex="0" role="region" aria-label={t('admin-cookies-detected')}>
            <table class={css.table} data-testid="cookie-detected">
              <thead><tr><th scope="col">{t('admin-cookies-col-name')}</th><th scope="col">{t('admin-cookies-col-kind')}</th><th scope="col">{t('admin-cookies-col-seen')}</th><th scope="col">{t('admin-cookies-col-last')}</th><th scope="col">{t('admin-cookies-col-actions')}</th></tr></thead>
              <tbody>
                <For each={list()}>{(row) => <DetectedItem row={row} api={api} run={run} after={reloadDetected} reloadInventory={reloadInventory} />}</For>
              </tbody>
            </table>
          </div>
        </Show>
      </AdminSubsection>

      <AdminSubsection title={t('admin-cookies-stats')} description={t('admin-cookies-stats-help')}
        actions={<>
          <label>{t('admin-cookies-range')} <select value={String(days())} onChange={(e) => setDays(Number(e.currentTarget.value) as (typeof RANGES)[number])}>
            <For each={RANGES}>{(d) => <option value={String(d)}>{t('admin-cookies-range-days', { days: d })}</option>}</For></select></label>{' '}
          <button type="button" class="btn" onClick={() => void reloadStats()}>{t('privacy-refresh')}</button>{' '}
          <button type="button" class="btn" onClick={() => void run(async () => { const to = Math.floor(Date.now() / 1000) + 1; download(await api.exportCsv(to - days() * DAY, to)); }, t('admin-cookies-exported'))}>{t('admin-cookies-export')}</button>
        </>}>
        <Show when={stats.error}><p role="alert">{describeAdminError(stats.error)}</p></Show>
        <Show when={stats.loading && !stats.latest}><InlineLoading /></Show>
        <Show when={!stats.error ? stats.latest : undefined}>{(value) => <StatsView stats={value()} />}</Show>
      </AdminSubsection>
    </SettingsGate>
  );
}

function download(csv: string): void {
  const url = URL.createObjectURL(new Blob([csv], { type: 'text/csv;charset=utf-8' }));
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = 'cookie-consents.csv';
  anchor.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

function AddRule(props: { api: CookiesApi; onSaved: () => void; onError: (e: unknown) => void }): JSX.Element {
  const [kind, setKind] = createSignal<StorageKind>('cookie');
  const [pattern, setPattern] = createSignal('');
  const [category, setCategory] = createSignal<Category>('analytics');
  const [note, setNote] = createSignal('');
  const valid = (): boolean => pattern().trim().length > 0 && /[^*]/.test(pattern());
  return (
    <form onSubmit={(e) => { e.preventDefault(); if (!valid()) return; void props.api.saveRule({ kind: kind(), pattern: pattern().trim(), category: category(), note: note() }).then(() => { setPattern(''); setNote(''); props.onSaved(); }, props.onError); }}>
      <h4>{t('admin-cookies-add-rule')}</h4>
      <label>{t('admin-cookies-col-kind')} <select value={kind()} onChange={(e) => setKind(e.currentTarget.value as StorageKind)}><For each={KIND_IDS}>{(k) => <option value={k}>{t(`admin-cookies-kind-${k}`)}</option>}</For></select></label>{' '}
      <label>{t('admin-cookies-col-name')} <input value={pattern()} maxLength={128} onInput={(e) => setPattern(e.currentTarget.value)} aria-describedby="cookie-pattern-help" /></label>{' '}
      <label>{t('admin-cookies-col-category')} <select value={category()} onChange={(e) => setCategory(e.currentTarget.value as Category)}>{categoryOptions()}</select></label>{' '}
      <label>{t('admin-cookies-note')} <input value={note()} maxLength={200} onInput={(e) => setNote(e.currentTarget.value)} /></label>{' '}
      <button type="submit" class="btn" disabled={!valid()}>{t('admin-cookies-add')}</button>
      <p id="cookie-pattern-help">{t('admin-cookies-pattern-help')}</p>
    </form>
  );
}

function DetectedItem(props: { row: DetectedRow; api: CookiesApi; run: (work: () => Promise<unknown>, done?: string) => Promise<void>; after: () => unknown; reloadInventory: () => unknown }): JSX.Element {
  const [category, setCategory] = createSignal<Category>('functional');
  const settle = async (work: () => Promise<unknown>, done: string): Promise<void> => props.run(async () => { await work(); await props.after(); await props.reloadInventory(); }, done);
  return (
    <tr data-detected={props.row.id}>
      <th scope="row"><code>{props.row.name}</code>{props.row.status === 'ignored' ? <> ({t('admin-cookies-ignored')})</> : null}</th>
      <td>{t(`admin-cookies-kind-${props.row.kind}`)}</td><td>{formatNumber(props.row.seenCount)}</td><td>{formatDateTime(props.row.lastSeen * 1000)}</td>
      <td>
        <select aria-label={t('admin-cookies-category-of', { name: props.row.name })} value={category()} onChange={(e) => setCategory(e.currentTarget.value as Category)}>{categoryOptions()}</select>{' '}
        <button type="button" class="btn" onClick={() => void settle(() => props.api.classify(props.row.id, category()), t('admin-cookies-classified'))}>{t('admin-cookies-classify')}</button>{' '}
        <Show when={props.row.status !== 'ignored'}><button type="button" class="btn btn--ghost" onClick={() => void settle(() => props.api.ignore(props.row.id), t('admin-cookies-saved'))}>{t('admin-cookies-ignore')}</button>{' '}</Show>
        <button type="button" class="btn btn--ghost" onClick={() => void settle(() => props.api.remove(props.row.id), t('admin-cookies-removed'))}>{t('admin-cookies-remove')}</button>
      </td>
    </tr>
  );
}

function StatsView(props: { stats: ConsentStats }): JSX.Element {
  const s = (): ConsentStats => props.stats;
  const decisions = (): { label: string; value: number }[] => [
    { label: t('admin-cookies-decision-acceptAll'), value: s().decisions.acceptAll },
    { label: t('admin-cookies-decision-rejectAll'), value: s().decisions.rejectAll },
    { label: t('admin-cookies-decision-custom'), value: s().decisions.custom },
    { label: t('admin-cookies-decision-notice'), value: s().decisions.notice },
  ];
  const times = (): number[] => s().series.map((p) => p.t);
  const day = (secs: number): string => formatDateTime(secs * 1000, { dateStyle: 'medium' } as Intl.DateTimeFormatOptions);
  return (
    <>
      <Show when={s().truncated}><p role="status">{t('admin-cookies-truncated')}</p></Show>
      <p data-testid="cookie-total">{t('admin-cookies-total', { count: s().total })}</p>
      <StatsSection title={t('admin-cookies-stats')} kind="charts">
        <BarChart title={t('admin-cookies-chart-decisions')} bars={decisions()} emptyText={t('admin-cookies-no-data')} testId="cookie-chart-decisions" />
        <BarChart title={t('admin-cookies-chart-categories')} bars={(['functional', 'analytics', 'marketing'] as const).map((c) => ({ label: `${t(`cookies-category-${c}`)} (${share(s().categories[c].accepted, s().total)} %)`, value: s().categories[c].accepted }))} emptyText={t('admin-cookies-no-data')} testId="cookie-chart-categories" />
        <ChartCard
          testId="cookie-chart-daily"
          title={t('admin-cookies-chart-daily')}
          unit={t('admin-cookies-chart-daily-unit')}
          legend={[{ label: t('admin-cookies-decision-acceptAll'), slot: 1 }, { label: t('admin-cookies-decision-rejectAll'), slot: 2 }]}
          summary={<>{t('admin-cookies-chart-daily-summary', { accept: s().decisions.acceptAll, reject: s().decisions.rejectAll, custom: s().decisions.custom })}</>}
          table={{ columns: [t('admin-cookies-col-day'), t('admin-cookies-decision-acceptAll'), t('admin-cookies-decision-rejectAll'), t('admin-cookies-decision-custom'), t('admin-cookies-decision-notice')], numeric: [false, true, true, true, true], rows: s().series.map((p) => [day(p.t), String(p.decisions.acceptAll), String(p.decisions.rejectAll), String(p.decisions.custom), String(p.decisions.notice)]) }}
          empty={s().total === 0 ? t('admin-cookies-no-data') : undefined}
        >
          <LineChart times={times()} series={[{ label: t('admin-cookies-decision-acceptAll'), slot: 1, values: s().series.map((p) => p.decisions.acceptAll) }, { label: t('admin-cookies-decision-rejectAll'), slot: 2, values: s().series.map((p) => p.decisions.rejectAll) }]} format={(n) => formatNumber(n, { maximumFractionDigits: 0 })} formatTime={day} whole />
        </ChartCard>
      </StatsSection>
      <div class={css.tableWrap} tabindex="0" role="region" aria-label={t('admin-cookies-regions')}>
        <table class={css.table}>
          <thead><tr><th scope="col">{t('admin-cookies-regions')}</th><th scope="col">{t('admin-cookies-col-count')}</th></tr></thead>
          <tbody><For each={s().regions} fallback={<tr><td colspan="2">{t('admin-cookies-no-data')}</td></tr>}>{(r) => <tr><th scope="row">{r.region}</th><td>{formatNumber(r.count)}</td></tr>}</For></tbody>
        </table>
      </div>
      <p>{t('admin-cookies-signals', { gpc: s().signals.gpc, dnt: s().signals.dnt })}</p>
    </>
  );
}
