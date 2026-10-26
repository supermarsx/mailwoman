import { createSignal, Show, type JSX } from 'solid-js';
import type { AdminAccount, AdminSession } from '../../../api/admin/admins.ts';
import { t } from '../../../i18n';
import { Icon } from '../../../components/Icon.tsx';
import { DataTable, type Column } from '../shell/DataTable.tsx';

export function AdminSessionTable(props: { sessions: AdminSession[]; accounts: AdminAccount[]; busy: boolean; canRevoke: boolean; onRevoke: (session: AdminSession) => void; onRevokeMany?: (sessions:AdminSession[])=>void }): JSX.Element {
  const [query, setQuery] = createSignal('');
  const [selected,setSelected]=createSignal(new Set<string>());
  const name = (row: AdminSession): string => props.accounts.find((account) => account.id === row.adminId)?.username ?? row.adminId;
  const shown=():AdminSession[]=>props.sessions.filter(row=>name(row).toLowerCase().includes(query().trim().toLowerCase()));
  const chosen=():AdminSession[]=>shown().filter(row=>selected().has(row.id));
  const toggle=(id:string,checked:boolean):void=>{setSelected(previous=>{const next=new Set(previous);if(checked)next.add(id);else next.delete(id);return next;});};
  const date = (value: number | undefined): string => value === undefined ? '—' : new Date(value * 1000).toLocaleString();
  const columns = (): Column<AdminSession>[] => [
    ...(props.onRevokeMany?[{id:'select',header:t('admin-session-select'),cell:(row:AdminSession)=><input type="checkbox" aria-label={t('admin-session-select-account',{account:name(row)})} checked={selected().has(row.id)} disabled={props.busy||!props.canRevoke} onChange={event=>toggle(row.id,event.currentTarget.checked)}/>}]:[]),
    { id: 'account', header: t('admin-session-account'), cell: name },
    { id: 'created', header: t('admin-session-created'), cell: (row) => Number.isNaN(Date.parse(row.createdAt)) ? row.createdAt : new Date(row.createdAt).toLocaleString() },
    { id: 'idle', header: t('admin-session-idle-expiry'), cell: (row) => date(row.idleExpiresAt) },
    { id: 'expiry', header: t('admin-session-expiry'), cell: (row) => date(row.expiresAt) },
    { id: 'status', header: t('admin-session-status'), cell: (row) => t(row.current ? 'admin-session-current' : 'admin-session-active') },
  ];
  return <section aria-label={t('admin-session-console-title')}>
    <h3>{t('admin-session-console-title')}</h3><p>{t('admin-session-console-metadata')}</p>
    <Show when={props.onRevokeMany&&props.canRevoke}><div class="field"><label><input type="checkbox" aria-label={t('admin-session-select-visible')} checked={shown().length>0&&chosen().length===shown().length} disabled={props.busy||shown().length===0} onChange={event=>setSelected(event.currentTarget.checked?new Set(shown().map(row=>row.id)):new Set<string>())}/>{t('admin-session-select-visible')}</label><button class="btn" disabled={props.busy||chosen().length===0} onClick={()=>props.onRevokeMany?.(chosen())}>{t('admin-session-revoke-selected',{count:chosen().length})}</button></div></Show>
    <DataTable caption={t('admin-session-console-title')} columns={columns()} state="ready" rowKey={(row) => row.id}
      rows={shown()}
      emptyText={t('admin-session-empty')} search={{ label: t('admin-session-account'), value: query(), onSearch: value=>{setQuery(value);setSelected(new Set<string>());} }}
      rowActions={(row) => <button class="btn" type="button" disabled={props.busy || !props.canRevoke}
        aria-label={t('admin-session-revoke-account', { account: name(row) })} title={t('admin-session-revoke-account', { account: name(row) })}
        onClick={() => props.onRevoke(row)}><Icon name="close" /></button>} />
  </section>;
}
