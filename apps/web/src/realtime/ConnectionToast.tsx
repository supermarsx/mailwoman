// Connection-status banner (plan §3 e6). Renders a single PERSISTENT banner off
// the realtime controller's connection model: connecting / offline / degraded
// (poll fallback) / auth-expired. These are states, not events, so they stay a
// banner for as long as they hold rather than a toast that times out (t32 f39:
// persistent banners are not toasts). The one transient message, "Reconnected"
// when the socket recovers, is a toast in the shared toast store
// (components/toast). It reads `useRealtime()` so it works with the app
// singleton or a test-provided controller, and offers a Reconnect action for
// the recoverable states.
//
// An ended session (auth-expired) offers "Sign in again" instead: reconnecting
// cannot revive it. The action is the store's `logout()`, which clears the
// session and so returns the shell to the login screen.
//
// It is mounted outside the signed-in branch of App.tsx, so it also renders on
// the login screen. There the model is `idle` (no push transport is wanted) and
// the banner shows nothing: it reports on a session's connection, not on the
// browser. The copy is in the `common` catalog, which ships in the entry chunk.

import { Show, createEffect, useContext, type JSX } from 'solid-js';
import { useRealtime } from './context.ts';
import { toast } from '../components/toast/index.ts';
import { AppContext } from '../state/context.ts';
import type { ConnectionState } from './connection.ts';
import { t } from '../i18n/index.ts';

/** The states that put a banner on screen. */
type BannerState = Exclude<ConnectionState, 'online' | 'idle'>;

// Literal ids at each call, so check-catalog.mjs can resolve them.
const MESSAGES: Record<BannerState, () => string> = {
  connecting: () => t('common-conn-connecting'),
  degraded: () => t('common-conn-degraded'),
  offline: () => t('common-conn-offline'),
  'auth-expired': () => t('common-conn-auth-expired'),
};

function isBannerState(s: ConnectionState): s is BannerState {
  return s !== 'online' && s !== 'idle';
}

/** The toast id of "Reconnected", so a second recovery replaces the first. */
const RECONNECTED_TOAST_ID = 'connection-reconnected';

/** These states offer a manual "Reconnect" action from the top of the ladder. */
const RECOVERABLE = new Set<ConnectionState>(['degraded']);

export function ConnectionToast(): JSX.Element {
  const rt = useRealtime();
  // Read directly rather than through `useApp()`, which throws without a
  // provider: the banner is also rendered on its own against a bare controller.
  const app = useContext(AppContext);

  function signInAgain(): void {
    // The server may refuse the logout request of a session it already ended;
    // `logout()` clears the local session either way, which is the point here.
    void app?.logout().catch(() => undefined);
  }
  const state = rt.connection.state;

  // Whether this session's connection has been down (offline, degraded or
  // auth-expired) since it was last online. A recovery passes through
  // 'connecting' on its way back, so the previous state alone cannot tell a
  // recovery from the first connect of a session (idle → connecting → online).
  let wasDown = false;

  createEffect(() => {
    const s = state();
    if (s === 'online') {
      if (wasDown) toast.success(t('common-conn-reconnected'), { id: RECONNECTED_TOAST_ID });
      wasDown = false;
    } else if (s === 'idle') {
      // The session ended; there is nothing left to have recovered.
      wasDown = false;
      toast.dismiss(RECONNECTED_TOAST_ID);
    } else {
      // The banner is back: an earlier "Reconnected" is no longer true.
      toast.dismiss(RECONNECTED_TOAST_ID);
      if (s !== 'connecting') wasDown = true;
    }
  });

  return (
    <>
      <Show when={isBannerState(state())}>
        {(() => {
          const s = state() as BannerState;
          return (
            <div
              class={`connection-toast connection-toast--${s}`}
              data-state={s}
              role="status"
              aria-live="polite"
            >
              <span>{MESSAGES[s]()}</span>
              <Show when={RECOVERABLE.has(s)}>
                <button type="button" class="connection-toast__action" onClick={() => rt.reconnect()}>
                  {t('common-conn-reconnect')}
                </button>
              </Show>
              <Show when={s === 'auth-expired' && app !== undefined}>
                <button type="button" class="connection-toast__action" onClick={signInAgain}>
                  {t('common-conn-sign-in-again')}
                </button>
              </Show>
            </div>
          );
        })()}
      </Show>
    </>
  );
}
