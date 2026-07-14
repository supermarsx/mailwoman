// V7 GAL search (SPEC §13, plan §3 e7). SCAFFOLD stub (e0): inert placeholder.
// e7 renders GAL search in recipient fields + expand-group-before-send; e14 wires
// it to /api/directory/*.

import type { JSX } from 'solid-js';

export interface DirectorySearchProps {
  /** The recipient-field query (bound by e7). */
  query?: string;
}

export function DirectorySearch(_props: DirectorySearchProps): JSX.Element {
  return <div data-module="directory">Directory/GAL search not yet implemented (t7 e7).</div>;
}
