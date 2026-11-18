import {stageLocalFile} from '../offline/localFiles.ts';
import {isOffline} from '../offline/runtime.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from './accountPreferenceLifecycle.ts';
import {offlineServerBase} from '../offline/cacheScope.ts';
export {request,responseFor,updateEmails,COUNTS_CALL_ID} from './jmapWire.ts';
import {request} from './jmapWire.ts';
// Pure JMAP request builders + response extraction. No I/O here so these are
// trivially unit-testable (see jmap.test.ts).

import {
  CAP_CORE,
  CAP_MAIL,
  CAP_SUBMISSION,
  type FilterCondition,
  type Id,
  type JmapRequest,
} from './jmap-types.ts';
import { apiFetch } from './http.ts';
import { settings } from './settings.ts';

// V2 (§2.1): the list view now fetches the modern-mail props so tags, pins,
// snooze, follow-up and the attachment indicator render straight off Email/get.
export const HEADER_PROPERTIES = [
  'id',
  'blobId',
  'from',
  'to',
  'subject',
  'receivedAt',
  'preview',
  'keywords',
  // Where the message is. A relocation's undo moves it back to these, and the
  // optimistic unread badge adjusts these folders; a server that projects
  // `properties` (RFC 8620 §5.1) leaves it out unless it is named here.
  'mailboxIds',
  'threadId',
  'pinned',
  'snoozedUntil',
  'followUpAt',
  'hasAttachment',
  'size',
  // t94: the server's Focused / Other decision, read from the index (no body).
  'mailwomanFocus',
  // The cached authentication summary the list needs for a brand logo (never computed per row).
  'mailwomanAuth',
  'mailwomanPriority',
  'mailwomanMentions',
] as const;

export const BODY_PROPERTIES = [
  'id',
  'blobId',
  'from',
  'to',
  'cc',
  'bcc',
  'replyTo',
  'subject',
  'receivedAt',
  'sentAt',
  'messageId',
  'inReplyTo',
  'references',
  'preview',
  'htmlBody',
  'textBody',
  'bodyValues',
  'attachments',
  'keywords',
  'mailboxIds',
  'threadId',
  'pinned',
  'snoozedUntil',
  'followUpAt',
  'hasAttachment',
  'size',
  'mailwomanPriority',
  'mailwomanMentions',
] as const;

/** Default cap so an upstream can send more; enough for V0 flows. */
const MAIL_USING = [CAP_CORE, CAP_MAIL];
const SUBMISSION_USING = [CAP_CORE, CAP_MAIL, CAP_SUBMISSION];


export function mailboxGet(accountId: Id, callId = 'c0'): JmapRequest {
  return request(MAIL_USING, [['Mailbox/get', { accountId, ids: null }, callId]]);
}

/**
 * Which slice of a query to fetch (t22-e4). Omitted entirely, these builders
 * emit exactly what they emitted before paging existed: the first `limit` ids
 * with the total calculated.
 *
 * `position` and `anchor` are the two ways RFC 8620 §5.5 lets a client name a
 * window. They are mutually exclusive; sending both is a client bug, so the
 * builders emit `anchor` and drop `position` when both are supplied rather than
 * letting the server arbitrate.
 */
export interface PageWindow {
  /** Absolute zero-based offset into the query result. */
  position?: number;
  /** Page relative to the position of this id instead (stable under inserts). */
  anchor?: Id;
  /** Offset applied to `anchor`; negative walks backwards. Ignored without one. */
  anchorOffset?: number;
  /**
   * Whether to ask for the query's total. Defaults to `true`, which is what the
   * un-paged builders always did. Pass `false` for the CONTINUATIONS of a query:
   * the total cannot change while you page through a fixed query state, and it
   * costs the server a `COUNT(*)` over the whole folder every time it is asked.
   */
  calculateTotal?: boolean;
}

/** The `Email/query` args for one page window, minus filter/sort/accountId. */
function windowArgs(limit: number, page: PageWindow): Record<string, unknown> {
  const args: Record<string, unknown> = {
    limit,
    calculateTotal: page.calculateTotal ?? true,
  };
  if (page.anchor !== undefined) {
    args['anchor'] = page.anchor;
    if (page.anchorOffset !== undefined) args['anchorOffset'] = page.anchorOffset;
  } else if (page.position !== undefined && page.position !== 0) {
    // Omit the default so an unpaged request is byte-identical to the pre-paging one.
    args['position'] = page.position;
  }
  return args;
}

/**
 * List a mailbox: Email/query for the newest ids, then Email/get header
 * properties for exactly those ids via a JMAP result reference (#ids from the
 * query), so the whole page is one round-trip.
 *
 * `limit` is the PAGE size, not the folder size: before t22 it was the only
 * bound and the app never sent anything but the default, so message 51 of a
 * folder was unreachable however far the list was scrolled. `page` names which
 * slice of the query this call wants.
 */
/**
 * The Events view (t78): not a stored folder, but an `inMailbox` id the server
 * expands to every message of the account with calendar content (an invitation,
 * reply, cancellation, counter-proposal or `.ics` file). Mirrors
 * `mw_engine::query::EVENTS_VIEW_ID`.
 */
export const EVENTS_VIEW_ID = 'mwViewEvents';

/** Extra conditions on a folder listing: the Events view's "upcoming only". */
export interface ListExtra {
  calendarEndAfter?: string;
  /** Leave out messages carrying this keyword (`mail.showSpam` off: `$junk`). */
  notKeyword?: string;
}

export function listMailbox(accountId: Id, mailboxId: Id, limit = 50, page: PageWindow = {}, extra: ListExtra = {}): JmapRequest {
  // The Events view's rows carry their event hint (read from the index, not the body).
  const properties: string[] = mailboxId === EVENTS_VIEW_ID ? [...listHeaderProperties(), 'mailwomanCalendar'] : listHeaderProperties();
  // Expiry marks in the list need the Expires headers; they are read only while `mail.expiry.hideExpired` is on.
  const expiry = settings()['mail.expiry.hideExpired']?.value;
  if (expiry === 'dim' || expiry === 'archive-suggestion' || expiry === 'hide') {
    for(const property of ['header:Expires:asText', 'header:Expiry-Date:asText'])if(!properties.includes(property))properties.push(property);
  }
  return request(MAIL_USING, [
    [
      'Email/query',
      {
        accountId,
        filter: { inMailbox: mailboxId, ...extra },
        sort: [{ property: 'receivedAt', isAscending: false }],
        ...windowArgs(limit, page),
      },
      'q',
    ],
    [
      'Email/get',
      {
        accountId,
        '#ids': { resultOf: 'q', name: 'Email/query', path: '/ids' },
        properties,
      },
      'g',
    ],
  ]);
}

/** The Events view's counts for its sidebar badge (t78): total and unread, no ids. */
export function eventsViewCounts(accountId: Id): JmapRequest {
  return request(MAIL_USING, [
    ['Email/query', { accountId, filter: { inMailbox: EVENTS_VIEW_ID }, limit: 0, calculateTotal: true }, 'all'],
    ['Email/query', { accountId, filter: { inMailbox: EVENTS_VIEW_ID, notKeyword: '$seen' }, limit: 0, calculateTotal: true }, 'unread'],
  ]);
}

/**
 * Search mail with the frozen `Email/query` filter set (§2.1). Any full-text /
 * attachment field routes engine-side to `mw-search`; the whole operator string
 * is carried in `filter.text` so the engine parses `from:`/`subject:`/… itself.
 * One round-trip: query for ids, then fetch header props for exactly those ids.
 * Pages exactly like `listMailbox`.
 */
export function searchEmails(
  accountId: Id,
  filter: FilterCondition,
  limit = 50,
  page: PageWindow = {},
): JmapRequest {
  return request(MAIL_USING, [
    [
      'Email/query',
      { accountId, filter, sort: [{ property: 'receivedAt', isAscending: false }], ...windowArgs(limit, page) },
      'q',
    ],
    [
      'Email/get',
      { accountId, '#ids': { resultOf: 'q', name: 'Email/query', path: '/ids' }, properties: listHeaderProperties() },
      'g',
    ],
  ]);
}

/**
 * The headers a mail server writes to say which address a message reached
 * (RFC 8621 §4.1.3 `header:<Name>:asAddresses`). Opening a message for a reply
 * reads them so the reply can be sent from the matching identity.
 */
export const DELIVERY_HEADER_PROPERTIES = [
  'header:Delivered-To:asAddresses:all',
  'header:X-Original-To:asAddresses:all',
  'header:Envelope-To:asAddresses:all',
  'header:X-Forwarded-To:asAddresses:all',
  'header:Resent-To:asAddresses:all',
] as const;

/** The optional message headers the reader shows as badges (`header:<Name>:asText`, answered by the engine). */
export const OPTIONAL_HEADER_PROPERTIES = [
  'header:Expires:asText',
  'header:Expiry-Date:asText',
  'header:Reply-By:asText',
  'header:Sensitivity:asText',
  'header:Supersedes:asText',
] as const;

/** Envelope metadata is ready with the list; no body values or attachment bytes. */
export function listHeaderProperties(): string[] {
  if(settings()['mail.cache.preloadHeaders']?.value===false)return [...HEADER_PROPERTIES];
  return [...HEADER_PROPERTIES,'cc','bcc','replyTo','sentAt','messageId','inReplyTo','references',...DELIVERY_HEADER_PROPERTIES,...OPTIONAL_HEADER_PROPERTIES];
}

export function emailGetFull(accountId: Id, id: Id, maxBodyValueBytes = 1_000_000): JmapRequest {
  return request(MAIL_USING, [
    [
      'Email/get',
      {
        accountId,
        ids: [id],
        properties: [...BODY_PROPERTIES, ...DELIVERY_HEADER_PROPERTIES, ...OPTIONAL_HEADER_PROPERTIES],
        fetchHTMLBodyValues: true,
        fetchTextBodyValues: true,
        maxBodyValueBytes,
      },
      'g',
    ],
  ]);
}

export * from './jmapCompose.ts';

// ── New-file blob upload (26.15 §1) ─────────────────────────────────────────
// The one piece of I/O in this module: POST a locally-picked file's bytes to the
// session `uploadUrl` for an account and read back the RFC 8620 §6.1 upload
// response. It lives beside the request builders (rather than on the JMAP
// `Client`) so the compose file picker drives it directly; the transport is
// injectable so it unit-tests without a live server, mirroring the Nextcloud
// module's `Fetcher` seam. The returned `blobId` is fed straight into the
// existing `Email/set` create `attachments` array (see `sendEnvelope`).

/** The JMAP upload response (RFC 8620 §6.1) for one stored blob. */
export interface UploadResponse {
  accountId: Id;
  blobId: Id;
  type: string;
  size: number;
}

/** Injectable transport for `uploadBlob` (defaults to a same-origin `fetch`). */
export type UploadFetcher = (input: string, init?: RequestInit) => Promise<Response>;

const defaultUploadFetcher: UploadFetcher = (input, init) =>
  apiFetch(input, { credentials: 'same-origin', ...init });

/**
 * Upload one file's bytes to the session `uploadUrl` for `accountId` and return
 * the stored blob's `{ accountId, blobId, type, size }`. `uploadUrl` is the raw
 * session template (`/jmap/upload/{accountId}`); its `{accountId}` placeholder is
 * substituted here. The request body is the file itself and its Content-Type is
 * the file's own type (or `application/octet-stream` when the browser reports
 * none). A non-2xx response throws with the status so the caller can surface a
 * concrete failure. Size limits are the caller's job (guard against the session's
 * `maxSizeUpload` BEFORE calling, to avoid a wasted round-trip).
 */
export async function uploadBlob(
  uploadUrl: string,
  accountId: Id,
  file: Blob & { name?: string },
  fetcher: UploadFetcher = defaultUploadFetcher,
): Promise<UploadResponse> {
  if(isOffline()||typeof navigator!=='undefined'&&navigator.onLine===false){
    const owner=preferenceAccount();if(!owner?.storageAuthority||owner.id!==accountId)throw new Error('Verified account is required for an offline attachment');
    const uploaded=await stageLocalFile({accountId,storageAuthority:owner.storageAuthority,base:offlineServerBase()},file);
    if(!preferenceAccountIsCurrent(owner))throw new DOMException('Attachment account changed','AbortError');return uploaded;
  }
  const url = uploadUrl.replace('{accountId}', encodeURIComponent(accountId));
  const contentType = file.type !== '' ? file.type : 'application/octet-stream';
  const res = await fetcher(url, {
    method: 'POST',
    headers: { 'content-type': contentType },
    body: file,
  });
  if (!res.ok) {
    throw new Error(`upload failed with ${res.status}`);
  }
  return (await res.json()) as UploadResponse;
}

// ── Threading headers of a stored message ───────────────────────────────────
// `Email/get` on this server returns no `messageId`, `inReplyTo` or
// `references`, so a reply reads them from the message itself: the header block
// of the raw RFC 5322 blob behind the session `downloadUrl`.

/** The ids a reply needs from the message it answers, without angle brackets. */
export interface ThreadingHeaders {
  /** The message's own `Message-ID`, or `null` when it has none. */
  messageId: string | null;
  /** Its `References`, oldest first; when that header is absent, its
   *  `In-Reply-To` if that names exactly one message (RFC 5322 §3.6.4). */
  references: string[];
}

/** Every `<id>` of a header value, without the brackets. Comments and anything
 *  else outside angle brackets are skipped. */
function messageIds(value: string | undefined): string[] {
  if (value === undefined) return [];
  return [...value.matchAll(/<([^<>\s]+)>/g)].map((m) => m[1]!);
}

/**
 * The reference chain of a message, given its `References` and `In-Reply-To`
 * ids: the former when there are any; otherwise the latter when it is a single
 * id. Several `In-Reply-To` ids say nothing about their order in the thread, so
 * they are not used.
 */
export function parentReferences(references: string[], inReplyTo: string[]): string[] {
  if (references.length > 0) return references;
  return inReplyTo.length === 1 ? inReplyTo : [];
}

/**
 * Read `Message-ID`, `References` and `In-Reply-To` out of a raw message (or
 * just its header block). Only the text before the first blank line is read.
 */
export function parseThreadingHeaders(raw: string): ThreadingHeaders {
  const end = /\r?\n\r?\n/.exec(raw);
  const block = (end === null ? raw : raw.slice(0, end.index)).replace(/\r?\n[ \t]+/g, ' ');
  const field = (name: string): string | undefined => {
    const m = new RegExp(`^${name}:(.*)$`, 'im').exec(block);
    return m === null ? undefined : m[1]!;
  };
  return {
    messageId: messageIds(field('Message-ID'))[0] ?? null,
    references: parentReferences(messageIds(field('References')), messageIds(field('In-Reply-To'))),
  };
}

/** The most header text `fetchThreadingHeaders` reads before giving up on
 *  finding the end of the header block. */
const MAX_HEADER_BYTES = 256 * 1024;

/**
 * Fetch the header block of the raw message at `url` (a substituted session
 * `downloadUrl` for the message's own blob) and parse its threading headers.
 * The body is not downloaded where the response can be read as a stream: the
 * read stops at the first blank line. A non-2xx response throws.
 */
export async function fetchThreadingHeaders(
  url: string,
  fetcher: UploadFetcher = defaultUploadFetcher,
): Promise<ThreadingHeaders> {
  // Ask for the head of the message only. A server without range support
  // answers 200 with the whole message; the read below stops either way.
  const res = await fetcher(url, { headers: { range: `bytes=0-${MAX_HEADER_BYTES - 1}` } });
  if (!res.ok) throw new Error(`message download failed with ${res.status}`);
  const reader = res.body?.getReader();
  if (reader === undefined) return parseThreadingHeaders(await res.text());
  // latin1 keeps one character per byte, so a multi-byte sequence split across
  // two chunks cannot be mangled; the ids read here are ASCII.
  const decoder = new TextDecoder('latin1');
  let head = '';
  for (;;) {
    const { done, value } = await reader.read();
    if (value !== undefined) head += decoder.decode(value, { stream: true });
    if (done) break;
    if (/\r?\n\r?\n/.test(head) || head.length > MAX_HEADER_BYTES) {
      await reader.cancel();
      break;
    }
  }
  return parseThreadingHeaders(head);
}

// ── V2 modern-mail operations (plan §1.5, §2.1) ─────────────────────────────
// All are single Email/set or EmailSubmission/set builders returning one
// request. Keyword changes round-trip to IMAP keywords; pinned/snoozedUntil/
// followUpAt are the engine-local Email props backed by `message_meta`.

/** Add (`on:true`) or remove (`on:false`) a keyword/label on one message. */
export function setEmailKeyword(accountId: Id, id: Id, keyword: string, on: boolean): JmapRequest {
  return request(MAIL_USING, [
    ['Email/set', { accountId, update: { [id]: { [`keywords/${keyword}`]: on ? true : null } } }, 'set'],
  ]);
}

/** Patch the engine-local Email meta props (pin / snooze / follow-up). */
export function setEmailMeta(
  accountId: Id,
  id: Id,
  patch: { pinned?: boolean; snoozedUntil?: string | null; followUpAt?: string | null },
): JmapRequest {
  return request(MAIL_USING, [['Email/set', { accountId, update: { [id]: { ...patch } } }, 'set']]);
}

/** Relocate a message to a new set of mailboxes (archive/move/spam/restore). */
export function moveEmail(accountId: Id, id: Id, mailboxIds: Record<Id, boolean>): JmapRequest {
  return request(MAIL_USING, [['Email/set', { accountId, update: { [id]: { mailboxIds } } }, 'set']]);
}

/** The call id of the `Mailbox/get` that {@link updateEmails} appends. */

/**
 * Patch several messages in ONE `Email/set` (t28-e16): a bulk action over a
 * selection is one request whatever its size, and the engine applies it as one
 * batch. `patches` maps each message id to its own patch (`keywords/<kw>`,
 * `mailboxIds`, …), so an undo can put each message back where it was.
 *
 * A `Mailbox/get` follows in the same request (call id {@link COUNTS_CALL_ID}).
 * The server runs method calls in order, so the folder counts it returns already
 * reflect the patch: that is how the sidebar's unread badges follow a mark-read
 * or a move without a second round trip.
 */


/** Permanently destroy a message (used by sweep "delete all"). */
export function destroyEmails(accountId: Id, ids: Id[], permanent = false): JmapRequest {
  return request(MAIL_USING, [['Email/set', { accountId, destroy: ids, ...(permanent ? { mailwomanPermanent: true } : {}) }, 'set']]);
}

/** Find every message from a sender (sweep preview + execute source). */
export function queryFromSender(accountId: Id, fromEmail: string, mailboxId?: Id): JmapRequest {
  const filter: Record<string, unknown> = { from: fromEmail };
  if (mailboxId !== undefined) filter['inMailbox'] = mailboxId;
  return request(MAIL_USING, [
    ['Email/query', { accountId, filter, sort: [{ property: 'receivedAt', isAscending: false }], calculateTotal: true }, 'q'],
    ['Email/get', { accountId, '#ids': { resultOf: 'q', name: 'Email/query', path: '/ids' }, properties: listHeaderProperties() }, 'g'],
  ]);
}

/** The sending identities (`Identity/get`) — configured + server-pulled froms. */
export function identityGet(accountId: Id): JmapRequest {
  return request(SUBMISSION_USING, [['Identity/get', { accountId, ids: null }, 'i']]);
}

/** The visible Outbox: `EmailSubmission/query` newest-first, then hydrate. */
export function outboxQuery(accountId: Id, limit = 100): JmapRequest {
  return request(SUBMISSION_USING, [
    ['EmailSubmission/query', { accountId, sort: [{ property: 'sendAt', isAscending: false }], limit }, 'q'],
    ['EmailSubmission/get', { accountId, '#ids': { resultOf: 'q', name: 'EmailSubmission/query', path: '/ids' } }, 'g'],
  ]);
}

/** Cancel a pending/scheduled submission before its window elapses (undo-send). */
export function cancelSubmission(accountId: Id, id: Id): JmapRequest {
  return request(SUBMISSION_USING, [
    ['EmailSubmission/set', { accountId, update: { [id]: { undoStatus: 'canceled' } } }, 'set'],
  ]);
}

/** Send a scheduled/held submission immediately (clear the delay). */
export function sendSubmissionNow(accountId: Id, id: Id): JmapRequest {
  return request(SUBMISSION_USING, [
    ['EmailSubmission/set', { accountId, update: { [id]: { sendAt: null, mailwomanHoldSeconds: 0 } } }, 'set'],
  ]);
}

/**
 * Move a scheduled submission to a new send time. `ifSendAt` is the time the
 * caller last saw: the engine refuses the change (`stateMismatch`) when the
 * row has moved on since, so two editors cannot overwrite each other.
 */
export function rescheduleSubmission(accountId: Id, id: Id, sendAt: string, ifSendAt: string | null): JmapRequest {
  const patch: Record<string, unknown> = { sendAt };
  if (ifSendAt !== null) patch['mailwomanIfSendAt'] = ifSendAt;
  return request(SUBMISSION_USING, [['EmailSubmission/set', { accountId, update: { [id]: patch } }, 'set']]);
}

/** Queue a message again for sending (a failed row's "Send again"). */
export function resendSubmission(accountId: Id, emailId: Id, identityId: Id | null): JmapRequest {
  const spec: Record<string, unknown> = { emailId };
  if (identityId !== null) spec['identityId'] = identityId;
  return request(SUBMISSION_USING, [['EmailSubmission/set', { accountId, create: { again: spec } }, 'set']]);
}

/** Extract the args of the method response matching `callId`, typed by caller. */
