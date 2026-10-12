// The location choice of a holiday-pack import. A `.hol` file with more than
// one `[Location]` section is not imported whole: this modal lists the
// locations, none ticked, and hands back the ones the user ticks. Cancelling
// imports nothing.
//
// Modal a11y mirrors ShareDialog: Tab is trapped within the dialog, Escape
// closes, focus lands on the first control on open and is restored to the
// invoker on close.

import { For, Show, createMemo, createSignal, onCleanup, onMount, type JSX } from 'solid-js';
import { t } from '../../i18n';
import type { HolLocation } from './hol-locations.ts';
import * as css from './calendar.css.ts';
import * as own from './HolLocationPicker.css.ts';

const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export interface HolLocationPickerProps {
  /** The pack's locations, in file order. */
  locations: readonly HolLocation[];
  /** Called with the ticked locations, in file order; never with none. */
  onImport: (chosen: HolLocation[]) => void;
  onClose: () => void;
}

export function HolLocationPicker(props: HolLocationPickerProps): JSX.Element {
  const [filter, setFilter] = createSignal('');
  // Ticked locations by their position in the file: two sections may share a name.
  const [picked, setPicked] = createSignal<ReadonlySet<number>>(new Set());

  const rows = createMemo(() => props.locations.map((location, index) => ({ location, index })));
  const shown = createMemo(() => {
    const needle = filter().trim().toLocaleLowerCase();
    return needle === '' ? rows() : rows().filter((r) => r.location.name.toLocaleLowerCase().includes(needle));
  });

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

  function toggle(index: number, on: boolean): void {
    const next = new Set(picked());
    if (on) next.add(index);
    else next.delete(index);
    setPicked(next);
  }

  function confirm(): void {
    const chosen = rows()
      .filter((r) => picked().has(r.index))
      .map((r) => r.location);
    if (chosen.length > 0) props.onImport(chosen);
  }

  return (
    <div class={css.dialogBackdrop} onClick={(e) => e.target === e.currentTarget && props.onClose()}>
      <div
        ref={dialogRef}
        class={css.dialog}
        role="dialog"
        tabindex={-1}
        aria-label={t('calendar-import-locations-title')}
        aria-modal="true"
        onKeyDown={onDialogKeyDown}
      >
        <h2 class={own.heading}>{t('calendar-import-locations-title')}</h2>
        <p class={css.dimText}>{t('calendar-import-locations-intro', { count: props.locations.length })}</p>

        <input
          class={css.input}
          type="search"
          value={filter()}
          onInput={(e) => setFilter(e.currentTarget.value)}
          placeholder={t('calendar-import-locations-filter')}
          aria-label={t('calendar-import-locations-filter')}
        />

        <ul class={own.list} aria-label={t('calendar-import-locations-list')}>
          <For each={shown()}>
            {(row) => (
              <li>
                <label class={own.item}>
                  <input
                    type="checkbox"
                    checked={picked().has(row.index)}
                    onChange={(e) => toggle(row.index, e.currentTarget.checked)}
                  />
                  <bdi>{row.location.name}</bdi>
                </label>
              </li>
            )}
          </For>
        </ul>
        <Show when={shown().length === 0}>
          <p class={css.dimText}>{t('calendar-import-locations-none')}</p>
        </Show>
        <p class={css.dimText} role="status">{t('calendar-import-locations-picked', { count: picked().size })}</p>

        <div class={css.dialogActions}>
          <button type="button" class={css.button} onClick={() => props.onClose()}>{t('common-cancel')}</button>
          <button type="button" class={css.primaryButton} disabled={picked().size === 0} onClick={confirm}>
            {t('calendar-import-locations-import')}
          </button>
        </div>
      </div>
    </div>
  );
}
