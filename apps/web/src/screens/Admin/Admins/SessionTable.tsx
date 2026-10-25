import { createSignal, type JSX } from 'solid-js';
import type { AdminAccount, AdminSession } from '../../../api/admin/admins.ts';
import { t } from '../../../i18n';
import { Icon } from '../../../components/Icon.tsx';
import { DataTable, type Column } from '../shell/DataTable.tsx';

export function AdminSessionTable(props: { sessions: AdminSession[]; accounts: AdminAccount[]; busy: boolean; canRevoke: boolean; onRevoke: (session: AdminSession) => void }): JSX.Element {
  const [query, setQuery] = createSignal('');
  const name = (row: AdminSession): string => props.accounts.find((account) => account.id === row.adminId)?.username ?? row.adminId;
  const date = (value: number | undefined): string => value === undefined ? '—' : new Date(value * 1000).toLocaleString();
  const columns = (): Column<AdminSession>[] => [
    { id: 'account', header: t('admin-session-account'), cell: name },
    { id: 'created', header: t('admin-session-created'), cell: (row) => Number.isNaN(Date.parse(row.createdAt)) ? row.createdAt : new Date(row.createdAt).toLocaleString() },
    { id: 'idle', header: t('admin-session-idle-expiry'), cell: (row) => date(row.idleExpiresAt) },
    { id: 'expiry', header: t('admin-session-expiry'), cell: (row) => date(row.expiresAt) },
    { id: 'status', header: t('admin-session-status'), cell: (row) => t(row.current ? 'admin-session-current' : 'admin-session-active') },
  ];
  return <section aria-label={t('admin-session-console-title')}>
    <h3>{t('admin-session-console-title')}</h3><p>{t('admin-session-console-metadata')}</p>
    <DataTable caption={t('admin-session-console-title')} columns={columns()} state="ready" rowKey={(row) => row.id}
      rows={props.sessions.filter((row) => name(row).toLowerCase().includes(query().trim().toLowerCase()))}
      emptyText={t('admin-session-empty')} search={{ label: t('admin-session-account'), value: query(), onSearch: setQuery }}
      rowActions={(row) => <button class="btn" type="button" disabled={props.busy || !props.canRevoke}
        aria-label={t('admin-session-revoke-account', { account: name(row) })} title={t('admin-session-revoke-account', { account: name(row) })}
        onClick={() => props.onRevoke(row)}><Icon name="close" /></button>} />
  </section>;
}
