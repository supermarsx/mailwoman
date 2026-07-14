import { For, Show, onMount, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t } from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';
import { outboxStateOf, type OutboxState } from '../state/slices/outbox.ts';
import type { EmailSubmission } from '../api/jmap-types.ts';

// The honest, visible Outbox (plan §1.3, §2.1): what the engine is holding —
// send-later rows waiting for their `sendAt`, held rows inside the undo-send
// window, plus finalized/canceled history. Backed by `EmailSubmission/query`.

const STATE_LABEL: Record<OutboxState, string> = {
  scheduled: 'mail-outbox-scheduled',
  holding: 'mail-outbox-holding',
  sent: 'mail-outbox-sent',
  canceled: 'mail-outbox-canceled',
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
    <section class="outbox" aria-label={t('mail-outbox-label')}>
      <header class="outbox__header">
        <h2>{t('mail-outbox-label')}</h2>
        <button type="button" class={`btn btn--ghost ${a11y.focusable}`} onClick={() => void app.refreshOutbox()}>
          {t('mail-outbox-refresh')}
        </button>
      </header>
      <Show when={app.outbox().length > 0} fallback={<p class="outbox__empty">{t('mail-outbox-empty')}</p>}>
        <ul class="outbox__items">
          <For each={app.outbox()}>
            {(sub) => {
              const state = () => outboxStateOf(sub);
              const cancelable = () => state() === 'scheduled' || state() === 'holding';
              return (
                <li class="outbox__row" data-state={state()}>
                  <span class="outbox__state" classList={{ [`outbox__state--${state()}`]: true }}>
                    {t(STATE_LABEL[state()])}
                  </span>
                  <span class="outbox__when">{whenText(sub)}</span>
                  <Show when={cancelable()}>
                    <span class="outbox__actions">
                      <button
                        type="button"
                        class={`btn btn--ghost ${a11y.focusable}`}
                        onClick={() => void app.sendOutboxNow(sub.id)}
                      >
                        {t('mail-outbox-send-now')}
                      </button>
                      <button
                        type="button"
                        class={`btn btn--ghost ${a11y.focusable}`}
                        onClick={() => void app.cancelOutbox(sub.id)}
                      >
                        {t('mail-outbox-cancel')}
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
