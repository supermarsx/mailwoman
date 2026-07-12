// Notes store slice (plan §2.5, §3 e0 → filled by e6). Frozen seam composed into
// `AppState`; e6 fills the signals + actions over the `Note/*` surface (mock
// until e10). Bodies are opaque to the client (sealed server-side).

import { createSignal, type Accessor } from 'solid-js';
import type { Note } from '../../api/pim-types.ts';
import type { SliceContext } from './context.ts';

export interface NotesSlice {
  notes: Accessor<Note[]>;
  /** Load the account's notes (pinned first) (e6 fills). */
  loadNotes(): Promise<void>;
}

export function createNotesSlice(_ctx: SliceContext): NotesSlice {
  const [notes] = createSignal<Note[]>([]);

  return {
    notes,
    loadNotes: () => Promise.resolve(),
  };
}
