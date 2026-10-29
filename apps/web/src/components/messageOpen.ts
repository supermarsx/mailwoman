// Where a message opened from the Mail list goes (t74).
//
// With the reading pane on (Right or Bottom) a single click, a tap or Enter
// shows the message in the reading pane beside the list: one reader, reused for
// the next message, not a tab. A double-click or Shift+Enter (the rebindable
// `openInTab` shortcut) opens the message in its own workspace tab. With the
// reading pane Off there is nothing beside the list, so every open is a tab;
// the same holds at phone width, where an open message always covers the list.

import type { ReadingPane } from './readingPane.ts';

/** How the user asked: a plain activation, or explicitly for a tab. */
export type OpenGesture = 'activate' | 'tab';

/** Where the message is shown. */
export type OpenTarget = 'pane' | 'tab';

export function messageOpenTarget(gesture: OpenGesture, layout: { readingPane: ReadingPane; narrow: boolean }): OpenTarget {
  if (gesture === 'tab') return 'tab';
  return !layout.narrow && layout.readingPane !== 'off' ? 'pane' : 'tab';
}

/** What a click on a message row does. */
export type RowClickAction = 'range' | 'toggle' | 'activate' | 'ignore';

/**
 * Shift+click selects a range and Ctrl/Cmd+click (or any click in selection
 * mode) toggles the row (t58); neither opens anything. The second click of a
 * double-click is ignored: the first already opened the message and the
 * `dblclick` that follows decides whether it also gets a tab. A keyboard Enter
 * reports `detail` 0 and always activates.
 */
export function classifyRowClick(
  e: { shiftKey: boolean; ctrlKey: boolean; metaKey: boolean; detail: number },
  state: { modifierSelection: boolean; selecting: boolean },
): RowClickAction {
  if (e.shiftKey && state.modifierSelection) return 'range';
  if (((e.ctrlKey || e.metaKey) && state.modifierSelection) || state.selecting) return 'toggle';
  return e.detail >= 2 ? 'ignore' : 'activate';
}

/** Whether a double-click on a row asks for a tab (a modified or selection-mode one does not). */
export function dblClickOpensTab(
  e: { shiftKey: boolean; ctrlKey: boolean; metaKey: boolean; altKey: boolean },
  state: { selecting: boolean },
): boolean {
  return !state.selecting && !e.shiftKey && !e.ctrlKey && !e.metaKey && !e.altKey;
}
