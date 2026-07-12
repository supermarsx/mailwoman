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
}

export interface Email {
  id: Id;
  mailboxIds: Record<Id, boolean>;
  from: EmailAddress[] | null;
  to: EmailAddress[] | null;
  subject: string | null;
  receivedAt: UtcDate;
  preview: string;
  htmlBody?: EmailBodyPart[];
  textBody?: EmailBodyPart[];
  bodyValues?: Record<string, EmailBodyValue>;
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

export interface FilterCondition {
  inMailbox?: Id;
  inMailboxOtherThan?: Id[];
}
export interface Comparator {
  property: string;
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

// ── Request / Response envelope ─────────────────────────────────────────────

export type MethodName =
  | 'Mailbox/get'
  | 'Email/query'
  | 'Email/get'
  | 'Email/set'
  | 'EmailSubmission/set';

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
