// Admin > Retention, second half: the retention classes (what lives in which
// class and how old it is), the archive tier of the long class (size, last
// verification, search, restore, export) and an overview of the rules users have
// set for their own mail. The limits themselves are edited in place with the same
// editor the rest of the screen uses. These are configurable controls; whether a
// configuration meets a given law is for the operator to decide.

import { For, Show, createResource, createSignal, type JSX } from 'solid-js';
import { formatDateTime, formatNumber, t } from '../../../i18n';
import {
  archiveExportPath,
  type ArchiveHit, type ArchiveSnapshot, type ArchiveVerify, type ClassesSnapshot, type UserRulesOverview,
} from '../../../api/admin/retention.ts';
import { adminUrl } from '../../../api/adminPath.ts';
import type { AdminSetting } from '../../../api/admin/settings.ts';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { InlineSettingEditor } from '../Settings/InlineSettingEditor.tsx';
import { BarChart } from '../shell/BarChart.tsx';
import { StatsSection } from '../shell/ChartCard.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import { StatTile } from '../shell/StatTile.tsx';
import * as css from '../admin.css.ts';
import { bucketLabels, classWarnings, formatBytes } from './model.ts';

export interface TiersApi {
  classes: () => Promise<ClassesSnapshot>;
  archive: () => Promise<ArchiveSnapshot>;
  userRules: () => Promise<UserRulesOverview>;
  verify: () => Promise<ArchiveVerify>;
  search: (store: string, q: { key?: string; account?: string }) => Promise<{ hits: ArchiveHit[] }>;
  restore: (segmentId: string) => Promise<{ restored: number }>;
  settings: () => Promise<AdminSetting[]>;
}

const when = (s: number | null | undefined): string => (!s ? t('admin-retention-never') : formatDateTime(s * 1000));

export function TiersPanel(props: { api: TiersApi }): JSX.Element {
  const api = props.api;
  const [classes] = createResource(() => api.classes());
  const [archive, { refetch: refetchArchive }] = createResource(() => api.archive());
  const [users] = createResource(() => api.userRules());
  const [rows] = createResource(() => api.settings().catch(() => [] as AdminSetting[]));
  const [busy, setBusy] = createSignal(false);
  const [message, setMessage] = createSignal<string | null>(null);
  const [failure, setFailure] = createSignal<unknown>(null);
  const [store, setStore] = createSignal('');
  const [key, setKey] = createSignal('');
  const [account, setAccount] = createSignal('');
  const [hits, setHits] = createSignal<ArchiveHit[] | null>(null);

  const settingRows = (keys: readonly string[]): AdminSetting[] => {
    const all = rows.error ? [] : rows.latest ?? [];
    return keys.flatMap((k) => all.filter((r) => r.key === k));
  };
  const run = async (work: () => Promise<void>): Promise<void> => {
    setBusy(true);
    setFailure(null);
    setMessage(null);
    try { await work(); } catch (e) { setFailure(e); } finally { setBusy(false); }
  };
  const verify = (): Promise<void> => run(async () => {
    const r = await api.verify();
    setMessage(r.failed.length === 0
      ? t('admin-retention-verify-ok', { count: r.checked })
      : t('admin-retention-verify-failed', { failed: r.failed.length, count: r.checked }));
    await refetchArchive();
  });
  const search = (): Promise<void> => run(async () => {
    const r = await api.search(store() || (archive.latest?.tables[0] ?? ''), { key: key().trim(), account: account().trim() });
    setHits(r.hits);
  });
  const restore = (id: string): Promise<void> => run(async () => {
    const r = await api.restore(id);
    setMessage(t('admin-retention-restored', { count: r.restored }));
  });

  return (
    <>
      <AdminSubsection title={t('admin-retention-classes')} description={t('admin-retention-classes-intro')}>
        <Show when={classes.error}><p role="alert">{describeAdminError(classes.error)}</p></Show>
        <Show when={classes.error ? undefined : classes.latest}>
          {(c) => (
            <>
              <Show when={classWarnings(c()).length > 0}>
                <ul data-testid="retention-class-warnings">
                  <For each={classWarnings(c())}>{(w) => <li role="note"><strong>{t(w)}</strong></li>}</For>
                </ul>
              </Show>
              <p class={css.note} data-testid="retention-class-enforce">
                {c().enforce ? t('admin-retention-class-enforce-on') : t('admin-retention-class-enforce-off')}
              </p>
              <For each={c().classes}>{(cl) => (
                <div data-testid={`retention-class-${cl.id}`}>
                  <h4>{t(`admin-retention-class-${cl.id}`)}</h4>
                  <p class={css.note}>
                    {cl.forever ? t('admin-retention-class-days-forever') : t('admin-retention-class-days', { days: cl.effectiveDays })}
                    {' '}{t(`admin-retention-class-${cl.id}-about`)}
                  </p>
                  <Show when={cl.forever}><p role="note"><strong>{t('admin-retention-forever-warning')}</strong></p></Show>
                  <ul>
                    <For each={cl.entries}>{(e) => (
                      <li data-testid={`retention-class-entry-${e.id}`}>
                        {t(`admin-retention-store-${e.id}`)}
                        <Show when={e.keptForever}>{' '}<strong>{t('admin-retention-limit-forever')}</strong></Show>
                        <Show when={e.exceedsClass}>{' '}<em>{t('admin-retention-exceeds-class')}</em></Show>
                        <Show when={e.archivable}>{' '}<span>{t('admin-retention-archivable')}</span></Show>
                      </li>
                    )}</For>
                  </ul>
                </div>
              )}</For>
              <StatsSection title={t('admin-retention-age-charts')} kind="charts">
                <For each={c().classes}>{(cl) => (
                  <BarChart
                    testId={`retention-age-${cl.id}`}
                    title={t('admin-retention-age-chart', { class: t(`admin-retention-class-${cl.id}`) })}
                    bars={bucketLabels(c().edgesDays, t).map((label, i) => ({ label, value: cl.ageBuckets[i] ?? 0 })).filter((b) => b.value > 0)}
                    emptyText={t('admin-retention-chart-empty')}
                  />
                )}</For>
              </StatsSection>
              <For each={settingRows([...c().classes.map((x) => x.key), c().enforceKey])}>{(row) => (
                <InlineSettingEditor settingKey={row.key} setting={row} noScoped />
              )}</For>
            </>
          )}
        </Show>
      </AdminSubsection>

      <AdminSubsection title={t('admin-retention-archive')} description={t('admin-retention-archive-intro')}>
        <Show when={archive.error}><p role="alert">{describeAdminError(archive.error)}</p></Show>
        <Show when={archive.error ? undefined : archive.latest}>
          {(a) => (
            <>
              <StatsSection title={t('admin-retention-archive-stats')} kind="tiles" testId="retention-archive-stats">
                <StatTile label={t('admin-retention-archive-state')} value={a().enabled ? t('admin-retention-state-on') : t('admin-retention-state-off')} testId="retention-archive-state" />
                <StatTile label={t('admin-retention-archive-segments')} value={a().totals.segments} testId="retention-archive-segments" />
                <StatTile label={t('admin-retention-archive-rows')} value={a().totals.rows} testId="retention-archive-rows" />
                <StatTile label={t('admin-retention-archive-size')} value={formatBytes(a().totals.bytes)} testId="retention-archive-size" />
                <StatTile label={t('admin-retention-archive-verified')} value={when(a().status?.verifiedAt)} testId="retention-archive-verified" />
                <StatTile label={t('admin-retention-archive-failing')} value={a().totals.failing} testId="retention-archive-failing" />
              </StatsSection>
              <Show when={!a().directoryValid}><p role="alert">{t('admin-retention-archive-dir-invalid')}</p></Show>
              <Show when={a().legalHold}><p role="note">{t('admin-retention-archive-hold')}</p></Show>
              <Show when={a().longForever}><p role="note"><strong>{t('admin-retention-forever-warning')}</strong></p></Show>
              <p class={css.note}>{t('admin-retention-archive-key', { generation: a().keyGeneration, directory: a().directory })}</p>
              <div class={css.headingRow}>
                <button type="button" class="btn" disabled={busy()} onClick={() => void verify()} data-testid="retention-archive-verify">{t('admin-retention-archive-verify')}</button>
              </div>
              <Show when={message()}><p role="status" data-testid="retention-archive-message">{message()}</p></Show>
              <Show when={failure()}><p role="alert">{describeAdminError(failure())}</p></Show>

              <h4>{t('admin-retention-archive-search')}</h4>
              <form class={css.headingRow} onSubmit={(e) => { e.preventDefault(); void search(); }}>
                <label>{t('admin-retention-archive-store')}{' '}
                  <select value={store()} onChange={(e) => setStore(e.currentTarget.value)} data-testid="retention-archive-store">
                    <For each={a().tables}>{(id) => <option value={id}>{t(`admin-retention-store-${id}`)}</option>}</For>
                  </select>
                </label>
                <label>{t('admin-retention-archive-row-key')}{' '}
                  <input value={key()} onInput={(e) => setKey(e.currentTarget.value)} data-testid="retention-archive-key" />
                </label>
                <label>{t('admin-retention-archive-account')}{' '}
                  <input value={account()} onInput={(e) => setAccount(e.currentTarget.value)} data-testid="retention-archive-account" />
                </label>
                <button type="submit" class="btn" disabled={busy() || (key().trim() === '' && account().trim() === '')} data-testid="retention-archive-find">{t('admin-retention-archive-find')}</button>
              </form>
              <Show when={hits()}>
                {(h) => (
                  <div data-testid="retention-archive-hits">
                    <Show when={h().length === 0}><p>{t('admin-retention-archive-nohits')}</p></Show>
                    <ul>
                      <For each={h()}>{(hit) => (
                        <li>
                          <code>{JSON.stringify(hit.row)}</code>{' '}
                          <button type="button" class="btn btn--ghost" disabled={busy()} onClick={() => void restore(hit.segmentId)} data-testid="retention-archive-restore">{t('admin-retention-archive-restore')}</button>
                        </li>
                      )}</For>
                    </ul>
                  </div>
                )}
              </Show>

              <table class={css.table} data-testid="retention-archive-table">
                <thead>
                  <tr>
                    <th scope="col">{t('admin-retention-col-store')}</th>
                    <th scope="col">{t('admin-retention-archive-seq')}</th>
                    <th scope="col">{t('admin-retention-col-rows')}</th>
                    <th scope="col">{t('admin-retention-col-size')}</th>
                    <th scope="col">{t('admin-retention-archive-range')}</th>
                    <th scope="col">{t('admin-retention-archive-check')}</th>
                    <th scope="col">{t('admin-retention-archive-export')}</th>
                  </tr>
                </thead>
                <tbody>
                  <For each={a().segments}>{(s) => (
                    <tr data-testid={`retention-segment-${s.id}`}>
                      <th scope="row">{t(`admin-retention-store-${s.storeId}`)}</th>
                      <td>{s.seq}</td>
                      <td>{formatNumber(s.rowCount)}</td>
                      <td>{formatBytes(s.bytes)}</td>
                      <td>{when(s.firstTs)} – {when(s.lastTs)}</td>
                      <td>{s.state === 'purged' ? t('admin-retention-archive-purged') : s.verifyStatus === 'ok' ? t('admin-retention-archive-ok') : s.verifyStatus}</td>
                      <td><Show when={s.state !== 'purged'}><a class="btn btn--ghost" href={adminUrl(archiveExportPath(s.id))} download="">{t('admin-retention-archive-export')}</a></Show></td>
                    </tr>
                  )}</For>
                </tbody>
              </table>
              <For each={settingRows(a().keys.filter((k) => k.startsWith('retention.archive.')))}>{(row) => (
                <InlineSettingEditor settingKey={row.key} setting={row} noScoped />
              )}</For>
            </>
          )}
        </Show>
      </AdminSubsection>

      <AdminSubsection title={t('admin-retention-userrules')} description={t('admin-retention-userrules-intro')}>
        <Show when={users.error}><p role="alert">{describeAdminError(users.error)}</p></Show>
        <Show when={users.error ? undefined : users.latest}>
          {(u) => (
            <>
              <StatsSection title={t('admin-retention-userrules')} kind="tiles" testId="retention-userrules-stats">
                <StatTile label={t('admin-retention-userrules-state')} value={u().enabled ? t('admin-retention-state-on') : t('admin-retention-state-off')} testId="retention-userrules-state" />
                <StatTile label={t('admin-retention-userrules-users')} value={u().usersWithRules} testId="retention-userrules-users" />
                <StatTile label={t('admin-retention-userrules-rules')} value={u().enabledRules} testId="retention-userrules-rules" />
                <StatTile label={t('admin-retention-userrules-changed')} value={u().messagesChanged30d} testId="retention-userrules-changed" />
                <StatTile label={t('admin-retention-userrules-lastjob')} value={u().lastJob ? when(u().lastJob?.at) : t('admin-retention-never')} testId="retention-userrules-lastjob" />
              </StatsSection>
              <For each={settingRows(u().keys)}>{(row) => (
                <InlineSettingEditor settingKey={row.key} setting={row} noScoped />
              )}</For>
            </>
          )}
        </Show>
      </AdminSubsection>
    </>
  );
}
