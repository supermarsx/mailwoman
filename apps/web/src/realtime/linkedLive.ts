// Linked mailbox live updates (t46). The server multiplexes every verified
// linked mailbox onto the sign-in session's own realtime stream as
// `MailboxChange` frames (contracts/push.ts), tagged with the link id and the
// mailbox's opaque storage authority. This module routes them:
//
// * a frame for the linked mailbox that is open now, under the authority this
//   tab verified when it opened it, refetches the datatypes that moved, as a
//   `StateChange` does for the sign-in mailbox;
// * every frame records that mailbox's Inbox unread count, so the mailbox
//   picker shows live counts for the mailboxes that are not open.
//
// A count is only read back under the authority the caller holds for that
// mailbox, so a frame naming a different authority for a link (a re-linked or
// re-pointed mailbox) never feeds a badge, and the sign-in mailbox's frames are
// accepted only under the sign-in authority.

import { createSignal } from 'solid-js';
import type { MailboxChange, TypeStates } from '../contracts/push.ts';
import type { ChangedType } from './changes.ts';

const TYPES: readonly ChangedType[] = ['Email', 'Mailbox', 'EmailSubmission', 'Thread'];

/** The linked mailbox open now, as this tab verified it. */
export interface ActiveLinkedMailbox {
  linkId: string;
  storageAuthority: string;
}

export interface LinkedLiveOptions {
  /** The linked mailbox open now; null while the sign-in mailbox is open. */
  active(): ActiveLinkedMailbox | null;
  /** The sign-in mailbox's storage authority, once known. */
  primaryAuthority(): string | undefined;
  /** The open linked mailbox moved: refetch these datatypes. */
  onActiveChanged(types: ChangedType[]): void;
  /** A frame for the open link named an authority this tab does not hold. */
  onAuthorityMismatch?(): void;
}

export interface LinkedLive {
  /** Route one decoded frame. */
  apply(frame: MailboxChange): void;
  /**
   * The live Inbox unread count of a mailbox (`null` = the sign-in mailbox, a
   * string = a link id) under the authority the caller holds for it. `null`
   * when not known yet, or when the last frame named another authority.
   */
  unread(linkId: string | null, storageAuthority?: string): number | null;
  /** Forget everything (sign-out, stream teardown). */
  reset(): void;
}

interface Count {
  authority: string;
  unread: number;
}

export function createLinkedLive(opts: LinkedLiveOptions): LinkedLive {
  const [counts, setCounts] = createSignal<Readonly<Record<string, Count>>>({});
  // Last state tokens acted on, per mailbox and authority.
  const known = new Map<string, TypeStates>();

  return {
    apply(frame): void {
      if (frame.link === null && frame.storageAuthority !== opts.primaryAuthority()) return;
      const key = frame.link ?? '';
      if (frame.inboxUnread !== undefined) {
        const unread = frame.inboxUnread;
        setCounts((c) => ({ ...c, [key]: { authority: frame.storageAuthority, unread } }));
      }
      const stateKey = `${key}\n${frame.storageAuthority}`;
      const prev = known.get(stateKey);
      const moved = TYPES.filter(
        (t) => frame.changed[t] !== undefined && (prev === undefined || prev[t] !== frame.changed[t]),
      );
      if (Object.keys(frame.changed).length > 0) known.set(stateKey, { ...prev, ...frame.changed });
      const active = opts.active();
      if (frame.link === null || active === null || active.linkId !== frame.link) return;
      if (active.storageAuthority !== frame.storageAuthority) {
        opts.onAuthorityMismatch?.();
        return;
      }
      if (moved.length > 0) opts.onActiveChanged(moved);
    },
    unread(linkId, storageAuthority): number | null {
      const entry = counts()[linkId ?? ''];
      if (entry === undefined) return null;
      const expected = linkId === null ? opts.primaryAuthority() : storageAuthority;
      return expected !== undefined && expected === entry.authority ? entry.unread : null;
    },
    reset(): void {
      setCounts({});
      known.clear();
    },
  };
}
