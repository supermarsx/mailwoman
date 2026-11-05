// A recalled message becomes a local draft; the mailbox shell opens it in a
// composer. Module state so the send path (toast Undo) and the Outbox can hand
// one over without a component in scope.

import { createSignal } from 'solid-js';

const [recalled, setRecalled] = createSignal<string | null>(null);

/** The id of a draft the shell should open in the composer, if any. */
export const recalledDraftId = recalled;

/** Ask the shell to open this draft in a composer. */
export function offerRecalledDraft(id: string): void {
  setRecalled(id);
}

/** Mark the offer handled. */
export function takeRecalledDraft(): string | null {
  const id = recalled();
  setRecalled(null);
  return id;
}
