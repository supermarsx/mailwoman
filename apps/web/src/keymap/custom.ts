import { chordId, parseChord } from './chords.ts';
import type { SettingsMap } from '../api/settings.ts';

export const SHORTCUT_ACTIONS = ['compose', 'archive', 'reply', 'replyAll', 'forward', 'next', 'previous', 'search', 'refresh', 'settings', 'trash', 'markRead', 'markUnread', 'spam', 'zoomIn', 'zoomOut', 'zoomReset', 'renameFolder'] as const;
export type ShortcutAction = typeof SHORTCUT_ACTIONS[number];

export function validBinding(value: string): boolean {
  if (value === 'inherit' || value === 'disabled') return true;
  if (value.length > 64) return false;
  const chord = parseChord(value);
  return chord !== null && (([...chord.key].length === 1 && chord.key.codePointAt(0)! >= 32 && chord.key.codePointAt(0) !== 127) || /^(Arrow(Up|Down|Left|Right)|Home|End|PageUp|PageDown|Delete|Backspace|Insert|Escape|Tab|Enter|F([1-9]|1[0-2]))$/.test(chord.key));
}

/** Conflicting bindings are inactive, rather than silently choosing an action. */
export function resolveCustomBindings(base: ReadonlyMap<string, string>, values: SettingsMap): { bindings: Map<string, string>; conflicts: string[] } {
  if (values['shortcuts.enabled']?.value === false) return { bindings: new Map(), conflicts: [] };
  const candidates = new Map<string, string[]>();
  const explicit = new Map<string, string>();
  for (const action of SHORTCUT_ACTIONS) {
    const value = values[`shortcuts.${action}`]?.value;
    if (typeof value === 'string' && value !== 'inherit') explicit.set(action, value);
  }
  function add(chord: string, action: string): void {
    candidates.set(chord, [...(candidates.get(chord) ?? []), action]);
  }
  for (const [chord, action] of base) if (!explicit.has(action)) add(chord, action);
  for (const [action, value] of explicit) {
    if (value === 'disabled' || !validBinding(value)) continue;
    const chord = parseChord(value);
    if (chord) add(chordId(chord), action);
  }
  const bindings = new Map<string, string>();
  const conflicts: string[] = [];
  for (const [chord, actions] of candidates) {
    if (actions.length === 1) bindings.set(chord, actions[0]!);
    else conflicts.push(...actions);
  }
  return { bindings, conflicts: [...new Set(conflicts)] };
}
