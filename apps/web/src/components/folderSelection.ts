// The folders picked with the marquee in the sidebar (ids). Bulk folder
// actions in the folder menu read it; it is cleared by a click on empty space,
// by choosing a folder, and when a folder disappears.
import { createSignal } from 'solid-js';

const [selection, setSelection] = createSignal<ReadonlySet<string>>(new Set<string>(), { equals: (a, b) => a.size === b.size && [...a].every(id => b.has(id)) });

export const selectedFolderIds = selection;
export function setSelectedFolders(ids: ReadonlySet<string>): void { setSelection(new Set(ids)); }
export function clearSelectedFolders(): void { setSelection(new Set<string>()); }
/** Drop ids that are no longer listed (deleted, hidden or collapsed away). */
export function pruneSelectedFolders(valid: ReadonlySet<string>): void {
  const current = selection();
  if ([...current].some(id => !valid.has(id))) setSelection(new Set([...current].filter(id => valid.has(id))));
}
