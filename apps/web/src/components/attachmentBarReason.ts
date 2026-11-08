/**
 * Whether the reader's attachments bar shows for a message, and, when the
 * message looks like it has attachments but the bar is missing or cannot be used
 * as usual, the single reason to tell the user (one toast per message).
 * Pure: the caller owns the toast and the actions.
 */

export type AttachmentBarReasonKind =
  | 'encrypted'
  | 'list-missing'
  | 'inline-only'
  | 'unavailable-parts'
  | 'policy-blocked'
  | 'policy-unavailable';

export interface AttachmentBarInput {
  /** The message's `hasAttachment` flag, when the server sent it. */
  hasAttachment: boolean | undefined;
  /** The `attachments` array as received, or undefined when the property was absent. */
  attachments: readonly { blobId?: string | null }[] | undefined;
  /** The message is encrypted and its content is not unlocked yet. */
  encrypted?: boolean;
  /** Parts known to be shown inside the body (inline images / cid resources). */
  inlineCount?: number;
  /** The attachment policy resource. */
  policy: 'idle' | 'loading' | 'ready' | 'error';
  /** Number of listed items that the policy refuses to download. */
  blockedCount?: number;
}

export interface AttachmentBarResult {
  /** The bar is rendered. */
  show: boolean;
  reason: AttachmentBarReasonKind | null;
  /** Parts counted by the reason (unavailable or blocked). */
  count: number;
}

export function attachmentBarReason(input: AttachmentBarInput): AttachmentBarResult {
  const list = input.attachments ?? [];
  const usable = list.filter((a) => a.blobId !== null && a.blobId !== undefined && a.blobId !== '').length;
  const unavailable = list.length - usable;
  if (usable > 0) {
    if (input.policy === 'error') return { show: true, reason: 'policy-unavailable', count: usable };
    const blocked = input.blockedCount ?? 0;
    if (input.policy === 'ready' && blocked >= usable) return { show: true, reason: 'policy-blocked', count: blocked };
    return { show: true, reason: null, count: 0 };
  }
  if (input.hasAttachment !== true && list.length === 0) {
    if ((input.inlineCount ?? 0) > 0) return { show: false, reason: 'inline-only', count: input.inlineCount ?? 0 };
    return { show: false, reason: null, count: 0 };
  }
  if (input.encrypted === true) return { show: false, reason: 'encrypted', count: 0 };
  if (unavailable > 0) return { show: false, reason: 'unavailable-parts', count: unavailable };
  if ((input.inlineCount ?? 0) > 0) return { show: false, reason: 'inline-only', count: input.inlineCount ?? 0 };
  return { show: false, reason: 'list-missing', count: 0 };
}

export {isInlineResourcePart} from './attachmentMetadata.ts';
