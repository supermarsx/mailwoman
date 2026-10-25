import { createResource, createSignal, For, Show, useContext, type JSX } from 'solid-js';
import { adminRequest } from './shell/adminApi.ts';
import { t } from '../../i18n';
import * as css from '../Settings/styles.css.ts';
import { ShellContext } from './shell/context.ts';
type Session = { handle: string; accountId: string; username: string; lastSeen: string; clientType: string; userAgent: string; ip: string };
type Listing = { sessions: Session[]; truncated: boolean };
export function AdminSessions(): JSX.Element {
  const shell = useContext(ShellContext);
  const canRevoke = () => shell?.can.can('sessions.revoke') ?? true;
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
    setBusy(true); setError('');
    try {
      const response = await adminRequest<Listing>('/sessions/revoke', { method: 'POST', body: { filter: filter(), dryRun: !commit, ...(commit ? { handles: preview()?.sessions.map((s) => s.handle) ?? [] } : {}) } });
      if (commit) { setPreview(undefined); await refetch(); } else setPreview(response.data);
    } catch (e) { setError(e instanceof Error ? e.message : t('security-center-error')); }
    finally { setBusy(false); }
  }
  return <section class={css.section}>
    <h2 class={css.heading}>{t('admin-nav-sessions')}</h2>
    <Show when={error()}><p role="alert">{error()}</p></Show>
    <label class={css.field}>{t('admin-sessions-domain')}<input class={css.input} value={domain()} onInput={(e) => change(setDomain, e.currentTarget.value)} /></label>
    <label class={css.field}>{t('admin-sessions-account')}<input class={css.input} value={account()} onInput={(e) => change(setAccount, e.currentTarget.value)} /></label>
    <label class={css.field}>{t('admin-sessions-client')}<input class={css.input} value={kind()} onInput={(e) => change(setKind, e.currentTarget.value)} /></label>
    <button type="button" disabled={busy() || !canRevoke()} title={shell?.can.reason('sessions.revoke') ?? undefined} onClick={() => void run(false)}>{t('admin-sessions-preview')}</button>
    <Show when={preview()}>{(review) => <><p role="status">{t('admin-sessions-count', { count: review().sessions.length })}</p><button type="button" disabled={busy() || review().sessions.length === 0} onClick={() => void run(true)}>{t('admin-sessions-confirm')}</button></>}</Show>
    <Show when={(preview() ?? sessions())?.truncated}><p>{t('admin-sessions-truncated')}</p></Show>
    <ul class={css.list}><For each={(preview() ?? sessions())?.sessions}>{(session) => <li class={css.item}><div>{session.username} · {session.clientType}<p class={css.meta}>{session.lastSeen} · {session.ip} · {session.userAgent}</p></div></li>}</For></ul>
  </section>;
}
