// Pure JMAP request builders + response extraction. No I/O here so these are
// trivially unit-testable (see jmap.test.ts).

import {
  CAP_CORE,
  CAP_MAIL,
  CAP_SUBMISSION,
  type EmailAddress,
  type Id,
  type Invocation,
  type JmapRequest,
  type JmapResponse,
} from './jmap-types.ts';

export const HEADER_PROPERTIES = [
  'id',
  'from',
  'to',
  'subject',
  'receivedAt',
  'preview',
] as const;

export const BODY_PROPERTIES = [
  'id',
  'from',
  'to',
  'subject',
  'receivedAt',
  'preview',
  'htmlBody',
  'textBody',
  'bodyValues',
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

  const submissionSet: Invocation = [
    'EmailSubmission/set',
    {
      accountId,
      create: {
        send: {
          emailId: '#draft',
          envelope: {
            mailFrom: { email: input.from.email },
            rcptTo: parseRecipients(input.to).map((r) => ({ email: r.email })),
          },
        },
      },
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
