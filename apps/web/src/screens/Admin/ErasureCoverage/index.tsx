// Admin > Compliance > Erasure and scrubbing: the Scrub person action with a dry
// run, the pseudonym key generations, the audit chain check, and the backup
// register with its "contains erased subjects" marker. These are controls over
// stored history and backups; whether a deployment meets a given law is for the
// operator to decide.
import { For, Show, createResource, createSignal, onMount, type JSX } from 'solid-js';
import { formatDateTime, formatNumber, loadCatalog, t } from '../../../i18n';
import {
  loadScrubState, previewScrub, registerBackup, removeBackup, rotateScrubKey, runScrub, sealAudit,
  type ScrubResult, type ScrubState,
} from '../../../api/admin/scrub.ts';
import { listAdminSettings, type AdminSetting } from '../../../api/admin/settings.ts';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { InlineSettingEditor } from '../Settings/InlineSettingEditor.tsx';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import * as css from '../admin.css.ts';

export interface ScrubApi {
  state: () => Promise<ScrubState>;
  preview: (address: string) => Promise<ScrubResult>;
  run: (address: string) => Promise<ScrubResult>;
  rotate: () => Promise<unknown>;
  seal: () => Promise<unknown>;
  register: (label: string, location: string) => Promise<unknown>;
  remove: (id: string) => Promise<unknown>;
  settings: () => Promise<AdminSetting[]>;
}

const liveApi: ScrubApi = {
  state: loadScrubState,
  preview: previewScrub,
  run: runScrub,
  rotate: rotateScrubKey,
  seal: sealAudit,
  register: (label, location) => registerBackup({ label, location }),
  remove: removeBackup,
  settings: async () => {
    const r = await listAdminSettings();
    return r.kind === 'ok' ? r.data : [];
  },
};

export const SCRUB_KEYS = [
  'scrub.enabled', 'scrub.pseudonymAlgorithm', 'scrub.includeLogs', 'scrub.autoRunOnErasure', 'scrub.keyRetentionDays',
] as const;
export const BACKUP_KEYS = ['backup.propagateErasure', 'backup.tombstoneRetentionDays'] as const;

const when = (ts: number | null): string => (ts === null ? '' : formatDateTime(new Date(ts * 1000)));

export default function ErasureCoverage(props: { api?: ScrubApi }): JSX.Element {
  const api = props.api ?? liveApi;
  onMount(() => void loadCatalog('admin-erasure'));
  const [state, { refetch }] = createResource(() => api.state());
  const [rows] = createResource(() => api.settings().catch(() => [] as AdminSetting[]));
  const [address, setAddress] = createSignal('');
  const [result, setResult] = createSignal<ScrubResult | null>(null);
  const [failure, setFailure] = createSignal<unknown>(null);
  const [busy, setBusy] = createSignal(false);
  const [confirm, setConfirm] = createSignal(false);
  const [label, setLabel] = createSignal('');
  const [location, setLocation] = createSignal('');
  const settingRows = (keys: readonly string[]): AdminSetting[] => {
    const all = rows.error ? [] : rows.latest ?? [];
    return keys.flatMap((k) => all.filter((r) => r.key === k));
  };
  const guard = async (fn: () => Promise<void>): Promise<void> => {
    setFailure(null);
    setBusy(true);
    try { await fn(); } catch (e) { setFailure(e); } finally { setBusy(false); }
  };
  const preview = (): Promise<void> => guard(async () => { setResult(await api.preview(address().trim())); });
  const run = async (): Promise<void> => { setResult(await api.run(address().trim())); setConfirm(false); await refetch(); };
  const act = (fn: () => Promise<unknown>): Promise<void> => guard(async () => { await fn(); await refetch(); });
  const counts = (r: ScrubResult): [string, number][] => Object.entries(r.counts).sort(([a], [b]) => a.localeCompare(b));

  return (
    <section class={css.section} aria-labelledby="admin-erasure-heading" data-testid="admin-erasure">
      <h2 id="admin-erasure-heading" class={css.heading}>{t('admin-erasure-title')}</h2>
      <p class={css.note}>{t('admin-erasure-intro')}</p>
      <p class={css.note}>{t('admin-erasure-docs')}</p>
      <Show when={state.error}><p role="alert">{describeAdminError(state.error)}</p></Show>
      <Show when={failure()}><p role="alert">{describeAdminError(failure())}</p></Show>

      <AdminSubsection title={t('admin-erasure-scrub')} description={t('admin-erasure-scrub-note')}>
        <label class="field">
          <span>{t('admin-erasure-address')}</span>
          <input type="email" value={address()} onInput={(e) => setAddress(e.currentTarget.value)} data-testid="erasure-address" autocomplete="off" />
        </label>
        <div class={css.headingRow}>
          <button type="button" class="btn" disabled={busy() || address().trim() === ''} onClick={() => void preview()} data-testid="erasure-preview">{t('admin-erasure-preview')}</button>
          <button type="button" class="btn btn--ghost" disabled={busy() || result() === null || !result()?.dryRun} onClick={() => setConfirm(true)} data-testid="erasure-run">{t('admin-erasure-run')}</button>
        </div>
        <Show when={result()}>
          {(r) => (
            <div role="status" data-testid="erasure-result">
              <p>{r().dryRun ? t('admin-erasure-dry-note') : t('admin-erasure-done-note', { pseudonym: r().pseudonym })}</p>
              <Show when={counts(r()).length > 0} fallback={<p>{t('admin-erasure-nothing')}</p>}>
                <table class={css.table}>
                  <thead><tr><th scope="col">{t('admin-erasure-col-table')}</th><th scope="col">{t('admin-erasure-col-rows')}</th></tr></thead>
                  <tbody><For each={counts(r())}>{([name, n]) => <tr><th scope="row">{name}</th><td>{formatNumber(n)}</td></tr>}</For></tbody>
                </table>
              </Show>
              <Show when={r().errors.length > 0}><ul role="alert"><For each={r().errors}>{(e) => <li>{e}</li>}</For></ul></Show>
            </div>
          )}
        </Show>
        <ConfirmDialog
          open={confirm()}
          title={t('admin-erasure-confirm-title')}
          body={t('admin-erasure-confirm-body')}
          confirmLabel={t('admin-erasure-run')}
          onConfirm={run}
          onCancel={() => setConfirm(false)}
          testId="erasure-confirm"
        />
      </AdminSubsection>

      <AdminSubsection title={t('admin-erasure-settings')}>
        <For each={settingRows([...SCRUB_KEYS, ...BACKUP_KEYS])}>{(row) => <InlineSettingEditor settingKey={row.key} setting={row} noScoped />}</For>
      </AdminSubsection>

      <Show when={state.latest}>
        {(s) => (
          <>
            <AdminSubsection title={t('admin-erasure-keys')} description={t('admin-erasure-keys-note')}
              actions={<button type="button" class="btn btn--ghost" disabled={busy()} onClick={() => void act(api.rotate)} data-testid="erasure-rotate">{t('admin-erasure-rotate')}</button>}>
              <table class={css.table} data-testid="erasure-keys">
                <thead><tr><th scope="col">{t('admin-erasure-col-gen')}</th><th scope="col">{t('admin-erasure-col-created')}</th><th scope="col">{t('admin-erasure-col-state')}</th></tr></thead>
                <tbody>
                  <For each={s().keys} fallback={<tr><td colspan="3">{t('admin-erasure-keys-none')}</td></tr>}>{(k) => (
                    <tr><th scope="row">{k.gen}</th><td>{when(k.createdAt)}</td>
                      <td>{k.retiredAt === null ? t('admin-erasure-key-current') : k.destroyAfter === null ? t('admin-erasure-key-retired') : t('admin-erasure-key-destroy', { when: when(k.destroyAfter) })}</td></tr>
                  )}</For>
                </tbody>
              </table>
            </AdminSubsection>

            <AdminSubsection title={t('admin-erasure-chain')} description={t('admin-erasure-chain-note')}
              actions={<button type="button" class="btn btn--ghost" disabled={busy()} onClick={() => void act(api.seal)} data-testid="erasure-seal">{t('admin-erasure-seal')}</button>}>
              <p data-testid="erasure-chain-state" role="status">
                {s().auditChain.ok
                  ? t('admin-erasure-chain-ok', { rows: formatNumber(s().auditChain.rows), unsealed: formatNumber(s().auditChain.unsealed) })
                  : t('admin-erasure-chain-bad', { seq: s().auditChain.firstBadSeq ?? 0 })}
              </p>
              <Show when={s().anchors.length > 0}>
                <table class={css.table} data-testid="erasure-anchors">
                  <thead><tr><th scope="col">{t('admin-erasure-col-time')}</th><th scope="col">{t('admin-erasure-col-range')}</th><th scope="col">{t('admin-erasure-col-rows')}</th></tr></thead>
                  <tbody><For each={s().anchors}>{(a) => <tr><th scope="row">{when(a.ts)}</th><td>{a.fromSeq} - {a.toSeq}</td><td>{formatNumber(a.rowsChanged)}</td></tr>}</For></tbody>
                </table>
              </Show>
            </AdminSubsection>

            <AdminSubsection title={t('admin-erasure-backups')} description={t('admin-erasure-backups-note', { watermark: formatNumber(s().tombstoneWatermark) })}>
              <table class={css.table} data-testid="erasure-backups">
                <thead>
                  <tr>
                    <th scope="col">{t('admin-erasure-col-label')}</th><th scope="col">{t('admin-erasure-col-location')}</th>
                    <th scope="col">{t('admin-erasure-col-created')}</th><th scope="col">{t('admin-erasure-col-watermark')}</th>
                    <th scope="col">{t('admin-erasure-col-marker')}</th><th scope="col">{t('admin-erasure-col-actions')}</th>
                  </tr>
                </thead>
                <tbody>
                  <For each={s().backups} fallback={<tr><td colspan="6">{t('admin-erasure-backups-none')}</td></tr>}>{(b) => (
                    <tr data-testid={`erasure-backup-${b.id}`}>
                      <th scope="row">{b.label}</th><td>{b.location}</td><td>{when(b.createdAt)}</td><td>{b.tombstoneWatermark}</td>
                      <td>{b.erasedSince > 0 ? <strong data-testid="erasure-marker">{t('admin-erasure-contains-erased', { count: b.erasedSince })}</strong> : t('admin-erasure-no-marker')}</td>
                      <td><button type="button" class="btn btn--ghost" disabled={busy()} onClick={() => void act(() => api.remove(b.id))}>{t('admin-erasure-forget')}</button></td>
                    </tr>
                  )}</For>
                </tbody>
              </table>
              <div class={css.grid}>
                <label class="field"><span>{t('admin-erasure-col-label')}</span><input value={label()} onInput={(e) => setLabel(e.currentTarget.value)} data-testid="erasure-backup-label" /></label>
                <label class="field"><span>{t('admin-erasure-col-location')}</span><input value={location()} onInput={(e) => setLocation(e.currentTarget.value)} data-testid="erasure-backup-location" /></label>
              </div>
              <button type="button" class="btn" disabled={busy() || label().trim() === '' || location().trim() === ''} data-testid="erasure-backup-add"
                onClick={() => void act(async () => { await api.register(label().trim(), location().trim()); setLabel(''); setLocation(''); })}>{t('admin-erasure-register')}</button>
            </AdminSubsection>
          </>
        )}
      </Show>
    </section>
  );
}
