// The composer header's collapse rules (pure) and its per-user memory.
//
// `mail.compose.headerCollapse` is the configured behaviour: `expanded` (always
// open until the user folds it), `collapsed` (folded until the user opens it)
// or `auto` (folded while the user is writing the body, open when a field is
// focused or needs attention). The user's own fold/unfold is remembered in this
// browser, under a key that survives reopening the composer.

import { parseRecipients } from '../../api/jmap.ts';

export type HeaderCollapseMode = 'expanded' | 'collapsed' | 'auto';
export const HEADER_COLLAPSE_MODES: readonly HeaderCollapseMode[] = ['expanded', 'collapsed', 'auto'];
export const HEADER_COLLAPSE_DEFAULT: HeaderCollapseMode = 'expanded';
export function isHeaderCollapseMode(value: unknown): value is HeaderCollapseMode {
  return typeof value === 'string' && (HEADER_COLLAPSE_MODES as readonly string[]).includes(value);
}

const STORAGE_KEY = 'mw.compose.headerOpen';

/** The remembered choice: `true` open, `false` folded, `null` none (or storage unavailable). */
export function readHeaderMemory(storage: Pick<Storage, 'getItem'> | undefined = safeStorage()): boolean | null {
  try {
    const raw = storage?.getItem(STORAGE_KEY);
    return raw === '1' ? true : raw === '0' ? false : null;
  } catch {
    return null;
  }
}

export function writeHeaderMemory(open: boolean, storage: Pick<Storage, 'setItem'> | undefined = safeStorage()): void {
  try {
    storage?.setItem(STORAGE_KEY, open ? '1' : '0');
  } catch {
    // Storage is blocked or full: the choice simply is not remembered.
  }
}

function safeStorage(): Storage | undefined {
  try {
    return typeof localStorage === 'undefined' ? undefined : localStorage;
  } catch {
    return undefined;
  }
}

export interface HeaderOpenInput {
  mode: HeaderCollapseMode;
  /** What the user chose last time, or `null`. Ignored in `auto`, which follows focus. */
  remembered: boolean | null;
  /** Focus is inside the header's fields. */
  focusWithin: boolean;
  /** Something needs attention (a problem was clicked in the status bar). */
  attention: boolean;
  /** The header has nothing to summarise (no recipient and no subject yet). */
  empty: boolean;
  /** The user opened it for a moment in `auto` mode (a click on the summary). */
  peek: boolean;
}

/** Whether the header's fields are on screen. */
export function headerOpen(input: HeaderOpenInput): boolean {
  if (input.focusWithin || input.attention) return true;
  if (input.empty) return true;
  if (input.mode === 'auto') return input.peek;
  return input.remembered ?? input.mode === 'expanded';
}

function shortName(raw: string): string {
  const [first] = parseRecipients(raw);
  if (!first) return raw.trim();
  return first.name?.trim() || first.email.split('@')[0] || first.email;
}

function countOf(raw: string): number {
  return parseRecipients(raw).filter((entry) => entry.email.trim() !== '').length;
}

export interface SummaryLabels { to: string; cc: string; bcc: string; subject: string; noSubject: string }

/** One line: `To: Ana +2 · Cc: 1 · Subject: Re: Invoice`. */
export function headerSummary(fields: { to: string; cc: string; bcc: string; subject: string }, labels: SummaryLabels): string {
  const parts: string[] = [];
  const toCount = countOf(fields.to);
  if (toCount > 0) parts.push(`${labels.to}: ${shortName(fields.to)}${toCount > 1 ? ` +${toCount - 1}` : ''}`);
  const ccCount = countOf(fields.cc);
  if (ccCount > 0) parts.push(`${labels.cc}: ${ccCount}`);
  const bccCount = countOf(fields.bcc);
  if (bccCount > 0) parts.push(`${labels.bcc}: ${bccCount}`);
  parts.push(`${labels.subject}: ${fields.subject.trim() === '' ? labels.noSubject : fields.subject.trim()}`);
  return parts.join(' · ');
}
