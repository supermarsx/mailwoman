import { For, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import type { Email, EmailAddress } from '../api/jmap-types.ts';

function senderLabel(from: EmailAddress[] | null): string {
  const first = from?.[0];
  if (first === undefined) return '(unknown sender)';
  return first.name && first.name.length > 0 ? first.name : first.email;
}

function formatDate(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return '';
  return d.toLocaleString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function MessageList(): JSX.Element {
  const app = useApp();

  return (
    <section class="list" aria-label="Messages">
      <Show
        when={!app.listLoading()}
        fallback={<p class="list__empty">Loading messages…</p>}
      >
        <Show
          when={app.messages().length > 0}
          fallback={<p class="list__empty">No messages</p>}
        >
          <ul class="list__items">
            <For each={app.messages()}>
              {(email: Email) => (
                <li>
                  <button
                    type="button"
                    class="list__row"
                    classList={{ 'list__row--active': app.openEmail()?.id === email.id }}
                    onClick={() => void app.openMessage(email.id)}
                  >
                    <span class="list__sender">{senderLabel(email.from)}</span>
                    <span class="list__subject">{email.subject ?? '(no subject)'}</span>
                    <span class="list__preview">{email.preview}</span>
                    <span class="list__date">{formatDate(email.receivedAt)}</span>
                  </button>
                </li>
              )}
            </For>
          </ul>
        </Show>
      </Show>
    </section>
  );
}
