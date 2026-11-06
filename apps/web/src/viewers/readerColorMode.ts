// Per-message colour mode for the reader.
//
// The modes are exactly the choices of the `reader.colorMode` setting
// (Settings > Reader). A choice made in the reader applies to the one message
// shown, never to the saved setting: it is kept in memory for the session (and
// in sessionStorage, so a reload of the same tab keeps it) keyed by
// account + message id. Admin-locked `reader.colorMode` ignores overrides.

import { createSignal } from 'solid-js';
import type { ReaderColorMode } from './readerAppearance.ts';

export const READER_COLOR_CHOICES = ['auto', 'original', 'adaptive', 'invert'] as const;
export type ReaderColorChoice = (typeof READER_COLOR_CHOICES)[number];

export function isReaderColorChoice(value: unknown): value is ReaderColorChoice {
  return typeof value === 'string' && (READER_COLOR_CHOICES as readonly string[]).includes(value);
}

/** The mode a choice renders as: automatic follows the theme. */
export function resolveReaderColorMode(choice: ReaderColorChoice, appearance: 'dark' | 'light' | string): ReaderColorMode {
  return choice === 'auto' ? (appearance === 'dark' ? 'adaptive' : 'original') : choice;
}

const STORAGE_KEY = 'mw.reader.colorOverrides';
const MAX_ENTRIES = 200;

function load(): Record<string, ReaderColorChoice> {
  try {
    const raw = typeof sessionStorage === 'undefined' ? null : sessionStorage.getItem(STORAGE_KEY);
    const parsed: unknown = raw === null ? {} : JSON.parse(raw);
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return {};
    const out: Record<string, ReaderColorChoice> = {};
    for (const [key, value] of Object.entries(parsed)) if (isReaderColorChoice(value)) out[key] = value;
    return out;
  } catch { return {}; }
}

const [overrides, setOverrides] = createSignal<Record<string, ReaderColorChoice>>(load());

export function readerColorKey(accountId: string, emailId: string): string {
  return `${accountId}:${emailId}`;
}

/** Reactive: the choice made for this message in this session, if any. */
export function messageColorOverride(key: string): ReaderColorChoice | undefined {
  return overrides()[key];
}

/** `null` clears the override (back to the user's setting). */
export function setMessageColorOverride(key: string, choice: ReaderColorChoice | null): void {
  const next = { ...overrides() };
  delete next[key];
  if (choice !== null) next[key] = choice;
  const keys = Object.keys(next);
  for (const old of keys.slice(0, Math.max(0, keys.length - MAX_ENTRIES))) delete next[old];
  setOverrides(next);
  try { sessionStorage.setItem(STORAGE_KEY, JSON.stringify(next)); } catch { /* storage unavailable: the in-memory copy still applies */ }
}

/** Test seam. */
export function resetMessageColorOverrides(): void {
  setOverrides({});
  try { sessionStorage.removeItem(STORAGE_KEY); } catch { /* ignore */ }
}
