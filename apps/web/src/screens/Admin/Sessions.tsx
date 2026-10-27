import { createResource, createSignal, Show, useContext, onCleanup, type JSX } from 'solid-js';
import { adminSessionGeneration } from '../../api/adminSession.ts';
import { adminRequest } from './shell/adminApi.ts';
import { t, formatDateTime } from '../../i18n';
import * as css from '../Settings/styles.css.ts';
import { DataTable, type Column } from './shell/DataTable.tsx';
import { ConfirmDialog } from './shell/ConfirmDialog.tsx';
import { Icon } from '../../components/Icon.tsx';
import * as sessionCss from './sessions.css.ts';
import { ShellContext } from './shell/context.ts';
import { revokeSessionBatch } from '../../api/sessionRevocation.ts';
type Session = { handle: string; accountId: string; username: string; lastSeen: string; lastActivity?: string; clientType: string; userAgent: string; ip: string; createdAt?: string; deviceLabel?: string; clientVersion?: string };
type Listing = { sessions: Session[]; truncated: boolean };
export function AdminSessions(): JSX.Element {
  const authority=adminSessionGeneration();let alive=true;onCleanup(()=>{alive=false;});
  const authorityCurrent=():boolean=>alive&&authority===adminSessionGeneration();
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
  const [selected,setSelected]=createSignal(new Set<string>());
  const [batch,setBatch]=createSignal<Session[]>();
  const [notice,setNotice]=createSignal<{text:string;problem:boolean}>();
  const [sessions, { refetch }] = createResource(async () => (await adminRequest<Listing>('/sessions')).data);
  const filter = () => ({ ...(domain() ? { domain: domain() } : {}), ...(account() ? { accountId: account() } : {}), ...(kind() ? { clientType: kind() } : {}) });
  function change(set: (value: string) => void, value: string): void { set(value); setPreview(undefined);setSelected(new Set<string>()); }
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
  const date = (value?: string): string => !value ? '—' : Number.isNaN(Date.parse(value)) ? value : formatDateTime(new Date(value));
  const chosen=():Session[]=>shown().filter(row=>selected().has(row.handle));
  async function revokeSelected():Promise<void>{
    const rows=batch();if(!rows||!canRevoke()||!authorityCurrent())return;setBusy(true);setError('');setNotice(undefined);
    try{
      const result=await revokeSessionBatch(rows,row=>row.handle,()=>false,async row=>{
        const response=await adminRequest<{revoked:number}>('/sessions/revoke',{method:'POST',body:{filter:{handle:row.handle},dryRun:false,handles:[row.handle]}});
        return response.data.revoked===1;
      },authorityCurrent);
      if(!authorityCurrent())return;
      setBatch(undefined);setSelected(new Set<string>());setPreview(undefined);
      setNotice({text:t('admin-session-batch-result',{revoked:result.revoked.length,failed:result.failed.length,skipped:result.skipped.length,unchanged:result.unchanged.length}),problem:result.failed.length>0||result.skipped.length>0});
      try{await refetch();}catch{if(authorityCurrent())setError(t('admin-session-refresh-failed'));}
    }finally{if(authorityCurrent())setBusy(false);}
  }
  const columns = (): Column<Session>[] => [
    {id:'select',header:t('admin-session-select'),cell:row=><input type="checkbox" checked={selected().has(row.handle)} disabled={busy()||!canRevoke()} aria-label={t('admin-session-select-account',{account:row.username})} onChange={event=>setSelected(previous=>{const next=new Set(previous);if(event.currentTarget.checked)next.add(row.handle);else next.delete(row.handle);return next;})}/>},
    { id: 'account', header: t('admin-session-account'), cell: (row) => <span class={sessionCss.cell}><strong>{row.username}</strong><p>{row.accountId}</p></span> },
    { id: 'device', header: t('admin-session-device'), cell: (row) => <span class={sessionCss.cell}>{row.deviceLabel || row.clientType || '—'} {row.clientVersion}<p>{row.userAgent || '—'}</p></span> },
    { id: 'ip', header: t('admin-session-ip'), cell: (row) => row.ip || '—' },
    { id: 'activity', header: t('admin-session-activity'), cell: (row) => date(row.lastActivity ?? row.lastSeen) },
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
    <Show when={notice()}>{value=><p role={value().problem?'alert':'status'}>{value().text}</p>}</Show>
    <Show when={canRevoke()}><div class={css.actions}><label><input type="checkbox" aria-label={t('admin-session-select-visible')} checked={shown().length>0&&chosen().length===shown().length} disabled={busy()||shown().length===0} onChange={event=>setSelected(event.currentTarget.checked?new Set(shown().map(row=>row.handle)):new Set<string>())}/>{t('admin-session-select-visible')}</label><button type="button" disabled={busy()||chosen().length===0} onClick={()=>setBatch(chosen())}>{t('admin-session-revoke-selected',{count:chosen().length})}</button></div></Show>
    <DataTable caption={t('admin-session-mailbox-title')} columns={columns()} rows={shown()} rowKey={(row) => row.handle}
      state={sessions.error ? 'error' : sessions.loading && !sessions.latest ? 'loading' : 'ready'}
      errorText={t('security-center-error')} onRetry={() => void refetch()} emptyText={t('admin-session-empty')}
      search={{ label: t('admin-session-search'), value: query(), onSearch: value=>{setQuery(value);setSelected(new Set<string>());} }}
      rowActions={(row) => <button type="button" class="btn" disabled={busy() || !canRevoke()} aria-label={t('admin-session-revoke-account', { account: row.username })} title={t('admin-session-revoke-account', { account: row.username })} onClick={() => setSingle(row)}><Icon name="close" /></button>} />
    <ConfirmDialog open={batch()!==undefined} title={t('admin-session-revoke-selected',{count:batch()?.length??0})} body={t('admin-session-batch-confirm')} confirmLabel={t('admin-session-batch-confirm-button')} onCancel={()=>setBatch(undefined)} onConfirm={revokeSelected}/>
    <ConfirmDialog open={single() !== undefined} title={t('admin-session-revoke-account', { account: single()?.username ?? '' })}
      body={t('admin-session-revoke-help')} confirmLabel={t('admin-session-revoke')} onCancel={() => setSingle(undefined)}
      onConfirm={async () => { const row = single(); if (!row || !canRevoke()) return;
        await adminRequest('/sessions/revoke', { method: 'POST', body: { filter: { handle: row.handle }, dryRun: false, handles: [row.handle] } });
        setSingle(undefined); setPreview(undefined); await refetch(); }} />
  </section>;
}
