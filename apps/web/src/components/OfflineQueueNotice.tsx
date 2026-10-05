import { For, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t, isolate } from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';
import * as css from './offlineQueueNotice.css.ts';
import type { QueuedItem, SendPayload } from '../offline/outbox.ts';

// The changes made offline that could not be applied (state/slices/offline.ts,
// `offlineFailed`): a move, label change or send the server refused on replay,
// or one whose replay kept erroring. Each stays here, with the reason the queue
// recorded, until the user retries it or discards it — the queue never replays
// a failed item by itself. Renders nothing while there are none.

/** What a queued item was going to do, in a few words. */
export function describeQueued(item: QueuedItem): string {
  switch (item.type) {
    case 'send': {
      const subject = ((item.payload as Partial<SendPayload> | null)?.draft?.subject ?? '').trim();
      return subject.length > 0
        ? t('mail-queue-kind-send', { subject: isolate(subject) })
        : t('mail-queue-kind-send-untitled');
    }
    case 'move':
      return t('mail-queue-kind-move');
    case 'flag':
      return t('mail-queue-kind-flag');
    case 'draft':
      return t('mail-queue-kind-draft');
    case 'pim':
      return t('mail-queue-kind-pim');
  }
}

export function OfflineQueueNotice(): JSX.Element {
  const app = useApp();
  return (
    <Show when={app.offlineFailed().length > 0}>
      <section class={css.notice} aria-label={t('mail-queue-notice-label')} data-testid="offline-queue-failed">
        <h3 class={css.title}>{t('mail-queue-notice-title', { count: app.offlineFailed().length })}</h3>
        <ul class={css.list}>
          <For each={app.offlineFailed()}>
            {(item) => (
              <li class={css.item} data-testid={`offline-failed-${item.id}`}>
                <span class={css.what}>
                  <span>{describeQueued(item)}</span>
                  <span class={css.reason}>
                    {item.lastError !== undefined && item.lastError.length > 0
                      ? t('mail-queue-reason', { reason: isolate(item.lastError) })
                      : t('mail-queue-reason-unknown')}
                  </span>
                </span>
                <span class={css.buttons}>
                  <button
                    type="button"
                    class={`${css.button} ${a11y.focusable}`}
                    aria-label={t('mail-queue-retry-item', { what: describeQueued(item) })}
                    onClick={() => void app.retryOffline(item.id)}
                  >
                    {t('mail-queue-retry')}
                  </button>
                  <button
                    type="button"
                    class={`${css.button} ${a11y.focusable}`}
                    aria-label={t('mail-queue-discard-item', { what: describeQueued(item) })}
                    onClick={() => void app.discardOffline(item.id)}
                  >
                    {t('mail-queue-discard')}
                  </button>
                </span>
              </li>
            )}
          </For>
        </ul>
      </section>
    </Show>
  );
}
