// Admin › Users (§19). Provision users, set quota, revoke sessions, toggle
// feature flags including the zero-access storage toggle (§9), force password
// change, and request a remote cache wipe. Every action audits server-side.

import { createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import type { UserFeatureFlags, UserSummary } from '../../state/slices/admin.ts';
import * as css from './admin.css.ts';

export function Users(): JSX.Element {
  const { api } = useAdmin();
  const [users, setUsers] = createSignal<UserSummary[]>([]);
  const [error, setError] = createSignal<string | null>(null);
  const [domain, setDomain] = createSignal('');
  const [username, setUsername] = createSignal('');
  const [bytes, setBytes] = createSignal('0');
  const [msgs, setMsgs] = createSignal('0');

  async function reload(): Promise<void> {
    try {
      setUsers(await api.listUsers());
      setError(null);
    } catch {
      setError('Could not load users');
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
      setError('Could not provision the user');
    }
  }

  async function patchFlag(u: UserSummary, key: keyof UserFeatureFlags, value: boolean): Promise<void> {
    try {
      if (key === 'zeroAccess') {
        await api.toggleZeroAccess(u.accountId, value);
      } else {
        await api.setFlags(u.accountId, { ...u.flags, [key]: value });
      }
      await reload();
    } catch {
      setError('Could not update the flag');
    }
  }

  async function onRevoke(u: UserSummary): Promise<void> {
    try {
      await api.revokeSessions(u.accountId);
      await reload();
    } catch {
      setError('Could not revoke sessions');
    }
  }

  return (
    <section class={css.section} aria-label="Users">
      <h2 class={css.heading}>Users</h2>
      <Show when={error()}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>

      <form class={css.card} onSubmit={(e) => void onProvision(e)} aria-label="Provision user">
        <div class={css.grid}>
          <label class="field">
            <span>Username</span>
            <input type="text" value={username()} onInput={(e) => setUsername(e.currentTarget.value)} />
          </label>
          <label class="field">
            <span>Domain</span>
            <input type="text" value={domain()} placeholder="example.com" onInput={(e) => setDomain(e.currentTarget.value)} />
          </label>
          <label class="field">
            <span>Quota bytes (0 = unlimited)</span>
            <input type="number" value={bytes()} onInput={(e) => setBytes(e.currentTarget.value)} />
          </label>
          <label class="field">
            <span>Quota messages (0 = unlimited)</span>
            <input type="number" value={msgs()} onInput={(e) => setMsgs(e.currentTarget.value)} />
          </label>
        </div>
        <button type="submit" class="btn btn--primary">
          Provision user
        </button>
      </form>

      <div class={css.card}>
        <Show when={users().length > 0} fallback={<p class={css.note}>No users yet.</p>}>
          <div class={css.tableWrap}>
            <table class={css.table}>
              <thead>
                <tr>
                  <th>Account</th>
                  <th>Quota (bytes/msgs)</th>
                  <th>Zero-access</th>
                  <th>Flags</th>
                  <th>Sessions</th>
                </tr>
              </thead>
              <tbody>
                <For each={users()}>
                  {(u) => (
                    <tr>
                      <td>{u.accountId}</td>
                      <td>{u.quota ? `${u.quota.bytesLimit} / ${u.quota.msgLimit}` : '—'}</td>
                      <td>
                        <label class="field">
                          <input
                            type="checkbox"
                            aria-label={`Zero-access for ${u.accountId}`}
                            checked={u.flags.zeroAccess}
                            onChange={(e) => void patchFlag(u, 'zeroAccess', e.currentTarget.checked)}
                          />
                        </label>
                      </td>
                      <td>
                        <label class="field">
                          <input
                            type="checkbox"
                            aria-label={`Disable ${u.accountId}`}
                            checked={u.flags.disabled}
                            onChange={(e) => void patchFlag(u, 'disabled', e.currentTarget.checked)}
                          />{' '}
                          disabled
                        </label>
                        <label class="field">
                          <input
                            type="checkbox"
                            aria-label={`Force password change for ${u.accountId}`}
                            checked={u.flags.forcePasswordChange}
                            onChange={(e) => void patchFlag(u, 'forcePasswordChange', e.currentTarget.checked)}
                          />{' '}
                          force change
                        </label>
                        <label class="field">
                          <input
                            type="checkbox"
                            aria-label={`Remote cache wipe for ${u.accountId}`}
                            checked={u.flags.remoteCacheWipe}
                            onChange={(e) => void patchFlag(u, 'remoteCacheWipe', e.currentTarget.checked)}
                          />{' '}
                          cache wipe
                        </label>
                      </td>
                      <td>
                        <button
                          type="button"
                          class="btn btn--ghost"
                          aria-label={`Revoke sessions for ${u.accountId}`}
                          onClick={() => void onRevoke(u)}
                        >
                          Revoke
                        </button>
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </Show>
      </div>
    </section>
  );
}
