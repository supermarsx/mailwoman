import {AsyncPending} from './AsyncState.tsx';
import { For, Show, createSignal, onCleanup, onMount, type JSX } from 'solid-js';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent,type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import { useApp } from '../state/context.ts';
import { formatDateTime, isolate, loadCatalog, t } from '../i18n/index.ts';
import { formatCountdown } from '../state/sendDelay.ts';
import * as a11y from './mailA11y.css.ts';
import * as css from './outbox.css.ts';
import './OutboxReader.css';
import { OutboxDetail } from './OutboxDetail.tsx';
import { OutboxEditTime } from './OutboxEditTime.tsx';
import { createContextMenu, type ContextAction } from './ContextMenu.tsx';
import { outboxStatus } from '../state/outboxStatus.ts';
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
  uncertain: 'mail-outbox-uncertain',
};

function whenText(sub: OutboxSubmission, now: number): string {
  const due = sub.mailwomanDueAt ?? sub.sendAt;
  if (due === null || due === undefined) return '';
  const state = outboxStateOf(sub, now);
  if (state === 'holding') {
    const left = (new Date(due).getTime() - now) / 1000;
    return left > 0 ? t('outbox-sends-in', { time: formatCountdown(left) }) : '';
  }
  if (state === 'scheduled') return t('outbox-sends-at', { when: formatDateTime(due) });
  return sub.sendAt === null ? '' : formatDateTime(sub.sendAt);
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
  if (sub.mailwomanDeliveryUncertain === true) return t('mail-outbox-uncertain-note');
  const error = sub.mailwomanLastError ?? '';
  if (error === '') return '';
  const count = sub.mailwomanAttempts ?? 0;
  return count > 1 ? t('mail-outbox-attempts', { count, error }) : t('mail-outbox-error', { error });
}

export function Outbox(): JSX.Element {
  const app = useApp();
  const [failed,setFailed]=createSignal(false);
  let disposed=false;
  onCleanup(()=>{disposed=true;});
  // One-second clock for the countdowns; it only matters while something waits.
  const [now,setNow]=createSignal(Date.now());
  const ticker=setInterval(()=>setNow(Date.now()),1000);
  onCleanup(()=>clearInterval(ticker));
  void loadCatalog('send-delay');
  // The row whose message is open in the reader, and the row whose send time is being edited.
  const [selectedId, setSelectedId] = createSignal<string | null>(null);
  const [editTimeId, setEditTimeId] = createSignal<string | null>(null);
  const [menuId, setMenuId] = createSignal<string | null>(null);
  const selected = () => app.outbox().find((s) => s.id === selectedId());
  const menuActions = (): ContextAction[] => {
    const sub = app.outbox().find((s) => s.id === menuId());
    if (sub === undefined) return [];
    const status = outboxStatus(sub, now());
    const rows: ContextAction[] = [{ id: 'open', label: t('outbox-open'), icon: 'mail', run: () => { setSelectedId(sub.id); } }];
    if (status.waiting) rows.push({ id: 'send-now', label: t(status.kind === 'held' ? 'mail-outbox-release' : 'mail-outbox-send-now'), icon: 'mail', run: () => app.sendOutboxNow(sub.id) });
    if (status.canEditTime) rows.push({ id: 'edit-time', label: t('outbox-edit-time'), icon: 'clock', run: () => { setEditTimeId(sub.id); } });
    if (status.waiting && status.kind !== 'held') {
      rows.push({ id: 'recall', label: t('outbox-recall-to-drafts'), icon: 'compose', run: () => app.cancelOutbox(sub.id) });
      rows.push({ id: 'edit-message', label: t('outbox-recall-edit'), icon: 'compose', run: () => app.recallOutbox(sub.id) });
    }
    if (status.canSendAgain) rows.push({ id: 'send-again', label: t('outbox-send-again'), icon: 'mail', run: async () => { await app.sendOutboxAgain(sub.id); } });
    if (status.waiting || status.kind === 'failed') rows.push({ id: 'delete', label: t('outbox-delete'), icon: 'delete', separatorBefore: true, run: async () => { await app.deleteOutbox(sub.id); } });
    return rows;
  };
  const rowMenu = createContextMenu({ label: () => t('mail-outbox-label'), scope: () => `outbox:${menuId()}`, actions: menuActions, onError: () => app.showToast('error', t('mail-context-failed')) });
  function refresh(owner:PreferenceAccount|null=preferenceAccount()):void {
    if(owner===null)return;
    setFailed(false);
    void app.refreshOutbox().catch(error=>{
      if(disposed||!preferenceAccountIsCurrent(owner)||error instanceof DOMException&&error.name==='AbortError')return;
      setFailed(true);
      app.showToast('error',t('mail-outbox-load-failed'));
    });
  }
  // Sign-in mounts the mailbox before activating its account authority. Wait
  // for that boundary, and reload whenever a retained tab changes mailbox.
  onMount(()=>onCleanup(observePreferenceAccount(owner=>{setFailed(false);if(owner)refresh(owner);})));

  return (
    <section class={`outbox ${css.root}`} data-mw-surface aria-label={t('mail-outbox-label')}>
      <header class={`outbox__header ${css.header}`}>
        <h2 class={css.title}>{t('mail-outbox-label')}</h2>
        <button type="button" class={`btn btn--ghost ${a11y.focusable}`} disabled={app.outboxLoading()} onClick={() => refresh()}>
          {t('mail-outbox-refresh')}
        </button>
      </header>
      <div class="outbox__split">
      <Show when={app.outbox().length > 0} fallback={<Show when={app.outboxLoading()} fallback={<p class={`outbox__empty ${css.empty}`}>{t(failed()?'mail-outbox-unavailable':'mail-outbox-empty')}</p>}><AsyncPending variant="list"/></Show>}>
        <ul class={`outbox__items ${css.items}`}>
          <For each={app.outbox()}>
            {(sub) => {
              const state = () => outboxStateOf(sub, now());
              const held = () => state() === 'held';
              const waiting = () => held() || state() === 'scheduled' || state() === 'holding';
              const message = () => app.outboxMessages()[sub.emailId];
              const recipients = () =>
                (message()?.to ?? []).map((a) => isolate(a.email)).join(', ');
              return (
                <li
                  class={`outbox__row ${css.row}`}
                  data-state={state()}
                  data-selected={selectedId() === sub.id ? 'true' : undefined}
                  tabIndex={0}
                  aria-current={selectedId() === sub.id ? 'true' : undefined}
                  onClick={(event) => { if (!(event.target as Element).closest('button')) setSelectedId(sub.id); }}
                  onKeyDown={(event) => {
                    if (event.target !== event.currentTarget) return;
                    if (event.key === 'Enter') setSelectedId(sub.id);
                    else { setMenuId(sub.id); rowMenu.keydown(event); }
                  }}
                  onContextMenu={(event) => { setMenuId(sub.id); rowMenu.open(event); }}
                >
                  <span class={`outbox__state outbox__state--${state()} ${css.state} ${css.stateOf[state()]}`}>
                    {t(STATE_LABEL[state()])}
                  </span>
                  <Show when={state() === 'scheduled'}>
                    <span class={`outbox__badge ${css.state}`} data-testid="outbox-scheduled-badge">{t('outbox-scheduled-badge')}</span>
                  </Show>
                  <span class={`outbox__when ${css.when}`}>{whenText(sub, now())}</span>
                  <Show when={message() !== undefined}>
                    <span class={`outbox__subject ${css.subject}`}>
                      {message()?.subject !== null && message()?.subject !== ''
                        ? isolate(message()?.subject ?? '')
                        : t('mail-outbox-no-subject')}
                    </span>
                    <Show when={recipients() !== ''}>
                      <span class={`outbox__to ${css.to}`}>{t('mail-outbox-to', { recipients: recipients() })}</span>
                    </Show>
                  </Show>
                  <Show when={held()}>
                    <p class={`outbox__note ${css.note}`}>{heldNote(sub.mailwomanOrigin)}</p>
                  </Show>
                  <Show when={!held() && sub.mailwomanOrigin}>
                    {(origin) => <p class={`outbox__note ${css.note}`}>{originText(origin())}</p>}
                  </Show>
                  <Show when={errorText(sub) !== ''}>
                    <p class={`outbox__error ${css.error}`} role="status">
                      {errorText(sub)}
                    </p>
                  </Show>
                  <Show when={state() === 'failed'}>
                    <span class={`outbox__actions ${css.actions}`}>
                      <button type="button" class={`btn btn--ghost ${a11y.focusable} ${css.action}`} data-action="send-again" onClick={() => void app.sendOutboxAgain(sub.id)}>
                        {t('outbox-send-again')}
                      </button>
                    </span>
                  </Show>
                  <Show when={waiting()}>
                    <span class={`outbox__actions ${css.actions}`}>
                      <button
                        type="button"
                        class={`btn btn--ghost ${a11y.focusable} ${css.action}`}
                        data-action="release"
                        onClick={() => void app.sendOutboxNow(sub.id)}
                      >
                        {t(held() ? 'mail-outbox-release' : 'mail-outbox-send-now')}
                      </button>
                      <Show when={state() === 'scheduled'}>
                        <button
                          type="button"
                          class={`btn btn--ghost ${a11y.focusable} ${css.action}`}
                          data-action="edit-time"
                          onClick={() => setEditTimeId(sub.id)}
                        >
                          {t('outbox-edit-time')}
                        </button>
                      </Show>
                      <Show when={!held()}>
                        <button
                          type="button"
                          class={`btn btn--ghost ${a11y.focusable} ${css.action}`}
                          data-action="recall-edit"
                          onClick={() => void app.recallOutbox(sub.id)}
                        >
                          {t('outbox-recall-edit')}
                        </button>
                      </Show>
                      <button
                        type="button"
                        class={`btn btn--ghost ${a11y.focusable} ${css.action}`}
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
      <Show when={selected()}>{(sub) => <aside class="outbox__reader"><OutboxDetail sub={sub()} now={now()} onClose={() => setSelectedId(null)} /></aside>}</Show>
      </div>
      {rowMenu.menu()}
      <Show when={editTimeId()}>
        {(id) => <OutboxEditTime current={app.outbox().find((s) => s.id === id())?.sendAt ?? null} save={(iso) => app.rescheduleOutbox(id(), iso)} onClose={() => setEditTimeId(null)} />}
      </Show>
    </section>
  );
}
