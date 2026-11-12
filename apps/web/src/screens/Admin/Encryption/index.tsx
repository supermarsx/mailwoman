// Admin > Compliance > Encryption: what is encrypted at rest and how that was
// determined, the database key and its age, a check that proves the database file
// is encrypted, a dry run of the migration, and the commands that change it.
// This reports configuration and observed file contents; it does not certify
// compliance with any law or standard.

import { For, Show, createResource, createSignal, onMount, useContext, type JSX } from 'solid-js';
import { formatDateTime, formatNumber, loadCatalog, t } from '../../../i18n';
import {
  loadEncryption, rehearseEncryption, verifyEncryption, encryptExistingFiles,
  type EncryptionSnapshot, type Rehearsal, type VerifyReport,
} from '../../../api/admin/encryption.ts';
import { listAdminSettings, type AdminSetting } from '../../../api/admin/settings.ts';
import {ShellContext} from '../shell/context.ts';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { InlineSettingEditor } from '../Settings/InlineSettingEditor.tsx';
import { StatsSection } from '../shell/ChartCard.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import { StatTile } from '../shell/StatTile.tsx';
import * as css from '../admin.css.ts';
import { checkLabel, counts, evidenceArgs, evidenceLabel, keyAgeDays, needsAttention, stateLabel, storeLabel, warningLabel } from './model.ts';

export interface EncryptionApi {
  load: () => Promise<EncryptionSnapshot>;
  verify: () => Promise<VerifyReport>;
  rehearse: () => Promise<Rehearsal>;
  settings: () => Promise<AdminSetting[]>;
  encryptFiles?:typeof encryptExistingFiles;
}

const liveApi: EncryptionApi = {
  load: loadEncryption,
  verify: verifyEncryption,
  rehearse: rehearseEncryption,
  settings: async () => {
    const r = await listAdminSettings();
    return r.kind === 'ok' ? r.data : [];
  },
};

export default function Encryption(props: { api?: EncryptionApi }): JSX.Element {
  const api = props.api ?? liveApi;
  const shell=useContext(ShellContext);const [migrated,setMigrated]=createSignal(false);
  onMount(() => void loadCatalog('admin-encryption'));
  const [snap, { refetch }] = createResource(() => api.load());
  const [rows] = createResource(() => api.settings().catch(() => [] as AdminSetting[]));
  const [busy, setBusy] = createSignal<'verify' | 'rehearse' | 'files' | null>(null);
  const [verify, setVerify] = createSignal<VerifyReport | null>(null);
  const [rehearsal, setRehearsal] = createSignal<Rehearsal | null>(null);
  const [failure, setFailure] = createSignal<unknown>(null);

  const act = async (kind: 'verify' | 'rehearse' | 'files'): Promise<void> => {
    setBusy(kind);
    setFailure(null);
    try {
      if(kind==='files'){await (api.encryptFiles??encryptExistingFiles)();setMigrated(true);await refetch();}
      else if (kind === 'verify') setVerify(await api.verify());
      else setRehearsal(await api.rehearse());
    } catch (error) {
      setFailure(error);
    } finally {
      setBusy(null);
    }
  };
  const settingRows = (keys: readonly string[]): AdminSetting[] => {
    const all = rows.error ? [] : rows.latest ?? [];
    return keys.flatMap((k) => all.filter((r) => r.key === k));
  };

  return (
    <section class={css.section} aria-labelledby="admin-encryption-heading" data-testid="admin-encryption">
      <h2 id="admin-encryption-heading" class={css.heading}>{t('admin-encryption-title')}</h2>
      <p class={css.note}>{t('admin-encryption-intro')}</p>
      <p class={css.note}>{t('admin-encryption-docs')}</p>
      <Show when={snap.error}><p role="alert">{describeAdminError(snap.error)}</p></Show>
      <Show when={snap.error ? undefined : snap.latest} fallback={<Show when={!snap.error}><p role="status">{t('admin-encryption-loading')}</p></Show>}>
        {(s) => (
          <>
            <StatsSection title={t('admin-encryption-summary')} kind="tiles" testId="encryption-stats">
              <StatTile label={t('admin-encryption-tile-database')} value={t(stateLabel(s().stores[0]?.state ?? 'unknown'))} testId="encryption-tile-database" />
              <StatTile label={t('admin-encryption-tile-mode')} value={t(`admin-encryption-mode-${s().mode}`)} testId="encryption-tile-mode" />
              <StatTile
                label={t('admin-encryption-tile-key')}
                value={s().key.configured ? (s().key.source?.kind ?? 'configured') : t('admin-encryption-key-none')}
                testId="encryption-tile-key"
              />
              <StatTile
                label={t('admin-encryption-tile-rotation')}
                value={s().rotation.dueAt ? formatDateTime(Date.parse(s().rotation.dueAt ?? '')) : t('admin-encryption-no-due')}
                testId="encryption-tile-rotation"
              />
            </StatsSection>

            <Show when={s().warnings.length > 0}>
              <AdminSubsection title={t('admin-encryption-findings')}>
                <ul data-testid="encryption-warnings">
                  <For each={s().warnings}>{(w) => <li role="note">{t(warningLabel(w))}</li>}</For>
                </ul>
              </AdminSubsection>
            </Show>

            <AdminSubsection title={t('admin-encryption-stores')} description={t('admin-encryption-stores-note')}>
              <table class={css.table} data-testid="encryption-stores">
                <thead>
                  <tr>
                    <th scope="col">{t('admin-encryption-col-store')}</th>
                    <th scope="col">{t('admin-encryption-col-state')}</th>
                    <th scope="col">{t('admin-encryption-col-evidence')}</th>
                  </tr>
                </thead>
                <tbody>
                  <For each={s().stores}>{(st) => (
                    <tr data-testid={`encryption-row-${st.id}`} data-state={st.state}>
                      <th scope="row">{t(storeLabel(st.id))}</th>
                      <td>
                        <strong>{t(stateLabel(st.state))}</strong>
                        <Show when={needsAttention(st.state)}>{' '}<span aria-hidden="true">!</span></Show>
                      </td>
                      <td>{t(evidenceLabel(st.evidence), evidenceArgs(st))}</td>
                    </tr>
                  )}</For>
                </tbody>
              </table>
              <p class={css.note}>
                {t('admin-encryption-counts', Object.fromEntries(Object.entries(counts(s().stores)).map(([k, v]) => [k.replace(/-/g, '_'), v])))}
              </p>
            </AdminSubsection>

            <AdminSubsection title={t('admin-encryption-key')}>
              <dl data-testid="encryption-key">
                <dt>{t('admin-encryption-key-source')}</dt>
                <dd>{s().key.source ? `${s().key.source?.kind}: ${s().key.source?.detail}` : t('admin-encryption-key-none')}</dd>
                <dt>{t('admin-encryption-key-fingerprint')}</dt>
                <dd>{s().key.fingerprint ?? '-'}</dd>
                <dt>{t('admin-encryption-key-age')}</dt>
                <dd>{keyAgeDays(s()) === null ? t('admin-encryption-age-unknown') : t('admin-encryption-age-days', { days: formatNumber(keyAgeDays(s()) ?? 0) })}</dd>
                <dt>{t('admin-encryption-key-rotation')}</dt>
                <dd>
                  {s().rotation.days === 0 ? t('admin-encryption-no-due') : t('admin-encryption-rotation-every', { days: s().rotation.days })}
                  <Show when={s().rotation.overdue}>{' '}<strong data-testid="encryption-overdue">{t('admin-encryption-overdue')}</strong></Show>
                </dd>
              </dl>
              <For each={settingRows(s().keys)}>{(row) => <InlineSettingEditor settingKey={row.key} setting={row} noScoped />}</For>
            </AdminSubsection>

            <AdminSubsection title={t('admin-encryption-actions')} description={t('admin-encryption-actions-note')}>
              <p>{t('admin-encryption-files-help')}</p>
              <Show when={migrated()}><p role="status">{t('admin-encryption-files-done')}</p></Show>
              <div class={css.headingRow}>
                <button type="button" class="btn" disabled={busy()!==null||!shell?.can.can('settings.write')} onClick={()=>void act('files')} data-testid="encryption-files">{t('admin-encryption-files-encrypt')}</button>
                <button type="button" class="btn" disabled={busy() !== null || s().backend !== 'sqlite'} onClick={() => void act('verify')} data-testid="encryption-verify">
                  {busy() === 'verify' ? t('admin-encryption-verifying') : t('admin-encryption-verify')}
                </button>
                <button type="button" class="btn" disabled={busy() !== null || s().backend !== 'sqlite'} onClick={() => void act('rehearse')} data-testid="encryption-rehearse">
                  {busy() === 'rehearse' ? t('admin-encryption-rehearsing') : t('admin-encryption-rehearse')}
                </button>
                <button type="button" class="btn btn--ghost" onClick={() => void refetch()}>{t('admin-encryption-refresh')}</button>
              </div>
              <Show when={verify()}>
                {(v) => (
                  <div role="status" data-testid="encryption-verify-result">
                    <p><strong>{v().passed ? t('admin-encryption-verify-passed') : t('admin-encryption-verify-failed')}</strong></p>
                    <ul>
                      <For each={v().checks}>{(c) => <li data-passed={c.passed}>{c.passed ? t('admin-encryption-pass') : t('admin-encryption-fail')}: {t(checkLabel(c.name))} ({c.detail})</li>}</For>
                    </ul>
                  </div>
                )}
              </Show>
              <Show when={rehearsal()}>
                {(r) => (
                  <p role="status" data-testid="encryption-rehearse-result">
                    {r().ok
                      ? t('admin-encryption-rehearse-ok', { tables: r().tables ?? 0, rows: formatNumber(r().rows ?? 0), millis: formatNumber(r().millis ?? 0) })
                      : t('admin-encryption-rehearse-failed', { error: r().error ?? '' })}
                  </p>
                )}
              </Show>
              <Show when={failure()}><p role="alert">{describeAdminError(failure())}</p></Show>
              <h4>{t('admin-encryption-runbook')}</h4>
              <p class={css.note}>{t('admin-encryption-runbook-note')}</p>
              <pre data-testid="encryption-commands"><code>{Object.values(s().commands).join('\n')}</code></pre>
            </AdminSubsection>

            <Show when={s().postgres}>
              {(pg) => (
                <AdminSubsection title={t('admin-encryption-postgres')} description={t('admin-encryption-postgres-note')}>
                  <Show when={!pg().error} fallback={<p role="alert">{pg().error}</p>}>
                    <dl data-testid="encryption-postgres">
                      <dt>{t('admin-encryption-pg-connection')}</dt>
                      <dd>{pg().connectionSsl === null || pg().connectionSsl === undefined ? t('admin-encryption-pg-unknown') : pg().connectionSsl ? t('admin-encryption-pg-tls-on', { version: pg().tlsVersion ?? '' }) : t('admin-encryption-pg-tls-off')}</dd>
                      <dt>{t('admin-encryption-pg-server-ssl')}</dt>
                      <dd>{pg().serverSsl ?? t('admin-encryption-pg-unknown')}</dd>
                      <dt>{t('admin-encryption-pg-checksums')}</dt>
                      <dd>{pg().dataChecksums ?? t('admin-encryption-pg-unknown')}</dd>
                      <dt>{t('admin-encryption-pg-pgcrypto')}</dt>
                      <dd>{pg().pgcryptoInstalled ? t('admin-encryption-pg-installed') : pg().pgcryptoAvailable ? t('admin-encryption-pg-available') : t('admin-encryption-pg-absent')}</dd>
                      <dt>{t('admin-encryption-pg-tde')}</dt>
                      <dd>{(pg().tdeIndicators?.settings.length ?? 0) + (pg().tdeIndicators?.extensions.length ?? 0) > 0 ? [...(pg().tdeIndicators?.settings ?? []), ...(pg().tdeIndicators?.extensions ?? [])].join(', ') : t('admin-encryption-pg-none-seen')}</dd>
                    </dl>
                  </Show>
                </AdminSubsection>
              )}
            </Show>

            <AdminSubsection title={t('admin-encryption-columns')} description={t('admin-encryption-columns-note')}>
              <table class={css.table} data-testid="encryption-columns">
                <thead>
                  <tr>
                    <th scope="col">{t('admin-encryption-col-table')}</th>
                    <th scope="col">{t('admin-encryption-col-column')}</th>
                    <th scope="col">{t('admin-encryption-col-holds')}</th>
                    <th scope="col">{t('admin-encryption-col-sealed')}</th>
                  </tr>
                </thead>
                <tbody>
                  <For each={s().columnCoverage}>{(c) => (
                    <tr data-sealed={c.sealed}>
                      <th scope="row">{c.table}</th>
                      <td>{c.column}</td>
                      <td>{c.what}</td>
                      <td>{c.sealed ? t('admin-encryption-sealed') : t('admin-encryption-not-sealed')}</td>
                    </tr>
                  )}</For>
                </tbody>
              </table>
            </AdminSubsection>
          </>
        )}
      </Show>
    </section>
  );
}
