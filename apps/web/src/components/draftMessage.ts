import type {Email, Mailbox} from '../api/jmap-types.ts';

/** Whether a message is a draft: it carries `$draft`, or sits only in a Drafts-role folder. */
export function isDraftMessage(email: Pick<Email, 'keywords' | 'mailboxIds'>, mailboxes: readonly Pick<Mailbox, 'id' | 'role'>[]): boolean {
  if (email.keywords?.['$draft'] === true) return true;
  const ids = Object.keys(email.mailboxIds ?? {});
  return ids.length > 0 && ids.every((id) => mailboxes.find((box) => box.id === id)?.role === 'drafts');
}
