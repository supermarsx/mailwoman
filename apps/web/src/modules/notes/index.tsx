// Notes module placeholder (plan §2.5, §3 e0 → filled by e6). e6 builds the
// notes list (pinned first, color chips, tag filter), the rich-text editor
// (reusing the sanitizer allowlist), tag/color/pin controls, search, and the
// `mailwoman:` cross-link picker over `state/slices/notes.ts` and `Note/*`.

import type { JSX } from 'solid-js';

export function NotesModule(): JSX.Element {
  return (
    <section aria-label="Notes" data-module="notes">
      <h1>Notes</h1>
      <p>The notes module mounts here (e6).</p>
    </section>
  );
}
