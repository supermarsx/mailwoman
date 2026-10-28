// The owner's mailbox arrangement (t40 e/f/g): which mailbox opens first, the
// order of the sign-in account and its links, and display-only nicknames. All
// three come from `GET /api/accounts` and are kept server-side per user.
//
// A nickname is a label for this person's own screens. It is never an identity:
// not a sender name, not a From header, and never sent to the mail server.
import { createSignal } from 'solid-js';
import type { LinkedMailboxList } from './linkedAccounts.ts';
import { primaryMailboxAuthority } from './linkedMailboxSession.ts';

/** The layout key of the sign-in account; links use their ids. */
export const MAIN_KEY = 'main';
export const NICKNAME_MAX = 64;

export interface MailboxLayout {
  revision: string;
  order: string[];
  nicknames: Record<string, string>;
}

export const keyOf = (linkId: string | null): string => linkId ?? MAIN_KEY;
export const linkOf = (key: string): string | null => (key === MAIN_KEY ? null : key);

/** The keys in display order: the saved order, then anything it misses. */
export function orderedKeys(list: Pick<LinkedMailboxList, 'accounts' | 'layout'>): string[] {
  const keys = [MAIN_KEY, ...list.accounts.map((account) => account.id)];
  const out: string[] = [];
  for (const key of list.layout?.order ?? []) if (keys.includes(key) && !out.includes(key)) out.push(key);
  if (!out.includes(MAIN_KEY)) out.unshift(MAIN_KEY);
  for (const key of keys) if (!out.includes(key)) out.push(key);
  return out;
}

// Characters a nickname may not hold (the server refuses the same set): C0/C1
// controls, zero-width and bidirectional formatting.
const REFUSED = /[\p{Cc}\u061C\u200B-\u200F\u202A-\u202E\u2060-\u206F\uFEFF\uFFF9-\uFFFB]/u;

/** `''` = cleared; `null` = not acceptable. */
export function cleanNickname(raw: string): string | null {
  const value = raw.trim();
  if (Array.from(value).length > NICKNAME_MAX || REFUSED.test(value)) return null;
  return value;
}

// The last list this tab saw for the signed-in owner: the source of the
// nickname shown in the sidebar head, the phone chip and popup titles. It is
// bound to the primary session's storage authority (the layout belongs to the
// owner, so it stays valid while a linked mailbox is open) and ignored once
// another sign-in replaces it.
const [known, setKnown] = createSignal<{ authority: string; list: LinkedMailboxList } | null>(null);

export function rememberLinkedList(list: LinkedMailboxList): void {
  setKnown({ authority: list.primary.storageAuthority, list });
}

/** The nickname of a mailbox, when the owner gave it one. */
export function mailboxNickname(linkId: string | null): string | undefined {
  const value = known();
  if (!value || value.authority !== primaryMailboxAuthority()) return undefined;
  const name = value.list.layout?.nicknames[keyOf(linkId)];
  return name ? name : undefined;
}

/** The arrangement with `keys` in a new order (from `moveTo` on the display order). */
export function layoutChange(list: LinkedMailboxList, edit: { order?: string[]; nickname?: { key: string; value: string } }): Omit<MailboxLayout, 'revision'> {
  const nicknames = { ...(list.layout?.nicknames ?? {}) };
  if (edit.nickname) {
    if (edit.nickname.value === '') delete nicknames[edit.nickname.key];
    else nicknames[edit.nickname.key] = edit.nickname.value;
  }
  return { order: edit.order ?? orderedKeys(list), nicknames };
}

/**
 * Save an arrangement change against the revision this tab read. When another
 * device saved first (409), re-read once and apply the same change on top of
 * its arrangement, so neither device's edit is silently lost.
 */
export async function saveLayout(
  api: { list(): Promise<LinkedMailboxList>; setLayout(layout: MailboxLayout): Promise<MailboxLayout> },
  current: LinkedMailboxList,
  change: (list: LinkedMailboxList) => Omit<MailboxLayout, 'revision'>,
): Promise<MailboxLayout> {
  try {
    return await api.setLayout({ revision: current.layout?.revision ?? '', ...change(current) });
  } catch (error) {
    if ((error as { status?: unknown } | null)?.status !== 409) throw error;
    const fresh = await api.list();
    return await api.setLayout({ revision: fresh.layout?.revision ?? '', ...change(fresh) });
  }
}
