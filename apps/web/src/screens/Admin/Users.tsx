import {UserAccessActions} from './UserAccessActions.tsx';
import * as auditCss from './shell/audit.css.ts';
import { ShellContext } from './shell/context.ts';
import { adminRequest } from './shell/adminApi.ts';
import { hrefFor, sectionPath } from './shell/router.ts';
import { AsyncPending } from '../../components/AsyncState.tsx';
// Admin › Users (§19). Provision users, set quota, revoke sessions, and set the
// per-account flags: disabled, force password change, and the zero-access flag.
// Every action audits server-side.
//
// There is no "cache wipe" checkbox: `remoteCacheWipe` is stored by
// `PUT /admin/users/{id}/flags` and read by nothing, so ticking it wiped no cache
// (26.20, t28-e8). The field stays in `UserFeatureFlags` because the server sends
// and expects it (`FlagsDto`, `crates/mw-server/src/admin.rs`), and is passed
// through unchanged when another flag is saved.

import { createSignal, createResource, useContext, For, Index, Show, onMount, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import type { UserFeatureFlags, UserSummary } from '../../state/slices/admin.ts';
import { t } from '../../i18n';
import { Feedback } from './shell/Feedback.tsx';
import * as css from './admin.css.ts';

export function Users(props: { accountId?: string; provision?: boolean } = {}): JSX.Element {
  const shell = useContext(ShellContext);
  const identity = () => props.accountId ?? shell?.router.route().rest[0];
  return <Show when={props.provision || identity()==='+new'} fallback={<Show when={identity()} keyed fallback={<UserList/>}>{id=><UserIdentity id={id}/>}</Show>}><UserList provision/></Show>;
}
function UserIdentity(props: { id: string }): JSX.Element {
  const [identity] = createResource(() => props.id, async id => (await adminRequest<{accountId:string;label:string;address:string;loginAddress?:string;directoryAddress?:string;legacyBinding?:boolean}>(`/users/${encodeURIComponent(id)}/identity`)).data);
  return <section class={css.section} aria-label={t('admin-users-title')}>
    <a href={hrefFor(sectionPath('users'))}>{t('admin-users-back')}</a>
    <h2 class={css.heading}>{t('admin-users-identity')}</h2>
    <Show when={identity.loading}><AsyncPending variant="status" /></Show>
    <Show when={identity.error}><p role="alert">{t('admin-users-load-error')}</p></Show>
    <Show when={!identity.error && identity.state === 'ready' ? identity.latest : undefined}>{value => <div class={css.card}><h3 dir="auto">{value().legacyBinding?value().loginAddress??value().label:value().label}</h3><p dir="auto">{value().loginAddress??value().address}</p><Show when={value().legacyBinding&&value().directoryAddress}><p class={css.note}>{t('admin-users-legacy-alias',{address:value().directoryAddress!})}</p></Show><p class={css.note}>{t('admin-users-id')}: <code class={auditCss.actor}>{value().accountId}</code></p></div>}</Show>
    <UserList accountId={props.id}/>
  </section>;
}
function UserList(props: { accountId?: string; provision?: boolean }): JSX.Element {
  const { api } = useAdmin();
  const [users, setUsers] = createSignal<UserSummary[]>([]);
  const [error, setError] = createSignal<string | null>(null);
  // An action's failure is a toast (t32 f39); a load failure stays on the page.
  const [failure, setFailure] = createSignal<string | null>(null);
  function fail(message: string): void {
    setFailure(null);
    setFailure(message);
  }
  const [domain, setDomain] = createSignal('');
  const [username, setUsername] = createSignal('');
  const [bytes, setBytes] = createSignal('0');
  const [msgs, setMsgs] = createSignal('0');

  async function reload(): Promise<void> {
    try {
      setUsers(await api.listUsers());
      setError(null);
    } catch {
      setError(t('admin-users-load-error'));
    }
  }
  onMount(() => void reload());

  async function onProvision(e: Event): Promise<void> {
    e.preventDefault();
    if (username().trim() === '' || domain().trim() === '') return;
    try {
      await api.provisionUser({
        domain: domain().trim(),
        username: username().trim(),
        quota: { bytesLimit: Number(bytes()) || 0, msgLimit: Number(msgs()) || 0 },
      });
      setUsername('');
      await reload();
    } catch {
      fail(t('admin-users-provision-error'));
    }
  }

  // Rows are an <Index> (t32-h12), so a reload updates them in place and keyboard
  // focus stays on the box just toggled. Solid writes `checked` only when the model
  // value changes, so the box is set back from the model by hand: to the stored
  // value after the reload, or to the old value when the change failed.
  async function patchFlag(u: UserSummary, key: keyof UserFeatureFlags, box: HTMLInputElement): Promise<void> {
    const value = box.checked;
    try {
      if (key === 'zeroAccess') {
        await api.toggleZeroAccess(u.accountId, value);
      } else {
        await api.setFlags(u.accountId, { ...u.flags, [key]: value });
      }
      await reload();
      const stored = users().find((x) => x.accountId === u.accountId);
      if (stored) box.checked = stored.flags[key];
    } catch {
      box.checked = !value;
      fail(t('admin-users-flag-error'));
    }
  }

  async function onRevoke(u: UserSummary): Promise<void> {
    try {
      await api.revokeSessions(u.accountId);
      await reload();
    } catch {
      fail(t('admin-users-revoke-error'));
    }
  }

  return (
    <section class={css.section} aria-label={t('admin-users-title')}>
      <Show when={!props.accountId}><h2 class={css.heading}>{t(props.provision?'admin-users-provision':'admin-users-title')}</h2></Show>
      <Show when={!props.accountId&&!props.provision}><a class="btn btn--primary" href={hrefFor(sectionPath('users',['+new']))}>{t('admin-users-provision')}</a></Show>
      <Show when={error()}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
      <Feedback kind="error" as="toast" message={failure()} />

      <Show when={props.provision}><a href={hrefFor(sectionPath('users'))}>{t('admin-users-back')}</a><form class={css.card} onSubmit={(e) => void onProvision(e)} aria-label={t('admin-users-provision')}>
        <div class={css.grid}>
          <label class="field">
            <span>{t('admin-users-username')}</span>
            <input type="text" value={username()} onInput={(e) => setUsername(e.currentTarget.value)} />
          </label>
          <label class="field">
            <span>{t('admin-users-domain')}</span>
            <input
              type="text"
              value={domain()}
              placeholder={t('admin-users-domain-placeholder')}
              onInput={(e) => setDomain(e.currentTarget.value)}
            />
          </label>
          <label class="field">
            <span>{t('admin-users-quota-bytes')}</span>
            <input type="number" value={bytes()} onInput={(e) => setBytes(e.currentTarget.value)} />
          </label>
          <label class="field">
            <span>{t('admin-users-quota-msgs')}</span>
            <input type="number" value={msgs()} onInput={(e) => setMsgs(e.currentTarget.value)} />
          </label>
        </div>
        <button type="submit" class="btn btn--primary">
          {t('admin-users-provision')}
        </button>
      </form></Show>

      <Show when={props.accountId}><Index each={users().filter(u=>u.accountId===props.accountId)}>{u=><>
        <section class={css.card}><h3>{t('admin-users-col-flags')}</h3><For each={[
          ['forcePasswordChange','admin-users-force-change-for','admin-users-force-change-help'],
          ['zeroAccess','admin-users-zeroaccess-for','admin-users-zeroaccess-help'],
        ] as const}>{([key,label,help])=><div class="field"><label class="field"><input type="checkbox" checked={u().flags[key]} disabled={u().status==='deleted'||u().status==='deleting'} aria-describedby={`user-${key}-help`} onChange={event=>void patchFlag(u(),key,event.currentTarget)}/>{t(label,{account:u().domain?`${u().username}@${u().domain}`:u().username})}</label><p class={css.note} id={`user-${key}-help`}>{t(help)}</p></div>}</For></section>
        <section class={css.card}><h3>{t('admin-users-col-quota')}</h3><p>{u().quota?`${u().quota?.bytesLimit} / ${u().quota?.msgLimit}`:'—'}</p></section>
        <section class={css.card}><h3>{t('admin-users-access-title')}</h3><p class={css.note}>{t('admin-users-access-help')}</p><UserAccessActions user={u()} onChanged={reload}/></section>
        <section class={css.card}><h3>{t('admin-users-col-sessions')}</h3><button type="button" class="btn btn--ghost" aria-label={t('admin-users-revoke-for',{account:u().accountId})} onClick={()=>void onRevoke(u())}>{t('admin-revoke')}</button></section>
      </>}</Index></Show>
      <Show when={!props.accountId&&!props.provision}><div class={css.card}><Show when={users().length>0} fallback={<p class={css.note}>{t('admin-users-empty')}</p>}><div class={css.tableWrap}><table class={css.table} aria-label={t('admin-users-title')}><thead><tr><th>{t('admin-users-col-account')}</th><th>{t('admin-users-col-quota')}</th><th>{t('admin-users-col-flags')}</th><th>{t('admin-users-configure')}</th></tr></thead><tbody><For each={users()}>{u=><tr><td dir="auto"><a href={hrefFor(sectionPath('users',[u.accountId]))}>{u.domain?`${u.username}@${u.domain}`:u.username}</a><Show when={u.lastSigninAt}><p class={css.note}>{t('admin-users-last-signin')}: {new Date(u.lastSigninAt!*1000).toLocaleString()}</p></Show></td><td>{u.quota?`${u.quota.bytesLimit} / ${u.quota.msgLimit}`:'—'}</td><td><Show when={u.status==='deleted'}><span class={css.badge}>{t('admin-users-deleted')}</span></Show><Show when={u.flags.disabled||u.status==='disabled'}><span class={css.badge}>{t('admin-users-disabled')}</span></Show><Show when={u.flags.forcePasswordChange}><span class={css.badge}>{t('admin-users-force-change')}</span></Show></td><td><UserAccessActions user={u} onChanged={reload} configureHref={hrefFor(sectionPath('users',[u.accountId]))}/></td></tr>}</For></tbody></table></div></Show></div></Show>
    </section>
  );
}
