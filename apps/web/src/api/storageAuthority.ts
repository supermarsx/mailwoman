import { preferenceAccount } from './accountPreferenceLifecycle.ts';
import { activeLinkedMailbox, primaryMailboxAuthority } from './linkedMailboxSession.ts';

/** Bind a private-settings request to the verified account captured by the UI. */
export function storageAuthorityHeaders(expected?: string): Record<string, string> {
  const authority = preferenceAccount()?.storageAuthority;
  if (!authority || (expected !== undefined && expected !== authority)) throw new Error('The signed-in account changed. Reload these settings.');
  return {'x-mailwoman-storage-authority': authority};
}

/** For a request that belongs to the signed-in person rather than to the open
 * mailbox (t87 D4, analytics consent): routed as `primary`, so it carries the
 * sign-in account's authority even while a linked mailbox is open. */
export function personStorageAuthorityHeaders(): Record<string, string> {
  if (!activeLinkedMailbox()) return storageAuthorityHeaders();
  const authority = primaryMailboxAuthority();
  if (!authority) throw new Error('The signed-in account changed. Reload these settings.');
  return {'x-mailwoman-storage-authority': authority};
}
