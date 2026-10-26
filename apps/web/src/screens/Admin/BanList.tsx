import { createSignal, onMount, useContext, type JSX } from 'solid-js';
import { formatDateTime } from '../../i18n/format.ts';
import { t } from '../../i18n';
import type { BanEntry } from '../../state/slices/admin.ts';
import { useAdmin } from './context.ts';
import { ShellContext } from './shell/context.ts';
import { sectionPath } from './shell/router.ts';
import { DataTable, type Column } from './shell/DataTable.tsx';
import { Feedback } from './shell/Feedback.tsx';
import * as css from './admin.css.ts';
export function BanList(): JSX.Element {
  const shell = useContext(ShellContext);
  const { api } = useAdmin();
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
    try { setBans(await api.listBans()); setError(null); }
    catch { setError(t('admin-obs-load-error')); }
    finally { setLoading(false); }
  }
  onMount(() => void reload());
  async function onUnban(ip: string): Promise<void> {
    if (shell && !shell.can.can('bans.write')) return;
    try {
      await api.removeBan(ip);
      await reload();
    } catch {
      fail(t('admin-obs-unban-error'));
    }
  }

  return <section class={css.section} aria-label={t('admin-obs-bans')}><Feedback kind="error" as="toast" message={failure()} />
      <div class={css.card}>
        <h3 class={css.heading}>{t('admin-obs-bans')}</h3>
        <p class={css.note} data-testid="admin-obs-bans-note">
          {t('admin-obs-bans-note')}
        </p>
        <button type="button" class="btn btn--primary" disabled={shell!==undefined && !shell.can.can('bans.write')} onClick={() => void shell?.router.navigate(sectionPath('security', ['bans', 'new']))}>{t('admin-obs-ban-add')}</button>
        <DataTable<BanEntry> caption={t('admin-obs-bans')} columns={banColumns()} rows={visibleBans()} rowKey={ban => ban.ip} state={loading() ? 'loading' : error() ? 'error' : 'ready'} errorText={error() ?? ''} onRetry={() => void reload()} emptyText={t('admin-obs-bans-empty')}
          search={{ label: t('admin-obs-ban-search'), value: query(), onSearch: setQuery }}
          filters={[{ id: 'active', label: t('admin-obs-ban-active-only'), active: onlyActive() }]} onToggleFilter={() => setOnlyActive(value => !value)}
          rowActions={ban => <button type="button" class="btn btn--ghost" disabled={shell!==undefined && !shell.can.can('bans.write')} aria-label={t('admin-obs-unban-for', { ip: ban.ip })} onClick={() => void onUnban(ban.ip)}>{t('admin-obs-unban-btn')}</button>}
        />
      </div>
  </section>;
}
