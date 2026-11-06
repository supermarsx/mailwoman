// A small status chip for the top bar: how many messages are waiting in the
// Outbox (queued, scheduled or held). It is not drawn when nothing waits.
// Opening it lists the queued messages soonest first with the next send time
// and a Recall and a Send now button on each, and a link to the full Outbox.

import { For, Show, createEffect, createSignal, onCleanup, type JSX } from 'solid-js';
import './OutboxReader.css';
import { formatDateTime, isolate, loadCatalog, t } from '../i18n/index.ts';
import { formatCountdown } from '../state/sendDelay.ts';
import { outboxStatus, queuedRows } from '../state/outboxStatus.ts';
import { useApp } from '../state/context.ts';
import { Icon } from './Icon.tsx';
import * as a11y from './mailA11y.css.ts';

export interface OutboxChipProps {
  onOpenOutbox: () => void;
}

export function OutboxChip(props: OutboxChipProps): JSX.Element {
  const app = useApp();
  void loadCatalog('send-delay');
  const [open, setOpen] = createSignal(false);
  const [now, setNow] = createSignal(Date.now());
  const rows = () => queuedRows(app.outbox(), now());
  // The clock only runs while the list is on screen.
  createEffect(() => {
    if (!open()) return;
    setNow(Date.now());
    const timer = setInterval(() => setNow(Date.now()), 1000);
    onCleanup(() => clearInterval(timer));
  });
  // Nothing left to show: close the panel with the chip.
  createEffect(() => { if (rows().length === 0 && open()) setOpen(false); });
  const next = () => rows().map((row) => outboxStatus(row, now())).find((status) => status.dueAt !== null)?.dueAt ?? null;
  const when = (id: string): string => {
    const row = rows().find((value) => value.id === id);
    if (row === undefined) return '';
    const status = outboxStatus(row, now());
    if (status.kind === 'held') return t('mail-outbox-held');
    if (status.dueAt === null) return '';
    return status.secondsLeft !== null && status.secondsLeft <= 3600
      ? t('outbox-sends-in', { time: formatCountdown(status.secondsLeft) })
      : t('outbox-sends-at', { when: formatDateTime(status.dueAt) });
  };

  return (
    <Show when={rows().length > 0}>
      <span class="outbox-chip" data-testid="outbox-chip">
        <button
          type="button"
          class={`btn btn--ghost outbox-chip__toggle ${a11y.focusable}`}
          aria-haspopup="dialog"
          aria-expanded={open()}
          aria-label={t('outbox-queued-count', { count: rows().length })}
          title={t('outbox-queued-count', { count: rows().length })}
          onClick={() => setOpen(!open())}
        >
          <Icon name="bell" />
          <span class="outbox-chip__count" data-testid="outbox-chip-count">{rows().length}</span>
        </button>
        <Show when={open()}>
          <div class="outbox-chip__panel" role="dialog" aria-label={t('outbox-chip-title')} onKeyDown={(event) => { if (event.key === 'Escape') setOpen(false); }}>
            <Show when={next() !== null}>
              <p class="outbox-chip__next" data-testid="outbox-chip-next">{t('outbox-chip-next', { when: formatDateTime(next() ?? '') })}</p>
            </Show>
            <ul class="outbox-chip__list">
              <For each={rows()}>
                {(row) => {
                  const message = () => app.outboxMessages()[row.emailId];
                  const held = () => outboxStatus(row, now()).kind === 'held';
                  return (
                    <li class="outbox-chip__item" data-sub-id={row.id}>
                      <span class="outbox-chip__subject">{message()?.subject ? isolate(message()?.subject ?? '') : t('mail-outbox-no-subject')}</span>
                      <span class="outbox-chip__when">{when(row.id)}</span>
                      <span class="outbox-chip__actions">
                        <button type="button" class={`btn btn--ghost ${a11y.focusable}`} data-action="send-now" onClick={() => void app.sendOutboxNow(row.id)}>
                          {t(held() ? 'mail-outbox-release' : 'mail-outbox-send-now')}
                        </button>
                        <Show when={!held()}>
                          <button type="button" class={`btn btn--ghost ${a11y.focusable}`} data-action="recall" onClick={() => void app.cancelOutbox(row.id)}>
                            {t('outbox-recall-to-drafts')}
                          </button>
                        </Show>
                      </span>
                    </li>
                  );
                }}
              </For>
            </ul>
            <button type="button" class={`btn btn--ghost ${a11y.focusable}`} data-action="open-outbox" onClick={() => { setOpen(false); props.onOpenOutbox(); }}>
              {t('outbox-chip-view-all')}
            </button>
          </div>
        </Show>
      </span>
    </Show>
  );
}
