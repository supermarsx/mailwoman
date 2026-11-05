// The app-level action history behind the Undo and Redo buttons.
//
// Every reversible mail action (archive, delete, move, spam, mark read, flag,
// label, pin, snooze, follow-up, whole-folder jobs) calls `recordUndoable` once
// per user gesture, with a label and the closures that reverse and repeat it.
// The 10-second toast Undo stays; it now runs through this history, so both
// agree on what has been undone.
//
// Per session and in memory only: nothing is persisted, and sign-out or an
// account switch clears it (`clearUndoHistory`). The stack holds at most
// `undoCapacity()` entries (the user's `ui.undoHistorySize` clamped by the
// administrator's `ui.undoHistoryMax`; 0 turns the history off). Recording a
// new action empties the redo stack. An action that can no longer be reversed
// (the message was moved or deleted elsewhere) fails with a toast and is
// dropped. The editor's own text history (compose/ComposerHistory) is separate.

import {createSignal} from 'solid-js';
import {settings} from '../api/settings.ts';
import {toast} from '../components/toast/toast.ts';
import {loadCatalog,t} from '../i18n/index.ts';

export const UNDO_HISTORY_SIZE_KEY = 'ui.undoHistorySize';
export const UNDO_HISTORY_CAP_KEY = 'ui.undoHistoryMax';
export const UNDO_HISTORY_LIMIT = 50;
export const UNDO_HISTORY_DEFAULT = 50;
/** The sizes the Settings selector offers (0 = off), before the administrator's cap. */
export const UNDO_HISTORY_CHOICES = [0, 10, 25, 50] as const;

/** Thrown by an undo/redo closure when its target changed or vanished; `reason` is shown to the user. */
export class UndoUnavailable extends Error {
  constructor(readonly reason?: string) {
    super(reason ?? 'unavailable');
  }
}

export interface UndoableAction {
  /** What the user did, as a short sentence ("Archived 3 messages"). */
  label: string;
  accountId?: string | null;
  /** Reverse the action. Throw `UndoUnavailable` when the target changed. */
  undo: () => Promise<void>;
  /** Repeat the action after an undo. */
  redo: () => Promise<void>;
  /** Called after this entry was undone or redone from the history (the toast follows it). */
  onSettled?: () => void;
}

export interface UndoEntry {
  readonly id: number;
  readonly label: string;
  readonly at: number;
  readonly accountId: string | null;
  readonly action: UndoableAction;
}

function intSetting(key: string, fallback: number): number {
  const value = settings()[key]?.value;
  return typeof value === 'number' && Number.isInteger(value) ? Math.min(UNDO_HISTORY_LIMIT, Math.max(0, value)) : fallback;
}

/** Reactive: the administrator's ceiling (default 50). */
export function undoHistoryCap(): number {
  return intSetting(UNDO_HISTORY_CAP_KEY, UNDO_HISTORY_LIMIT);
}

/** Reactive: how many actions are kept. The user's size (or the administrator's default or lock), never above the cap. */
export function undoCapacity(): number {
  return Math.min(intSetting(UNDO_HISTORY_SIZE_KEY, UNDO_HISTORY_DEFAULT), undoHistoryCap());
}

/** Reactive: whether the history, its buttons and its shortcuts are on. */
export function undoEnabled(): boolean {
  return undoCapacity() > 0;
}

// Oldest first. The redo stack's last element is the next redo.
const [undoList, setUndoList] = createSignal<readonly UndoEntry[]>([]);
const [redoList, setRedoList] = createSignal<readonly UndoEntry[]>([]);
const [running, setRunning] = createSignal(false);
const [progress, setProgress] = createSignal<{kind: 'undo' | 'redo'; done: number; total: number} | null>(null);
const [announcement, setAnnouncement] = createSignal('');
let sequence = 0;
let replaying = false;

/** Undoable actions, newest first, at most the capacity. */
export function undoEntries(): readonly UndoEntry[] {
  return undoList().slice(-undoCapacity()).reverse();
}
/** Redoable actions, the next redo first. */
export function redoEntries(): readonly UndoEntry[] {
  return redoList().slice(-undoCapacity()).reverse();
}
/** An undo or redo is running. */
export const undoBusy = running;
/** Multi-step progress, or null. */
export const undoProgress = progress;
/** The last result, for an aria-live region. */
export const undoAnnouncement = announcement;
/** True while the history replays an entry: the action sites record nothing and raise no toast then. */
export function undoReplaying(): boolean {
  return replaying;
}

/** Forget everything (sign-out, account switch, tests). */
export function clearUndoHistory(): void {
  setUndoList([]);
  setRedoList([]);
  setProgress(null);
  setAnnouncement('');
  setRunning(false);
  replaying = false;
}

/** Record a finished action. Returns the entry, or null when the history is off or a replay is running. */
export function recordUndoable(action: UndoableAction): UndoEntry | null {
  const capacity = undoCapacity();
  if (capacity === 0 || replaying) return null;
  void loadCatalog('undo'); // the messages of a later undo or redo
  const entry: UndoEntry = {id: ++sequence, label: action.label, at: Date.now(), accountId: action.accountId ?? null, action};
  setUndoList((list) => [...list, entry].slice(-capacity));
  setRedoList([]);
  return entry;
}

function failureText(entry: UndoEntry, kind: 'undo' | 'redo', error: unknown): string {
  const reason = error instanceof UndoUnavailable ? error.reason : undefined;
  if (reason) return t(kind === 'undo' ? 'undo-cant-undo-reason' : 'undo-cant-redo-reason', {reason});
  return t(kind === 'undo' ? 'undo-cant-undo' : 'undo-cant-redo', {label: entry.label});
}

async function step(kind: 'undo' | 'redo', entry: UndoEntry): Promise<boolean> {
  const from = kind === 'undo' ? undoList : redoList;
  const setFrom = kind === 'undo' ? setUndoList : setRedoList;
  const setTo = kind === 'undo' ? setRedoList : setUndoList;
  replaying = true;
  try {
    await (kind === 'undo' ? entry.action.undo() : entry.action.redo());
  } catch (error) {
    replaying = false;
    setFrom(from().filter((item) => item !== entry));
    const text = failureText(entry, kind, error);
    setAnnouncement(text);
    toast.error(text, {id: 'undo-history'});
    return false;
  }
  replaying = false;
  setFrom(from().filter((item) => item !== entry));
  setTo((list) => [...list, entry].slice(-Math.max(1, undoCapacity())));
  const text = t(kind === 'undo' ? 'undo-done' : 'redo-done', {label: entry.label});
  setAnnouncement(text);
  toast.info(text, {id: 'undo-history'});
  entry.action.onSettled?.();
  return true;
}

async function through(kind: 'undo' | 'redo', id: number | null): Promise<number | null> {
  if (running()) return null;
  const stack = kind === 'undo' ? undoList : redoList;
  const chain = [...stack()].reverse();
  const stop = id === null ? 0 : chain.findIndex((entry) => entry.id === id);
  if (chain.length === 0 || stop < 0) return null;
  const todo = chain.slice(0, stop + 1);
  setRunning(true);
  let done = 0;
  try {
    for (const entry of todo) {
      if (todo.length > 1) setProgress({kind, done, total: todo.length});
      if (!(await step(kind, entry))) break;
      done++;
    }
  } finally {
    setProgress(null);
    setRunning(false);
  }
  return done;
}

/** Undo just `id` (the toast's Undo). Resolves 1 or 0, or null when `id` is not in the history or another replay is running. */
export async function undoOnly(id: number): Promise<number | null> {
  const entry = undoList().find((item) => item.id === id);
  if (entry === undefined || running()) return null;
  setRunning(true);
  try {
    return (await step('undo', entry)) ? 1 : 0;
  } finally {
    setRunning(false);
  }
}

/** Undo the newest action. Resolves the number undone, or null when nothing ran. */
export const undoLast = (): Promise<number | null> => through('undo', null);
/** Redo the most recently undone action. */
export const redoLast = (): Promise<number | null> => through('redo', null);
/** Undo `id` and every newer action, newest first, stopping at the first failure. null when `id` is not in the history. */
export const undoThrough = (id: number): Promise<number | null> => through('undo', id);
/** Redo back through `id`, nearest first, stopping at the first failure. */
export const redoThrough = (id: number): Promise<number | null> => through('redo', id);

/** The element that is typing, where Ctrl+Z belongs to the field and not to the app. */
export function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  if (target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement) return true;
  if (target instanceof HTMLInputElement) {
    return !['button', 'checkbox', 'radio', 'submit', 'reset', 'image', 'range', 'color', 'file'].includes(target.type);
  }
  return target.closest('[contenteditable=""], [contenteditable="true"], .cm-editor, .ProseMirror') !== null;
}

/**
 * Handle Ctrl/Cmd+Z (undo) and Ctrl/Cmd+Shift+Z or Ctrl+Y (redo). Leaves the key
 * alone in text fields, in modal dialogs, while another handler used it, and
 * when the history is off or has nothing for it. True when it ran an action.
 */
export function handleUndoKey(event: KeyboardEvent): boolean {
  if (event.defaultPrevented || event.isComposing || event.altKey) return false;
  if (!(event.ctrlKey || event.metaKey)) return false;
  const key = event.key.toLowerCase();
  const redo = (key === 'z' && event.shiftKey) || (key === 'y' && event.ctrlKey && !event.metaKey && !event.shiftKey);
  const undo = key === 'z' && !event.shiftKey;
  if (!undo && !redo) return false;
  if (!undoEnabled() || isTypingTarget(event.target)) return false;
  const target = event.target;
  if (target instanceof Element && target.closest('[aria-modal="true"], dialog[open]') !== null) return false;
  if (running() || (redo ? redoList() : undoList()).length === 0) return false;
  event.preventDefault();
  void (redo ? redoLast() : undoLast());
  return true;
}
