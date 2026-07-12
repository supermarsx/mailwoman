// Connection-status toast upgrade (plan §3 e6). Renders a single banner off the
// realtime controller's connection model: offline / degraded (poll fallback) /
// auth-expired, plus a transient "Reconnected" when the socket recovers. It
// reads `useRealtime()` so it works with the app singleton or a test-provided
// controller, and offers a Reconnect action for the recoverable states.

import { Show, createEffect, createSignal, onCleanup, type JSX } from 'solid-js';
import { useRealtime } from './context.ts';
import type { ConnectionState } from './connection.ts';

const MESSAGES: Record<Exclude<ConnectionState, 'online'>, string> = {
  connecting: 'Connecting…',
  degraded: 'Realtime updates paused — reconnecting',
  offline: 'You are offline — changes sync when you reconnect',
  'auth-expired': 'Your session expired — please sign in again',
};

/** These states offer a manual "Reconnect" action from the top of the ladder. */
const RECOVERABLE = new Set<ConnectionState>(['degraded', 'auth-expired']);

export function ConnectionToast(): JSX.Element {
  const rt = useRealtime();
  const state = rt.connection.state;

  const [reconnected, setReconnected] = createSignal(false);
  let prev: ConnectionState = state();
  let timer: ReturnType<typeof setTimeout> | undefined;

  createEffect(() => {
    const s = state();
    // A move to 'online' from any dropped state is a recovery worth surfacing.
    if (s === 'online' && prev !== 'online' && prev !== 'connecting') {
      setReconnected(true);
      if (timer !== undefined) clearTimeout(timer);
      timer = setTimeout(() => setReconnected(false), 2500);
    }
    prev = s;
  });
  onCleanup(() => {
    if (timer !== undefined) clearTimeout(timer);
  });

  return (
    <>
      <Show when={reconnected()}>
        <div class="connection-toast connection-toast--reconnected" role="status" aria-live="polite">
          Reconnected
        </div>
      </Show>
      <Show when={state() !== 'online' && !reconnected()}>
        {(() => {
          const s = state() as Exclude<ConnectionState, 'online'>;
          return (
            <div
              class={`connection-toast connection-toast--${s}`}
              data-state={s}
              role="status"
              aria-live="polite"
            >
              <span>{MESSAGES[s]}</span>
              <Show when={RECOVERABLE.has(s)}>
                <button type="button" class="connection-toast__action" onClick={() => rt.reconnect()}>
                  Reconnect
                </button>
              </Show>
            </div>
          );
        })()}
      </Show>
    </>
  );
}
