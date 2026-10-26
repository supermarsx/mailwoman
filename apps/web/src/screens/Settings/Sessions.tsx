// Active-session listing + revocation (t16 e15, SPEC §19 — S11).
//
// Lists the account's live sessions (metadata only; raw ids never leave the
// server) and revokes them: one by handle, or "everywhere else" (all but the
// current). Rides the `crates/mw-server/src/twofa_routes.rs` session routes.

import { createResource, createSignal, For, Show, onMount, onCleanup, type JSX } from 'solid-js';
import { observePreferenceAccount } from '../../api/accountPreferenceLifecycle.ts';
import { t, loadCatalog, isolate } from '../../i18n';
import { SettingsService } from './service.ts';
import type { SessionMeta } from './types.ts';
import { toast } from '../../components/toast/index.ts';
import { Icon } from '../../components/Icon.tsx';
import {revokeSessionBatch} from '../../api/sessionRevocation.ts';
import {SessionRevokeConfirm} from './SessionRevokeConfirm.tsx';
import * as table from './sessions.css.ts';
import * as css from './styles.css.ts';

export interface SessionsProps {
  service?: SettingsService;
}

export function Sessions(props: SessionsProps): JSX.Element {
  const service = props.service ?? new SettingsService();
  onMount(() => void loadCatalog('settings'));

  const [authority,setAuthority]=createSignal(0);let alive=true;
  const [sessions, { refetch }] = createResource(authority,() => service.sessions());
  const [query, setQuery] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [selected,setSelected]=createSignal(new Set<string>());
  const [batch,setBatch]=createSignal<SessionMeta[]>();
  const [result,setResult]=createSignal<{text:string;problem:boolean}>();
  let observed=false;
  const unsubscribe=observePreferenceAccount(()=>{if(!observed){observed=true;return;}setAuthority(value=>value+1);setSelected(new Set<string>());setBatch(undefined);setResult(undefined);setBusy(false);});
  onCleanup(()=>{alive=false;unsubscribe();});

  // A failed action is a toast (t32 f39).
  function fail(e: unknown): void {
    toast.error(e instanceof Error ? e.message : t('settings-sessions-error'));
  }

  async function revoke(handle: string): Promise<void> {
    setBusy(true);
    try {
      await service.revokeSession(handle);
      await refetch();
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  }

  async function revokeOthers(): Promise<void> {
    setBusy(true);
    try {
      await service.revokeOtherSessions();
      await refetch();
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  }

  const loaded = (): SessionMeta[] => sessions.error ? [] : sessions.latest ?? [];
  const others = (): number => loaded().filter((s) => !s.current).length;
  const shown=():SessionMeta[]=>loaded().filter(row=>[row.username,row.ip,row.deviceLabel,row.userAgent].some(value=>value?.toLowerCase().includes(query().trim().toLowerCase())));
  const eligible=():SessionMeta[]=>shown().filter(row=>!row.current);
  const chosen=():SessionMeta[]=>eligible().filter(row=>selected().has(row.handle));
  async function revokeSelected():Promise<void>{
    const rows=batch();if(!rows)return;const epoch=authority();const current=():boolean=>alive&&epoch===authority();setBusy(true);setResult(undefined);
    try{
      const value=await revokeSessionBatch(rows,row=>row.handle,()=>false,row=>service.revokeSession(row.handle),current);
      if(!current())return;
      setBatch(undefined);setSelected(new Set<string>());
      setResult({text:t('settings-sessions-selected-result',{revoked:value.revoked.length,unchanged:value.unchanged.length,failed:value.failed.length,skipped:value.skipped.length}),problem:value.failed.length>0||value.skipped.length>0});
      try{await refetch();}catch{if(current())fail(new Error(t('settings-sessions-refresh-failed')));}
    }finally{if(current())setBusy(false);}
  }

  return (
    <section class={css.section} aria-label={t('settings-sessions-title')}>
      <h2 class={css.heading}>{t('settings-sessions-title')}</h2>
      <p class={css.prose}>{t('settings-sessions-intro')}</p>

      <label class={css.field}>{t('settings-sessions-search')}<input class={css.input} type="search" value={query()} onInput={(event) => {setQuery(event.currentTarget.value);setSelected(new Set<string>());}} /></label>
      <p class={css.prose}>{t('settings-sessions-current-protected')}</p>
      <div class={css.actions}><label><input type="checkbox" aria-label={t('settings-sessions-select-visible')} checked={eligible().length>0&&chosen().length===eligible().length} disabled={busy()||eligible().length===0} onChange={event=>setSelected(event.currentTarget.checked?new Set(eligible().map(row=>row.handle)):new Set<string>())}/>{t('settings-sessions-select-visible')}</label><button class={css.danger} disabled={busy()||chosen().length===0} onClick={()=>setBatch(chosen())}>{t('settings-sessions-selected-title',{count:chosen().length})}</button></div>
      <Show when={result()}>{value=><p role={value().problem?'alert':'status'}>{value().text}</p>}</Show>
      <Show when={sessions.error}><p role="alert">{t('settings-sessions-error')}</p></Show>
      <div class={table.wrap} aria-busy={sessions.loading}>
        <table class={table.table} data-testid="session-list"><caption>{t('settings-sessions-title')}</caption>
          <thead><tr><th aria-label={t('settings-sessions-select')}/><th>{t('settings-sessions-account')}</th><th>{t('settings-sessions-device')}</th><th>{t('settings-sessions-location')}</th><th>{t('settings-sessions-activity')}</th><th>{t('settings-sessions-actions')}</th></tr></thead>
          <tbody><For each={shown()}>{(s) => <tr>
            <td><input type="checkbox" aria-label={t('settings-sessions-select-device',{device:s.deviceLabel||s.clientType||s.username})} checked={selected().has(s.handle)} disabled={busy()||s.current} onChange={event=>setSelected(previous=>{const next=new Set(previous);if(event.currentTarget.checked)next.add(s.handle);else next.delete(s.handle);return next;})}/></td>
            <td>{isolate(s.username)}</td><td>{s.deviceLabel || s.clientType || '—'} {s.clientVersion}<p class={css.meta}>{s.userAgent || '—'}</p></td>
            <td>{s.ip || '—'}<p class={css.meta}>{s.country || t('settings-sessions-location-unavailable')}</p></td>
            <td>{t('settings-sessions-last-seen', { when: s.lastActivity ?? s.lastSeen })}<p class={css.meta}>{t('security-center-first-seen', { when: s.createdAt })}</p></td>
            <td><Show when={s.current} fallback={<button type="button" class={css.danger} disabled={busy()} aria-label={t('settings-sessions-revoke-device', { device: s.deviceLabel || s.clientType || s.username })} title={t('settings-sessions-revoke-device', { device: s.deviceLabel || s.clientType || s.username })} onClick={() => void revoke(s.handle)}><Icon name="close" /></button>}><span class={css.badge}>{t('settings-sessions-current')}</span></Show></td>
          </tr>}</For></tbody>
        </table>
      </div>

      <Show when={others() > 0}>
        <div class={css.actions}>
          <button type="button" class={css.danger} disabled={busy()} onClick={() => void revokeOthers()} data-testid="revoke-others">
            {t('settings-sessions-revoke-others', { count: others() })}
          </button>
        </div>
      </Show>
      <Show when={batch()}>{rows=><SessionRevokeConfirm count={rows().length} busy={busy()} onCancel={()=>setBatch(undefined)} onConfirm={revokeSelected}/>}</Show>
    </section>
  );
}

export default Sessions;
