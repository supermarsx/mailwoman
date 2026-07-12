// Contacts module placeholder (plan §2.5, §3 e0 → filled by e7). e7 builds
// address books, the contact list (favorites, groups/lists), contact detail +
// edit (business-card layout), merge-duplicates, import/export vCard + CSV, and
// the Compose autocomplete hook over `state/slices/contacts.ts` and the frozen
// `AddressBook/*`/`ContactCard/*`/`ContactGroup/*` surface.

import type { JSX } from 'solid-js';

export function ContactsModule(): JSX.Element {
  return (
    <section aria-label="Contacts" data-module="contacts">
      <h1>Contacts</h1>
      <p>The contacts module mounts here (e7).</p>
    </section>
  );
}
