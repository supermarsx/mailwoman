// Calendar sharing dialog (P1). A focus-trapped modal that lists a calendar's
// existing `shareWith` grants (principal + read / read-write access) and lets the
// owner add a grant, change its access, or remove it. Every mutation goes through
// the controller's `shareCalendar` / `unshareCalendar` (a `Calendar/set` update of
// the `shareWith` ACL — the Mailwoman-native sharing surface e11 backs).
//
// Modal a11y mirrors EventEditor: Tab is trapped within the dialog, Escape closes,
// focus lands on the first control on open and is restored to the invoker on close.

import { For, Show, createSignal, onCleanup, onMount, type JSX } from 'solid-js';
import { t, isolate } from '../../i18n';
import type { Calendar, CalendarShare } from '../../api/pim-types.ts';
import type { CalendarController } from './controller.ts';
import * as css from './calendar.css.ts';

const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export interface ShareDialogProps {
  controller: CalendarController;
  /** The calendar whose sharing is being edited. */
  calendar: Calendar;
  onClose: () => void;
}

export function ShareDialog(props: ShareDialogProps): JSX.Element {
  const [newPrincipal, setNewPrincipal] = createSignal('');
  const [newAccess, setNewAccess] = createSignal<CalendarShare['access']>('read');
  const [busy, setBusy] = createSignal(false);

  // The live grant list comes from the controller so it re-renders after a
  // mutation reload without the dialog holding its own stale copy.
  const grants = (): CalendarShare[] =>
    props.controller.calendars().find((c) => c.id === props.calendar.id)?.shareWith ?? props.calendar.shareWith;

  let dialogRef!: HTMLDivElement;
  let restoreEl: HTMLElement | null = null;
  onMount(() => {
    restoreEl = (document.activeElement as HTMLElement | null) ?? null;
    const first = dialogRef.querySelector<HTMLElement>(FOCUSABLE);
    (first ?? dialogRef).focus();
  });
  onCleanup(() => restoreEl?.focus?.());

  function onDialogKeyDown(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
      e.preventDefault();
      props.onClose();
      return;
    }
    if (e.key !== 'Tab') return;
    const nodes = Array.from(dialogRef.querySelectorAll<HTMLElement>(FOCUSABLE));
    if (nodes.length === 0) return;
    const first = nodes[0]!;
    const last = nodes[nodes.length - 1]!;
    const activeEl = document.activeElement;
    if (e.shiftKey && activeEl === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && activeEl === last) {
      e.preventDefault();
      first.focus();
    }
  }

  async function addGrant(): Promise<void> {
    const principal = newPrincipal().trim();
    if (principal === '' || busy()) return;
    setBusy(true);
    try {
      await props.controller.shareCalendar(props.calendar.id, principal, newAccess());
      setNewPrincipal('');
    } finally {
      setBusy(false);
    }
  }

  async function changeAccess(principal: string, access: CalendarShare['access']): Promise<void> {
    setBusy(true);
    try {
      await props.controller.shareCalendar(props.calendar.id, principal, access);
    } finally {
      setBusy(false);
    }
  }

  async function removeGrant(principal: string): Promise<void> {
    setBusy(true);
    try {
      await props.controller.unshareCalendar(props.calendar.id, principal);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div class={css.dialogBackdrop} onClick={(e) => e.target === e.currentTarget && props.onClose()}>
      <div
        ref={dialogRef}
        class={css.dialog}
        role="dialog"
        tabindex={-1}
        aria-label={t('calendar-share-title', { name: isolate(props.calendar.name) })}
        aria-modal="true"
        onKeyDown={onDialogKeyDown}
      >
        <h2 style={{ margin: 0, 'font-size': '1.1rem' }}>
          {t('calendar-share-title', { name: isolate(props.calendar.name) })}
        </h2>
        <p class={css.dimText}>{t('calendar-share-intro')}</p>

        <div class={css.field}>
          <label class={css.label}>{t('calendar-share-people')}</label>
          <Show
            when={grants().length > 0}
            fallback={<p class={css.dimText}>{t('calendar-share-empty')}</p>}
          >
            <ul class={css.attendeeList}>
              <For each={grants()}>
                {(g) => (
                  <li class={css.attendeeRow}>
                    <span class={css.attendeeEmail}><bdi>{g.principal}</bdi></span>
                    <select
                      class={css.input}
                      value={g.access}
                      disabled={busy()}
                      onChange={(e) => void changeAccess(g.principal, e.currentTarget.value as CalendarShare['access'])}
                      aria-label={t('calendar-share-access-for', { principal: isolate(g.principal) })}
                    >
                      <option value="read">{t('calendar-share-read')}</option>
                      <option value="readWrite">{t('calendar-share-readwrite')}</option>
                    </select>
                    <button
                      type="button"
                      class={css.button}
                      disabled={busy()}
                      aria-label={t('calendar-share-remove', { principal: isolate(g.principal) })}
                      onClick={() => void removeGrant(g.principal)}
                    >
                      ×
                    </button>
                  </li>
                )}
              </For>
            </ul>
          </Show>
        </div>

        <div class={css.field}>
          <label class={css.label} for="cal-share-principal">{t('calendar-share-add')}</label>
          <div class={css.row}>
            <input
              id="cal-share-principal"
              class={css.input}
              placeholder="name@example.com"
              value={newPrincipal()}
              onInput={(e) => setNewPrincipal(e.currentTarget.value)}
              onKeyDown={(e) => e.key === 'Enter' && (e.preventDefault(), void addGrant())}
            />
            <select
              class={css.input}
              value={newAccess()}
              onChange={(e) => setNewAccess(e.currentTarget.value as CalendarShare['access'])}
              aria-label={t('calendar-share-new-access')}
            >
              <option value="read">{t('calendar-share-read')}</option>
              <option value="readWrite">{t('calendar-share-readwrite')}</option>
            </select>
            <button type="button" class={css.primaryButton} disabled={busy()} onClick={() => void addGrant()}>
              {t('common-add')}
            </button>
          </div>
        </div>

        <div class={css.dialogActions}>
          <span class={css.spacer} />
          <button type="button" class={css.button} onClick={props.onClose}>{t('common-close')}</button>
        </div>
      </div>
    </div>
  );
}
