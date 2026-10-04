// Admin › Observability (§19, §21). The append-only audit-log viewer + JSONL
// export, and the login monitor's ban list with add + unban.
//
// There is no telemetry form. The log level, OTLP DSN and metrics toggle it used
// to carry were saved to a settings row that nothing applies: the running log
// filter, OTLP exporter and `/metrics` endpoint are set from the server's
// environment at start (`crates/mw-server/src/observability.rs`,
// `ObservabilityConfig::from_env`). The form returns when a save changes the
// running server (26.20, t28-e8).
//
// The ban list is a record, and the screen says so: no request path refuses a
// listed address (`mw_admin::BanEntry`, `crates/mw-admin/src/lib.rs`).

import { createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import type { AuditLogEntry, BanEntry } from '../../state/slices/admin.ts';
import { t } from '../../i18n';
import * as css from './admin.css.ts';

export function Observability(): JSX.Element {
  const { api } = useAdmin();
  const [audit, setAudit] = createSignal<AuditLogEntry[]>([]);
  const [bans, setBans] = createSignal<BanEntry[]>([]);
  const [error, setError] = createSignal<string | null>(null);
  const [banIp, setBanIp] = createSignal('');
  const [banReason, setBanReason] = createSignal('');

  async function reload(): Promise<void> {
    try {
      const [a, b] = await Promise.all([api.listAudit(100), api.listBans()]);
      setAudit(a);
      setBans(b);
      setError(null);
    } catch {
      setError(t('admin-obs-load-error'));
    }
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
      setError(t('admin-obs-export-error'));
    }
  }

  async function onAddBan(e: Event): Promise<void> {
    e.preventDefault();
    if (banIp().trim() === '') return;
    try {
      await api.addBan({ ip: banIp().trim(), reason: banReason().trim(), expiresAt: null });
      setBanIp('');
      setBanReason('');
      await reload();
    } catch {
      setError(t('admin-obs-ban-add-error'));
    }
  }

  async function onUnban(ip: string): Promise<void> {
    try {
      await api.removeBan(ip);
      await reload();
    } catch {
      setError(t('admin-obs-unban-error'));
    }
  }

  return (
    <section class={css.section} aria-label={t('admin-obs-title')}>
      <h2 class={css.heading}>{t('admin-obs-title')}</h2>
      <Show when={error()}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>

      <div class={css.card}>
        <p class={css.note}>{t('admin-obs-telemetry-note')}</p>
      </div>

      <div class={css.card}>
        <div style={{ display: 'flex', 'justify-content': 'space-between', 'align-items': 'center' }}>
          <h3 class={css.heading}>{t('admin-obs-audit')}</h3>
          <button type="button" class="btn btn--ghost" onClick={() => void onExport()}>
            {t('admin-obs-export')}
          </button>
        </div>
        <Show when={audit().length > 0} fallback={<p class={css.note}>{t('admin-obs-audit-empty')}</p>}>
          <div class={css.tableWrap}>
            <table class={css.table}>
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
                      <td class={css.mono}>{a.ts}</td>
                      <td>
                        <span dir="auto">{a.actor}</span> <span class={css.badge}>{a.actorKind}</span>
                      </td>
                      <td dir="auto">{a.action}</td>
                      <td dir="auto">{a.target ?? '—'}</td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </Show>
      </div>

      <div class={css.card}>
        <h3 class={css.heading}>{t('admin-obs-bans')}</h3>
        <p class={css.note} data-testid="admin-obs-bans-note">
          {t('admin-obs-bans-note')}
        </p>
        <form onSubmit={(e) => void onAddBan(e)} aria-label={t('admin-obs-ban-add')} class={css.grid}>
          <label class="field">
            <span>{t('admin-obs-ban-ip')}</span>
            <input type="text" value={banIp()} onInput={(e) => setBanIp(e.currentTarget.value)} />
          </label>
          <label class="field">
            <span>{t('admin-obs-ban-reason')}</span>
            <input type="text" value={banReason()} onInput={(e) => setBanReason(e.currentTarget.value)} />
          </label>
          <button type="submit" class="btn btn--primary">
            {t('admin-obs-ban-btn')}
          </button>
        </form>
        <Show when={bans().length > 0} fallback={<p class={css.note}>{t('admin-obs-bans-empty')}</p>}>
          <For each={bans()}>
            {(b) => (
              <div class={css.listRow}>
                <div>
                  <span class={css.mono} dir="auto">
                    {b.ip}
                  </span>{' '}
                  <span class={css.note} dir="auto">
                    {b.reason}
                  </span>
                </div>
                <button
                  type="button"
                  class="btn btn--ghost"
                  aria-label={t('admin-obs-unban-for', { ip: b.ip })}
                  onClick={() => void onUnban(b.ip)}
                >
                  {t('admin-obs-unban-btn')}
                </button>
              </div>
            )}
          </For>
        </Show>
      </div>
    </section>
  );
}
