// Key chords: the binding syntax of `PRESET_BINDINGS` (screens/Settings/prefs.ts)
// parsed into something a `KeyboardEvent` can be compared with.
//
// A binding is one key with optional modifiers: `e`, `/`, `Backspace`, `↓`,
// `Ctrl+N`. `Ctrl` is the platform modifier: Ctrl, or Cmd on a Mac, so one
// preset works on both. Letters compare case-insensitively; a binding that wants
// Shift says `Shift+`.

/** A parsed binding, or a pressed key, in one comparable shape. */
export interface Chord {
  /** `KeyboardEvent.key`, lower-cased when it is a single character. */
  key: string;
  /** Ctrl or Cmd. */
  mod: boolean;
  alt: boolean;
  shift: boolean;
}

/** Display names in the preset table for keys whose `KeyboardEvent.key` differs. */
const KEY_ALIASES: Record<string, string> = {
  '↓': 'ArrowDown',
  '↑': 'ArrowUp',
  '←': 'ArrowLeft',
  '→': 'ArrowRight',
  Esc: 'Escape',
  Space: ' ',
  Del: 'Delete',
};

function normalKey(key: string): string {
  const aliased = KEY_ALIASES[key] ?? key;
  return aliased.length === 1 ? aliased.toLowerCase() : aliased;
}

/** Parse a binding such as `Ctrl+N`. `null` for an empty or unreadable one. */
export function parseChord(spec: string): Chord | null {
  const trimmed = spec.trim();
  if (trimmed === '') return null;
  // `+` alone, or a trailing `+` (as in `Ctrl++`), is the plus key itself.
  const parts = trimmed === '+' ? ['+'] : trimmed.split(/\+(?!$)/);
  const keyPart = parts.pop();
  if (keyPart === undefined || keyPart === '') return null;
  const chord: Chord = { key: normalKey(keyPart), mod: false, alt: false, shift: false };
  for (const raw of parts) {
    const m = raw.trim().toLowerCase();
    if (m === 'ctrl' || m === 'cmd' || m === 'meta' || m === 'mod') chord.mod = true;
    else if (m === 'alt' || m === 'option') chord.alt = true;
    else if (m === 'shift') chord.shift = true;
    else return null;
  }
  return chord;
}

/** The chord a key press makes. Shift counts for letters and named keys, and
 *  not for other printed characters: `?` is Shift+/ on one layout and not on
 *  another, so the character alone decides. */
export function chordOf(e: KeyboardEvent): Chord {
  const caseless = e.key.length === 1 && e.key.toLowerCase() === e.key.toUpperCase();
  return {
    key: normalKey(e.key),
    mod: e.ctrlKey || e.metaKey,
    alt: e.altKey,
    shift: caseless ? false : e.shiftKey,
  };
}

/** A stable string for a chord, for use as a map key. */
export function chordId(c: Chord): string {
  return `${c.mod ? 'mod+' : ''}${c.alt ? 'alt+' : ''}${c.shift ? 'shift+' : ''}${c.key}`;
}
