import { createSignal, For, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { MessageList } from '../components/MessageList.tsx';
import { Reader } from '../components/Reader.tsx';
import { Compose } from '../components/Compose.tsx';

export function MailboxScreen(): JSX.Element {
  const app = useApp();
  const [composing, setComposing] = createSignal(false);

  return (
    <div class="shell">
      <aside class="sidebar">
        <div class="sidebar__head">
          <span class="sidebar__brand">Mailwoman</span>
          <Show when={app.me()}>{(m) => <span class="sidebar__user">{m().username}</span>}</Show>
        </div>
        <button type="button" class="btn btn--primary sidebar__compose" onClick={() => setComposing(true)}>
          Compose
        </button>
        <nav class="sidebar__nav" aria-label="Mailboxes">
          <For each={app.mailboxes()}>
            {(box) => (
              <button
                type="button"
                class="sidebar__box"
                classList={{ 'sidebar__box--active': app.selectedMailboxId() === box.id }}
                onClick={() => void app.selectMailbox(box.id)}
              >
                <span class="sidebar__box-name">{box.name}</span>
                <Show when={box.unreadEmails > 0}>
                  <span class="sidebar__badge">{box.unreadEmails}</span>
                </Show>
              </button>
            )}
          </For>
        </nav>
        <button type="button" class="btn btn--ghost sidebar__logout" onClick={() => void app.logout()}>
          Log out
        </button>
        <Show when={!app.online()}>
          <span class="sidebar__offline" aria-live="polite">
            Offline
          </span>
        </Show>
      </aside>
      <MessageList />
      <Reader />
      <Show when={composing()}>
        <Compose onClose={() => setComposing(false)} />
      </Show>
    </div>
  );
}
