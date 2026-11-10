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
  type EmailQueryResponse,
  type Id,
  type JmapRequest,
} from '../api/jmap-types.ts';
import { viewerKindFor } from '../contracts/viewer.ts';
import { apiFetch } from '../api/http.ts';
import {activeLinkedMailbox} from '../api/linkedMailboxSession.ts';
import {linkedResourceUrl} from '../api/linkedAccountRouting.ts';
import {transportBase} from '../api/transport.ts';
import { formatBytes } from '../i18n/format.ts';

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
  /** Owning folders; absent on older callers' locally constructed items. */
  mailboxIds?: Record<Id, boolean>;
}

function firstFrom(from: EmailAddress[] | null): string {
  const a = from?.[0];
  if (a === undefined) return '';
  return a.name !== null && a.name.length > 0 ? `${a.name} <${a.email}>` : a.email;
}

/** One-round-trip query for every message with an attachment + its attachment list. */
export function attachmentsQuery(accountId: Id, limit = 200, position = 0): JmapRequest {
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
          position,
          calculateTotal: true,
        },
        'aq',
      ],
      [
        'Email/get',
        {
          accountId,
          '#ids': { resultOf: 'aq', name: 'Email/query', path: '/ids' },
          properties: ['id', 'from', 'subject', 'receivedAt', 'size', 'mailboxIds', 'attachments'],
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
        ...(e.mailboxIds ? {mailboxIds: {...e.mailboxIds}} : {}),
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

export interface AttachmentPage {
  items: AttachmentItem[];
  position: number;
  nextPosition: number | null;
}

export interface AttachmentSearchProgress {
  items: AttachmentItem[];
  pages: number;
  limited: boolean;
}

/** Search message pages sequentially; never download attachment content. */
export async function searchAttachmentPages(
  load: (position: number) => Promise<AttachmentPage>,
  filters: AttachmentFilters,
  signal: AbortSignal,
  progress: (value: AttachmentSearchProgress) => void,
): Promise<AttachmentSearchProgress> {
  const items: AttachmentItem[] = [];
  const seen = new Set<string>();
  let position = 0;
  let pages = 0;
  while (!signal.aborted) {
    const page = await load(position);
    signal.throwIfAborted();
    if (page.position !== position || (page.nextPosition !== null && (!Number.isSafeInteger(page.nextPosition) || page.nextPosition <= position))) {
      throw new Error('Invalid attachment search page');
    }
    pages++;
    for (const item of filterAttachments(page.items, filters)) {
      const key = JSON.stringify([item.emailId, item.blobId, item.name]);
      if (seen.has(key)) continue;
      seen.add(key);
      items.push(item);
      if (items.length >= 1000) break;
    }
    const result = {items: [...items], pages, limited: items.length >= 1000};
    progress(result);
    if (result.limited || page.nextPosition === null) return result;
    position = page.nextPosition;
  }
  signal.throwIfAborted();
  return {items, pages, limited: false};
}

/** Page by messages, not flattened files: one message can contain many files. */
export async function loadAttachmentPage(client: Client, accountId: Id, position = 0): Promise<AttachmentPage> {
  const limit = 50;
  const res = await client.jmap(attachmentsQuery(accountId, limit, position));
  const query = responseFor<EmailQueryResponse>(res, 'aq');
  const got = responseFor<EmailGetResponse>(res, 'ag');
  if (query.accountId !== accountId || got.accountId !== accountId || query.position !== position) {
    throw new Error('Attachment response does not match the requested account or page');
  }
  const next = position + query.ids.length;
  return {
    items: parseAttachments(got.list as EmailWithAttachments[]),
    position,
    nextPosition: query.ids.length === 0 || (query.total !== undefined ? next >= query.total : query.ids.length < limit) ? null : next,
  };
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
  const url=template
    .replace(/\{accountId\}/g, encodeURIComponent(ref.accountId))
    .replace(/\{blobId\}/g, encodeURIComponent(ref.blobId))
    .replace(/\{name\}/g, encodeURIComponent(ref.name))
    .replace(/\{type\}/g, encodeURIComponent(ref.mime));
  const linked=activeLinkedMailbox();
  if(!linked)return url;
  if(linked.accountId!==ref.accountId)throw new Error('Mailbox changed');
  return linkedResourceUrl(url,linked.linkId,new URL(`${transportBase()}/`,globalThis.location.href).href);
}

/** Fetch a blob (cookie-authed, same-origin) and wrap it in an object URL. */
export async function fetchObjectUrl(url: string,signal?:AbortSignal): Promise<string> {
  const res = await apiFetch(url, { credentials: 'same-origin',...(signal?{signal}:{}) });
  if (!res.ok) throw new Error(`attachment download failed: ${res.status}`);
  const blob=await res.blob();signal?.throwIfAborted();return URL.createObjectURL(blob);
}

// ── Filtering + operator search ─────────────────────────────────────────────

export type TypeCategory = 'image' | 'pdf' | 'audio' | 'video' | 'text' | 'message' | 'document' | 'spreadsheet' | 'archive' | 'other';

/** Group a MIME type using the very same routing as the viewers (`viewerKindFor`). */
export function categoryOf(mime: string, name = ''): TypeCategory {
  const normalized = mime.split(';', 1)[0]!.trim().toLowerCase();
  const spreadsheet = /^(?:application\/(?:vnd\.ms-excel(?:\..*)?|vnd\.openxmlformats-officedocument\.spreadsheetml\..*|vnd\.oasis\.opendocument\.spreadsheet(?:-template)?)|text\/(?:csv|tab-separated-values))$/;
  const document = /^application\/(?:msword|rtf|postscript|epub\+zip|vnd\.ms-word\..*|vnd\.openxmlformats-officedocument\.(?:wordprocessingml|presentationml)\..*|vnd\.oasis\.opendocument\.(?:text|presentation)(?:-template)?|vnd\.ms-powerpoint(?:\..*)?)$/;
  const archive = /^application\/(?:zip|x-zip-compressed|x-7z-compressed|x-rar-compressed|vnd\.rar|gzip|x-gzip|x-tar|x-bzip2?|x-xz|zstd)$/;
  if (spreadsheet.test(normalized)) return 'spreadsheet';
  if (document.test(normalized) || normalized === 'text/rtf') return 'document';
  const kind = viewerKindFor(normalized);
  if (kind === 'unsupported' || normalized === 'application/zip') {
    if (/\.(?:xlsx?|xlsm|xlsb|xltx?|ods|ots|csv|tsv)$/i.test(name)) return 'spreadsheet';
    if (/\.(?:docx?|docm|dotx?|odt|ott|rtf|pptx?|pptm|odp|epub)$/i.test(name)) return 'document';
    if (archive.test(normalized) || /\.(?:zip|7z|rar|tar|gz|bz2?|xz|zst)$/i.test(name)) return 'archive';
  }
  // A calendar file is grouped with text, as before it had its own viewer (t53).
  return kind === 'unsupported' ? 'other' : kind === 'calendar' ? 'text' : kind;
}

export interface AttachmentFilters {
  /** Free words, each matching name, sender, or subject (case-insensitive). */
  text?: string;
  /** A dedicated filename substring. */
  filename?: string;
  subject?: string;
  /** OR within this list; leading dots and case are ignored. */
  extensions?: readonly string[];
  mailboxId?: Id;
  /** Additional filters combined with AND, including advanced query + form. */
  and?: readonly AttachmentFilters[];
  category?: TypeCategory | 'all';
  /** Sender substring (case-insensitive). */
  from?: string;
  minSize?: number;
  maxSize?: number;
  /** ISO instants inclusive; date-only values include their entire UTC day. */
  before?: string;
  after?: string;
}

export function filterAttachments(items: AttachmentItem[], f: AttachmentFilters): AttachmentItem[] {
  const budget = {remaining: 64};
  function predicate(filters: AttachmentFilters, depth = 0): (item: AttachmentItem) => boolean {
    if (depth > 8 || --budget.remaining < 0) return () => false;
    const children = filters.and?.map(child => predicate(child, depth + 1)) ?? [];
    const words = tokenize(filters.text ?? '').map(word => word.replace(/"/g, '').toLowerCase()).filter(Boolean);
    const filename = filters.filename?.trim().toLowerCase();
    const from = filters.from?.trim().toLowerCase();
    const subject = filters.subject?.trim().toLowerCase();
    const extensions = filters.extensions?.flatMap(value => value.split(/[,;\s]+/)).map(value => value.trim().replace(/^\.+/, '').toLowerCase()).filter(Boolean) ?? [];
    const after = dateBound(filters.after, false), before = dateBound(filters.before, true);
    return item => {
      if (Number.isNaN(after) || Number.isNaN(before)) return false;
      if ([filters.minSize, filters.maxSize].some(value => value !== undefined && (!Number.isFinite(value) || value < 0))) return false;
      if (!children.every(matches => matches(item))) return false;
      if (filters.category !== undefined && filters.category !== 'all' && categoryOf(item.mime, item.name) !== filters.category) return false;
      const name = item.name.toLowerCase(), sender = item.from.toLowerCase(), title = item.subject.toLowerCase();
      if (!words.every(word => name.includes(word) || sender.includes(word) || title.includes(word))) return false;
      if (filename && !name.includes(filename)) return false;
      if (from && !sender.includes(from)) return false;
      if (subject && !title.includes(subject)) return false;
      if (extensions.length && !extensions.some(extension => name.endsWith(`.${extension}`))) return false;
      if (filters.mailboxId && item.mailboxIds?.[filters.mailboxId] !== true) return false;
      if (filters.minSize !== undefined && item.size < filters.minSize) return false;
      if (filters.maxSize !== undefined && item.size > filters.maxSize) return false;
      if (after !== undefined || before !== undefined) {
        const received = Date.parse(item.receivedAt);
        if (!Number.isFinite(received) || (after !== undefined && received < after) || (before !== undefined && received > before)) return false;
      }
      return true;
    };
  }
  return items.filter(predicate(f));
}

function dateBound(value: string | undefined, end: boolean): number | undefined {
  if (!value) return undefined;
  const date = Date.parse(value);
  if (!Number.isFinite(date)) return Number.NaN;
  if (/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    if (new Date(date).toISOString().slice(0, 10) !== value) return Number.NaN;
    return date + (end ? 86_400_000 - 1 : 0);
  }
  return date;
}

/** Parse `1mb` / `500kb` / `2048` into bytes (undefined if unparseable). */
export function parseSize(raw: string): number | undefined {
  const m = /^(\d+(?:\.\d+)?)\s*(b|kb|mb|gb)?$/i.exec(raw.trim());
  if (m === null || m[1] === undefined) return undefined;
  const n = Number.parseFloat(m[1]);
  const unit = (m[2] ?? 'b').toLowerCase();
  const mult = unit === 'gb' ? 1e9 : unit === 'mb' ? 1e6 : unit === 'kb' ? 1e3 : 1;
  const bytes = Math.round(n * mult);
  return Number.isSafeInteger(bytes) && bytes >= 0 ? bytes : undefined;
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
    case 'message':
    case 'eml':
    case 'tnef':
      return 'message';
    case 'document':
    case 'documents':
    case 'doc':
    case 'docx':
    case 'word':
      return 'document';
    case 'spreadsheet':
    case 'spreadsheets':
    case 'sheet':
    case 'xls':
    case 'xlsx':
    case 'csv':
      return 'spreadsheet';
    case 'archive':
    case 'archives':
    case 'zip':
    case 'rar':
    case '7z':
      return 'archive';
    case 'all':
    case 'any':
      return 'all';
    default:
      return 'other';
  }
}

function tokenize(q: string): string[] {
  return q.match(/(?:[^\s"]+|"[^"]*")+/g) ?? [];
}

/**
 * Parse a search string using the shared operators (`filename:`/`type:`/`from:`/
 * `larger:`/`smaller:`/`before:`/`after:`); bare words match message/file context.
 */
export function parseAttachmentQuery(q: string): AttachmentFilters {
  const f: AttachmentFilters = {};
  const free: string[] = [];
  for (const tok of tokenize(q)) {
    const m = /^([a-z]+):(.*)$/i.exec(tok);
    const op = m?.[1]?.toLowerCase();
    const val = m?.[2]?.replace(/"/g, '');
    if (op === undefined || val === undefined || val.length === 0) {
      free.push(tok);
      continue;
    }
    switch (op) {
      case 'filename':
      case 'name':
        f.filename = val;
        break;
      case 'type':
      case 'kind':
        f.category = normalizeCategory(val);
        break;
      case 'from':
        f.from = val;
        break;
      case 'subject':
        f.subject = val;
        break;
      case 'ext':
      case 'extension':
      case 'extensions':
        f.extensions = val.split(/[,;]+/).map(value => value.replace(/^\.+/, '').toLowerCase()).filter(Boolean);
        break;
      case 'in':
      case 'folder':
        f.mailboxId = val;
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
  if (free.length > 0) f.text = free.join(' ');
  return f;
}

/** Human-readable byte size for the grid (e.g. `1.4 MB`), in the app locale. */
export function formatSize(bytes: number): string {
  return formatBytes(bytes);
}
