import * as ux from '../shell/ux.css.ts';
import { AuditActor } from '../shell/AuditActor.tsx';
import * as auditCss from '../shell/audit.css.ts';
// Admin › Dashboard, the console's landing page (t32-admin.md §2.2, t32-ext.md §K).
//
// Real data only. Every figure comes from `GET /admin/dashboard`; a figure the
// server cannot read in its mode or platform arrives as `null` and is shown as
// "Not available" with the reason. Until the endpoint exists (stage A2, t32-e13)
// the page says the figures are not available yet and shows no numbers at all.
//
// Not on this page because the server cannot read them honestly: quota usage
// (nothing measures it), backups (host-side), recent error log lines (log viewer).

import { For, Match, Show, Switch, createResource, createSignal, createEffect, onCleanup, type JSX } from 'solid-js';
import { AsyncPending } from '../../../components/AsyncState.tsx';
import { resourceValue } from '../../../components/resourceValue.ts';
import { isolate, t } from '../../../i18n';
import { useOptionalPersonalPreferences } from '../shell/personalPreferences.tsx';
import { dashboardPreferences, type DashboardPreferences, type DashboardWidgetId } from './preferences.ts';
import { useShell } from '../shell/context.ts';
import { findSection, SECTIONS } from '../shell/sections.ts';
import { hrefFor, sectionPath } from '../shell/router.ts';
import { isConsoleLocation } from '../../../api/adminPath.ts';
import { StatTile } from '../shell/StatTile.tsx';
import { RequestHistory } from '../RequestHistory.tsx';
import { BarChart } from '../shell/BarChart.tsx';
import { Feedback } from '../shell/Feedback.tsx';
import { Icon } from '../shell/icons.tsx';
import type { Capability } from '../shell/can.ts';
import type { ChecklistItem, Dashboard as DashboardData } from '../../../api/admin/dashboard.ts';
import { HealthPanel } from './HealthPanel.tsx';
import { formatBytes, formatCount, formatDateTime, formatDuration } from './format.ts';
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
const WARNINGS: Record<string, { text: (count: number) => string; section: string | null; rest?: string[] }> = {
  'blocked-accounts': { text: (count) => t('admin-dash-warn-blocked-accounts', { count }), section: 'domains' },
  'no-trusted-proxy-with-ip-rules': { text: (count) => t('admin-dash-warn-no-trusted-proxy', { count }), section: 'access' },
  'allowlist-bypass-env': { text: (count) => t('admin-dash-warn-allowlist-bypass', { count }), section: 'access' },
  'metrics-token-unset': { text: (count) => t('admin-dash-warn-metrics-token', { count }), section: 'observability' },
  'pending-migrations': { text: (count) => t('admin-dash-warn-pending-migrations', { count }), section: null },
  'upstream-cert-expiring': { text: (count) => t('admin-dash-warn-cert-expiring', { count }), section: 'domains' },
  'geoip-database-missing': { text: () => t('admin-dash-warn-geoip-missing'), section: 'security', rest: ['geoip'] },
  'geoip-database-stale': { text: () => t('admin-dash-warn-geoip-stale'), section: 'security', rest: ['geoip'] },
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
  const preferences = useOptionalPersonalPreferences();
  const layout = () => dashboardPreferences(preferences?.data()['dashboard']);
  const [resource, { refetch }] = createResource(() => shellCtx.services.loadDashboard());
  // Never `resource()` (t75): read while the timed refresh or the Refresh button
  // reloads, it suspends the console's page boundary and the whole dashboard is
  // swapped for the page skeleton and back. The last numbers stay while it reloads.
  const current = () => resourceValue(resource);

  createEffect(() => {
    const seconds = layout().refreshSecs;
    if (!seconds) return;
    const timer = setInterval(() => { if (document.visibilityState === 'visible' && !resource.loading) void refetch(); }, seconds * 1000);
    onCleanup(() => clearInterval(timer));
  });

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
        <a class={`btn btn--ghost ${ux.iconAction}`} aria-label={t('admin-dash-customize')} title={t('admin-dash-customize')} href={hrefFor(sectionPath('preferences'))} onClick={e => inPlace(e, () => open(sectionPath('preferences')))}><Icon name="settings"/></a>
        <button type="button" class={`btn btn--ghost ${ux.iconAction}`} aria-label={t('admin-dash-refresh')} title={t('admin-dash-refresh')} onClick={() => void refetch()} disabled={resource.loading}>
          <Icon name="refresh"/>
        </button>
      </div>
      <Switch>
        <Match when={resource.loading && current() === undefined}>
          <div class={shell.state}><AsyncPending variant="page" message={t('admin-shell-loading')} onRetry={() => void refetch()} /></div>
        </Match>
        <Match when={current()?.kind === 'unavailable'}>
          <div class={shell.field} data-testid="dash-unavailable">
            <Feedback kind="info" message={t('admin-dash-unavailable')} />
            <p class={admin.note}>{t('admin-dash-unavailable-detail')}</p>
          </div>
        </Match>
        <Match when={current()?.kind === 'error' || resource.error}>
          <div class={[shell.state, shell.stateError].join(' ')} data-testid="dash-error">
            <Feedback
              kind="error"
              message={(() => {
                const r = current();
                const status = r?.kind === 'error' ? r.status : null;
                return status === null ? t('admin-dash-error-network') : t('admin-dash-error-status', { status });
              })()}
            />
            <button type="button" class="btn btn--ghost" onClick={() => void refetch()}>
              {t('admin-shell-retry')}
            </button>
          </div>
        </Match>
        <Match when={(() => { const r = current(); return r?.kind === 'ok' ? r.data : undefined; })()}>
          {(d) => <DashboardBody data={d()} layout={layout()} sectionLink={sectionLink} open={open} username={shellCtx.username()} />}
        </Match>
      </Switch>
    </section>
  );
}

interface BodyProps {
  layout: DashboardPreferences;
  data: DashboardData;
  sectionLink: (id: string | null, rest?: string[]) => string | null;
  open: (path: string) => void;
  username: string;
}

function Widget(props: { id: DashboardWidgetId; layout: DashboardPreferences; deprioritized?: boolean; children: JSX.Element }): JSX.Element {
  return <Show when={!props.layout.hidden.includes(props.id)}><div data-dashboard-widget={props.id} data-deprioritized={props.deprioritized ? 'true' : undefined} class={props.deprioritized ? css.completedWidget : css.widgetOrder[props.layout.order.indexOf(props.id) as keyof typeof css.widgetOrder]}>{props.children}</div></Show>;
}

function DashboardBody(props: BodyProps): JSX.Element {
  const d = (): DashboardData => props.data;
  const completedChecklist = (): boolean => !!d().checklist?.length && d().checklist!.every(item => item.done) && !props.layout.keepCompletedChecklistProminent;
  const proxy = (): boolean => d().server?.mode === 'proxy';
  const indexReason = (): string => (proxy() || d().health?.components?.search?.status === 'disabled' ? t('admin-dash-na-index-proxy') : t('admin-dash-na-index-memory'));

  return (
    <div class={css.widgets}>
      <Show when={d().unavailable && Object.keys(d().unavailable!).length > 0}>
        <details class={css.panel}>
          <summary class={css.panelTitle}>{t('admin-dash-limitations')}</summary>
          <ul class={css.list}><For each={[...new Set(Object.values(d().unavailable ?? {}))]}>{reason => <li>{reason}</li>}</For></ul>
        </details>
      </Show>
      <Show when={d().warnings.length > 0}>
        <section class={css.panel} aria-labelledby="admin-dash-warnings">
          <h3 id="admin-dash-warnings" class={css.panelTitle}>
            {t('admin-dash-warnings')}
          </h3>
          <ul class={css.list}>
            <For each={d().warnings}>
              {(w) => {
                const known = WARNINGS[w.code];
                const link = props.sectionLink(known?.section ?? null, known?.rest);
                return (
                  <li class={css.warning} data-warning={w.code}>
                    <Icon name="alert" />
                    <span>{known ? known.text(w.count) : t('admin-dash-warn-unknown', { code: isolate(w.code), count: w.count })}</span>
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

      <Show when={(d().blockedAccounts?.total ?? 0) > 0}>
        <BlockedAccounts data={d()} sectionLink={props.sectionLink} open={props.open} />
      </Show>

      <Show when={d().checklist && d().checklist!.length > 0}>
        <Widget id="checklist" layout={props.layout} deprioritized={completedChecklist()}><Checklist items={d().checklist!} open={props.open} username={props.username} deprioritized={completedChecklist()} /></Widget>
      </Show>

      <Widget id="overview" layout={props.layout}>
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
              upstream: d().counts.domainsWithUpstream ?? t('admin-shell-not-available'),
              disabled: d().counts.domainsDisabled ?? t('admin-shell-not-available'),
            })}
          />
          <StatTile label={t('admin-dash-sessions')} value={d().counts.activeSessions} format={formatCount} />
          <StatTile label={t('admin-dash-admin-sessions')} value={d().counts.activeAdminSessions} format={formatCount} />
        </div>
      </section>
      </Widget>

      <Show when={d().security}>{security => (
      <Widget id="security" layout={props.layout}>
      <section class={css.panel} aria-labelledby="admin-dash-security">
        <h3 id="admin-dash-security" class={css.panelTitle}>
          {t('admin-dash-security')}
        </h3>
        <div class={shell.tiles}>
          <StatTile label={t('admin-dash-failed-webmail')} value={security().failedLogins24h.webmail} format={formatCount} />
          <StatTile label={t('admin-dash-failed-admin')} value={security().failedLogins24h.admin} format={formatCount} />
          <StatTile label={t('admin-dash-active-blocks')} value={security().activeBlocks} format={formatCount} />
          <StatTile
            label={t('admin-dash-allowlists')}
            value={t('admin-dash-allowlists-value', {
              admin: security().allowlists.admin === 'on' ? t('admin-dash-on') : t('admin-dash-off'),
              webmail: security().allowlists.webmail === 'on' ? t('admin-dash-on') : t('admin-dash-off'),
            })}
          />
          <StatTile
            label={t('admin-dash-trusted-proxies')}
            value={security().trustedProxiesConfigured ? t('admin-dash-configured') : t('admin-dash-not-configured')}
            detail={t('admin-dash-untrusted-forwarded', { count: security().forwardedFromUntrustedPeer24h })}
          />
        </div>
      </section>
      </Widget>
      )}</Show>

      <Show when={d().traffic}>
      <Widget id="traffic" layout={props.layout}>
        <RequestHistory refreshToken={d()}/>
      </Widget>
      </Show>

      <Show when={d().health}>{health => <Widget id="health" layout={props.layout}><HealthPanel health={health()} sectionLink={props.sectionLink} open={props.open} /></Widget>}</Show>

      <Show when={d().storage}>{storage => (
      <Widget id="storage" layout={props.layout}>
      <section class={css.panel} aria-labelledby="admin-dash-storage">
        <h3 id="admin-dash-storage" class={css.panelTitle}>
          {t('admin-dash-storage')}
        </h3>
        <div class={shell.tiles}>
          <StatTile label={t('admin-dash-db-size')} value={storage().dbBytes} format={formatBytes} />
          <StatTile
            label={t('admin-dash-wal-size')}
            value={storage().walBytes}
            format={formatBytes}
            unavailableReason={t('admin-dash-na-wal')}
          />
          <StatTile
            label={t('admin-dash-index-size')}
            value={storage().searchIndexBytes}
            format={formatBytes}
            unavailableReason={indexReason()}
          />
          <StatTile
            testId="tile-volume"
            label={t('admin-dash-volume-free')}
            value={storage().dataVolume?.freeBytes ?? null}
            format={formatBytes}
            detail={
              storage().dataVolume
                ? t('admin-dash-volume-of', { total: formatBytes(storage().dataVolume!.totalBytes) })
                : undefined
            }
            unavailableReason={t('admin-dash-na-volume')}
          />
        </div>
      </section>
      </Widget>
      )}</Show>

      <Widget id="audit" layout={props.layout}>
      <section class={css.panel} aria-labelledby="admin-dash-audit">
        <h3 id="admin-dash-audit" class={css.panelTitle}>
          {t('admin-dash-recent-audit')}
        </h3>
        <Show
          when={d().recentAudit.length > 0}
          fallback={<p class={admin.note}>{t('admin-dash-audit-empty')}</p>}
        >
          <div class={admin.tableWrap}>
            <table class={`${admin.table} ${auditCss.auditTable}`}>
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
                      <td data-label={t('admin-dash-audit-when')}>{formatDateTime(row.ts)}</td>
                      <td data-label={t('admin-dash-audit-actor')}><AuditActor {...row} /></td>
                      <td class={admin.mono} data-label={t('admin-dash-audit-action')}>{row.action}</td>
                      <td data-label={t('admin-dash-audit-target')}><span class={auditCss.identifier} title={row.target ?? undefined}>{row.target}</span></td>
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
      </Widget>

      <Show when={d().server}>{server => (
      <Widget id="server" layout={props.layout}>
      <section class={css.panel} aria-labelledby="admin-dash-server">
        <h3 id="admin-dash-server" class={css.panelTitle}>
          {t('admin-dash-server')}
        </h3>
        <dl class={css.facts}>
          <dt>{t('admin-dash-version')}</dt>
          <dd>{server().version}</dd>
          <dt>{t('admin-dash-build')}</dt>
          <dd class={admin.mono}>{server().buildSha ?? t('admin-shell-not-available')}</dd>
          <dt>{t('admin-dash-mode')}</dt>
          <dd>{proxy() ? t('admin-dash-mode-proxy') : t('admin-dash-mode-engine')}</dd>
          <dt>{t('admin-dash-uptime')}</dt>
          <dd>{formatDuration(server().uptimeSecs)}</dd>
          <dt>{t('admin-dash-base-path')}</dt>
          <dd class={admin.mono}>{server().basePath === '' ? t('admin-dash-base-root') : server().basePath}</dd>
        </dl>
      </section>
      </Widget>
      )}</Show>
    </div>
  );
}

function BlockedAccounts(props: {
  data: DashboardData;
  sectionLink: BodyProps['sectionLink'];
  open: BodyProps['open'];
}): JSX.Element {
  const b = (): NonNullable<DashboardData['blockedAccounts']> => props.data.blockedAccounts!;
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
                    {(host) => <span class={admin.note}> {t('admin-dash-blocked-seen', { host: isolate(host()) })}</span>}
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
                      {t('admin-dash-configure-domain', { domain: isolate(row.domain) })}
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

function Checklist(props: { items: ChecklistItem[]; open: (path: string) => void; username: string; deprioritized: boolean }): JSX.Element {
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
      <details open={!props.deprioritized}>
      <summary class={css.checkSummary}><h3 id="admin-dash-checklist" class={css.checkTitle}>
        {t('admin-dash-checklist', { open: openCount() })}
      </h3></summary>
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
              <Show when={!item.done && isConsoleLocation(item.href)}>
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
      </details>
    </section>
  );
}
