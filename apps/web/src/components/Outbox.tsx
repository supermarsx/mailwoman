import { For, Show, onMount, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { isolate, t } from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';
import {
  outboxStateOf,
  type OutboxState,
  type OutboxSubmission,
  type SubmissionOrigin,
} from '../state/slices/outbox.ts';

// The honest, visible Outbox (plan §1.3, §2.1): what the engine is holding —
// rows held until the owner releases them, send-later rows waiting for their
// `sendAt`, rows inside the undo-send window — plus the sent, canceled and
// not-sent history. Backed by `EmailSubmission/query`.
//
// A held row is one something other than this client created on the owner's
// behalf (an MCP `mail.send` from an API key without unattended send). The
// server does not send it; "Release" here is what does. So the row says who
// created it and what it would send, before the button.

const STATE_LABEL: Record<OutboxState, string> = {
  held: 'mail-outbox-held',
  scheduled: 'mail-outbox-scheduled',
  holding: 'mail-outbox-holding',
  sent: 'mail-outbox-sent',
  canceled: 'mail-outbox-canceled',
  failed: 'mail-outbox-failed',
};

function whenText(sub: OutboxSubmission): string {
  if (sub.sendAt === null) return '';
  const d = new Date(sub.sendAt);
  return Number.isNaN(d.getTime()) ? '' : d.toLocaleString();
}

/** The line under a held row: that it is held, and who created it. */
function heldNote(origin: SubmissionOrigin | null | undefined): string {
  if (origin === null || origin === undefined) return t('mail-outbox-held-note');
  const name = isolate(origin.name);
  return origin.kind === 'apiKey'
    ? t('mail-outbox-held-note-api-key', { name })
    : t('mail-outbox-held-note-oauth', { name });
}

/** Who created a row that is no longer held (sent, canceled), if not the owner. */
function originText(origin: SubmissionOrigin): string {
  const name = isolate(origin.name);
  return origin.kind === 'apiKey'
    ? t('mail-outbox-origin-api-key', { name })
    : t('mail-outbox-origin-oauth', { name });
}

/** The engine's last error for a row, with the attempt count when it retried. */
function errorText(sub: OutboxSubmission): string {
  const error = sub.mailwomanLastError ?? '';
  if (error === '') return '';
  const count = sub.mailwomanAttempts ?? 0;
  return count > 1 ? t('mail-outbox-attempts', { count, error }) : t('mail-outbox-error', { error });
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
              const held = () => state() === 'held';
              const waiting = () => held() || state() === 'scheduled' || state() === 'holding';
              const message = () => app.outboxMessages()[sub.emailId];
              const recipients = () =>
                (message()?.to ?? []).map((a) => isolate(a.email)).join(', ');
              return (
                <li class="outbox__row" data-state={state()}>
                  <span class="outbox__state" classList={{ [`outbox__state--${state()}`]: true }}>
                    {t(STATE_LABEL[state()])}
                  </span>
                  <span class="outbox__when">{whenText(sub)}</span>
                  <Show when={message() !== undefined}>
                    <span class="outbox__subject">
                      {message()?.subject !== null && message()?.subject !== ''
                        ? isolate(message()?.subject ?? '')
                        : t('mail-outbox-no-subject')}
                    </span>
                    <Show when={recipients() !== ''}>
                      <span class="outbox__to">{t('mail-outbox-to', { recipients: recipients() })}</span>
                    </Show>
                  </Show>
                  <Show when={held()}>
                    <p class="outbox__note">{heldNote(sub.mailwomanOrigin)}</p>
                  </Show>
                  <Show when={!held() && sub.mailwomanOrigin}>
                    {(origin) => <p class="outbox__note">{originText(origin())}</p>}
                  </Show>
                  <Show when={errorText(sub) !== ''}>
                    <p class="outbox__error" role="status">
                      {errorText(sub)}
                    </p>
                  </Show>
                  <Show when={waiting()}>
                    <span class="outbox__actions">
                      <button
                        type="button"
                        class={`btn btn--ghost ${a11y.focusable}`}
                        data-action="release"
                        onClick={() => void app.sendOutboxNow(sub.id)}
                      >
                        {t(held() ? 'mail-outbox-release' : 'mail-outbox-send-now')}
                      </button>
                      <button
                        type="button"
                        class={`btn btn--ghost ${a11y.focusable}`}
                        data-action="cancel"
                        onClick={() => void app.cancelOutbox(sub.id)}
                      >
                        {t(held() ? 'mail-outbox-discard' : 'mail-outbox-cancel')}
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
