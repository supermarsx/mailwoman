import { onMount, createEffect, Show, Switch, Match, type JSX } from 'solid-js';
import { AppContext } from './state/context.ts';
import { createAppState } from './state/store.ts';
import { createClient } from './api/client.ts';
import { Login } from './screens/Login.tsx';
import { MailboxScreen } from './screens/Mailbox.tsx';
import { Toast } from './components/Toast.tsx';
import { ConnectionToast } from './realtime/ConnectionToast.tsx';

export function App(): JSX.Element {
  const client = createClient();
  const app = createAppState(client);

  onMount(() => {
    void app.init();
  });

  // Open the realtime push transport once a session exists, and tear it down on
  // logout (plan §2.2). Inert under jsdom (no WebSocket/EventSource).
  createEffect(() => {
    if (app.me() !== null) app.startRealtime();
    else app.stopRealtime();
  });

  // Keep the offline slice's cached header slice in sync with the visible list,
  // so the reduced offline search + offline reads have data to work from (§2.5).
  createEffect(() => {
    app.cacheHeaders(app.messages());
  });

  return (
    <AppContext.Provider value={app}>
      <Show when={app.authChecked()} fallback={<div class="boot">Loading…</div>}>
        <Switch>
          <Match when={app.me() === null}>
            <Login />
          </Match>
          <Match when={app.me() !== null}>
            <MailboxScreen />
          </Match>
        </Switch>
      </Show>
      <Toast />
      <ConnectionToast />
    </AppContext.Provider>
  );
}
