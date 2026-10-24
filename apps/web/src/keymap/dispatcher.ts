// The app's one keyboard-shortcut dispatcher (t28-e16).
//
// One `keydown` listener on the document resolves a key press to an ACTION ID
// through the active bindings, then runs the handler registered for that id.
// Surfaces do not listen for shortcut keys themselves: they register handlers
// in the one action table (`register('archive', …)`), and whichever surface is
// mounted answers. Before this file the presets in Settings were a preview
// table and no key did anything.
//
// What it leaves alone, so nothing element-local is replaced:
//   - a key an element already handled (`defaultPrevented`): the message list's
//     arrow keys, a menu's Escape, a dialog's focus trap;
//   - typing: inputs, textareas, selects and contenteditable regions (the
//     composer), and IME composition;
//   - anything inside an open modal dialog;
//   - a press with no binding, or whose handler declines (returns `false`).
//
// Extension points for the shortcut registry that builds on this (t32-e56:
// sequences, presets, conflict detection, admin locks): `bindings` is a
// function the dispatcher calls on every press, so a richer resolver replaces
// it without touching surfaces; the action table is keyed by action id, so new
// actions are new `register` calls; `dispatch` is exposed for tests and for
// replaying a press.

import { chordId, chordOf, parseChord } from './chords.ts';
import { loadPrefs, PRESET_BINDINGS, type KeyboardPreset } from '../screens/Settings/prefs.ts';

/** A canonical action name, as `PRESET_BINDINGS` uses them (`archive`, `next`, …). */
export type ActionId = string;

/** Runs an action. Return `false` to decline (the key then does what it would
 *  have done without the dispatcher); anything else means handled. */
export type ActionHandler = (e: KeyboardEvent) => boolean | void;

/** Bindings as the dispatcher reads them: chord id → action id. */
export type BindingMap = ReadonlyMap<string, ActionId>;

export interface Keymap {
  /** Add a handler for `action`. The newest registration for an id answers;
   *  removing it lets the previous one answer again. Returns the remover. */
  register(action: ActionId, handler: ActionHandler): () => void;
  /** Resolve and run one key press. True when an action handled it. */
  dispatch(e: KeyboardEvent): boolean;
  /** Detach the document listener and forget every handler. */
  dispose(): void;
}

/** The chord → action map for a preset. A binding the parser cannot read is
 *  skipped rather than bound to something else. */
export function bindingsFor(preset: KeyboardPreset): BindingMap {
  const out = new Map<string, ActionId>();
  for (const { action, keys } of PRESET_BINDINGS[preset] ?? []) {
    const chord = parseChord(keys);
    if (chord !== null && !out.has(chordId(chord))) out.set(chordId(chord), action);
  }
  return out;
}

/** The bindings of the preset chosen in Settings, read at each press so a change
 *  there applies to the next key without a reload. */
export function activeBindings(): BindingMap {
  return bindingsFor(loadPrefs().keyboardPreset);
}

/** Where typing happens: keys there are text, not shortcuts. */
function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  if (target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement) return true;
  if (target instanceof HTMLInputElement) {
    // Buttons and checkboxes take no text; every other input type does.
    return !['button', 'checkbox', 'radio', 'submit', 'reset', 'image', 'range', 'color', 'file'].includes(
      target.type,
    );
  }
  return target.closest('[contenteditable=""], [contenteditable="true"]') !== null;
}

function insideModal(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest('[aria-modal="true"], dialog[open]') !== null;
}

export interface KeymapOptions {
  /** Where the listener goes. Defaults to `document`. */
  target?: Pick<Document, 'addEventListener' | 'removeEventListener'>;
  /** The bindings in force. Defaults to {@link activeBindings}. */
  bindings?: () => BindingMap;
}

/** Build a dispatcher. The app uses the one from {@link getKeymap}; tests build their own. */
export function createKeymap(opts: KeymapOptions = {}): Keymap {
  const bindings = opts.bindings ?? activeBindings;
  const handlers = new Map<ActionId, ActionHandler[]>();

  function dispatch(e: KeyboardEvent): boolean {
    if (e.defaultPrevented || e.isComposing) return false;
    if (isTypingTarget(e.target) || insideModal(e.target)) return false;
    const action = bindings().get(chordId(chordOf(e)));
    if (action === undefined) return false;
    const stack = handlers.get(action);
    const handler = stack?.[stack.length - 1];
    if (handler === undefined) return false;
    if (handler(e) === false) return false;
    e.preventDefault();
    return true;
  }

  const listener = (e: Event): void => {
    if (e instanceof KeyboardEvent) dispatch(e);
  };
  const target = opts.target ?? (typeof document !== 'undefined' ? document : undefined);
  target?.addEventListener('keydown', listener);

  return {
    register(action, handler) {
      const stack = handlers.get(action) ?? [];
      stack.push(handler);
      handlers.set(action, stack);
      return () => {
        const current = handlers.get(action);
        if (current === undefined) return;
        const at = current.lastIndexOf(handler);
        if (at >= 0) current.splice(at, 1);
        if (current.length === 0) handlers.delete(action);
      };
    },
    dispatch,
    dispose() {
      target?.removeEventListener('keydown', listener);
      handlers.clear();
    },
  };
}

let shared: Keymap | undefined;

/** The app's dispatcher, created on first use. Every surface registers here. */
export function getKeymap(): Keymap {
  shared ??= createKeymap();
  return shared;
}
