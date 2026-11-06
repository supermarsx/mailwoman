// Linked-mailbox sessions: how the account they come from is named and found.
import type { SessionMeta } from './types.ts';

/** The strings a linked row shows: origin, the mailbox it opens, or neither when hidden. */
export interface LinkedOriginView {
  /** Origin address as the server sent it (already masked per setting); undefined when hidden or removed. */
  readonly origin?: string;
  readonly removed: boolean;
  /** Target mailbox, only when it differs from the origin. */
  readonly target?: string;
  /** Domain to look a brand up by (origin's, else target's). */
  readonly brandHost?: string;
}

export function isLinkedSession(row: SessionMeta): boolean {
  return row.linkKind === 'linked';
}

export function linkedOriginView(row: SessionMeta): LinkedOriginView | undefined {
  if (!isLinkedSession(row)) return undefined;
  const removed = row.originRemoved === true;
  const origin = removed ? undefined : row.originEmail;
  const target = row.targetEmail && row.targetEmail !== origin ? row.targetEmail : undefined;
  const host = (origin ?? row.targetEmail)?.split('@')[1];
  return { removed, ...(origin ? { origin } : {}), ...(target ? { target } : {}), ...(host ? { brandHost: host } : {}) };
}

/** Every string the search box should match for a row, origin and target included. */
export function sessionSearchValues(row: SessionMeta): (string | undefined)[] {
  return [row.username, row.ip, row.deviceLabel, row.userAgent, row.originEmail, row.targetEmail];
}
