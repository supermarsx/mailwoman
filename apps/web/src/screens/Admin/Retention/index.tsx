// Admin > Retention: every store that keeps personal or behavioural data with its
// retention limit, size, last sweep and rows removed; a dry run that counts what
// the sweep would remove before it runs; the limits and the log rotation settings,
// each edited in place. These are configurable controls; whether a configuration
// meets a given law is for the operator to decide.

import { For, Show, createResource, createSignal, onMount, type JSX } from 'solid-js';
import { formatDateTime, formatNumber, loadCatalog, t } from '../../../i18n';
import {
  loadArchive, loadClasses, loadRetention, loadUserRules, previewRetention, restoreArchive, runRetention, searchArchive, verifyArchive,
  type DryRun, type RetentionSnapshot, type RetentionStore,
} from '../../../api/admin/retention.ts';
import { listAdminSettings, type AdminSetting } from '../../../api/admin/settings.ts';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { InlineSettingEditor } from '../Settings/InlineSettingEditor.tsx';
import { BarChart } from '../shell/BarChart.tsx';
import { StatsSection } from '../shell/ChartCard.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import { StatTile } from '../shell/StatTile.tsx';
import * as css from '../admin.css.ts';
import { TiersPanel, type TiersApi } from './TiersPanel.tsx';
import { formatBytes, keepsForever, limitKeys, limitMessage, storeData, storeLabel, topBy } from './model.ts';

const when = (s: number | null): string => (s === null ? t('admin-retention-never') : formatDateTime(s * 1000));

export interface RetentionApi {
  load: () => Promise<RetentionSnapshot>;
  preview: () => Promise<DryRun>;
  run: () => Promise<{ runId: string }>;
  settings: () => Promise<AdminSetting[]>;
  /** Classes, archive tier and user rules; the panel is shown when present. */
  tiers?: TiersApi;
}

const loadSettings = async (): Promise<AdminSetting[]> => {
  const r = await listAdminSettings();
  return r.kind === 'ok' ? r.data : [];
};

const liveApi: RetentionApi = {
  load: loadRetention,
  preview: previewRetention,
  run: runRetention,
  settings: loadSettings,
  tiers: {
    classes: loadClasses, archive: loadArchive, userRules: loadUserRules, verify: verifyArchive,
    search: searchArchive, restore: restoreArchive, settings: loadSettings,
  },
};

function Limit(props: { store: RetentionStore }): JSX.Element {
  const m = (): ReturnType<typeof limitMessage> => limitMessage(props.store);
  return (
    <span>
      {t(m().id, m().args)}
      <Show when={keepsForever(props.store)}>
        {' '}<strong role="note" data-testid={`retention-warning-${props.store.id}`}>{t('admin-retention-forever-warning')}</strong>
      </Show>
    </span>
  );
}

export default function Retention(props: { api?: RetentionApi }): JSX.Element {
  const api = props.api ?? liveApi;
  onMount(() => void loadCatalog('admin-retention'));
  const [snap, { refetch }] = createResource(() => api.load());
  const [rows] = createResource(() => api.settings().catch(() => [] as AdminSetting[]));
  const [focusKey, setFocusKey] = createSignal<string | undefined>(undefined);
  const [preview, setPreview] = createSignal<DryRun | null>(null);
  const [busy, setBusy] = createSignal<'preview' | 'run' | null>(null);
  const [message, setMessage] = createSignal<string | null>(null);
  const [failure, setFailure] = createSignal<unknown>(null);

  const settingRows = (keys: readonly string[]): AdminSetting[] => {
    const all = rows.error ? [] : rows.latest ?? [];
    return keys.flatMap((k) => all.filter((r) => r.key === k));
  };
  const act = async (kind: 'preview' | 'run'): Promise<void> => {
    setBusy(kind);
    setFailure(null);
    setMessage(null);
    try {
      if (kind === 'preview') setPreview(await api.preview());
      else {
        await api.run();
        setPreview(null);
        setMessage(t('admin-retention-run-started'));
      }
    } catch (error) {
      setFailure(error);
    } finally {
      setBusy(null);
    }
  };
  const label = (id: string): string => t(storeLabel(id));

  return (
    <section class={css.section} aria-labelledby="admin-retention-heading" data-testid="admin-retention">
      <h2 id="admin-retention-heading" class={css.heading}>{t('admin-retention-title')}</h2>
      <p class={css.note}>{t('admin-retention-intro')}</p>
      <p class={css.note}>{t('admin-retention-docs')}</p>
      <Show when={snap.error}><p role="alert">{describeAdminError(snap.error)}</p></Show>
      <Show when={snap.error ? undefined : snap.latest} fallback={<Show when={!snap.error}><p role="status">{t('admin-retention-loading')}</p></Show>}>
        {(s) => (
          <>
            <StatsSection title={t('admin-retention-stats')} kind="tiles" testId="retention-stats">
              <StatTile label={t('admin-retention-tile-last')} value={s().sweeper.lastRunAt === null ? t('admin-retention-never') : when(s().sweeper.lastRunAt)} testId="retention-tile-last" />
              <StatTile label={t('admin-retention-tile-next')} value={s().sweeper.enabled && s().sweeper.nextRunAt !== null ? when(s().sweeper.nextRunAt) : t('admin-retention-none')} testId="retention-tile-next" />
              <StatTile label={t('admin-retention-tile-removed')} value={s().sweeper.lastTotal} unavailableReason={t('admin-retention-never')} testId="retention-tile-removed" />
              <StatTile label={t('admin-retention-tile-state')} value={s().sweeper.running ? t('admin-retention-state-running') : s().sweeper.enabled ? t('admin-retention-state-on') : t('admin-retention-state-off')} testId="retention-tile-state" />
            </StatsSection>

            <AdminSubsection title={t('admin-retention-run-heading')}>
              <div class={css.headingRow}>
                <button type="button" class="btn" disabled={busy() !== null} onClick={() => void act('preview')} data-testid="retention-preview">
                  {busy() === 'preview' ? t('admin-retention-dry-running') : t('admin-retention-dry-run')}
                </button>
                <button type="button" class="btn btn--ghost" onClick={() => void refetch()}>{t('admin-retention-run-refresh')}</button>
              </div>
              <Show when={preview()}>
                {(p) => (
                  <div role="status" data-testid="retention-dry-result">
                    <p>{p().total === 0 ? t('admin-retention-dry-none') : t('admin-retention-dry-result', { total: formatNumber(p().total) })}</p>
                    <Show when={p().heldAccounts > 0}><p>{t('admin-retention-held', { count: p().heldAccounts })}</p></Show>
                    <ul>
                      <For each={p().stores.filter((x) => x.rows > 0 || x.error)}>{(x) => (
                        <li>{label(x.id)}: {formatNumber(x.rows)}{x.more ? '+' : ''}<Show when={x.error}> ({x.error})</Show></li>
                      )}</For>
                    </ul>
                    <button type="button" class="btn" disabled={busy() !== null} onClick={() => void act('run')} data-testid="retention-run">{t('admin-retention-run-confirm')}</button>
                  </div>
                )}
              </Show>
              <Show when={message()}><p role="status">{message()}</p></Show>
              <Show when={failure()}><p role="alert">{describeAdminError(failure())}</p></Show>
            </AdminSubsection>

            <AdminSubsection title={t('admin-retention-stores')}>
              <table class={css.table} data-testid="retention-stores">
                <thead>
                  <tr>
                    <th scope="col">{t('admin-retention-col-store')}</th>
                    <th scope="col">{t('admin-retention-col-data')}</th>
                    <th scope="col">{t('admin-retention-col-limit')}</th>
                    <th scope="col">{t('admin-retention-col-rows')}</th>
                    <th scope="col">{t('admin-retention-col-size')}</th>
                    <th scope="col">{t('admin-retention-col-last')}</th>
                    <th scope="col">{t('admin-retention-col-purged')}</th>
                    <th scope="col">{t('admin-retention-col-owner')}</th>
                  </tr>
                </thead>
                <tbody>
                  <For each={s().stores}>{(st) => (
                    <tr data-testid={`retention-row-${st.id}`}>
                      <th scope="row">{label(st.id)}</th>
                      <td>{t(storeData(st.id))}</td>
                      <td>
                        <Limit store={st} />
                        <Show when={st.key && st.owner === 'sweeper'}>
                          {' '}<button type="button" class="btn btn--ghost" onClick={() => setFocusKey(st.key ?? undefined)} data-testid={`retention-edit-${st.id}`}>{t('admin-retention-edit')}</button>
                        </Show>
                        <Show when={st.error}><div role="alert">{t('admin-retention-error', { error: st.error ?? '' })}</div></Show>
                      </td>
                      <td>{st.rows === null ? '' : formatNumber(st.rows)}</td>
                      <td>{st.bytes === null ? '' : formatBytes(st.bytes)}</td>
                      <td>{st.owner === 'sweeper' ? when(st.lastSweepAt) : ''}</td>
                      <td>{st.owner === 'sweeper' && st.purged !== null ? formatNumber(st.purged) : ''}</td>
                      <td>{st.owner === 'sweeper' ? t('admin-retention-owner-sweeper') : t('admin-retention-owner-existing')}</td>
                    </tr>
                  )}</For>
                </tbody>
              </table>
            </AdminSubsection>

            <StatsSection title={t('admin-retention-charts')} kind="charts">
              <BarChart testId="retention-chart-rows" title={t('admin-retention-chart-rows')} bars={topBy(s().stores, (x) => x.rows).map((x) => ({ label: label(x.id), value: x.value }))} emptyText={t('admin-retention-chart-empty')} />
              <BarChart testId="retention-chart-size" title={t('admin-retention-chart-size')} bars={topBy(s().stores, (x) => x.bytes).map((x) => ({ label: label(x.id), value: x.value }))} format={formatBytes} emptyText={t('admin-retention-chart-empty')} />
              <BarChart testId="retention-chart-purged" title={t('admin-retention-chart-purged')} bars={topBy(s().stores, (x) => (x.owner === 'sweeper' ? x.purged : null)).map((x) => ({ label: label(x.id), value: x.value }))} emptyText={t('admin-retention-chart-empty')} />
            </StatsSection>

            <Show when={api.tiers}>{(tiers) => <TiersPanel api={tiers()} />}</Show>

            <AdminSubsection title={t('admin-retention-limits')} description={t('admin-retention-limits-note')}>
              <For each={settingRows([...Object.values(s().sweeper.keys), ...limitKeys(s().stores)])}>{(row) => (
                <InlineSettingEditor settingKey={row.key} setting={row} focus={focusKey() === row.key} noScoped />
              )}</For>
            </AdminSubsection>

            <AdminSubsection title={t('admin-retention-logs')} description={t('admin-retention-logs-intro')}>
              <p data-testid="retention-log-status">
                {s().logs.status.fileEnabled
                  ? t('admin-retention-logs-status', {
                    directory: s().logs.status.directory,
                    current: formatBytes(s().logs.status.currentBytes),
                    files: s().logs.status.rotatedFiles,
                    rotated: formatBytes(s().logs.status.rotatedBytes),
                  })
                  : t('admin-retention-logs-off')}
              </p>
              <For each={settingRows(s().logs.keys)}>{(row) => (
                <InlineSettingEditor settingKey={row.key} setting={row} noScoped />
              )}</For>
            </AdminSubsection>
          </>
        )}
      </Show>
    </section>
  );
}
