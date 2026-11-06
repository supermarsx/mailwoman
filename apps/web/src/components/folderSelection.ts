// The folders picked in the sidebar (ids): by the marquee, Ctrl/Cmd+click, Shift+click, Shift+Arrow or Ctrl+A.
// One selection and one anchor for all of them (see folderSelectionModel.ts). Bulk folder actions in the
// folder menu and the selection bar read it; it is cleared by Escape, a click on empty space, by choosing a
// folder with a plain click, and when a folder disappears.
import { createSignal } from 'solid-js';
import { EMPTY_SELECTION, applyClick, extendByKey, pruneSelection, selectAll, type ClickModifiers, type SelectionState } from './folderSelectionModel.ts';

const [state, setState] = createSignal<SelectionState>(EMPTY_SELECTION);
const [selection, setSelection] = createSignal<ReadonlySet<string>>(new Set<string>(), { equals: (a, b) => a.size === b.size && [...a].every(id => b.has(id)) });
function commit(next: SelectionState): void { setState(next); setSelection(new Set(next.selected)); }

export const selectedFolderIds = selection;
/** The marquee's result: replaces the picks, keeps the anchor. */
export function setSelectedFolders(ids: ReadonlySet<string>): void { commit({ ...state(), selected: new Set(ids), rangeAdded: [] }); }
export function clearSelectedFolders(): void { commit({ selected: new Set<string>(), anchor: null, rangeAdded: [] }); }
/** Drop ids that are no longer listed (deleted, hidden or collapsed away). */
export function pruneSelectedFolders(valid: ReadonlySet<string>): void { const next = pruneSelection(state(), valid); if (next !== state()) commit(next); }
/** A click with modifiers on a visible row. Returns whether the selection changed (a plain click only moves the anchor). */
export function clickFolder(visible: readonly string[], id: string, mod: ClickModifiers, selectable?: (id: string) => boolean): boolean {
  const before = selection();
  commit(applyClick(state(), visible, id, mod, selectable));
  return selection() !== before;
}
/** Shift+Arrow from `focus`: returns the id to focus next, or null at an end of the list. */
export function extendFolderSelection(visible: readonly string[], focus: string, dir: 1 | -1, selectable?: (id: string) => boolean): string | null {
  const result = extendByKey(state(), visible, focus, dir, selectable);
  if (!result) return null;
  commit(result.state);
  return result.focus;
}
export function selectAllFolders(visible: readonly string[], selectable?: (id: string) => boolean): void { commit(selectAll(visible, selectable)); }
export const folderSelectionAnchor = (): string | null => state().anchor;
