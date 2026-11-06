// The composer's Reply-To row: when it shows, how it is checked, what the status
// bar says about it. Pure; Compose.tsx holds the state.

import { invalidRecipients, parseRecipients } from '../../api/jmap.ts';

export type ReplyToShow = 'auto' | 'always' | 'never';
export const isReplyToShow = (v: unknown): v is ReplyToShow => v === 'auto' || v === 'always' || v === 'never';

export interface ReplyToVisibility {
  show: ReplyToShow;
  /** The row holds an address (an identity default or a draft's value). */
  hasValue: boolean;
  /** The user opened it with the toggle. */
  toggled: boolean;
  /** The message answers one that had a Reply-To of its own. */
  repliesToReplyTo: boolean;
}

/** Whether the row itself is shown. The toggle button follows `toggleAvailable`. */
export function replyToRowVisible(v: ReplyToVisibility): boolean {
  if (v.show === 'never') return false;
  if (v.show === 'always') return true;
  return v.toggled || v.hasValue || v.repliesToReplyTo;
}

/** Whether the toggle button is offered: not when "never", and not when the row is already forced open. */
export function replyToToggleAvailable(show: ReplyToShow): boolean {
  return show === 'auto';
}

export const domainOfAddress = (email: string): string => email.slice(email.lastIndexOf('@') + 1).trim().toLowerCase();

export interface ReplyToCheck {
  invalid: string[];
  /** Domains of Reply-To addresses that differ from the From domain, once each. */
  external: string[];
}

export function checkReplyTo(text: string, fromEmail: string): ReplyToCheck {
  const invalid = invalidRecipients(text);
  const from = domainOfAddress(fromEmail);
  const external = new Set<string>();
  for (const r of parseRecipients(text)) {
    if (invalid.includes(r.email)) continue;
    const d = domainOfAddress(r.email);
    // A sub-domain of the From domain, or the other way round, is still the same organisation.
    if (from !== '' && d !== from && !d.endsWith(`.${from}`) && !from.endsWith(`.${d}`)) external.add(d);
  }
  return { invalid, external: [...external] };
}

/**
 * What goes into the send. `undefined` = leave the identity's own Reply-To (the
 * user did not touch the row, or the administrator forbids changes); a string,
 * possibly empty, is the user's choice ("" removes the header).
 */
export function replyToForSend(opts: { edited: boolean; text: string; allowOverride: boolean }): string | undefined {
  if (!opts.allowOverride || !opts.edited) return undefined;
  return opts.text.trim();
}
