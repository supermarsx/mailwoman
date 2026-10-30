// Local draft persistence for the universal Drafts drawer (W9).
//
// Composer drafts are auto-saved to `localStorage` under a single JSON array so a
// closed/refreshed composer can be resumed without a round-trip. These copies
// are account-bound saved state of an unsent composition: the composer does not
// write drafts to the server's Drafts mailbox. They are plaintext (recipients,
// subject, body) and belong to whoever is signed in, so `clearDrafts()` is
// called on logout. The drawer is written prop-first so a server list can back
// it later without a component change.

const KEY = 'mw.compose.drafts.v2';
const LEGACY_KEY='mw.compose.drafts.v1';
const MAX_DRAFTS = 25;

/** An image placed in the body, as a draft keeps it: the uploaded blob and the
 *  Content-ID the body refers to it by (`inlineImages.ts`). */
export interface StoredInlineImage {
  cid: string;
  blobId: string;
  type: string;
  name: string;
  size: number;
}

export interface StoredAttachment {
  name: string;
  blobId: string;
  size: number;
  contentType: string | null;
}

export interface StoredDraft {
  id: string;
  accountId: string;
  storageAuthority: string;
  /** Revision last confirmed by the account Drafts API; never inferred from a newer server copy. */
  serverRevision?: string;
  serverStatus?:'saved'|'uncertain'|'refused';
  identityId?: string;
  fromOverride?: string;
  importance?: 'high'|'normal'|'low';
  bodyFormat?: 'plain' | 'html';
  insertedSignatures?: string[];
  to: string;
  /** Absent on a draft stored before Cc/Bcc and replies existed. */
  cc?: string;
  bcc?: string;
  inReplyTo?: string[];
  references?: string[];
  subject: string;
  bodyHtml: string;
  bodyText: string;
  savedAt: number;
  /** This message's send options (t29-e12); absent = off. */
  attachPublicKey?: boolean;
  includeFileHashes?: boolean;
  sentMailboxId?: string;
  requestReceipt?: boolean;
  requestDeliveryReceipt?: boolean;
  openTracking?: boolean;
  /** Images placed in the body; absent = none. */
  inline?: StoredInlineImage[];
  attachments?: StoredAttachment[];
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

const isString = (v: unknown): boolean => typeof v === 'string';
const isStringList = (v: unknown): boolean => Array.isArray(v) && v.every(isString);
const optional = (v: unknown, ok: (v: unknown) => boolean): boolean => v === undefined || ok(v);
const isBool = (v: unknown): boolean => typeof v === 'boolean';
const isInlineList = (v: unknown): boolean =>
  Array.isArray(v) &&
  v.every((i: unknown) => {
    if (typeof i !== 'object' || i === null) return false;
    const r = i as Record<string, unknown>;
    return isString(r.cid) && isString(r.blobId) && isString(r.type) && isString(r.name) && typeof r.size === 'number';
  });

const isAttachments = (v: unknown): boolean => Array.isArray(v) && v.length <= 100 && v.every(item => {
  if (typeof item !== 'object' || item === null) return false;
  const file = item as Record<string, unknown>;
  return isString(file.name) && typeof file.blobId === 'string' && file.blobId.length > 0
    && typeof file.size === 'number' && Number.isSafeInteger(file.size) && file.size >= 0
    && (file.contentType === null || isString(file.contentType));
});

function isDraft(d: unknown): d is StoredDraft {
  if (typeof d !== 'object' || d === null) return false;
  const r = d as Record<string, unknown>;
  return (
    typeof r.id === 'string' &&
    typeof r.accountId === 'string' && r.accountId.length > 0 &&
    typeof r.storageAuthority==='string'&&/^[A-Za-z0-9_-]{16,256}$/.test(r.storageAuthority)&&
    optional(r.importance,value=>value==='high'||value==='normal'||value==='low') &&
    optional(r.fromOverride, value => typeof value === 'string' && value.length <= 254 && !/[\r\n]/.test(value)) &&
    optional(r.serverStatus,value=>value==='saved'||value==='uncertain'||value==='refused') &&
    optional(r.serverRevision, isString) &&
    optional(r.identityId, isString) &&
    optional(r.bodyFormat, value => value === 'plain' || value === 'html') &&
    optional(r.insertedSignatures, isStringList) &&
    typeof r.to === 'string' &&
    typeof r.subject === 'string' &&
    typeof r.bodyHtml === 'string' &&
    typeof r.bodyText === 'string' &&
    typeof r.savedAt === 'number' &&
    optional(r.cc, isString) &&
    optional(r.bcc, isString) &&
    optional(r.inReplyTo, isStringList) &&
    optional(r.references, isStringList) &&
    optional(r.attachPublicKey, isBool) &&
    optional(r.includeFileHashes, isBool) &&
    optional(r.sentMailboxId, isString) &&
    optional(r.requestReceipt, isBool) &&
    optional(r.requestDeliveryReceipt, isBool) &&
    optional(r.openTracking, isBool) &&
    optional(r.inline, isInlineList) && optional(r.attachments, isAttachments)
  );
}

function write(drafts: StoredDraft[]): void {
  try {
    const counts = new Map<string, number>();
    localStorage.setItem(KEY, JSON.stringify(drafts.filter(draft => {
      const count = counts.get(draft.storageAuthority) ?? 0;
      counts.set(draft.storageAuthority, count + 1);
      return count < MAX_DRAFTS;
    })));
  } catch {
    // Storage full / disabled (private mode): drafts degrade to in-session only.
  }
}

/** Only this authenticated account drafts, newest first. Legacy unowned rows
 * are not attributed to whichever account happens to sign in next. */
export function listDrafts(accountId: string | null,storageAuthority:string|null|undefined): StoredDraft[] {
  if (!accountId||!storageAuthority) return [];
  return read().filter(draft => draft.accountId === accountId && draft.storageAuthority===storageAuthority).sort((a, b) => b.savedAt - a.savedAt);
}

/** A draft is worth saving only once it has some recipient / subject / body. */
export function draftHasContent(d: Pick<StoredDraft, 'to' | 'cc' | 'bcc' | 'subject' | 'bodyText' | 'attachments' | 'inline'>): boolean {
  return [d.to, d.cc ?? '', d.bcc ?? '', d.subject, d.bodyText].some((v) => v.trim() !== '') || (d.attachments?.length ?? 0) > 0 || (d.inline?.length ?? 0) > 0;
}

/** Insert or update a draft by id (no-op when it has no content yet). */
export function saveDraft(draft: StoredDraft): void {
  if (!draft.accountId || !isDraft(draft) || !draftHasContent(draft)) return;
  const rest = read().filter((d) => d.id !== draft.id || d.accountId !== draft.accountId || d.storageAuthority!==draft.storageAuthority);
  write([draft, ...rest]);
}

/** Remove a draft by id (e.g. after it is sent or discarded). */
export function deleteDraft(id: string, accountId: string | null,storageAuthority:string|null|undefined): void {
  if (!accountId||!storageAuthority) return;
  write(read().filter((d) => d.id !== id || d.accountId !== accountId || d.storageAuthority!==storageAuthority));
}

/** Remove every stored draft (logout: the next person at this browser must not
 *  be offered the previous account's unsent mail). */
export function clearDrafts(): void {
  try {
    localStorage.removeItem(KEY);
    localStorage.removeItem(LEGACY_KEY);
  } catch {
    // Storage disabled: nothing was stored.
  }
}

/** A fresh, collision-resistant draft id for a composer session. */
export function newDraftId(): string {
  return `d-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}
