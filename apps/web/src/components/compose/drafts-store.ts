// Local draft persistence for the universal Drafts drawer (W9).
//
// Composer drafts are auto-saved to `localStorage` under a single JSON array so a
// closed/refreshed composer can be resumed without a round-trip. This is the
// client-side recovery layer; server-side draft sync (the JMAP Drafts mailbox)
// remains the durable store and is out of this executor's file ownership — the
// drawer is written prop-first so a server list can back it later without a
// component change.

const KEY = 'mw.compose.drafts.v1';
const MAX_DRAFTS = 25;

export interface StoredDraft {
  id: string;
  to: string;
  subject: string;
  bodyHtml: string;
  bodyText: string;
  savedAt: number;
}

function read(): StoredDraft[] {
  try {
    const raw = localStorage.getItem(KEY);
    if (raw === null) return [];
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((d): d is StoredDraft => isDraft(d));
  } catch {
    return [];
  }
}

function isDraft(d: unknown): d is StoredDraft {
  if (typeof d !== 'object' || d === null) return false;
  const r = d as Record<string, unknown>;
  return (
    typeof r.id === 'string' &&
    typeof r.to === 'string' &&
    typeof r.subject === 'string' &&
    typeof r.bodyHtml === 'string' &&
    typeof r.bodyText === 'string' &&
    typeof r.savedAt === 'number'
  );
}

function write(drafts: StoredDraft[]): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(drafts.slice(0, MAX_DRAFTS)));
  } catch {
    // Storage full / disabled (private mode): drafts degrade to in-session only.
  }
}

/** All stored drafts, newest first. */
export function listDrafts(): StoredDraft[] {
  return read().sort((a, b) => b.savedAt - a.savedAt);
}

/** A draft is worth saving only once it has some recipient / subject / body. */
export function draftHasContent(d: Pick<StoredDraft, 'to' | 'subject' | 'bodyText'>): boolean {
  return d.to.trim() !== '' || d.subject.trim() !== '' || d.bodyText.trim() !== '';
}

/** Insert or update a draft by id (no-op when it has no content yet). */
export function saveDraft(draft: StoredDraft): void {
  if (!draftHasContent(draft)) return;
  const rest = read().filter((d) => d.id !== draft.id);
  write([draft, ...rest]);
}

/** Remove a draft by id (e.g. after it is sent or discarded). */
export function deleteDraft(id: string): void {
  write(read().filter((d) => d.id !== id));
}

/** A fresh, collision-resistant draft id for a composer session. */
export function newDraftId(): string {
  return `d-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}
