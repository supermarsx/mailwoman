// The Outbox reader: a queued message with a status header (queued, scheduled,
// sending, retrying, failed), the due time with a live countdown, the retry
// count and last error, and the actions that apply to its state. The body is
// shown as plain text; the message is not delivered yet and nothing in it is
// rendered as markup.

import { Show, createResource, createSignal, type JSX } from 'solid-js';
import './OutboxReader.css';
import { formatDateTime, isolate, t } from '../i18n/index.ts';
import { formatCountdown } from '../state/sendDelay.ts';
import { outboxStatus, type OutboxStatus } from '../state/outboxStatus.ts';
import type { OutboxSubmission } from '../state/slices/outbox.ts';
import { useApp } from '../state/context.ts';
import { OutboxEditTime } from './OutboxEditTime.tsx';
import * as a11y from './mailA11y.css.ts';

export interface OutboxDetailProps {
  sub: OutboxSubmission;
  now: number;
  onClose?: () => void;
}

const LONG_COUNTDOWN_SECONDS = 3600;

/** The line under the status label: when it goes, or what is being retried. */
export function statusWhen(status: OutboxStatus): string {
  if (status.kind === 'scheduled' || status.kind === 'queued') {
    if (status.dueAt === null) return '';
    const when = formatDateTime(status.dueAt);
    return status.secondsLeft !== null && status.secondsLeft <= LONG_COUNTDOWN_SECONDS
      ? t('outbox-status-sends-at-in', { when, time: formatCountdown(status.secondsLeft) })
      : t('outbox-sends-at', { when });
  }
  if (status.kind === 'retrying' && status.nextAttemptAt !== null) {
    return t('outbox-status-next-attempt', { when: formatDateTime(status.nextAttemptAt) });
  }
  return '';
}

export function OutboxDetail(props: OutboxDetailProps): JSX.Element {
  const app = useApp();
  const status = (): OutboxStatus => outboxStatus(props.sub, props.now);
  const [body] = createResource(() => props.sub.emailId, (id) => app.outboxBody(id));
  const [editing, setEditing] = createSignal(false);
  const [confirmDelete, setConfirmDelete] = createSignal(false);
  const btn = `btn btn--ghost ${a11y.focusable}`;
  const addresses = (list: { name?: string | null; email: string }[] | undefined): string => (list ?? []).map((a) => isolate(a.email)).join(', ');

  return (
    <article class="outbox-reader" data-testid="outbox-reader" aria-label={t('outbox-reader-label')}>
      <div class="outbox-reader__status" data-kind={status().kind} role="status">
        <span class="outbox-reader__label" data-testid="outbox-status-label">{t(`outbox-status-${status().kind}`)}</span>
        <Show when={statusWhen(status()) !== ''}>
          <p class="outbox-reader__meta" data-testid="outbox-status-when">{statusWhen(status())}</p>
        </Show>
        <Show when={status().attempts > 0}>
          <p class="outbox-reader__meta" data-testid="outbox-status-attempts">{t('outbox-status-attempts', { count: status().attempts })}</p>
        </Show>
        <Show when={status().kind === 'uncertain'}>
          <p class="outbox-reader__error">{t('mail-outbox-uncertain-note')}</p>
        </Show>
        <Show when={status().lastError !== '' && status().kind !== 'uncertain'}>
          <p class="outbox-reader__error" data-testid="outbox-status-error">{t('outbox-status-last-error', { error: isolate(status().lastError) })}</p>
        </Show>
        <Show when={status().kind === 'failed'}>
          <p class="outbox-reader__meta">{t('outbox-failed-drafts')}</p>
        </Show>
      </div>

      <div class="outbox-reader__actions" role="toolbar" aria-label={t('outbox-reader-actions')}>
        <Show when={status().waiting}>
          <button type="button" class={btn} data-action="send-now" onClick={() => void app.sendOutboxNow(props.sub.id)}>
            {t(status().kind === 'held' ? 'mail-outbox-release' : 'mail-outbox-send-now')}
          </button>
        </Show>
        <Show when={status().canEditTime}>
          <button type="button" class={btn} data-action="edit-time" onClick={() => setEditing(true)}>{t('outbox-edit-time')}</button>
        </Show>
        <Show when={status().waiting && status().kind !== 'held'}>
          <button type="button" class={btn} data-action="recall" onClick={() => void app.cancelOutbox(props.sub.id)}>{t('outbox-recall-to-drafts')}</button>
          <button type="button" class={btn} data-action="edit-message" onClick={() => void app.recallOutbox(props.sub.id)}>{t('outbox-recall-edit')}</button>
        </Show>
        <Show when={status().canSendAgain}>
          <button type="button" class={btn} data-action="send-again" onClick={() => void app.sendOutboxAgain(props.sub.id)}>{t('outbox-send-again')}</button>
        </Show>
        <Show when={status().waiting || status().kind === 'failed'}>
          <button
            type="button"
            class={btn}
            data-action="delete"
            onClick={() => {
              if (!confirmDelete()) { setConfirmDelete(true); return; }
              void app.deleteOutbox(props.sub.id).then((gone) => { if (gone) props.onClose?.(); else setConfirmDelete(false); });
            }}
          >
            {t(confirmDelete() ? 'outbox-delete-confirm' : 'outbox-delete')}
          </button>
        </Show>
      </div>

      <Show when={body()} fallback={<p class="outbox-reader__meta">{body.loading ? t('outbox-reader-loading') : t('outbox-reader-unavailable')}</p>}>
        {(message) => (
          <>
            <h3 class="outbox-reader__subject">{message().subject ? isolate(message().subject ?? '') : t('mail-outbox-no-subject')}</h3>
            <p class="outbox-reader__meta">{t('outbox-reader-to', { recipients: addresses(message().to) })}</p>
            <Show when={message().cc.length > 0}>
              <p class="outbox-reader__meta">{t('outbox-reader-cc', { recipients: addresses(message().cc) })}</p>
            </Show>
            <pre class="outbox-reader__body">{message().text}</pre>
          </>
        )}
      </Show>

      <Show when={editing()}>
        <OutboxEditTime current={props.sub.sendAt} save={(iso) => app.rescheduleOutbox(props.sub.id, iso)} onClose={() => setEditing(false)} />
      </Show>
    </article>
  );
}
