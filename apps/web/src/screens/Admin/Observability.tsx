// Admin › Observability (§19, §21). Log level + OTLP DSN + auth-gated Prometheus
// metrics toggle; the append-only audit-log viewer + JSONL export; the login
// monitor / ban list (fail2ban-compatible) with add + unban.

import { createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import type { AuditLogEntry, BanEntry, ObservabilityConfig } from '../../state/slices/admin.ts';
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
      setError('Could not load observability data');
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
      setError('Could not save observability config');
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
      setError('Could not export the audit log');
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
      setError('Could not add the ban');
    }
  }

  async function onUnban(ip: string): Promise<void> {
    try {
      await api.removeBan(ip);
      await reload();
    } catch {
      setError('Could not remove the ban');
    }
  }

  return (
    <section class={css.section} aria-label="Observability">
      <h2 class={css.heading}>Observability</h2>
      <Show when={error()}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>

      <form class={css.card} onSubmit={(e) => void onSaveConfig(e)} aria-label="Logging and telemetry">
        <div class={css.grid}>
          <label class="field">
            <span>Log level</span>
            <input type="text" value={obs().logLevel} onInput={(e) => patch('logLevel', e.currentTarget.value)} />
          </label>
          <label class="field">
            <span>OTLP DSN</span>
            <input
              type="text"
              value={obs().otlpDsn ?? ''}
              placeholder="https://otlp.example.org"
              onInput={(e) => patch('otlpDsn', e.currentTarget.value === '' ? null : e.currentTarget.value)}
            />
          </label>
        </div>
        <label class="field">
          <input
            type="checkbox"
            checked={obs().metricsEnabled}
            aria-label="Enable Prometheus metrics endpoint"
            onChange={(e) => patch('metricsEnabled', e.currentTarget.checked)}
          />{' '}
          Enable auth-gated Prometheus /metrics
        </label>
        <button type="submit" class="btn btn--primary">
          Save telemetry
        </button>
        <Show when={saved()}>
          <p class={css.note} role="status">
            Saved.
          </p>
        </Show>
      </form>

      <div class={css.card}>
        <div style={{ display: 'flex', 'justify-content': 'space-between', 'align-items': 'center' }}>
          <h3 class={css.heading}>Audit log</h3>
          <button type="button" class="btn btn--ghost" onClick={() => void onExport()}>
            Export JSONL
          </button>
        </div>
        <Show when={audit().length > 0} fallback={<p class={css.note}>No audit entries.</p>}>
          <div class={css.tableWrap}>
            <table class={css.table}>
              <thead>
                <tr>
                  <th>Time</th>
                  <th>Actor</th>
                  <th>Action</th>
                  <th>Target</th>
                </tr>
              </thead>
              <tbody>
                <For each={audit()}>
                  {(a) => (
                    <tr>
                      <td class={css.mono}>{a.ts}</td>
                      <td>
                        {a.actor} <span class={css.badge}>{a.actorKind}</span>
                      </td>
                      <td>{a.action}</td>
                      <td>{a.target ?? '—'}</td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </Show>
      </div>

      <div class={css.card}>
        <h3 class={css.heading}>Login monitor / ban list</h3>
        <form onSubmit={(e) => void onAddBan(e)} aria-label="Add ban" class={css.grid}>
          <label class="field">
            <span>IP address</span>
            <input type="text" value={banIp()} onInput={(e) => setBanIp(e.currentTarget.value)} />
          </label>
          <label class="field">
            <span>Reason</span>
            <input type="text" value={banReason()} onInput={(e) => setBanReason(e.currentTarget.value)} />
          </label>
          <button type="submit" class="btn btn--primary">
            Ban IP
          </button>
        </form>
        <Show when={bans().length > 0} fallback={<p class={css.note}>No active bans.</p>}>
          <For each={bans()}>
            {(b) => (
              <div class={css.listRow}>
                <div>
                  <span class={css.mono}>{b.ip}</span> <span class={css.note}>{b.reason}</span>
                </div>
                <button type="button" class="btn btn--ghost" aria-label={`Unban ${b.ip}`} onClick={() => void onUnban(b.ip)}>
                  Unban
                </button>
              </div>
            )}
          </For>
        </Show>
      </div>
    </section>
  );
}
