// Client-local Settings preferences (t16 e15 — W14 keyboard presets, W16 offline
// eviction policy, W20 interface direction preview).
//
// These are per-device UI preferences, not account state: they persist to
// localStorage under a namespaced key and are read by the shell (keymap dispatch),
// the OPFS offline cache (eviction), and the Settings panel (direction preview).
// Everything is guarded for the no-`localStorage` case (SSR) and never throws.

const STORAGE_KEY = 'mw.settings.prefs.v1';

/** Keyboard-shortcut preset (W14). The shell keymap (`src/keymap/`) reads the
 *  active preset on every key press. `off` binds no key at all, which is the way
 *  to turn single-character shortcuts off (WCAG 2.1.4). */
export type KeyboardPreset = 'default' | 'gmail' | 'outlook' | 'vim' | 'off';

/** When an opened message is marked read (t28-e16). `on-open` marks it after
 *  `markReadDelayMs` of it staying open; `never` leaves that to the user. */
export type MarkReadMode = 'on-open' | 'never';

/** Offline cache eviction strategy (W16). */
export type EvictionStrategy = 'lru' | 'oldest' | 'manual';

/** Interface writing direction (W20). "auto" follows the negotiated locale. */
export type DirectionPref = 'auto' | 'ltr' | 'rtl';

export interface SettingsPrefs {
  keyboardPreset: KeyboardPreset;
  /** Offline cache budget in megabytes (0 = do not cache). */
  offlineBudgetMb: number;
  /** How long to retain cached items, in days (0 = no age limit). */
  offlineRetentionDays: number;
  eviction: EvictionStrategy;
  direction: DirectionPref;
  markRead: MarkReadMode;
  /** How long a message must stay open before it is marked read, in ms (0 = at once). */
  markReadDelayMs: number;
}

export const DEFAULT_PREFS: SettingsPrefs = {
  keyboardPreset: 'default',
  offlineBudgetMb: 250,
  offlineRetentionDays: 30,
  eviction: 'lru',
  direction: 'auto',
  markRead: 'on-open',
  markReadDelayMs: 1000,
};

/** Upper bound on the mark-read delay: past a minute it is not a dwell any more. */
export const MAX_MARK_READ_DELAY_MS = 60_000;

const KEYBOARD_PRESETS: readonly KeyboardPreset[] = ['default', 'gmail', 'outlook', 'vim', 'off'];
const MARK_READ_MODES: readonly MarkReadMode[] = ['on-open', 'never'];
const EVICTION_STRATEGIES: readonly EvictionStrategy[] = ['lru', 'oldest', 'manual'];
const DIRECTIONS: readonly DirectionPref[] = ['auto', 'ltr', 'rtl'];

function store(): Storage | null {
  try {
    return typeof localStorage !== 'undefined' ? localStorage : null;
  } catch {
    return null; // access can throw in a sandboxed/blocked context.
  }
}

function clampNumber(value: unknown, fallback: number): number {
  return typeof value === 'number' && Number.isFinite(value) && value >= 0 ? Math.floor(value) : fallback;
}

function oneOf<T extends string>(value: unknown, allowed: readonly T[], fallback: T): T {
  return typeof value === 'string' && (allowed as readonly string[]).includes(value) ? (value as T) : fallback;
}

/** Read the persisted prefs, falling back per-field to defaults on any junk. */
export function loadPrefs(): SettingsPrefs {
  const s = store();
  if (s === null) return { ...DEFAULT_PREFS };
  const raw = s.getItem(STORAGE_KEY);
  if (raw === null) return { ...DEFAULT_PREFS };
  try {
    const parsed = JSON.parse(raw) as Partial<SettingsPrefs>;
    return {
      keyboardPreset: oneOf(parsed.keyboardPreset, KEYBOARD_PRESETS, DEFAULT_PREFS.keyboardPreset),
      offlineBudgetMb: clampNumber(parsed.offlineBudgetMb, DEFAULT_PREFS.offlineBudgetMb),
      offlineRetentionDays: clampNumber(parsed.offlineRetentionDays, DEFAULT_PREFS.offlineRetentionDays),
      eviction: oneOf(parsed.eviction, EVICTION_STRATEGIES, DEFAULT_PREFS.eviction),
      direction: oneOf(parsed.direction, DIRECTIONS, DEFAULT_PREFS.direction),
      markRead: oneOf(parsed.markRead, MARK_READ_MODES, DEFAULT_PREFS.markRead),
      markReadDelayMs: Math.min(
        clampNumber(parsed.markReadDelayMs, DEFAULT_PREFS.markReadDelayMs),
        MAX_MARK_READ_DELAY_MS,
      ),
    };
  } catch {
    return { ...DEFAULT_PREFS };
  }
}

/** Persist prefs (best-effort; a blocked/full store is swallowed, never throws). */
export function savePrefs(prefs: SettingsPrefs): void {
  const s = store();
  if (s === null) return;
  try {
    s.setItem(STORAGE_KEY, JSON.stringify(prefs));
  } catch {
    /* quota / blocked — a device pref failing to persist is non-fatal. */
  }
}

/**
 * The keybindings each preset maps a canonical action to. The one key dispatcher
 * (`src/keymap/dispatcher.ts`) resolves key presses through this table, and the
 * Settings preview lists it, so what the preview shows is what the keys do.
 * Key syntax: a single key (`e`, `/`, `Backspace`), `↑`/`↓` for the arrow keys,
 * and `Ctrl+` for the platform modifier (Ctrl, or Cmd on a Mac).
 */
export const PRESET_BINDINGS: Record<KeyboardPreset, ReadonlyArray<{ action: string; keys: string }>> = {
  default: [
    { action: 'compose', keys: 'c' },
    { action: 'archive', keys: 'e' },
    { action: 'reply', keys: 'r' },
    { action: 'next', keys: 'j' },
    { action: 'previous', keys: 'k' },
    { action: 'search', keys: '/' },
  ],
  gmail: [
    { action: 'compose', keys: 'c' },
    { action: 'archive', keys: 'e' },
    { action: 'reply', keys: 'r' },
    { action: 'next', keys: 'j' },
    { action: 'previous', keys: 'k' },
    { action: 'search', keys: '/' },
  ],
  outlook: [
    { action: 'compose', keys: 'Ctrl+N' },
    { action: 'archive', keys: 'Backspace' },
    { action: 'reply', keys: 'Ctrl+R' },
    { action: 'next', keys: '↓' },
    { action: 'previous', keys: '↑' },
    { action: 'search', keys: 'Ctrl+E' },
  ],
  vim: [
    { action: 'compose', keys: 'o' },
    { action: 'archive', keys: 'x' },
    { action: 'reply', keys: 'r' },
    { action: 'next', keys: 'j' },
    { action: 'previous', keys: 'k' },
    { action: 'search', keys: '/' },
  ],
  off: [],
};
