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

import { createSignal, createResource, useContext, Index, Show, onMount, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import type { UserFeatureFlags, UserSummary } from '../../state/slices/admin.ts';
import { t } from '../../i18n';
import { Feedback } from './shell/Feedback.tsx';
import * as css from './admin.css.ts';

export function Users(): JSX.Element {
  const shell = useContext(ShellContext);
  const identity = () => shell?.router.route().rest[0];
  return <Show when={identity()} fallback={<UserList />}>{id => <UserIdentity id={id()} />}</Show>;
}
function UserIdentity(props: { id: string }): JSX.Element {
  const [identity] = createResource(() => props.id, async id => (await adminRequest<{accountId:string;label:string;address:string}>(`/users/${encodeURIComponent(id)}/identity`)).data);
  return <section class={css.section} aria-label={t('admin-users-title')}>
    <a href={hrefFor(sectionPath('users'))}>{t('admin-users-back')}</a>
    <h2 class={css.heading}>{t('admin-users-identity')}</h2>
    <Show when={identity.loading}><AsyncPending /></Show>
    <Show when={identity.error}><p role="alert">{t('admin-users-load-error')}</p></Show>
    <Show when={identity()}>{value => <div class={css.card}><h3 dir="auto">{value().label}</h3><p dir="auto">{value().address}</p><p class={css.note}>{t('admin-users-id')}: <code class={auditCss.actor}>{value().accountId}</code></p></div>}</Show>
  </section>;
}
function UserList(): JSX.Element {
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
      <h2 class={css.heading}>{t('admin-users-title')}</h2>
      <Show when={error()}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
      <Feedback kind="error" as="toast" message={failure()} />

      <form class={css.card} onSubmit={(e) => void onProvision(e)} aria-label={t('admin-users-provision')}>
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
      </form>

      <div class={css.card}>
        <Show when={users().length > 0} fallback={<p class={css.note}>{t('admin-users-empty')}</p>}>
          <div class={css.tableWrap}>
            <table class={css.table}>
              <thead>
                <tr>
                  <th>{t('admin-users-col-account')}</th>
                  <th>{t('admin-users-col-quota')}</th>
                  <th>{t('admin-users-col-zeroaccess')}</th>
                  <th>{t('admin-users-col-flags')}</th>
                  <th>{t('admin-users-col-sessions')}</th>
                </tr>
              </thead>
              <tbody>
                <Index each={users()}>
                  {(u, i) => (
                    <tr>
                      <td dir="auto"><a href={hrefFor(sectionPath('users', [u().accountId]))}>{u().domain ? `${u().username}@${u().domain}` : u().username}</a><Show when={u().lastSigninAt}><p class={css.note}>{t('admin-users-last-signin')}: {new Date(u().lastSigninAt! * 1000).toLocaleString()}</p></Show></td>
                      <td>{u().quota ? `${u().quota?.bytesLimit} / ${u().quota?.msgLimit}` : '—'}</td>
                      <td>
                        <label class="field">
                          <input
                            type="checkbox"
                            aria-label={t('admin-users-zeroaccess-for', { account: u().accountId })}
                            aria-describedby={`admin-users-zeroaccess-help-${i}`}
                            checked={u().flags.zeroAccess}
                            onChange={(e) => void patchFlag(u(), 'zeroAccess', e.currentTarget)}
                          />
                        </label>
                        <p class={css.note} id={`admin-users-zeroaccess-help-${i}`}>
                          {t('admin-users-zeroaccess-help')}
                        </p>
                      </td>
                      <td>
                        <label class="field">
                          <input
                            type="checkbox"
                            aria-label={t('admin-users-disable-for', { account: u().accountId })}
                            aria-describedby={`admin-users-disabled-help-${i}`}
                            checked={u().flags.disabled}
                            onChange={(e) => void patchFlag(u(), 'disabled', e.currentTarget)}
                          />{' '}
                          {t('admin-users-disabled')}
                        </label>
                        <p class={css.note} id={`admin-users-disabled-help-${i}`}>
                          {t('admin-users-disabled-help')}
                        </p>
                        <label class="field">
                          <input
                            type="checkbox"
                            aria-label={t('admin-users-force-change-for', { account: u().accountId })}
                            aria-describedby={`admin-users-force-change-help-${i}`}
                            checked={u().flags.forcePasswordChange}
                            onChange={(e) => void patchFlag(u(), 'forcePasswordChange', e.currentTarget)}
                          />{' '}
                          {t('admin-users-force-change')}
                        </label>
                        <p class={css.note} id={`admin-users-force-change-help-${i}`}>
                          {t('admin-users-force-change-help')}
                        </p>
                      </td>
                      <td>
                        <button
                          type="button"
                          class="btn btn--ghost"
                          aria-label={t('admin-users-revoke-for', { account: u().accountId })}
                          onClick={() => void onRevoke(u())}
                        >
                          {t('admin-revoke')}
                        </button>
                      </td>
                    </tr>
                  )}
                </Index>
              </tbody>
            </table>
          </div>
        </Show>
      </div>
    </section>
  );
}
