// Linked-mailbox rows in the administrator's session listing. The server decides
// what an administrator may see (users.read for the origin's domain, and the
// owner's privacy.sessions.showLinkedOrigin); this only shapes what it sent.
import { hrefFor, sectionPath } from './shell/router.ts';

export interface AdminLinkFields {
  linkKind?: string;
  originEmail?: string;
  originAccountId?: string;
  originDeviceLabel?: string;
  originRemoved?: boolean;
  originRestricted?: boolean;
  targetEmail?: string;
  linkCreatedAt?: number;
}

export interface AdminOriginView {
  /** Origin address as the server sent it (already masked per setting); undefined when hidden, restricted or removed. */
  readonly origin?: string;
  readonly removed: boolean;
  readonly restricted: boolean;
  /** Device label of the origin's own session (never its address or location). */
  readonly deviceLabel?: string;
  /** Href of the origin account's admin page, only when the server sent the origin account id. */
  readonly accountHref?: string;
}

export function adminOriginView(row: AdminLinkFields): AdminOriginView | undefined {
  if (row.linkKind !== 'linked') return undefined;
  const removed = row.originRemoved === true;
  return {
    removed,
    restricted: row.originRestricted === true,
    ...(!removed && row.originEmail ? { origin: row.originEmail } : {}),
    ...(!removed && row.originDeviceLabel ? { deviceLabel: row.originDeviceLabel } : {}),
    ...(!removed && row.originAccountId ? { accountHref: hrefFor(sectionPath('users', [row.originAccountId])) } : {}),
  };
}
