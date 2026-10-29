// Counter bubbles (t39): which folders show a count, what the count shows, and
// whether the mailbox name in the sidebar head carries one.
//
//   mail.folderCounts       none | unread | unreadIfAny | total | both
//                           `unread` shows the unread count, 0 included; `unreadIfAny`
//                           shows it only while there is unread mail.
//   mail.folderCountScope   all | main   (`main`: folders with a role — Inbox, Drafts,
//                           Sent, Archive, Junk, Trash, Outbox and any other
//                           special-use role the server reports)
//   mail.accountUnreadBadge the bubble beside the mailbox name; off by default. The
//                           per-mailbox counts inside the picker menu always show.
//
// Each is a user setting with an administrator default and lock, and the server
// resolves them for the sign-in account, so every linked mailbox shows the same.

import { settings } from './settings.ts';

export type FolderCountMode = 'none' | 'unread' | 'unreadIfAny' | 'total' | 'both';
export type FolderCountScope = 'all' | 'main';
export const FOLDER_COUNT_MODES: readonly FolderCountMode[] = ['none', 'unread', 'unreadIfAny', 'total', 'both'];
export const FOLDER_COUNT_SCOPES: readonly FolderCountScope[] = ['all', 'main'];

export function folderCountMode(): FolderCountMode {
  const value = settings()['mail.folderCounts']?.value;
  return FOLDER_COUNT_MODES.includes(value as FolderCountMode) ? (value as FolderCountMode) : 'both';
}

export function folderCountScope(): FolderCountScope {
  return settings()['mail.folderCountScope']?.value === 'main' ? 'main' : 'all';
}

/** The bubble beside the mailbox name in the sidebar head (default off). */
export function accountUnreadBadge(): boolean {
  return settings()['mail.accountUnreadBadge']?.value === true;
}

/** A main folder is one with a role; custom folders have none. */
export function isMainFolder(folder: { role?: string | null }): boolean {
  return typeof folder.role === 'string' && folder.role !== '';
}

/** Whether a folder shows a count bubble at all under `mode` and `scope`. */
export function showsFolderCount(
  folder: { role?: string | null; unreadEmails: number; totalEmails: number },
  mode: FolderCountMode,
  scope: FolderCountScope,
): boolean {
  if (mode === 'none' || !Number.isFinite(folder.totalEmails)) return false;
  if (scope === 'main' && !isMainFolder(folder)) return false;
  if (mode === 'unreadIfAny') return folder.unreadEmails > 0;
  return true;
}

/** What a count bubble reads, or null for none (the Focused/Other tabs). */
export function listCount(mode: FolderCountMode, total: number, unread: number): number | null {
  if (mode === 'none') return null;
  if (mode === 'unread') return unread;
  if (mode === 'unreadIfAny') return unread > 0 ? unread : null;
  return total;
}
