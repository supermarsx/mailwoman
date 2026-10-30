// Admin › Dashboard › Health (t103).
//
// Every row is a server probe: a status (OK / Warning / Failing / Disabled), the
// server's reason, then the numbers it measured. A disabled component says how
// to turn it on. "Check again" reloads this panel alone from
// `GET /admin/dashboard/health?fresh=1`; the rest of the page stays put.
// Durations read in the unit that fits (μs below 1 ms, t90's formatter).

import { For, Show, createEffect, createSignal, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { Icon, type IconName } from '../shell/icons.tsx';
import { Sparkline, type SparkPoint } from '../shell/Sparkline.tsx';
import { hrefFor } from '../shell/router.ts';
import {
  HEALTH_COMPONENTS,
  createHealthLoader,
  list,
  num,
  obj,
  str,
  type DashboardHealth,
  type HealthComponent,
  type HealthComponentId,
  type HealthLoader,
  type HealthSample,
  type HealthStatus,
} from '../../../api/admin/health.ts';
import { formatBytes, formatCount, formatDuration, formatMs, formatMsSet, formatTime } from './format.ts';
import * as dash from './dashboard.css.ts';
import * as css from './health.css.ts';
import * as shell from '../shell/shell.css.ts';

const LABEL: Record<HealthComponentId, () => string> = {
  database: () => t('admin-dash-h-db'),
  migrations: () => t('admin-dash-h-migrations'),
  sync: () => t('admin-dash-h-engine'),
  outbox: () => t('admin-dash-h-outbox'),
  search: () => t('admin-dash-h-index'),
  render: () => t('admin-dash-h-sandbox'),
  plugins: () => t('admin-dash-h-plugins'),
};

const STATUS_ICON: Record<HealthStatus, IconName> = { ok: 'check', warn: 'alert', fail: 'close', disabled: 'minus' };
const STATUS_TEXT: Record<HealthStatus, () => string> = {
  ok: () => t('admin-dash-h-status-ok'),
  warn: () => t('admin-dash-h-status-warn'),
  fail: () => t('admin-dash-h-status-fail'),
  disabled: () => t('admin-dash-h-status-disabled'),
};

const statusText = (s: HealthStatus): string => (STATUS_TEXT[s] ?? STATUS_TEXT.fail)();
const statusIcon = (s: HealthStatus): IconName => STATUS_ICON[s] ?? 'alert';
const chipClass = (s: HealthStatus): string => css.chip[s] ?? css.chip.fail;

/** Related pages: section id and its link label. */
const LINKS: Record<HealthComponentId, { section: string; label: () => string }[]> = {
  database: [{ section: 'diagnostics', label: () => t('admin-dash-h-open-diagnostics') }],
  migrations: [{ section: 'diagnostics', label: () => t('admin-dash-h-open-diagnostics') }],
  sync: [{ section: 'logs', label: () => t('admin-dash-h-open-logs') }, { section: 'diagnostics', label: () => t('admin-dash-h-open-diagnostics') }],
  outbox: [{ section: 'logs', label: () => t('admin-dash-h-open-logs') }, { section: 'diagnostics', label: () => t('admin-dash-h-open-diagnostics') }],
  search: [{ section: 'maintenance', label: () => t('admin-dash-h-open-maintenance') }],
  render: [{ section: 'diagnostics', label: () => t('admin-dash-h-open-diagnostics') }, { section: 'logs', label: () => t('admin-dash-h-open-logs') }],
  plugins: [{ section: 'plugins', label: () => t('admin-dash-h-open-plugins') }, { section: 'logs', label: () => t('admin-dash-h-open-logs') }],
};

type Stat = { label: string; value: string | string[] };

const none = (): string => t('admin-dash-h-m-none');
const n = (v: number | null): string => (v === null ? '—' : formatCount(v));
const ms = (v: number | null): string => (v === null ? '—' : formatMs(v));
/** An age in seconds: sub-second ages read in ms or μs, longer ones in s/min/h/d. */
const age = (secs: number | null): string => (secs === null ? '—' : secs < 1 ? formatMs(secs * 1000) : formatDuration(secs));
const ago = (secs: number | null): string => (secs === null ? '—' : t('admin-dash-h-m-ago', { age: age(secs) }));

function migrationList(m: Record<string, unknown>, key: string): string {
  const rows = list(m, key);
  return rows.length === 0 ? none() : rows.map((r) => `${String(r.version)} ${String(r.description ?? '')}`.trim()).join(', ');
}

/** The stat rows for one component, from its metrics. */
export function statsFor(id: HealthComponentId, c: HealthComponent): Stat[] {
  const m = c.metrics;
  if (c.status === 'disabled' && id !== 'plugins') return [];
  switch (id) {
    case 'database': {
      const rows: Stat[] = [
        { label: t('admin-dash-h-m-ping'), value: ms(num(m, 'pingMs')) },
        {
          label: t('admin-dash-h-m-pool'),
          value: t('admin-dash-h-m-pool-value', { inUse: n(num(m, 'poolInUse')), idle: n(num(m, 'poolIdle')), size: n(num(m, 'poolSize')) }),
        },
        { label: t('admin-dash-h-m-engine'), value: `${str(m, 'backend') ?? '—'} ${str(m, 'serverVersion') ?? ''}`.trim() },
      ];
      const place = [str(m, 'name'), str(m, 'host') && `${str(m, 'host')}${num(m, 'port') ? `:${num(m, 'port')}` : ''}`].filter(Boolean).join(' @ ');
      if (place) rows.push({ label: t('admin-dash-h-m-location'), value: place });
      if (num(m, 'dbBytes') !== null) rows.push({ label: t('admin-dash-h-m-db-size'), value: formatBytes(num(m, 'dbBytes')!) });
      if (num(m, 'walBytes') !== null) rows.push({ label: t('admin-dash-h-m-wal-size'), value: formatBytes(num(m, 'walBytes')!) });
      if (str(m, 'journalMode')) rows.push({ label: t('admin-dash-h-m-journal'), value: str(m, 'journalMode')! });
      rows.push({
        label: t('admin-dash-h-m-slow'),
        value: num(m, 'slowQueriesLastHour') === null ? (c.notes?.slowQueriesLastHour ?? t('admin-dash-h-not-tracked')) : n(num(m, 'slowQueriesLastHour')),
      });
      return rows;
    }
    case 'migrations': {
      const latest = obj(m, 'latest');
      return [
        { label: t('admin-dash-h-m-applied'), value: t('admin-dash-h-m-applied-value', { applied: n(num(m, 'applied')), expected: n(num(m, 'expected')) }) },
        { label: t('admin-dash-h-m-latest'), value: latest.version === undefined ? none() : `${String(latest.version)} ${String(latest.description ?? '')}`.trim() },
        { label: t('admin-dash-h-m-pending'), value: migrationList(m, 'pending') },
        ...(list(m, 'failed').length ? [{ label: t('admin-dash-h-m-failed'), value: migrationList(m, 'failed') }] : []),
        ...(list(m, 'unknown').length ? [{ label: t('admin-dash-h-m-unknown'), value: migrationList(m, 'unknown') }] : []),
      ];
    }
    case 'sync': {
      const rows: Stat[] = [
        {
          label: t('admin-dash-h-m-accounts'),
          value: t('admin-dash-h-m-accounts-value', {
            total: n(num(m, 'accounts')),
            syncing: n(num(m, 'syncing')),
            idle: n(num(m, 'idle')),
            erroring: n(num(m, 'erroring')),
          }),
        },
        {
          label: t('admin-dash-h-m-last-sync'),
          value:
            num(m, 'lastSyncAgeMaxSecs') === null
              ? none()
              : t('admin-dash-h-m-last-sync-value', { p50: age(num(m, 'lastSyncAgeP50Secs')), max: age(num(m, 'lastSyncAgeMaxSecs')) }),
        },
        { label: t('admin-dash-h-m-never-synced'), value: n(num(m, 'neverSynced')) },
        { label: t('admin-dash-h-m-queue'), value: n(num(m, 'queueDepth')) },
        { label: t('admin-dash-h-m-errors-hour'), value: n(num(m, 'errorsLastHour')) },
      ];
      const kinds = Object.entries(obj(m, 'byKind'));
      rows.push({
        label: t('admin-dash-h-m-by-kind'),
        value: kinds.length === 0
          ? none()
          : kinds.map(([kind, v]) => {
              const k = v as Record<string, unknown>;
              return t('admin-dash-h-m-kind-value', { kind: kind.toUpperCase(), accounts: n(num(k, 'accounts')), watch: n(num(k, 'watchConnections')) });
            }),
      });
      const errors = list(m, 'recentErrors');
      if (errors.length) {
        rows.push({ label: t('admin-dash-h-m-recent-errors'), value: errors.map((e) => `${String(e.kind ?? '').toUpperCase()}: ${String(e.error ?? '')} (${ago(num(e, 'ageSecs'))})`) });
      }
      return rows;
    }
    case 'outbox': {
      const sends = obj(m, 'sendsLastHour');
      const last = obj(m, 'lastSend');
      const sub = obj(m, 'submission');
      const outcome = str(last, 'outcome');
      const rows: Stat[] = [
        { label: t('admin-dash-h-m-queued'), value: t('admin-dash-h-m-queued-value', { queued: n(num(m, 'queued')), deferred: n(num(m, 'deferred')), sending: n(num(m, 'sending')) }) },
        { label: t('admin-dash-h-m-oldest'), value: num(m, 'oldestDueAgeSecs') === null ? none() : age(num(m, 'oldestDueAgeSecs')) },
        { label: t('admin-dash-h-m-scheduled'), value: t('admin-dash-h-m-scheduled-value', { scheduled: n(num(m, 'scheduled')), held: n(num(m, 'held')) }) },
        { label: t('admin-dash-h-m-failed-total'), value: t('admin-dash-h-m-failed-total-value', { failed: n(num(m, 'failed')), uncertain: n(num(m, 'uncertain')) }) },
        {
          label: t('admin-dash-h-m-sends-hour'),
          value: t('admin-dash-h-m-sends-hour-value', { sent: n(num(sends, 'sent')), deferred: n(num(sends, 'deferred')), failed: n((num(sends, 'failed') ?? 0) + (num(sends, 'uncertain') ?? 0)) }),
        },
        {
          label: t('admin-dash-h-m-last-send'),
          value: outcome === null
            ? t('admin-dash-h-m-last-send-none')
            : [t('admin-dash-h-m-last-send-value', { outcome: outcomeText(outcome), age: age(num(last, 'ageSecs')) }), ...(str(last, 'error') ? [str(last, 'error')!] : [])].join(' — '),
        },
      ];
      rows.push({
        label: t('admin-dash-h-m-submission'),
        value: num(sub, 'configured') === null
          ? t('admin-dash-h-m-submission-pending')
          : num(sub, 'configured') === 0
            ? t('admin-dash-h-m-submission-none')
            : t('admin-dash-h-m-submission-value', {
                reachable: n(num(sub, 'reachable')),
                checked: n(num(sub, 'checked')),
                slowest: ms(num(sub, 'slowestHandshakeMs')),
                age: age(num(sub, 'ageSecs')),
              }),
      });
      return rows;
    }
    case 'search': {
      const progress = num(m, 'rebuildProgressPct');
      return [
        { label: t('admin-dash-h-m-documents'), value: t('admin-dash-h-m-documents-value', { documents: n(num(m, 'documents')), messages: n(num(m, 'indexableMessages')) }) },
        { label: t('admin-dash-h-m-index-pending'), value: n(num(m, 'pending')) },
        { label: t('admin-dash-h-m-index-size'), value: m.persistent === false ? t('admin-dash-h-m-index-memory') : num(m, 'bytes') === null ? '—' : formatBytes(num(m, 'bytes')!) },
        { label: t('admin-dash-h-m-last-commit'), value: m.persistent === false ? t('admin-dash-h-m-index-memory') : ago(num(m, 'lastCommitAgeSecs')) },
        {
          label: t('admin-dash-h-m-rebuild'),
          value: m.rebuilding === true
            ? t('admin-dash-h-m-rebuild-running', { pct: progress === null ? '—' : `${Math.round(progress)} %`, done: n(num(m, 'rebuildDone')), total: n(num(m, 'rebuildTotal')) })
            : t('admin-dash-h-m-rebuild-idle'),
        },
      ];
    }
    case 'render': {
      if (m.workerConfigured === false) {
        return [
          { label: t('admin-dash-h-m-version'), value: t('admin-dash-h-m-worker-missing') },
          { label: t('admin-dash-h-m-jail'), value: m.jailRequired === true ? t('admin-dash-h-m-jail-required') : t('admin-dash-h-m-jail-optional') },
        ];
      }
      return [
        { label: t('admin-dash-h-m-version'), value: str(m, 'version') ?? '—' },
        { label: t('admin-dash-h-m-render-ping'), value: ms(num(m, 'pingMs')) },
        { label: t('admin-dash-h-m-render-jobs'), value: n(num(m, 'inFlight')) },
        {
          label: t('admin-dash-h-m-render-failures'),
          value: t('admin-dash-h-m-render-failures-value', { failed: n(num(m, 'failuresLastHour')), timeouts: n(num(m, 'timeoutsLastHour')), busy: n(num(m, 'busyRefusalsLastHour')) }),
        },
        {
          label: t('admin-dash-h-m-jail'),
          value: str(m, 'jailDegraded')
            ? t('admin-dash-h-m-jail-degraded', { why: str(m, 'jailDegraded')! })
            : m.jailRequired === true ? t('admin-dash-h-m-jail-required') : t('admin-dash-h-m-jail-optional'),
        },
      ];
    }
    case 'plugins': {
      if (num(m, 'installed') === null) return [];
      const plugins = list(m, 'plugins');
      const rows: Stat[] = [
        { label: t('admin-dash-h-m-plugins'), value: t('admin-dash-h-m-plugins-value', { installed: n(num(m, 'installed')), enabled: n(num(m, 'enabled')), loaded: n(num(m, 'loaded')) }) },
      ];
      if (plugins.length) {
        rows.push({
          label: t('admin-dash-h-m-plugin-list'),
          value: plugins.map((p) => {
            const limits = obj(p, 'limits');
            const state = p.loaded === true ? t('admin-dash-h-m-plugin-loaded') : t('admin-dash-h-m-plugin-not-loaded', { reason: String(p.notLoadedReason ?? '—') });
            const lim = t('admin-dash-h-m-plugin-limits', {
              memory: num(limits, 'memoryMb') === null ? t('admin-dash-h-unlimited') : `${formatCount(num(limits, 'memoryMb')!)} MB`,
              deadline: num(limits, 'deadlineMs') === null ? t('admin-dash-h-unlimited') : formatMs(num(limits, 'deadlineMs')!),
              fuel: num(limits, 'fuel') === null ? t('admin-dash-h-unlimited') : formatCount(num(limits, 'fuel')!),
            });
            return `${String(p.name ?? p.id)} ${String(p.version ?? '')} — ${state}; ${lim}`;
          }),
        });
      }
      rows.push({
        label: t('admin-dash-h-m-plugin-calls'),
        value: t('admin-dash-h-m-plugin-calls-value', { traps: n(num(m, 'trapsLastHour')), limits: n(num(m, 'limitTripsLastHour')), errors: n(num(m, 'runtimeErrorsLastHour')) }),
      });
      if (num(m, 'hostPingMs') !== null) rows.push({ label: t('admin-dash-h-m-host-ping'), value: ms(num(m, 'hostPingMs')) });
      return rows;
    }
  }
}

function outcomeText(outcome: string): string {
  switch (outcome) {
    case 'sent': return t('admin-dash-h-outcome-sent');
    case 'deferred': return t('admin-dash-h-outcome-deferred');
    case 'failed': return t('admin-dash-h-outcome-failed');
    case 'uncertain': return t('admin-dash-h-outcome-uncertain');
    default: return outcome;
  }
}

/** One sentence naming the thresholds a component is judged by, or null. */
export function thresholdsFor(id: HealthComponentId, c: HealthComponent): string | null {
  const th = c.thresholds;
  switch (id) {
    case 'database':
      return th.pingWarnMs === undefined ? null : t('admin-dash-h-t-ping', { warn: formatMs(th.pingWarnMs), fail: formatMs(th.pingFailMs ?? 0) });
    case 'outbox':
      return th.oldestDueWarnSecs === undefined ? null : t('admin-dash-h-t-oldest', { warn: formatDuration(th.oldestDueWarnSecs), fail: formatDuration(th.oldestDueFailSecs ?? 0) });
    case 'search':
      return th.pendingWarn === undefined ? null : t('admin-dash-h-t-pending', { n: formatCount(th.pendingWarn) });
    case 'render':
      return th.pingWarnMs === undefined ? null : t('admin-dash-h-t-render', { warn: formatMs(th.pingWarnMs), timeout: formatMs(th.pingTimeoutMs ?? 0) });
    case 'sync':
      return th.erroringFailRatio === undefined ? null : t('admin-dash-h-t-sync', { pct: `${Math.round(th.erroringFailRatio * 100)} %` });
    default:
      return null;
  }
}

function points(history: HealthSample[], key: keyof HealthSample): SparkPoint[] {
  return history.map((s) => ({ t: s.t, value: typeof s[key] === 'number' ? (s[key] as number) : null }));
}

export interface HealthPanelProps {
  health: DashboardHealth;
  sectionLink: (id: string | null, rest?: string[]) => string | null;
  open: (path: string) => void;
  loader?: HealthLoader | undefined;
}

export function HealthPanel(props: HealthPanelProps): JSX.Element {
  const load = props.loader ?? createHealthLoader();
  const [health, setHealth] = createSignal<DashboardHealth>(props.health);
  const [busy, setBusy] = createSignal(false);
  const [failed, setFailed] = createSignal<number | null | undefined>(undefined);
  // A whole-page reload hands in a new result; take it.
  createEffect(() => setHealth(props.health));

  const refresh = async (): Promise<void> => {
    setBusy(true);
    const r = await load(true);
    setBusy(false);
    if (r.kind === 'ok') {
      setHealth(r.data);
      setFailed(undefined);
    } else {
      setFailed(r.status);
    }
  };

  const inPlace = (e: MouseEvent, path: string): void => {
    if (e.button !== 0 || e.ctrlKey || e.metaKey || e.shiftKey || e.altKey) return;
    e.preventDefault();
    props.open(path);
  };

  const history = (): HealthSample[] => health().history ?? [];

  return (
    <section class={dash.panel} aria-labelledby="admin-dash-health" aria-busy={busy() ? 'true' : undefined} data-testid="dash-health">
      <div class={css.head}>
        <h3 id="admin-dash-health" class={dash.panelTitle}>
          {t('admin-dash-health')}
        </h3>
        <span class={css.checked} data-testid="dash-health-checked">
          {t('admin-dash-h-checked', { time: formatTime(health().checkedAt) })}
        </span>
        <button type="button" class="btn btn--ghost" onClick={() => void refresh()} disabled={busy()} data-testid="dash-health-refresh">
          <Icon name="refresh" />
          <span>{t('admin-dash-h-refresh')}</span>
        </button>
      </div>
      <Show when={failed() !== undefined}>
        <p class={css.note} role="status">
          {failed() === null ? t('admin-dash-h-refresh-network') : t('admin-dash-h-refresh-failed', { status: String(failed()) })}
        </p>
      </Show>
      <ul class={css.items}>
        <For each={HEALTH_COMPONENTS.filter((id) => health().components?.[id])}>
          {(id) => {
            const c = (): HealthComponent => health().components[id];
            const links = (): { href: string; label: string }[] =>
              LINKS[id].flatMap((l) => {
                const href = props.sectionLink(l.section);
                return href === null ? [] : [{ href, label: l.label() }];
              });
            return (
              <li class={css.item} data-testid={`health-${id}`} data-status={c().status}>
                <div class={css.itemHead}>
                  <span class={chipClass(c().status)} data-testid={`health-${id}-status`}>
                    <Icon name={statusIcon(c().status)} />
                    <span>{statusText(c().status)}</span>
                  </span>
                  <span class={css.label}>{LABEL[id]()}</span>
                  <span class={css.reason} data-testid={`health-${id}-reason`}>{c().reason}</span>
                  <Show when={links().length > 0}>
                    <span class={css.links}>
                      <For each={links()}>{(l) => <a href={hrefFor(l.href)} onClick={(e) => inPlace(e, l.href)}>{l.label}</a>}</For>
                    </span>
                  </Show>
                </div>
                <Show when={c().enable}>
                  <p class={css.note} data-testid={`health-${id}-enable`}>
                    {t('admin-dash-h-enable')}: {c().enable}
                  </p>
                </Show>
                <Show when={statsFor(id, c()).length > 0}>
                  <dl class={css.stats} data-testid={`health-${id}-stats`}>
                    <For each={statsFor(id, c())}>
                      {(s) => (
                        <>
                          <dt>{s.label}</dt>
                          <dd>
                            {Array.isArray(s.value) ? (
                              <ul>
                                <For each={s.value}>{(v) => <li>{v}</li>}</For>
                              </ul>
                            ) : (
                              s.value
                            )}
                          </dd>
                        </>
                      )}
                    </For>
                  </dl>
                </Show>
                <Show when={thresholdsFor(id, c()) || typeof c().probeUs === 'number'}>
                  <p class={css.note}>
                    {[thresholdsFor(id, c()), typeof c().probeUs === 'number' ? t('admin-dash-h-probe-time', { time: formatMs(c().probeUs! / 1000) }) : null]
                      .filter(Boolean)
                      .join(' · ')}
                  </p>
                </Show>
              </li>
            );
          }}
        </For>
      </ul>
      <Show when={history().length >= 2}>
        <div class={css.charts} data-testid="dash-health-history">
          <span class={shell.srOnly}>{t('admin-dash-h-history', { n: history().length })}</span>
          <Sparkline title={t('admin-dash-h-chart-db')} points={points(history(), 'dbPingMs')} format={formatMs} formatSet={formatMsSet} />
          <Show when={history().some((s) => typeof s.renderPingMs === 'number')}>
            <Sparkline title={t('admin-dash-h-chart-render')} points={points(history(), 'renderPingMs')} format={formatMs} formatSet={formatMsSet} />
          </Show>
          <Show when={history().some((s) => typeof s.outboxQueued === 'number')}>
            <Sparkline title={t('admin-dash-h-chart-outbox')} points={points(history(), 'outboxQueued')} format={formatCount} />
          </Show>
          <Show when={history().some((s) => typeof s.searchPending === 'number')}>
            <Sparkline title={t('admin-dash-h-chart-search')} points={points(history(), 'searchPending')} format={formatCount} />
          </Show>
        </div>
      </Show>
    </section>
  );
}
