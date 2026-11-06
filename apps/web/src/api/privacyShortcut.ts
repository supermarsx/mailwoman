// A request type chosen from the Your data dashboard ("Download my data",
// "Delete my account"); the data requests form below reads it and pre-selects.
import { createSignal } from 'solid-js';

export type ShortcutKind = 'access' | 'portability' | 'erasure' | 'restriction';

const [kind, setKind] = createSignal<ShortcutKind | null>(null);

export const requestedKind = kind;

/** Pre-select a request type and bring the form into view. */
export function chooseRequest(next: ShortcutKind): void {
  setKind(next);
  const target = document.getElementById('your-data-requests');
  target?.scrollIntoView?.({ block: 'start' });
  (document.getElementById('dr-kind') as HTMLElement | null)?.focus?.();
}
