// Data layer for the Attachments module (plan §2.4) + the thumbnail strip.
//
// The global grid is `Email/query{filter:{hasAttachment:true}}` + an `Email/get`
// for each message's `attachments` list, one round-trip via a JMAP result ref.
// Filtering (type / sender / size / date) and the operator search
// (`filename:` / `type:` / `larger:` / `from:` …) run over the parsed list —
// the same operator vocabulary the engine `mw-search` backs online (e9). The
// pure functions here are what the module's filter tests exercise.

import type { Client } from '../api/client.ts';
import { responseFor } from '../api/jmap.ts';
import {
  CAP_CORE,
  CAP_MAIL,
  type Email,
  type EmailAddress,
  type EmailBodyPart,
  type EmailGetResponse,
  type Id,
  type JmapRequest,
} from '../api/jmap-types.ts';
import { viewerKindFor } from '../contracts/viewer.ts';

/** JMAP `attachments` parts carry a filename/disposition beyond the body-part core. */
export interface AttachmentPart extends EmailBodyPart {
  name?: string | null;
  disposition?: string | null;
  cid?: string | null;
}
interface EmailWithAttachments extends Email {
  attachments?: AttachmentPart[];
}

/** One attachment flattened with its message context for the global grid. */
export interface AttachmentItem {
  emailId: Id;
  blobId: string;
  name: string;
  mime: string;
  size: number;
  from: string;
  subject: string;
  receivedAt: string;
}

function firstFrom(from: EmailAddress[] | null): string {
  const a = from?.[0];
  if (a === undefined) return '';
  return a.name !== null && a.name.length > 0 ? `${a.name} <${a.email}>` : a.email;
}

/** One-round-trip query for every message with an attachment + its attachment list. */
export function attachmentsQuery(accountId: Id, limit = 200): JmapRequest {
  return {
    using: [CAP_CORE, CAP_MAIL],
    methodCalls: [
      [
        'Email/query',
        {
          accountId,
          filter: { hasAttachment: true },
          sort: [{ property: 'receivedAt', isAscending: false }],
          limit,
        },
        'aq',
      ],
      [
        'Email/get',
        {
          accountId,
          '#ids': { resultOf: 'aq', name: 'Email/query', path: '/ids' },
          properties: ['id', 'from', 'subject', 'receivedAt', 'size', 'attachments'],
        },
        'ag',
      ],
    ],
  };
}

/** Flatten `Email/get` results into per-attachment rows (skips inline-only cids). */
export function parseAttachments(emails: EmailWithAttachments[]): AttachmentItem[] {
  const out: AttachmentItem[] = [];
  for (const e of emails) {
    for (const a of e.attachments ?? []) {
      if (a.blobId === null || a.blobId === undefined || a.blobId === '') continue;
      out.push({
        emailId: e.id,
        blobId: a.blobId,
        name: a.name !== null && a.name !== undefined && a.name.length > 0 ? a.name : '(unnamed)',
        mime: a.type.length > 0 ? a.type : 'application/octet-stream',
        size: a.size,
        from: firstFrom(e.from),
        subject: e.subject ?? '(no subject)',
        receivedAt: e.receivedAt,
      });
    }
  }
  return out;
}

/** Live loader: runs the query and flattens it. Used by `<Attachments>` in-app. */
export async function loadAttachments(client: Client, accountId: Id): Promise<AttachmentItem[]> {
  const res = await client.jmap(attachmentsQuery(accountId));
  const got = responseFor<EmailGetResponse>(res, 'ag');
  return parseAttachments(got.list as EmailWithAttachments[]);
}

// ── Download URL + blob fetch ───────────────────────────────────────────────

export interface BlobRef {
  accountId: Id;
  blobId: string;
  name: string;
  mime: string;
}

/** Substitute a JMAP `downloadUrl` URI template (RFC 8620 §6.2). */
export function buildDownloadUrl(template: string, ref: BlobRef): string {
  return template
    .replace(/\{accountId\}/g, encodeURIComponent(ref.accountId))
    .replace(/\{blobId\}/g, encodeURIComponent(ref.blobId))
    .replace(/\{name\}/g, encodeURIComponent(ref.name))
    .replace(/\{type\}/g, encodeURIComponent(ref.mime));
}

/** Fetch a blob (cookie-authed, same-origin) and wrap it in an object URL. */
export async function fetchObjectUrl(url: string): Promise<string> {
  const res = await fetch(url, { credentials: 'same-origin' });
  if (!res.ok) throw new Error(`attachment download failed: ${res.status}`);
  return URL.createObjectURL(await res.blob());
}

// ── Filtering + operator search ─────────────────────────────────────────────

export type TypeCategory = 'image' | 'pdf' | 'audio' | 'video' | 'text' | 'other';

/** Group a MIME type using the very same routing as the viewers (`viewerKindFor`). */
export function categoryOf(mime: string): TypeCategory {
  const kind = viewerKindFor(mime);
  return kind === 'unsupported' ? 'other' : kind;
}

export interface AttachmentFilters {
  /** Filename substring (case-insensitive). */
  text?: string;
  category?: TypeCategory | 'all';
  /** Sender substring (case-insensitive). */
  from?: string;
  minSize?: number;
  maxSize?: number;
  /** ISO dates; `after` is inclusive-lower, `before` inclusive-upper. */
  before?: string;
  after?: string;
}

export function filterAttachments(items: AttachmentItem[], f: AttachmentFilters): AttachmentItem[] {
  const text = f.text?.toLowerCase();
  const from = f.from?.toLowerCase();
  return items.filter((it) => {
    if (f.category !== undefined && f.category !== 'all' && categoryOf(it.mime) !== f.category) {
      return false;
    }
    if (text !== undefined && text.length > 0 && !it.name.toLowerCase().includes(text)) return false;
    if (from !== undefined && from.length > 0 && !it.from.toLowerCase().includes(from)) return false;
    if (f.minSize !== undefined && it.size < f.minSize) return false;
    if (f.maxSize !== undefined && it.size > f.maxSize) return false;
    if (f.after !== undefined && it.receivedAt < f.after) return false;
    if (f.before !== undefined && it.receivedAt > f.before) return false;
    return true;
  });
}

/** Parse `1mb` / `500kb` / `2048` into bytes (undefined if unparseable). */
export function parseSize(raw: string): number | undefined {
  const m = /^(\d+(?:\.\d+)?)\s*(b|kb|mb|gb)?$/i.exec(raw.trim());
  if (m === null || m[1] === undefined) return undefined;
  const n = Number.parseFloat(m[1]);
  const unit = (m[2] ?? 'b').toLowerCase();
  const mult = unit === 'gb' ? 1e9 : unit === 'mb' ? 1e6 : unit === 'kb' ? 1e3 : 1;
  return Math.round(n * mult);
}

function normalizeCategory(v: string): TypeCategory | 'all' {
  switch (v.toLowerCase()) {
    case 'image':
    case 'images':
    case 'img':
      return 'image';
    case 'pdf':
      return 'pdf';
    case 'audio':
    case 'sound':
      return 'audio';
    case 'video':
    case 'movie':
      return 'video';
    case 'text':
    case 'txt':
      return 'text';
    case 'all':
    case 'any':
      return 'all';
    default:
      return 'other';
  }
}

function tokenize(q: string): string[] {
  return (q.match(/(?:[^\s"]+|"[^"]*")+/g) ?? []).map((t) => t.replace(/"/g, ''));
}

/**
 * Parse a search string using the shared operators (`filename:`/`type:`/`from:`/
 * `larger:`/`smaller:`/`before:`/`after:`); bare words fall back to filename.
 */
export function parseAttachmentQuery(q: string): AttachmentFilters {
  const f: AttachmentFilters = {};
  const free: string[] = [];
  for (const tok of tokenize(q)) {
    const m = /^([a-z]+):(.*)$/i.exec(tok);
    const op = m?.[1]?.toLowerCase();
    const val = m?.[2];
    if (op === undefined || val === undefined || val.length === 0) {
      free.push(tok);
      continue;
    }
    switch (op) {
      case 'filename':
      case 'name':
        f.text = val;
        break;
      case 'type':
      case 'kind':
        f.category = normalizeCategory(val);
        break;
      case 'from':
        f.from = val;
        break;
      case 'larger': {
        const n = parseSize(val);
        if (n !== undefined) f.minSize = n;
        break;
      }
      case 'smaller': {
        const n = parseSize(val);
        if (n !== undefined) f.maxSize = n;
        break;
      }
      case 'before':
        f.before = val;
        break;
      case 'after':
        f.after = val;
        break;
      default:
        free.push(tok);
    }
  }
  if (free.length > 0 && f.text === undefined) f.text = free.join(' ');
  return f;
}

/** Human-readable byte size for the grid (e.g. `1.4 MB`). */
export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ['KB', 'MB', 'GB'];
  let n = bytes / 1024;
  let i = 0;
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024;
    i++;
  }
  return `${n.toFixed(1)} ${units[i]}`;
}
