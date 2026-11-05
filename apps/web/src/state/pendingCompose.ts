// A composer another module asks for (Contacts: "Send contact by email"). The
// mailbox shell owns the composer panes, so the request waits here, as module
// state, until the shell takes it. Same hand-over shape as `recalledDraft.ts`.

import { createSignal } from 'solid-js';
import type { ComposeInitial } from '../components/compose/reply.ts';

const [pending, setPending] = createSignal<ComposeInitial | null>(null);

/** The draft the shell should open in a new composer, if any. */
export const pendingCompose = pending;

/** Ask the shell to open this draft in a composer. */
export function requestCompose(draft: ComposeInitial): void {
  setPending(draft);
}

/** Mark the request handled and return it. */
export function takePendingCompose(): ComposeInitial | null {
  const draft = pending();
  setPending(null);
  return draft;
}
