import { onMount, Show, Switch, Match, type JSX } from 'solid-js';
import { AppContext } from './state/context.ts';
import { createAppState } from './state/store.ts';
import { createClient } from './api/client.ts';
import { Login } from './screens/Login.tsx';
import { MailboxScreen } from './screens/Mailbox.tsx';
import { Toast } from './components/Toast.tsx';

export function App(): JSX.Element {
  const client = createClient();
  const app = createAppState(client);

  onMount(() => {
    void app.init();
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
    </AppContext.Provider>
  );
}
