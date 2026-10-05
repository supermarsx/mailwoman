// Reply / Reply all / Forward: who the new message goes to, what its subject
// is, and which message ids it carries. Pure functions over the open `Email` —
// no DOM and no ProseMirror, so the reader can import this on the entry chunk.
// The quoted body is built separately, in `quote.ts`.

import type { Email, EmailAddress } from '../../api/jmap-types.ts';
import type { ThreadingHeaders } from '../../api/jmap.ts';

export type ReplyMode = 'reply' | 'reply-all' | 'forward';

/** What a composer opened from a message starts with. */
export interface ComposeInitial {
  mode: ReplyMode;
  /** Recipient fields in the composer's own syntax (`Name <addr>, …`). */
  to: string;
  cc: string;
  subject: string;
  /** The quoted original as HTML already reduced to the compose schema, and
   *  the same content as plain text for the plain-text body. */
  bodyHtml: string;
  bodyText: string;
  /** Message ids without angle brackets; empty for a forward, and for a reply
   *  to a message whose `Message-ID` could not be read. */
  inReplyTo: string[];
  references: string[];
  /** The original's attachments, carried by blob id (forward only). */
  attachments: { name: string; blobId: string; size: number; contentType: string | null }[];
  /** True when the quoted text is the decrypted content of an encrypted
   *  message. The composer then asks before sending it unencrypted, and does
   *  not auto-save it to local storage. */
  quotesDecrypted: boolean;
}

/** The addresses that are the user's own, for leaving them out of a reply-all. */
export interface OwnAddresses {
  /** Identity addresses and the account's login name, as the server gave them. */
  addresses: string[];
}

const REPLY_PREFIX = /^\s*re\s*:\s*/i;
const FORWARD_PREFIX = /^\s*(?:fwd?|fw)\s*:\s*/i;

/** Remove every leading occurrence of `prefix` from `subject`. */
function stripLeading(subject: string, prefix: RegExp): string {
  let rest = subject;
  while (prefix.test(rest)) rest = rest.replace(prefix, '');
  return rest.trim();
}

/** `Re: <subject>`, with exactly one `Re:` however many the original had. */
export function replySubject(subject: string | null): string {
  return `Re: ${stripLeading(subject ?? '', REPLY_PREFIX)}`.trimEnd();
}

/** `Fwd: <subject>`, with exactly one `Fwd:` however many the original had. */
export function forwardSubject(subject: string | null): string {
  return `Fwd: ${stripLeading(subject ?? '', FORWARD_PREFIX)}`.trimEnd();
}

/**
 * Whether `email` is one of the user's own addresses. An own entry with an `@`
 * matches that address, case-insensitively. An own entry WITHOUT one is a bare
 * login name (an engine account's identity can be just that); it matches an
 * address whose local part is that name, since the server does not say which
 * domain the account receives at.
 */
export function isOwnAddress(email: string, own: OwnAddresses): boolean {
  const addr = email.trim().toLowerCase();
  const local = addr.split('@')[0] ?? '';
  return own.addresses.some((o) => {
    const mine = o.trim().toLowerCase();
    if (mine === '') return false;
    return mine.includes('@') ? mine === addr : mine === local;
  });
}

/** Drop repeated addresses (case-insensitive), keeping the first of each. */
function unique(list: EmailAddress[], seen = new Set<string>()): EmailAddress[] {
  const out: EmailAddress[] = [];
  for (const a of list) {
    const key = a.email.trim().toLowerCase();
    if (key === '' || seen.has(key)) continue;
    seen.add(key);
    out.push(a);
  }
  return out;
}

/**
 * The To and Cc of a reply.
 *
 * - `reply`: the original's `Reply-To` when it has one, otherwise its `From`.
 * - `reply-all`: the same in To, followed by the original's other To
 *   recipients; its Cc recipients in Cc. The user's own addresses are left out
 *   of both, and no address appears twice.
 * - A message the user sent themselves (its `From` is one of their addresses)
 *   is answered to the people it was sent to, not to the user.
 */
export function replyRecipients(
  email: Email,
  mode: 'reply' | 'reply-all',
  own: OwnAddresses,
): { to: EmailAddress[]; cc: EmailAddress[] } {
  const from = email.from ?? [];
  const origTo = email.to ?? [];
  const origCc = email.cc ?? [];
  const notOwn = (a: EmailAddress): boolean => !isOwnAddress(a.email, own);
  const sentByUser = from.length > 0 && from.every((a) => !notOwn(a));

  if (sentByUser) {
    const seen = new Set<string>();
    const to = unique(origTo, seen);
    return { to, cc: mode === 'reply-all' ? unique(origCc, seen) : [] };
  }

  const replyTo = email.replyTo ?? [];
  const author = replyTo.length > 0 ? replyTo : from;
  if (mode === 'reply') return { to: unique(author), cc: [] };

  const seen = new Set<string>();
  const to = unique([...author, ...origTo.filter(notOwn)], seen);
  const cc = unique(origCc.filter(notOwn), seen);
  return { to, cc };
}

/** One address in the composer's field syntax. A display name holding a
 *  character the field parser treats as structure is double-quoted; angle
 *  brackets are dropped from it, because the parser reads `<`…`>` as the
 *  address even inside quotes. */
export function formatAddress(a: EmailAddress): string {
  const name = (a.name ?? '').replace(/[<>]/g, '').trim();
  if (name === '') return a.email;
  const shown = /[,;<>"\\()]/.test(name) ? `"${name.replace(/(["\\])/g, '\\$1')}"` : name;
  return `${shown} <${a.email}>`;
}

/** A recipient list in the composer's field syntax. */
export function formatAddressList(list: EmailAddress[]): string {
  return list.map(formatAddress).join(', ');
}

/** The `inReplyTo` and `references` a reply carries: the original's
 *  `Message-ID`, and its `References` followed by that id (RFC 5322 §3.6.4).
 *  Both are empty when the original's id is not known. */
export function replyThreading(original: ThreadingHeaders | null): { inReplyTo: string[]; references: string[] } {
  const id = original?.messageId ?? null;
  if (original === null || id === null) return { inReplyTo: [], references: [] };
  return { inReplyTo: [id], references: [...original.references.filter((r) => r !== id), id] };
}
