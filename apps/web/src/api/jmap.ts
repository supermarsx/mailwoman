// Pure JMAP request builders + response extraction. No I/O here so these are
// trivially unit-testable (see jmap.test.ts).

import {
  CAP_CORE,
  CAP_MAIL,
  CAP_SUBMISSION,
  type EmailAddress,
  type FilterCondition,
  type Id,
  type Invocation,
  type JmapRequest,
  type JmapResponse,
} from './jmap-types.ts';

// V2 (§2.1): the list view now fetches the modern-mail props so tags, pins,
// snooze, follow-up and the attachment indicator render straight off Email/get.
export const HEADER_PROPERTIES = [
  'id',
  'from',
  'to',
  'subject',
  'receivedAt',
  'preview',
  'keywords',
  'threadId',
  'pinned',
  'snoozedUntil',
  'followUpAt',
  'hasAttachment',
  'size',
] as const;

export const BODY_PROPERTIES = [
  'id',
  'blobId',
  'from',
  'to',
  'subject',
  'receivedAt',
  'preview',
  'htmlBody',
  'textBody',
  'bodyValues',
  'attachments',
  'keywords',
  'threadId',
  'pinned',
  'snoozedUntil',
  'followUpAt',
  'hasAttachment',
  'size',
] as const;

/** Default cap so an upstream can send more; enough for V0 flows. */
const MAIL_USING = [CAP_CORE, CAP_MAIL];
const SUBMISSION_USING = [CAP_CORE, CAP_MAIL, CAP_SUBMISSION];

export function request(using: string[], methodCalls: Invocation[]): JmapRequest {
  return { using, methodCalls };
}

export function mailboxGet(accountId: Id, callId = 'c0'): JmapRequest {
  return request(MAIL_USING, [['Mailbox/get', { accountId, ids: null }, callId]]);
}

/**
 * List a mailbox: Email/query for the newest ids, then Email/get header
 * properties for exactly those ids via a JMAP result reference (#ids from the
 * query), so the whole page is one round-trip.
 */
export function listMailbox(accountId: Id, mailboxId: Id, limit = 50): JmapRequest {
  return request(MAIL_USING, [
    [
      'Email/query',
      {
        accountId,
        filter: { inMailbox: mailboxId },
        sort: [{ property: 'receivedAt', isAscending: false }],
        limit,
        calculateTotal: true,
      },
      'q',
    ],
    [
      'Email/get',
      {
        accountId,
        '#ids': { resultOf: 'q', name: 'Email/query', path: '/ids' },
        properties: [...HEADER_PROPERTIES],
      },
      'g',
    ],
  ]);
}

/**
 * Search mail with the frozen `Email/query` filter set (§2.1). Any full-text /
 * attachment field routes engine-side to `mw-search`; the whole operator string
 * is carried in `filter.text` so the engine parses `from:`/`subject:`/… itself.
 * One round-trip: query for ids, then fetch header props for exactly those ids.
 */
export function searchEmails(accountId: Id, filter: FilterCondition, limit = 50): JmapRequest {
  return request(MAIL_USING, [
    [
      'Email/query',
      { accountId, filter, sort: [{ property: 'receivedAt', isAscending: false }], limit, calculateTotal: true },
      'q',
    ],
    [
      'Email/get',
      { accountId, '#ids': { resultOf: 'q', name: 'Email/query', path: '/ids' }, properties: [...HEADER_PROPERTIES] },
      'g',
    ],
  ]);
}

export function emailGetFull(accountId: Id, id: Id, maxBodyValueBytes = 1_000_000): JmapRequest {
  return request(MAIL_USING, [
    [
      'Email/get',
      {
        accountId,
        ids: [id],
        properties: [...BODY_PROPERTIES],
        fetchHTMLBodyValues: true,
        fetchTextBodyValues: true,
        maxBodyValueBytes,
      },
      'g',
    ],
  ]);
}

export interface DraftInput {
  from: EmailAddress;
  to: string;
  subject: string;
  htmlBody: string;
  draftMailboxId: Id;
  sentMailboxId?: Id;
  /** V2: send via a specific sending identity (§2.1 `Identity`). */
  identityId?: Id;
  /** V2 send-later: scheduled send time (ISO 8601 UTC); omitted = send now. */
  sendAt?: string;
  /**
   * V2 undo-send: engine-held delay before SMTP dispatch, in seconds. The
   * client shows a Cancel toast for this window; the engine only dials SMTP
   * after it elapses (plan §1.3).
   */
  holdSeconds?: number;
}

/**
 * Compose + send in ONE request using creation-id back-references:
 * Email/set creates the draft under key `draft`; EmailSubmission/set references
 * it as `#draft`; on success the email is moved out of Drafts into Sent.
 */
export function sendEnvelope(accountId: Id, input: DraftInput): JmapRequest {
  const mailboxIds: Record<Id, boolean> = { [input.draftMailboxId]: true };
  const emailSet: Invocation = [
    'Email/set',
    {
      accountId,
      create: {
        draft: {
          mailboxIds,
          keywords: { $draft: true, $seen: true },
          from: [input.from],
          to: parseRecipients(input.to),
          subject: input.subject,
          htmlBody: [{ partId: 'body', type: 'text/html' }],
          bodyValues: { body: { value: input.htmlBody } },
        },
      },
    },
    'set',
  ];

  const onSuccessUpdate: Record<string, Record<string, unknown>> = {};
  if (input.sentMailboxId !== undefined) {
    onSuccessUpdate['#send'] = {
      mailboxIds: { [input.sentMailboxId]: true },
      'keywords/$draft': null,
    };
  }

  // V2 (§2.1): the submission is a persisted, engine-held row. `sendAt` /
  // `mailwomanHoldSeconds` ride the create so the engine can enqueue instead of
  // dialing SMTP synchronously; both are extra fields on the frozen submission
  // object, so no seam change is needed.
  const sendCreate: Record<string, unknown> = {
    emailId: '#draft',
    envelope: {
      mailFrom: { email: input.from.email },
      rcptTo: parseRecipients(input.to).map((r) => ({ email: r.email })),
    },
  };
  if (input.identityId !== undefined) sendCreate['identityId'] = input.identityId;
  if (input.sendAt !== undefined) sendCreate['sendAt'] = input.sendAt;
  if (input.holdSeconds !== undefined) sendCreate['mailwomanHoldSeconds'] = input.holdSeconds;

  const submissionSet: Invocation = [
    'EmailSubmission/set',
    {
      accountId,
      create: { send: sendCreate },
      ...(input.sentMailboxId !== undefined ? { onSuccessUpdateEmail: onSuccessUpdate } : {}),
    },
    'submit',
  ];

  return request(SUBMISSION_USING, [emailSet, submissionSet]);
}

export function parseRecipients(raw: string): EmailAddress[] {
  return raw
    .split(/[,;]/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0)
    .map((email) => ({ name: null, email }));
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

/** Permanently destroy a message (used by sweep "delete all"). */
export function destroyEmails(accountId: Id, ids: Id[]): JmapRequest {
  return request(MAIL_USING, [['Email/set', { accountId, destroy: ids }, 'set']]);
}

/** Find every message from a sender (sweep preview + execute source). */
export function queryFromSender(accountId: Id, fromEmail: string, mailboxId?: Id): JmapRequest {
  const filter: Record<string, unknown> = { from: fromEmail };
  if (mailboxId !== undefined) filter['inMailbox'] = mailboxId;
  return request(MAIL_USING, [
    ['Email/query', { accountId, filter, sort: [{ property: 'receivedAt', isAscending: false }], calculateTotal: true }, 'q'],
    ['Email/get', { accountId, '#ids': { resultOf: 'q', name: 'Email/query', path: '/ids' }, properties: [...HEADER_PROPERTIES] }, 'g'],
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

/** Extract the args of the method response matching `callId`, typed by caller. */
export function responseFor<T>(res: JmapResponse, callId: string): T {
  const found = res.methodResponses.find((m) => m[2] === callId);
  if (found === undefined) {
    throw new Error(`no method response for call "${callId}"`);
  }
  if (found[0] === 'error') {
    const err = found[1] as { type?: string; description?: string };
    throw new Error(`JMAP method error: ${err.type ?? 'unknown'} ${err.description ?? ''}`.trim());
  }
  return found[1] as T;
}
