import { createResource, createSignal, Show, useContext, type JSX } from 'solid-js';
import { adminRequest } from './shell/adminApi.ts';
import { t } from '../../i18n';
import * as css from '../Settings/styles.css.ts';
import { DataTable, type Column } from './shell/DataTable.tsx';
import { ConfirmDialog } from './shell/ConfirmDialog.tsx';
import { Icon } from '../../components/Icon.tsx';
import * as sessionCss from './sessions.css.ts';
import { ShellContext } from './shell/context.ts';
type Session = { handle: string; accountId: string; username: string; lastSeen: string; clientType: string; userAgent: string; ip: string; createdAt?: string; deviceLabel?: string; clientVersion?: string };
type Listing = { sessions: Session[]; truncated: boolean };
export function AdminSessions(): JSX.Element {
  const shell = useContext(ShellContext);
  const canRevoke = () => shell?.can.can('sessions.revoke') ?? true;
  const [query, setQuery] = createSignal('');
  const [single, setSingle] = createSignal<Session>();
  const [domain, setDomain] = createSignal('');
  const [account, setAccount] = createSignal('');
  const [kind, setKind] = createSignal('');
  const [preview, setPreview] = createSignal<Listing>();
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  const [sessions, { refetch }] = createResource(async () => (await adminRequest<Listing>('/sessions')).data);
  const filter = () => ({ ...(domain() ? { domain: domain() } : {}), ...(account() ? { accountId: account() } : {}), ...(kind() ? { clientType: kind() } : {}) });
  function change(set: (value: string) => void, value: string): void { set(value); setPreview(undefined); }
  async function run(commit: boolean): Promise<void> {
    if (!canRevoke()) return;
    setBusy(true); setError('');
    try {
      const response = await adminRequest<Listing>('/sessions/revoke', { method: 'POST', body: { filter: filter(), dryRun: !commit, ...(commit ? { handles: preview()?.sessions.map((s) => s.handle) ?? [] } : {}) } });
      if (commit) { setPreview(undefined); await refetch(); } else setPreview(response.data);
    } catch (e) { setError(e instanceof Error ? e.message : t('security-center-error')); }
    finally { setBusy(false); }
  }
  const shown = (): Session[] => {
    const q = query().trim().toLowerCase();
    const rows = preview()?.sessions ?? (sessions.error ? [] : sessions.latest?.sessions) ?? [];
    return rows.filter((row) => (!domain() || row.username.split('@').at(-1)?.toLowerCase() === domain().toLowerCase())
      && (!account() || row.accountId === account()) && (!kind() || row.clientType === kind())
      && (!q || [row.username, row.accountId, row.ip, row.userAgent, row.deviceLabel].some((v) => v?.toLowerCase().includes(q))));
  };
  const date = (value?: string): string => !value ? '—' : Number.isNaN(Date.parse(value)) ? value : new Date(value).toLocaleString();
  const columns = (): Column<Session>[] => [
    { id: 'account', header: t('admin-session-account'), cell: (row) => <span class={sessionCss.cell}><strong>{row.username}</strong><p>{row.accountId}</p></span> },
    { id: 'device', header: t('admin-session-device'), cell: (row) => <span class={sessionCss.cell}>{row.deviceLabel || row.clientType || '—'} {row.clientVersion}<p>{row.userAgent || '—'}</p></span> },
    { id: 'ip', header: t('admin-session-ip'), cell: (row) => row.ip || '—' },
    { id: 'activity', header: t('admin-session-activity'), cell: (row) => date(row.lastSeen) },
    { id: 'created', header: t('admin-session-created'), cell: (row) => date(row.createdAt) },
  ];
  return <section class={css.section}>
    <h2 class={css.heading}>{t('admin-nav-sessions')}</h2>
    <Show when={error()}><p role="alert">{error()}</p></Show>
    <label class={css.field}>{t('admin-sessions-domain')}<input class={css.input} value={domain()} onInput={(e) => change(setDomain, e.currentTarget.value)} /></label>
    <label class={css.field}>{t('admin-sessions-account')}<input class={css.input} value={account()} onInput={(e) => change(setAccount, e.currentTarget.value)} /></label>
    <label class={css.field}>{t('admin-sessions-client')}<input class={css.input} value={kind()} onInput={(e) => change(setKind, e.currentTarget.value)} /></label>
    <button type="button" disabled={busy() || !canRevoke()} title={shell?.can.reason('sessions.revoke') ?? undefined} onClick={() => void run(false)}>{t('admin-sessions-preview')}</button>
    <Show when={preview()}>{(review) => <><p role="status">{t('admin-sessions-count', { count: review().sessions.length })}</p><button type="button" disabled={busy() || !canRevoke() || review().sessions.length === 0} onClick={() => void run(true)}>{t('admin-sessions-confirm')}</button></>}</Show>
    <Show when={(preview() ?? (sessions.error ? undefined : sessions.latest))?.truncated}><p>{t('admin-sessions-truncated')}</p></Show>
    <DataTable caption={t('admin-session-mailbox-title')} columns={columns()} rows={shown()} rowKey={(row) => row.handle}
      state={sessions.error ? 'error' : sessions.loading && !sessions.latest ? 'loading' : 'ready'}
      errorText={t('security-center-error')} onRetry={() => void refetch()} emptyText={t('admin-session-empty')}
      search={{ label: t('admin-session-search'), value: query(), onSearch: setQuery }}
      rowActions={(row) => <button type="button" class="btn" disabled={busy() || !canRevoke()} aria-label={t('admin-session-revoke-account', { account: row.username })} title={t('admin-session-revoke-account', { account: row.username })} onClick={() => setSingle(row)}><Icon name="close" /></button>} />
    <ConfirmDialog open={single() !== undefined} title={t('admin-session-revoke-account', { account: single()?.username ?? '' })}
      body={t('admin-session-revoke-help')} confirmLabel={t('admin-session-revoke')} onCancel={() => setSingle(undefined)}
      onConfirm={async () => { const row = single(); if (!row || !canRevoke()) return;
        await adminRequest('/sessions/revoke', { method: 'POST', body: { filter: { handle: row.handle }, dryRun: false, handles: [row.handle] } });
        setSingle(undefined); setPreview(undefined); await refetch(); }} />
  </section>;
}
