import { formatDateTime } from '../../i18n/format.ts';
import { AuditActor } from './shell/AuditActor.tsx';
import * as auditCss from './shell/audit.css.ts';
// Admin › Observability (§19, §21). The append-only audit-log viewer + JSONL
// export, and the login monitor's ban list with add + unban.
//
// Telemetry has a dedicated editor under /observability/telemetry. Its typed
// configuration is applied at server startup, with explicit environment pins.
//
// Active bans are enforced against the trusted client IP on application requests.

import { createSignal, For, Show, onMount, useContext, lazy, type JSX } from 'solid-js';
import { ShellContext } from './shell/context.ts';
import { sectionPath } from './shell/router.ts';
import { useAdmin } from './context.ts';
import type { AuditLogEntry, BanEntry } from '../../state/slices/admin.ts';
import { t } from '../../i18n';
import { DataTable, type Column } from './shell/DataTable.tsx';
import { Feedback } from './shell/Feedback.tsx';
import * as css from './admin.css.ts';

const SigninGeographyPage = lazy(() => import('./SigninGeography.tsx').then(module => ({ default: module.SigninGeographyPage })));
const AutomaticLocks = lazy(() => import('./AutomaticLocks.tsx').then(module => ({ default: module.AutomaticLocks })));
const BanEditor = lazy(() => import('./BanEditor.tsx').then(module => ({ default: module.BanEditor })));
const Telemetry = lazy(() => import('./Telemetry.tsx').then(module => ({ default: module.Telemetry })));
export function Observability(): JSX.Element {
  const shell = useContext(ShellContext);
  return <Show when={shell?.router.route().rest[0] === 'geography'} fallback={<Show when={shell?.router.route().rest[0] === 'locks'} fallback={<Show when={shell?.router.route().rest[0] === 'telemetry'} fallback={<Show when={shell?.router.route().rest[0] === 'bans' && shell?.router.route().rest[1] === 'new'} fallback={<ObservabilityAudit/>}><BanEditor/></Show>}><Telemetry/></Show>}><AutomaticLocks/></Show>}><SigninGeographyPage/></Show>;
}
function ObservabilityAudit(): JSX.Element {
  const shell = useContext(ShellContext);
  const { api } = useAdmin();
  const [audit, setAudit] = createSignal<AuditLogEntry[]>([]);
  const [bans, setBans] = createSignal<BanEntry[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [query, setQuery] = createSignal('');
  const [onlyActive, setOnlyActive] = createSignal(false);
  const expired = (ban: BanEntry): boolean => !!ban.expiresAt && Number.isFinite(Date.parse(ban.expiresAt)) && Date.parse(ban.expiresAt) <= Date.now();
  const visibleBans = (): BanEntry[] => bans().filter(ban => (!onlyActive() || !expired(ban)) && `${ban.ip} ${ban.reason}`.toLowerCase().includes(query().trim().toLowerCase()));
  const banColumns = (): Column<BanEntry>[] => [
    { id: 'ip', header: t('admin-obs-ban-ip'), cell: ban => <span class={css.mono}>{ban.ip}</span> },
    { id: 'reason', header: t('admin-obs-ban-reason'), cell: ban => <span dir="auto">{ban.reason}</span> },
    { id: 'expiry', header: t('admin-obs-ban-status'), cell: ban => <span>{ban.expiresAt && Number.isFinite(Date.parse(ban.expiresAt)) ? `${t(expired(ban) ? 'admin-obs-ban-expired' : 'admin-obs-ban-until')} ${formatDateTime(ban.expiresAt)}` : t('admin-obs-ban-permanent')}</span> },
  ];
  const [error, setError] = createSignal<string | null>(null);
  // An action's failure is a toast (t32 f39); a load failure stays on the page.
  const [failure, setFailure] = createSignal<string | null>(null);
  function fail(message: string): void {
    setFailure(null);
    setFailure(message);
  }

  async function reload(): Promise<void> {
    setLoading(true);
    try {
      const [a, b] = await Promise.all([api.listAudit(100), api.listBans()]);
      setAudit(a);
      setBans(b);
      setError(null);
    } catch {
      setError(t('admin-obs-load-error'));
    } finally { setLoading(false); }
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
      fail(t('admin-obs-export-error'));
    }
  }

  async function onUnban(ip: string): Promise<void> {
    try {
      await api.removeBan(ip);
      await reload();
    } catch {
      fail(t('admin-obs-unban-error'));
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
      <Feedback kind="error" as="toast" message={failure()} />
      <Show when={shell?.can.can('bans.read')}><div class={css.card}><p class={css.note}>{t('admin-obs-locks-note')}</p><button class="btn" onClick={()=>void shell?.router.navigate(sectionPath('observability',['locks']))}>{t('admin-obs-locks-title')}</button><button class="btn btn--ghost" onClick={()=>void shell?.router.navigate(sectionPath('observability',['geography']))}>{t('admin-obs-geo-title')}</button></div></Show>

      <div class={css.card}>
        <p class={css.note}>{t('admin-obs-telemetry-note')}</p>
        <Show when={shell?.can.can('observability.read')}><button class="btn" onClick={()=>void shell?.router.navigate(sectionPath('observability',['telemetry']))}>{t('admin-obs-telemetry-open')}</button></Show>
      </div>

      <div class={css.card}>
        <div class={css.headingRow}>
          <h3 class={css.heading}>{t('admin-obs-audit')}</h3>
          <button type="button" class="btn btn--ghost" onClick={() => void onExport()}>
            {t('admin-obs-export')}
          </button>
        </div>
        <Show when={audit().length > 0} fallback={<p class={css.note}>{t('admin-obs-audit-empty')}</p>}>
          <div class={css.tableWrap}>
            <table class={`${css.table} ${auditCss.auditTable}`}>
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
                      <td data-label={t('admin-obs-col-time')}><time dateTime={a.ts} title={a.ts}>{formatDateTime(a.ts) || a.ts}</time></td>
                      <td data-label={t('admin-obs-col-actor')}>
                        <div><AuditActor {...a} /><span class={css.badge}>{a.actorKind}</span></div>
                      </td>
                      <td data-label={t('admin-obs-col-action')} dir="auto">{a.action}</td>
                      <td data-label={t('admin-obs-col-target')} dir="auto"><span class={auditCss.identifier} title={a.target ?? undefined}>{a.target ?? '—'}</span></td>
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
        <button type="button" class="btn btn--primary" onClick={() => void shell?.router.navigate(sectionPath('observability', ['bans', 'new']))}>{t('admin-obs-ban-add')}</button>
        <DataTable<BanEntry> caption={t('admin-obs-bans')} columns={banColumns()} rows={visibleBans()} rowKey={ban => ban.ip} state={loading() ? 'loading' : error() ? 'error' : 'ready'} errorText={error() ?? ''} onRetry={() => void reload()} emptyText={t('admin-obs-bans-empty')}
          search={{ label: t('admin-obs-ban-search'), value: query(), onSearch: setQuery }}
          filters={[{ id: 'active', label: t('admin-obs-ban-active-only'), active: onlyActive() }]} onToggleFilter={() => setOnlyActive(value => !value)}
          rowActions={ban => <button type="button" class="btn btn--ghost" aria-label={t('admin-obs-unban-for', { ip: ban.ip })} onClick={() => void onUnban(ban.ip)}>{t('admin-obs-unban-btn')}</button>}
        />
      </div>
    </section>
  );
}
