// Admin › Dashboard, the console's landing page (t32-admin.md §2.2, t32-ext.md §K).
//
// Real data only. Every figure comes from `GET /admin/dashboard`; a figure the
// server cannot read in its mode or platform arrives as `null` and is shown as
// "Not available" with the reason. Until the endpoint exists (stage A2, t32-e13)
// the page says the figures are not available yet and shows no numbers at all.
//
// Not on this page because the server cannot read them honestly: quota usage
// (nothing measures it), backups (host-side), recent error log lines (log viewer).

import { For, Match, Show, Switch, createResource, createSignal, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { useShell } from '../shell/context.ts';
import { findSection, SECTIONS } from '../shell/sections.ts';
import { hrefFor, sectionPath } from '../shell/router.ts';
import { StatTile } from '../shell/StatTile.tsx';
import { Sparkline } from '../shell/Sparkline.tsx';
import { BarChart } from '../shell/BarChart.tsx';
import { Feedback } from '../shell/Feedback.tsx';
import { Icon } from '../shell/icons.tsx';
import type { Capability } from '../shell/can.ts';
import type { ChecklistItem, Dashboard as DashboardData, DashboardHealth } from '../../../api/admin/dashboard.ts';
import { formatBytes, formatCount, formatDateTime, formatDuration, formatMs, formatTime } from './format.ts';
import * as shell from '../shell/shell.css.ts';
import * as admin from '../admin.css.ts';
import * as css from './dashboard.css.ts';

/** Follow a plain click in place; modified clicks open a new tab. */
function inPlace(e: MouseEvent, go: () => void): void {
  if (e.button !== 0 || e.ctrlKey || e.metaKey || e.shiftKey || e.altKey) return;
  e.preventDefault();
  go();
}

/** Warning code → its sentence and the section that fixes it. */
const WARNINGS: Record<string, { text: (count: number) => string; section: string | null }> = {
  'blocked-accounts': { text: (count) => t('admin-dash-warn-blocked-accounts', { count }), section: 'domains' },
  'no-trusted-proxy-with-ip-rules': { text: (count) => t('admin-dash-warn-no-trusted-proxy', { count }), section: 'access' },
  'allowlist-bypass-env': { text: (count) => t('admin-dash-warn-allowlist-bypass', { count }), section: 'access' },
  'metrics-token-unset': { text: (count) => t('admin-dash-warn-metrics-token', { count }), section: 'observability' },
  'pending-migrations': { text: (count) => t('admin-dash-warn-pending-migrations', { count }), section: null },
  'upstream-cert-expiring': { text: (count) => t('admin-dash-warn-cert-expiring', { count }), section: 'domains' },
};

const CHECKLIST: Record<string, () => string> = {
  'domain-exists': () => t('admin-dash-check-domain-exists'),
  'domain-upstream': () => t('admin-dash-check-domain-upstream'),
  'test-connection': () => t('admin-dash-check-test-connection'),
  'preferred-domain': () => t('admin-dash-check-preferred-domain'),
  'second-admin': () => t('admin-dash-check-second-admin'),
  'trusted-proxies': () => t('admin-dash-check-trusted-proxies'),
  'security-policy': () => t('admin-dash-check-security-policy'),
  'metrics-token': () => t('admin-dash-check-metrics-token'),
  'branding-reviewed': () => t('admin-dash-check-branding'),
};

const HIDDEN_KEY = (user: string): string => `mw.admin.checklist.hidden.${user}`;

function readHidden(user: string): string[] {
  try {
    const raw = localStorage.getItem(HIDDEN_KEY(user));
    const v: unknown = raw === null ? [] : JSON.parse(raw);
    return Array.isArray(v) ? v.filter((x): x is string => typeof x === 'string') : [];
  } catch {
    return [];
  }
}

function writeHidden(user: string, ids: string[]): void {
  try {
    localStorage.setItem(HIDDEN_KEY(user), JSON.stringify(ids));
  } catch {
    /* private mode: hiding lasts for this page only */
  }
}

export function Dashboard(): JSX.Element {
  const shellCtx = useShell();
  const [data, { refetch }] = createResource(() => shellCtx.services.loadDashboard());

  /** A link to a section, only when the section exists and the admin may open it. */
  function sectionLink(id: string | null, rest: string[] = []): string | null {
    if (id === null) return null;
    const s = findSection(id, SECTIONS);
    if (!s || !shellCtx.can.can(s.capability as Capability)) return null;
    return sectionPath(id, rest);
  }
  const open = (path: string): void => void shellCtx.router.navigate(path);

  return (
    <section class={css.page} aria-labelledby="admin-dash-title">
      <div class={css.head}>
        <h2 id="admin-dash-title" class={admin.heading}>
          {t('admin-shell-nav-dashboard')}
        </h2>
        <button type="button" class="btn btn--ghost" onClick={() => void refetch()} disabled={data.loading}>
          {t('admin-dash-refresh')}
        </button>
      </div>
      <Switch>
        <Match when={data.loading && data() === undefined}>
          <p class={shell.state} role="status">
            <span class={shell.busy} aria-hidden="true" /> {t('admin-shell-loading')}
          </p>
        </Match>
        <Match when={data()?.kind === 'unavailable'}>
          <div class={shell.state} data-testid="dash-unavailable">
            <Feedback kind="info" message={t('admin-dash-unavailable')} />
            <p class={admin.note}>{t('admin-dash-unavailable-detail')}</p>
          </div>
        </Match>
        <Match when={data()?.kind === 'error' || data.error}>
          <div class={[shell.state, shell.stateError].join(' ')} data-testid="dash-error">
            <Feedback
              kind="error"
              message={(() => {
                const r = data();
                const status = r?.kind === 'error' ? r.status : null;
                return status === null ? t('admin-dash-error-network') : t('admin-dash-error-status', { status });
              })()}
            />
            <button type="button" class="btn btn--ghost" onClick={() => void refetch()}>
              {t('admin-shell-retry')}
            </button>
          </div>
        </Match>
        <Match when={(() => { const r = data(); return r?.kind === 'ok' ? r.data : undefined; })()}>
          {(d) => <DashboardBody data={d()} sectionLink={sectionLink} open={open} username={shellCtx.username()} />}
        </Match>
      </Switch>
    </section>
  );
}

interface BodyProps {
  data: DashboardData;
  sectionLink: (id: string | null, rest?: string[]) => string | null;
  open: (path: string) => void;
  username: string;
}

function DashboardBody(props: BodyProps): JSX.Element {
  const d = (): DashboardData => props.data;
  const proxy = (): boolean => d().server.mode === 'proxy';
  const reasonProxy = (): string => (proxy() ? t('admin-dash-na-proxy') : t('admin-dash-na-unreported'));

  return (
    <>
      <Show when={d().warnings.length > 0}>
        <section class={css.panel} aria-labelledby="admin-dash-warnings">
          <h3 id="admin-dash-warnings" class={css.panelTitle}>
            {t('admin-dash-warnings')}
          </h3>
          <ul class={css.list}>
            <For each={d().warnings}>
              {(w) => {
                const known = WARNINGS[w.code];
                const link = props.sectionLink(known?.section ?? null);
                return (
                  <li class={css.warning} data-warning={w.code}>
                    <Icon name="alert" />
                    <span>{known ? known.text(w.count) : t('admin-dash-warn-unknown', { code: w.code, count: w.count })}</span>
                    <Show when={link}>
                      {(href) => (
                        <a href={hrefFor(href())} onClick={(e) => inPlace(e, () => props.open(href()))}>
                          {t('admin-dash-warn-fix')}
                        </a>
                      )}
                    </Show>
                  </li>
                );
              }}
            </For>
          </ul>
        </section>
      </Show>

      <Show when={d().blockedAccounts.total > 0}>
        <BlockedAccounts data={d()} sectionLink={props.sectionLink} open={props.open} />
      </Show>

      <Show when={d().checklist && d().checklist!.length > 0}>
        <Checklist items={d().checklist!} open={props.open} username={props.username} />
      </Show>

      <section class={css.panel} aria-labelledby="admin-dash-overview">
        <h3 id="admin-dash-overview" class={css.panelTitle}>
          {t('admin-dash-overview')}
        </h3>
        <div class={shell.tiles}>
          <StatTile testId="tile-users" label={t('admin-dash-users')} value={d().counts.provisionedUsers} format={formatCount} />
          <StatTile label={t('admin-dash-signed-in-24h')} value={d().counts.accountsSignedIn24h} format={formatCount} />
          <StatTile label={t('admin-dash-signed-in-7d')} value={d().counts.accountsSignedIn7d} format={formatCount} />
          <StatTile
            label={t('admin-dash-domains')}
            value={d().counts.domains}
            format={formatCount}
            detail={t('admin-dash-domains-detail', {
              upstream: d().counts.domainsWithUpstream,
              disabled: d().counts.domainsDisabled,
            })}
          />
          <StatTile label={t('admin-dash-sessions')} value={d().counts.activeSessions} format={formatCount} />
          <StatTile label={t('admin-dash-admin-sessions')} value={d().counts.activeAdminSessions} format={formatCount} />
        </div>
      </section>

      <section class={css.panel} aria-labelledby="admin-dash-security">
        <h3 id="admin-dash-security" class={css.panelTitle}>
          {t('admin-dash-security')}
        </h3>
        <div class={shell.tiles}>
          <StatTile label={t('admin-dash-failed-webmail')} value={d().security.failedLogins24h.webmail} format={formatCount} />
          <StatTile label={t('admin-dash-failed-admin')} value={d().security.failedLogins24h.admin} format={formatCount} />
          <StatTile label={t('admin-dash-active-blocks')} value={d().security.activeBlocks} format={formatCount} />
          <StatTile
            label={t('admin-dash-allowlists')}
            value={t('admin-dash-allowlists-value', {
              admin: d().security.allowlists.admin === 'on' ? t('admin-dash-on') : t('admin-dash-off'),
              webmail: d().security.allowlists.webmail === 'on' ? t('admin-dash-on') : t('admin-dash-off'),
            })}
          />
          <StatTile
            label={t('admin-dash-trusted-proxies')}
            value={d().security.trustedProxiesConfigured ? t('admin-dash-configured') : t('admin-dash-not-configured')}
            detail={t('admin-dash-untrusted-forwarded', { count: d().security.forwardedFromUntrustedPeer24h })}
          />
        </div>
      </section>

      <section class={css.panel} aria-labelledby="admin-dash-traffic">
        <h3 id="admin-dash-traffic" class={css.panelTitle}>
          {t('admin-dash-traffic', { mins: d().traffic.windowMins })}
        </h3>
        <p class={admin.note}>{t('admin-dash-traffic-note')}</p>
        <div class={css.charts}>
          <Sparkline
            testId="spark-requests"
            title={t('admin-dash-requests')}
            points={d().traffic.series.map((p) => ({ t: p.t, value: p.requests }))}
            format={formatCount}
            formatTime={formatTime}
            emptyText={t('admin-dash-traffic-empty')}
          />
          <Sparkline
            testId="spark-errors"
            title={t('admin-dash-errors')}
            tone="danger"
            points={d().traffic.series.map((p) => ({ t: p.t, value: p.errors5xx }))}
            format={formatCount}
            formatTime={formatTime}
            emptyText={t('admin-dash-traffic-empty')}
          />
          <Sparkline
            testId="spark-p95"
            title={t('admin-dash-p95')}
            points={d().traffic.series.map((p) => ({ t: p.t, value: p.p95Ms }))}
            format={formatMs}
            formatTime={formatTime}
            emptyText={t('admin-dash-traffic-empty')}
          />
        </div>
      </section>

      <Health health={d().health} reasonProxy={reasonProxy()} />

      <section class={css.panel} aria-labelledby="admin-dash-storage">
        <h3 id="admin-dash-storage" class={css.panelTitle}>
          {t('admin-dash-storage')}
        </h3>
        <div class={shell.tiles}>
          <StatTile label={t('admin-dash-db-size')} value={d().storage.dbBytes} format={formatBytes} />
          <StatTile
            label={t('admin-dash-wal-size')}
            value={d().storage.walBytes}
            format={formatBytes}
            unavailableReason={t('admin-dash-na-wal')}
          />
          <StatTile
            label={t('admin-dash-index-size')}
            value={d().storage.searchIndexBytes}
            format={formatBytes}
            unavailableReason={t('admin-dash-na-index')}
          />
          <StatTile
            testId="tile-volume"
            label={t('admin-dash-volume-free')}
            value={d().storage.dataVolume?.freeBytes ?? null}
            format={formatBytes}
            detail={
              d().storage.dataVolume
                ? t('admin-dash-volume-of', { total: formatBytes(d().storage.dataVolume!.totalBytes) })
                : undefined
            }
            unavailableReason={t('admin-dash-na-volume')}
          />
        </div>
      </section>

      <section class={css.panel} aria-labelledby="admin-dash-audit">
        <h3 id="admin-dash-audit" class={css.panelTitle}>
          {t('admin-dash-recent-audit')}
        </h3>
        <Show
          when={d().recentAudit.length > 0}
          fallback={<p class={admin.note}>{t('admin-dash-audit-empty')}</p>}
        >
          <div class={admin.tableWrap}>
            <table class={admin.table}>
              <caption class={shell.srOnly}>{t('admin-dash-recent-audit')}</caption>
              <thead>
                <tr>
                  <th scope="col">{t('admin-dash-audit-when')}</th>
                  <th scope="col">{t('admin-dash-audit-actor')}</th>
                  <th scope="col">{t('admin-dash-audit-action')}</th>
                  <th scope="col">{t('admin-dash-audit-target')}</th>
                </tr>
              </thead>
              <tbody>
                <For each={d().recentAudit.slice(0, 10)}>
                  {(row) => (
                    <tr>
                      <td>{formatDateTime(row.ts)}</td>
                      <td>{row.actor}</td>
                      <td class={admin.mono}>{row.action}</td>
                      <td>{row.target}</td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </Show>
        <Show when={props.sectionLink('observability')}>
          {(href) => (
            <p class={admin.note}>
              <a class={css.link} href={hrefFor(href())} onClick={(e) => inPlace(e, () => props.open(href()))}>
                {t('admin-dash-audit-all')}
              </a>
            </p>
          )}
        </Show>
      </section>

      <section class={css.panel} aria-labelledby="admin-dash-server">
        <h3 id="admin-dash-server" class={css.panelTitle}>
          {t('admin-dash-server')}
        </h3>
        <dl class={css.facts}>
          <dt>{t('admin-dash-version')}</dt>
          <dd>{d().server.version}</dd>
          <dt>{t('admin-dash-build')}</dt>
          <dd class={admin.mono}>{d().server.buildSha ?? t('admin-shell-not-available')}</dd>
          <dt>{t('admin-dash-mode')}</dt>
          <dd>{proxy() ? t('admin-dash-mode-proxy') : t('admin-dash-mode-engine')}</dd>
          <dt>{t('admin-dash-uptime')}</dt>
          <dd>{formatDuration(d().server.uptimeSecs)}</dd>
          <dt>{t('admin-dash-base-path')}</dt>
          <dd class={admin.mono}>{d().server.basePath === '' ? t('admin-dash-base-root') : d().server.basePath}</dd>
        </dl>
      </section>
    </>
  );
}

function BlockedAccounts(props: {
  data: DashboardData;
  sectionLink: BodyProps['sectionLink'];
  open: BodyProps['open'];
}): JSX.Element {
  const b = (): DashboardData['blockedAccounts'] => props.data.blockedAccounts;
  return (
    <section class={[css.panel, css.alertPanel].join(' ')} aria-labelledby="admin-dash-blocked" data-testid="dash-blocked">
      <h3 id="admin-dash-blocked" class={css.panelTitle}>
        {t('admin-dash-blocked-title', { count: b().total })}
      </h3>
      <p class={admin.note}>{t('admin-dash-blocked-note')}</p>
      <BarChart
        title={t('admin-dash-blocked-chart')}
        bars={b().byDomain.map((r) => ({ label: r.domain, value: r.accounts }))}
        format={formatCount}
      />
      <ul class={css.list}>
        <For each={b().byDomain}>
          {(row) => {
            const link = props.sectionLink('domains', [row.domain]);
            return (
              <li class={css.blockedRow}>
                <span>
                  <strong>{row.domain}</strong>{' '}
                  {t('admin-dash-blocked-row', { count: row.accounts })}
                  <Show when={row.seenHost}>
                    {(host) => <span class={admin.note}> {t('admin-dash-blocked-seen', { host: host() })}</span>}
                  </Show>
                </span>
                <Show
                  when={link}
                  fallback={<span class={admin.note}>{t('admin-dash-blocked-no-access')}</span>}
                >
                  {(href) => (
                    <a
                      class="btn btn--primary"
                      href={hrefFor(href())}
                      onClick={(e) => inPlace(e, () => props.open(href()))}
                    >
                      {t('admin-dash-configure-domain', { domain: row.domain })}
                    </a>
                  )}
                </Show>
              </li>
            );
          }}
        </For>
      </ul>
    </section>
  );
}

function Checklist(props: { items: ChecklistItem[]; open: (path: string) => void; username: string }): JSX.Element {
  const [hidden, setHidden] = createSignal<string[]>(readHidden(props.username));
  const [showHidden, setShowHidden] = createSignal(false);
  const shown = (): ChecklistItem[] => props.items.filter((i) => showHidden() || !hidden().includes(i.id));
  const openCount = (): number => props.items.filter((i) => !i.done && !i.optional).length;

  function toggleHide(id: string): void {
    const next = hidden().includes(id) ? hidden().filter((x) => x !== id) : [...hidden(), id];
    setHidden(next);
    writeHidden(props.username, next);
  }

  return (
    <section class={css.panel} aria-labelledby="admin-dash-checklist" data-testid="dash-checklist">
      <h3 id="admin-dash-checklist" class={css.panelTitle}>
        {t('admin-dash-checklist', { open: openCount() })}
      </h3>
      <ul class={css.list}>
        <For each={shown()}>
          {(item) => (
            <li class={css.checkRow} data-check={item.id} data-done={item.done ? 'true' : 'false'}>
              <span class={item.done ? css.checkDone : css.checkOpen}>
                <Icon name={item.done ? 'check' : 'info'} />
                <span>{item.done ? t('admin-dash-check-done') : t('admin-dash-check-open')}</span>
              </span>
              <span class={css.checkLabel}>
                {(CHECKLIST[item.id] ?? (() => item.id))()}
                <Show when={item.optional}>
                  <span class={admin.note}> {t('admin-dash-check-optional')}</span>
                </Show>
              </span>
              <Show when={!item.done && item.href.startsWith('/admin')}>
                <a href={hrefFor(item.href)} onClick={(e) => inPlace(e, () => props.open(item.href))}>
                  {t('admin-dash-check-go')}
                </a>
              </Show>
              <button type="button" class="btn btn--ghost" onClick={() => toggleHide(item.id)}>
                {hidden().includes(item.id) ? t('admin-dash-check-unhide') : t('admin-dash-check-hide')}
              </button>
            </li>
          )}
        </For>
      </ul>
      <Show when={hidden().length > 0}>
        <p class={admin.note}>
          {t('admin-dash-check-hidden', { count: hidden().length })}{' '}
          <button type="button" class="btn btn--ghost" onClick={() => setShowHidden(!showHidden())}>
            {showHidden() ? t('admin-dash-check-hide-hidden') : t('admin-dash-check-show-hidden')}
          </button>
        </p>
      </Show>
    </section>
  );
}

function Health(props: { health: DashboardHealth; reasonProxy: string }): JSX.Element {
  const h = (): DashboardHealth => props.health;
  type Row = { label: string; ok: boolean | null; text: string };
  const rows = (): Row[] => {
    const m = h().migration;
    const e = h().engine;
    const o = h().outbox;
    const s = h().searchIndex;
    return [
      { label: t('admin-dash-h-db'), ok: true, text: t('admin-dash-h-db-ok', { ms: formatMs(h().dbPingMs) }) },
      {
        label: t('admin-dash-h-migrations'),
        ok: m.applied === m.expected,
        text:
          m.applied === m.expected
            ? t('admin-dash-h-migrations-ok', { n: m.applied })
            : t('admin-dash-h-migrations-behind', { applied: m.applied, expected: m.expected }),
      },
      {
        label: t('admin-dash-h-engine'),
        ok: e === null ? null : e.accountsWithSyncError === 0,
        text:
          e === null
            ? `${t('admin-shell-not-available')}: ${props.reasonProxy}`
            : t('admin-dash-h-engine-value', { accounts: e.registeredAccounts, errors: e.accountsWithSyncError }),
      },
      {
        label: t('admin-dash-h-outbox'),
        ok: o === null ? null : o.failed24h === 0,
        text:
          o === null
            ? `${t('admin-shell-not-available')}: ${props.reasonProxy}`
            : t('admin-dash-h-outbox-value', {
                pending: o.pending,
                failed: o.failed24h,
                oldest: o.oldestAgeSecs === null ? t('admin-dash-h-outbox-none') : formatDuration(o.oldestAgeSecs),
              }),
      },
      {
        label: t('admin-dash-h-index'),
        ok: s === null ? null : !s.rebuilding,
        text:
          s === null
            ? `${t('admin-shell-not-available')}: ${t('admin-dash-na-index')}`
            : s.rebuilding
              ? t('admin-dash-h-index-rebuilding', { n: s.documents })
              : t('admin-dash-h-index-value', { n: s.documents }),
      },
      {
        label: t('admin-dash-h-sandbox'),
        ok: h().renderSandbox === 'available',
        text:
          h().renderSandbox === 'available'
            ? t('admin-dash-h-sandbox-ok')
            : h().renderSandbox === 'missing'
              ? t('admin-dash-h-sandbox-missing')
              : t('admin-dash-h-sandbox-required'),
      },
      {
        label: t('admin-dash-h-plugins'),
        ok: h().pluginRuntime === 'available' ? true : null,
        text: h().pluginRuntime === 'available' ? t('admin-dash-h-plugins-ok') : t('admin-dash-h-plugins-off'),
      },
    ];
  };
  return (
    <section class={css.panel} aria-labelledby="admin-dash-health">
      <h3 id="admin-dash-health" class={css.panelTitle}>
        {t('admin-dash-health')}
      </h3>
      <dl class={css.facts} data-testid="dash-health">
        <For each={rows()}>
          {(r) => (
            <>
              <dt>{r.label}</dt>
              <dd class={css.healthValue}>
                <Show when={r.ok !== null}>
                  <span class={r.ok ? css.healthOk : css.healthBad}>
                    <Icon name={r.ok ? 'check' : 'alert'} />
                    <span class={shell.srOnly}>{r.ok ? t('admin-dash-h-ok') : t('admin-dash-h-attention')}</span>
                  </span>
                </Show>
                <span>{r.text}</span>
              </dd>
            </>
          )}
        </For>
      </dl>
    </section>
  );
}
