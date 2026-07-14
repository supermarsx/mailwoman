// Admin › Observability (§19, §21). Log level + OTLP DSN + auth-gated Prometheus
// metrics toggle; the append-only audit-log viewer + JSONL export; the login
// monitor / ban list (fail2ban-compatible) with add + unban.

import { createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import type { AuditLogEntry, BanEntry, ObservabilityConfig } from '../../state/slices/admin.ts';
import { t } from '../../i18n';
import * as css from './admin.css.ts';

const DEFAULT_OBS: ObservabilityConfig = {
  logLevel: 'info',
  otlpDsn: null,
  metricsEnabled: false,
  sentryDsn: null,
};

export function Observability(): JSX.Element {
  const { api } = useAdmin();
  const [obs, setObs] = createSignal<ObservabilityConfig>(DEFAULT_OBS);
  const [audit, setAudit] = createSignal<AuditLogEntry[]>([]);
  const [bans, setBans] = createSignal<BanEntry[]>([]);
  const [error, setError] = createSignal<string | null>(null);
  const [saved, setSaved] = createSignal(false);
  const [banIp, setBanIp] = createSignal('');
  const [banReason, setBanReason] = createSignal('');

  async function reload(): Promise<void> {
    try {
      const [o, a, b] = await Promise.all([api.getObservability(), api.listAudit(100), api.listBans()]);
      setObs(o);
      setAudit(a);
      setBans(b);
      setError(null);
    } catch {
      setError(t('admin-obs-load-error'));
    }
  }
  onMount(() => void reload());

  function patch<K extends keyof ObservabilityConfig>(key: K, value: ObservabilityConfig[K]): void {
    setObs({ ...obs(), [key]: value });
    setSaved(false);
  }

  async function onSaveConfig(e: Event): Promise<void> {
    e.preventDefault();
    try {
      await api.setObservability(obs());
      setSaved(true);
    } catch {
      setError(t('admin-obs-save-error'));
    }
  }

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

      <form class={css.card} onSubmit={(e) => void onSaveConfig(e)} aria-label={t('admin-obs-config')}>
        <div class={css.grid}>
          <label class="field">
            <span>{t('admin-obs-log-level')}</span>
            <input type="text" value={obs().logLevel} onInput={(e) => patch('logLevel', e.currentTarget.value)} />
          </label>
          <label class="field">
            <span>{t('admin-obs-otlp')}</span>
            <input
              type="text"
              value={obs().otlpDsn ?? ''}
              placeholder={t('admin-obs-otlp-placeholder')}
              onInput={(e) => patch('otlpDsn', e.currentTarget.value === '' ? null : e.currentTarget.value)}
            />
          </label>
        </div>
        <label class="field">
          <input
            type="checkbox"
            checked={obs().metricsEnabled}
            aria-label={t('admin-obs-metrics-label')}
            onChange={(e) => patch('metricsEnabled', e.currentTarget.checked)}
          />{' '}
          {t('admin-obs-metrics')}
        </label>
        <button type="submit" class="btn btn--primary">
          {t('admin-obs-save')}
        </button>
        <Show when={saved()}>
          <p class={css.note} role="status">
            {t('admin-saved')}
          </p>
        </Show>
      </form>

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
