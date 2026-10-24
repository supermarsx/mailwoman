// Keyboard shortcuts: one dispatcher, one action table keyed by action id.
// See dispatcher.ts for what it handles and what it leaves to elements.

import { onCleanup } from 'solid-js';
import { getKeymap, type ActionHandler, type ActionId } from './dispatcher.ts';

export { bindingsFor, activeBindings, createKeymap, getKeymap } from './dispatcher.ts';
export type { ActionHandler, ActionId, BindingMap, Keymap, KeymapOptions } from './dispatcher.ts';
export { chordId, chordOf, parseChord, type Chord } from './chords.ts';

/** Register `handlers` with the app's dispatcher for the life of the calling
 *  component (call it during component setup). */
export function useActions(handlers: Partial<Record<ActionId, ActionHandler>>): void {
  const keymap = getKeymap();
  const removers = Object.entries(handlers)
    .filter((entry): entry is [ActionId, ActionHandler] => entry[1] !== undefined)
    .map(([action, handler]) => keymap.register(action, handler));
  onCleanup(() => {
    for (const remove of removers) remove();
  });
}
