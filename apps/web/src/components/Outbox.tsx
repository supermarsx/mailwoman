import { For, Show, onMount, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { outboxStateOf, type OutboxState } from '../state/slices/outbox.ts';
import type { EmailSubmission } from '../api/jmap-types.ts';

// The honest, visible Outbox (plan §1.3, §2.1): what the engine is holding —
// send-later rows waiting for their `sendAt`, held rows inside the undo-send
// window, plus finalized/canceled history. Backed by `EmailSubmission/query`.

const STATE_LABEL: Record<OutboxState, string> = {
  scheduled: 'Scheduled',
  holding: 'Sending soon',
  sent: 'Sent',
  canceled: 'Canceled',
};

function whenText(sub: EmailSubmission): string {
  if (sub.sendAt === null) return '';
  const d = new Date(sub.sendAt);
  return Number.isNaN(d.getTime()) ? '' : d.toLocaleString();
}

export function Outbox(): JSX.Element {
  const app = useApp();
  onMount(() => void app.refreshOutbox());

  return (
    <section class="outbox" aria-label="Outbox">
      <header class="outbox__header">
        <h2>Outbox</h2>
        <button type="button" class="btn btn--ghost" onClick={() => void app.refreshOutbox()}>
          Refresh
        </button>
      </header>
      <Show when={app.outbox().length > 0} fallback={<p class="outbox__empty">Nothing waiting to send</p>}>
        <ul class="outbox__items">
          <For each={app.outbox()}>
            {(sub) => {
              const state = () => outboxStateOf(sub);
              const cancelable = () => state() === 'scheduled' || state() === 'holding';
              return (
                <li class="outbox__row" data-state={state()}>
                  <span class="outbox__state" classList={{ [`outbox__state--${state()}`]: true }}>
                    {STATE_LABEL[state()]}
                  </span>
                  <span class="outbox__when">{whenText(sub)}</span>
                  <Show when={cancelable()}>
                    <span class="outbox__actions">
                      <button
                        type="button"
                        class="btn btn--ghost"
                        onClick={() => void app.sendOutboxNow(sub.id)}
                      >
                        Send now
                      </button>
                      <button
                        type="button"
                        class="btn btn--ghost"
                        onClick={() => void app.cancelOutbox(sub.id)}
                      >
                        Cancel
                      </button>
                    </span>
                  </Show>
                </li>
              );
            }}
          </For>
        </ul>
      </Show>
    </section>
  );
}
