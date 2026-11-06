import { UsersTable } from './Users/UsersTable.tsx';
import { UserDetail } from './Users/UserDetail.tsx';
import { loadCatalog } from '../../i18n/catalog.ts';
import { ShellContext } from './shell/context.ts';
import { hrefFor, sectionPath } from './shell/router.ts';
// Admin › Users (§19). Provision users, set quota, revoke sessions, and set the
// per-account flags: disabled, force password change, and the zero-access flag.
// Every action audits server-side.
//
// There is no "cache wipe" checkbox: `remoteCacheWipe` is stored by
// `PUT /admin/users/{id}/flags` and read by nothing, so ticking it wiped no cache
// (26.20, t28-e8). The field stays in `UserFeatureFlags` because the server sends
// and expects it (`FlagsDto`, `crates/mw-server/src/admin.rs`), and is passed
// through unchanged when another flag is saved.

import { createSignal, useContext, Show, onMount, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import { t } from '../../i18n';
import { Feedback } from './shell/Feedback.tsx';
import * as css from './admin.css.ts';

export function Users(props: { accountId?: string; tab?: string; provision?: boolean } = {}): JSX.Element {
  const shell = useContext(ShellContext);
  const identity = () => props.accountId ?? shell?.router.route().rest[0];
  return <Show when={props.provision || identity()==='+new'} fallback={<Show when={identity()} keyed fallback={<UserList/>}>{id=><UserIdentity id={id} tab={props.tab}/>}</Show>}><UserList provision/></Show>;
}
function UserIdentity(props: { id: string; tab?: string | undefined }): JSX.Element {
  return <UserDetail id={props.id} {...(props.tab ? { tab: props.tab } : {})} />;
}
function UserList(props: { provision?: boolean }): JSX.Element {
  const { api } = useAdmin();
  
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
      await api.listUsers();
      setError(null);
    } catch {
      setError(t('admin-users-load-error'));
    }
  }
  onMount(() => { void loadCatalog('admin-users'); if (props.provision) void reload(); });

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

  return (
    <section class={css.section} aria-label={t('admin-users-title')}>
      <h2 class={css.heading}>{t(props.provision?'admin-users-provision':'admin-users-title')}</h2>
      <Show when={!props.provision}><a class="btn btn--primary" href={hrefFor(sectionPath('users',['+new']))}>{t('admin-users-provision')}</a></Show>
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

      <Show when={!props.provision}><UsersTable/></Show>
    </section>
  );
}
