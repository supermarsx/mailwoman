// Dirty-state guard (t32-ext.md §K).
//
// A form calls `useDirtyGuard(() => edited())`. While any mounted form reports
// unsaved changes:
//   • the admin router asks before an in-app navigation (`createLeaveCheck`), and
//     the shell shows the "leave and discard?" dialog;
//   • closing or reloading the tab gets the browser's own `beforeunload` prompt.
// The registry is module-level so the router needs no reference to the forms.

import { onCleanup, type Accessor } from 'solid-js';
import type { LeaveCheck } from './router.ts';

const guards = new Set<Accessor<boolean>>();

/** True when any mounted form has unsaved changes. */
export function anyDirty(): boolean {
  for (const dirty of guards) if (dirty()) return true;
  return false;
}

function onBeforeUnload(e: BeforeUnloadEvent): void {
  if (!anyDirty()) return;
  e.preventDefault();
  // Older engines show the prompt only when returnValue is set.
  e.returnValue = '';
}

/** Register a form's dirty state for the lifetime of the calling component. */
export function useDirtyGuard(dirty: Accessor<boolean>): void {
  if (guards.size === 0) window.addEventListener('beforeunload', onBeforeUnload);
  guards.add(dirty);
  onCleanup(() => {
    guards.delete(dirty);
    if (guards.size === 0) window.removeEventListener('beforeunload', onBeforeUnload);
  });
}

/** A router leave check: allow at once when nothing is dirty, otherwise `ask`. */
export function createLeaveCheck(ask: () => Promise<boolean>): LeaveCheck {
  return async () => (anyDirty() ? ask() : true);
}
