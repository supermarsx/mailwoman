// Connection-status toast upgrade (plan §3 e6). Renders a single banner off the
// realtime controller's connection model: offline / degraded (poll fallback) /
// auth-expired, plus a transient "Reconnected" when the socket recovers. It
// reads `useRealtime()` so it works with the app singleton or a test-provided
// controller, and offers a Reconnect action for the recoverable states.
//
// It is mounted outside the signed-in branch of App.tsx, so it also renders on
// the login screen. There the model is `idle` (no push transport is wanted) and
// the banner shows nothing: it reports on a session's connection, not on the
// browser. The copy is in the `common` catalog, which ships in the entry chunk.

import { Show, createEffect, createSignal, onCleanup, type JSX } from 'solid-js';
import { useRealtime } from './context.ts';
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

/** These states offer a manual "Reconnect" action from the top of the ladder. */
const RECOVERABLE = new Set<ConnectionState>(['degraded', 'auth-expired']);

export function ConnectionToast(): JSX.Element {
  const rt = useRealtime();
  const state = rt.connection.state;

  const [reconnected, setReconnected] = createSignal(false);
  // Whether this session's connection has been down (offline, degraded or
  // auth-expired) since it was last online. A recovery passes through
  // 'connecting' on its way back, so the previous state alone cannot tell a
  // recovery from the first connect of a session (idle → connecting → online).
  let wasDown = false;
  let timer: ReturnType<typeof setTimeout> | undefined;

  createEffect(() => {
    const s = state();
    if (s === 'online') {
      if (wasDown) {
        setReconnected(true);
        if (timer !== undefined) clearTimeout(timer);
        timer = setTimeout(() => setReconnected(false), 2500);
      }
      wasDown = false;
    } else if (s === 'idle') {
      // The session ended; there is nothing left to have recovered.
      wasDown = false;
      if (timer !== undefined) clearTimeout(timer);
      setReconnected(false);
    } else if (s !== 'connecting') {
      wasDown = true;
    }
  });
  onCleanup(() => {
    if (timer !== undefined) clearTimeout(timer);
  });

  return (
    <>
      <Show when={reconnected()}>
        <div class="connection-toast connection-toast--reconnected" role="status" aria-live="polite">
          {t('common-conn-reconnected')}
        </div>
      </Show>
      <Show when={isBannerState(state()) && !reconnected()}>
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
            </div>
          );
        })()}
      </Show>
    </>
  );
}
