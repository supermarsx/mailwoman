// Minimal hand-authored JMAP (RFC 8620/8621) types for the surface Mailwoman V0 uses.
// Mirrors what mw-jmap emits; intentionally partial.

export type Id = string;
export type UtcDate = string;

/** Standard JMAP capability URNs used by V0. */
export const CAP_CORE = 'urn:ietf:params:jmap:core';
export const CAP_MAIL = 'urn:ietf:params:jmap:mail';
export const CAP_SUBMISSION = 'urn:ietf:params:jmap:submission';

export interface Account {
  name: string;
  isPersonal: boolean;
  isReadOnly: boolean;
  accountCapabilities: Record<string, unknown>;
}

export interface JmapSession {
  capabilities: Record<string, unknown>;
  accounts: Record<Id, Account>;
  primaryAccounts: Record<string, Id>;
  username: string;
  apiUrl: string;
  downloadUrl: string;
  uploadUrl: string;
  eventSourceUrl: string;
  state: string;
}

export interface EmailAddress {
  name: string | null;
  email: string;
}

export interface EmailBodyPart {
  partId: string | null;
  blobId: string | null;
  size: number;
  type: string;
  charset?: string | null;
}

export interface EmailBodyValue {
  value: string;
  isEncodingProblem?: boolean;
  isTruncated?: boolean;
}

export interface Mailbox {
  id: Id;
  name: string;
  parentId: Id | null;
  role: string | null;
  sortOrder: number;
  totalEmails: number;
  unreadEmails: number;
  /** Set on a saved-search / search folder (V2 §2.1); absent on real mailboxes. */
  mailwomanSearchQuery?: string;
}

export interface Email {
  id: Id;
  /** The raw RFC 5322 message blob (for EML export / download). */
  blobId?: Id;
  mailboxIds: Record<Id, boolean>;
  from: EmailAddress[] | null;
  to: EmailAddress[] | null;
  subject: string | null;
  receivedAt: UtcDate;
  preview: string;
  htmlBody?: EmailBodyPart[];
  textBody?: EmailBodyPart[];
  bodyValues?: Record<string, EmailBodyValue>;
  // ── V2 additions (frozen §2.1). All optional so V1 fetches stay valid; e7
  // adds them to HEADER_PROPERTIES / BODY_PROPERTIES as it lands the UX. ──
  /** Labels + system keywords (`$seen/$flagged/$answered/$draft/…`). */
  keywords?: Record<string, boolean>;
  threadId?: Id;
  /** Engine-local metadata surfaced as Email props (plan §1.5). */
  pinned?: boolean;
  snoozedUntil?: UtcDate | null;
  followUpAt?: UtcDate | null;
  hasAttachment?: boolean;
  size?: number;
}

// ── Method-call argument / response shapes ──────────────────────────────────

export interface MailboxGetArgs {
  accountId: Id;
  ids: Id[] | null;
  properties?: string[] | null;
}
export interface MailboxGetResponse {
  accountId: Id;
  state: string;
  list: Mailbox[];
  notFound: Id[];
}

/**
 * The frozen `Email/query` filter set (§2.1). Any full-text/attachment field
 * (`text`/`from`/`to`/`cc`/`subject`/`body`/`filename`/`hasAttachment`) routes
 * to `mw-search` engine-side; a pure `inMailbox` filter stays the SQL fast path.
 */
export interface FilterCondition {
  inMailbox?: Id;
  inMailboxOtherThan?: Id[];
  text?: string;
  from?: string;
  to?: string;
  cc?: string;
  subject?: string;
  body?: string;
  hasKeyword?: string;
  notKeyword?: string;
  hasAttachment?: boolean;
  before?: UtcDate;
  after?: UtcDate;
  minSize?: number;
  maxSize?: number;
  /** Attachment filename substring. */
  filename?: string;
  /** V7 Assist (§14.3): opt-in semantic (embedding) re-ranking of the text query.
   *  Omitted for an ordinary keyword search; set only when the user enables the
   *  semantic-search toggle (gated on the `search-semantic` Assist capability). */
  semantic?: boolean;
}
/** Frozen `Email/query` sort properties (§2.1); default is `receivedAt` desc. */
export type SortProperty = 'receivedAt' | 'size' | 'from' | 'subject';
export interface Comparator {
  property: SortProperty | string;
  isAscending?: boolean;
}
export interface EmailQueryArgs {
  accountId: Id;
  filter?: FilterCondition;
  sort?: Comparator[];
  position?: number;
  limit?: number;
  calculateTotal?: boolean;
}
export interface EmailQueryResponse {
  accountId: Id;
  queryState: string;
  ids: Id[];
  position: number;
  total?: number;
}

export interface EmailGetArgs {
  accountId: Id;
  ids: Id[] | null;
  properties?: string[] | null;
  fetchHTMLBodyValues?: boolean;
  fetchTextBodyValues?: boolean;
  maxBodyValueBytes?: number;
}
export interface EmailGetResponse {
  accountId: Id;
  state: string;
  list: Email[];
  notFound: Id[];
}

export interface EmailCreate {
  mailboxIds: Record<Id, boolean>;
  from?: EmailAddress[];
  to?: EmailAddress[];
  subject?: string;
  htmlBody?: Array<{ partId: string; type: string }>;
  bodyValues?: Record<string, { value: string }>;
  keywords?: Record<string, boolean>;
}
export interface SetError {
  type: string;
  description?: string | null;
}
export interface EmailSetArgs {
  accountId: Id;
  create?: Record<string, EmailCreate>;
  update?: Record<Id, Record<string, unknown>>;
  destroy?: Id[];
}
export interface EmailSetResponse {
  accountId: Id;
  oldState: string | null;
  newState: string;
  created: Record<string, Partial<Email> & { id: Id }> | null;
  updated: Record<Id, unknown> | null;
  destroyed: Id[] | null;
  notCreated: Record<string, SetError> | null;
  notUpdated: Record<Id, SetError> | null;
  notDestroyed: Record<Id, SetError> | null;
}

export interface EmailSubmissionCreate {
  emailId: string;
  identityId?: string | null;
  envelope?: {
    mailFrom: { email: string };
    rcptTo: Array<{ email: string }>;
  } | null;
}
export interface EmailSubmissionSetArgs {
  accountId: Id;
  create?: Record<string, EmailSubmissionCreate>;
  onSuccessUpdateEmail?: Record<string, Record<string, unknown>> | null;
}
export interface EmailSubmissionSetResponse {
  accountId: Id;
  created: Record<string, { id: Id }> | null;
  notCreated: Record<string, SetError> | null;
}

// ── V2: the real, persisted EmailSubmission (§2.1) — undo-send / send-later /
//    the visible Outbox (`EmailSubmission/query`). ──
export type UndoStatus = 'pending' | 'final' | 'canceled';
export interface EmailSubmission {
  id: Id;
  emailId: Id;
  identityId: Id | null;
  /** Scheduled send time for send-later; `null` = fire after the hold window. */
  sendAt: UtcDate | null;
  undoStatus: UndoStatus;
  /** Engine-held delay before SMTP dispatch (the undo-send window), in seconds. */
  mailwomanHoldSeconds: number;
}
export interface EmailSubmissionGetResponse {
  accountId: Id;
  state: string;
  list: EmailSubmission[];
  notFound: Id[];
}
export interface EmailSubmissionQueryResponse {
  accountId: Id;
  queryState: string;
  ids: Id[];
  position: number;
  total?: number;
}

// ── V2: sending identities (§2.1) — multiple from-addresses + signatures. ──
export interface Identity {
  id: Id;
  name: string;
  email: string;
  replyTo: string | null;
  signatureHtml: string | null;
  signatureText: string | null;
  sentMailboxId: Id | null;
}
export interface IdentityGetResponse {
  accountId: Id;
  state: string;
  list: Identity[];
  notFound: Id[];
}

// ── V2: real state + changes (§2.1). Shape shared by Email/changes,
//    Mailbox/changes, EmailSubmission/changes. ──
export interface ChangesResponse {
  accountId: Id;
  oldState: string;
  newState: string;
  created: Id[];
  updated: Id[];
  destroyed: Id[];
  hasMoreChanges: boolean;
}
export interface QueryChangesResponse {
  accountId: Id;
  oldQueryState: string;
  newQueryState: string;
  removed: Id[];
  added: Array<{ id: Id; index: number }>;
}

// ── Request / Response envelope ─────────────────────────────────────────────

export type MethodName =
  | 'Mailbox/get'
  | 'Mailbox/changes'
  | 'Email/query'
  | 'Email/queryChanges'
  | 'Email/get'
  | 'Email/set'
  | 'Email/changes'
  | 'EmailSubmission/set'
  | 'EmailSubmission/get'
  | 'EmailSubmission/query'
  | 'EmailSubmission/changes'
  | 'Identity/get'
  | 'Identity/query';

export type Invocation = [name: string, args: Record<string, unknown>, callId: string];

export interface JmapRequest {
  using: string[];
  methodCalls: Invocation[];
  createdIds?: Record<string, Id>;
}

export interface JmapResponse {
  methodResponses: Invocation[];
  sessionState: string;
  createdIds?: Record<string, Id>;
}
