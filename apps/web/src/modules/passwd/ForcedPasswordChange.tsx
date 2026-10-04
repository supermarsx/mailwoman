// The forced password-change screen (t27, OH-1).
//
// An administrator can set `force_password_change` on an account. The server
// then lets the account sign in but answers everything except `/api/me`, logout,
// session rotation and the two password endpoints with
// `403 {"error":"password change required","passwordChangeRequired":true}`. The
// shell (`App.tsx`) renders this screen INSTEAD of the mailbox for such a
// session, so no mail request is issued while the hold is in force.
//
// The hold ends only when the server says so: after a change is accepted,
// `/api/me` is read again, and the mailbox mounts only if the flag is gone from
// it. Nothing on the client clears the hold on its own.
//
// The server may have no backend able to change this user's password (the
// default `Local` backend only knows accounts with a local hash). The change is
// then refused and the account stays here; the screen says that, and says who
// can resolve it, rather than implying a retry will work.

import { createSignal, onMount, Show, type JSX } from 'solid-js';
import { useApp } from '../../state/context.ts';
import { t, isolate, loadCatalog } from '../../i18n';
import { PasswordChange } from './PasswordChange.tsx';
import * as css from './styles.css.ts';

type After = 'idle' | 'loading' | 'still-required' | 'reload-failed';

export function ForcedPasswordChange(): JSX.Element {
  const app = useApp();
  onMount(() => void loadCatalog('passwd'));

  const [after, setAfter] = createSignal<After>('idle');

  /** The server accepted a change: ask it whether the hold is over. */
  async function reload(): Promise<void> {
    setAfter('loading');
    try {
      // Re-reads `/api/me`. Without the flag this loads mail and the shell
      // swaps this screen for the mailbox; with it, `me()` stays held.
      await app.init();
      if (app.me()?.passwordChangeRequired === true) setAfter('still-required');
    } catch {
      setAfter('reload-failed');
    }
  }

  return (
    <main class={css.forced} data-testid="forced-password-change">
      <div class={css.forcedBody}>
        <h1 class={css.forcedTitle}>{t('passwd-forced-title')}</h1>
        <p class={css.prose}>{t('passwd-forced-signed-in-as', { username: isolate(app.me()?.username) })}</p>
        <p class={css.prose}>{t('passwd-forced-explain')}</p>

        <PasswordChange accountId={app.me()?.accountId ?? ''} onChanged={() => void reload()} />

        <Show when={after() === 'still-required'}>
          <p class={css.banner} role="alert" data-testid="forced-still-required">
            {t('passwd-forced-still-required')}
          </p>
        </Show>
        <Show when={after() === 'reload-failed'}>
          <p class={css.banner} role="alert" data-testid="forced-reload-failed">
            {t('passwd-forced-reload-failed')}
          </p>
          <button type="button" class="btn btn--primary" onClick={() => void reload()}>
            {t('passwd-forced-retry')}
          </button>
        </Show>

        <p class={css.meta}>{t('passwd-forced-backend-note')}</p>

        <button type="button" class="btn btn--ghost" onClick={() => void app.logout()}>
          {t('passwd-forced-sign-out')}
        </button>
      </div>
    </main>
  );
}

export default ForcedPasswordChange;
