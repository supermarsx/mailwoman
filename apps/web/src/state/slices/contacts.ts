// Contacts store slice (plan §2.5, §3 e0 → filled by e7). Frozen seam composed
// into `AppState`; e7 fills the signals + actions over the `AddressBook/*` /
// `ContactCard/*` / `ContactGroup/*` surface (mock until e10), incl. the Compose
// autocomplete hook wired by e10.

import { createSignal, type Accessor } from 'solid-js';
import type { AddressBook, ContactCard, ContactGroup } from '../../api/pim-types.ts';
import type { SliceContext } from './context.ts';

export interface ContactsSlice {
  addressBooks: Accessor<AddressBook[]>;
  contacts: Accessor<ContactCard[]>;
  contactGroups: Accessor<ContactGroup[]>;
  /** Load the account's address books + contacts (e7 fills). */
  loadContacts(): Promise<void>;
}

export function createContactsSlice(_ctx: SliceContext): ContactsSlice {
  const [addressBooks] = createSignal<AddressBook[]>([]);
  const [contacts] = createSignal<ContactCard[]>([]);
  const [contactGroups] = createSignal<ContactGroup[]>([]);

  return {
    addressBooks,
    contacts,
    contactGroups,
    loadContacts: () => Promise.resolve(),
  };
}
