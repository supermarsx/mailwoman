// Contacts store slice (plan §2.5, §3 e7). Address books, the contact list
// (favorites, groups / distribution lists), contact detail + edit, merge-
// duplicates, vCard/CSV import, group management, and the favorite toggle — all
// over the frozen `AddressBook/*` / `ContactCard/*` / `ContactGroup/*` surface
// (plan §2.2), mock-backed until e10 swaps in the real engine. The recipient
// autocomplete hook lives in `modules/contacts/autocomplete.ts` (wired into
// Compose by e10); the slice just exposes `contacts()` as its card source.
//
// Disjoint file — no `store.ts` collision with the other PIM slices (same slice
// discipline as V2). `store.ts` spreads whatever this factory returns into the
// frozen `AppState`, so the interface below is additive and self-contained.

import { createSignal, createMemo, batch, onCleanup, type Accessor } from 'solid-js';
import { observePreferenceAccount } from '../../api/accountPreferenceLifecycle.ts';
import { CAP_CONTACTS, type AddressBook, type ContactCard, type ContactGroup } from '../../api/pim-types.ts';
import type { Id } from '../../api/jmap-types.ts';
import { responseFor } from '../../api/jmap.ts';
import { keyedAttempt } from '../../api/recovery.ts';
import {suggestionDisplay,type ContactSuggestion} from '../../modules/contacts/autocomplete.ts';
import {
  addressBooksGet,
  contactGroupSet,
  contactGroupsGet,
  contactMerge,
  contactAutocomplete,
  contactSet,
  contactsQueryGet,
  type AddressBookGetResponse,
  type ContactCreate,
  type ContactGetResponse,
  type ContactGroupGetResponse,
  type ContactGroupSetResponse,
  type ContactMergeResponse,
  type ContactSetResponse,
} from '../../modules/contacts/api.ts';
import { mergeCards } from '../../modules/contacts/merge.ts';
import type { ParsedContact } from '../../modules/contacts/vcard.ts';
import type { SliceContext } from './context.ts';

/** The fields a contact editor supplies on create/edit (a subset of `ContactCard`). */
export type ContactDraft = Partial<Omit<ContactCard, 'id'>>;

/** An empty structured name (editor / import default). */
function emptyName(): ContactCard['name'] {
  return { full: '', given: '', surname: '', prefix: '', suffix: '' };
}

/** Best-effort unique id for optimistic inserts before the server answers. */
function localId(prefix: string): string {
  const c = globalThis.crypto;
  if (c !== undefined && typeof c.randomUUID === 'function') return `${prefix}-${c.randomUUID()}`;
  return `${prefix}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

/** Materialize a full `ContactCard` from a partial draft (fills every field). */
export function draftToCard(id: Id, addressBookId: Id, draft: ContactDraft): ContactCard {
  return {
    id,
    addressBookId,
    uid: draft.uid ?? id,
    kind: draft.kind ?? 'individual',
    name: draft.name ?? emptyName(),
    nicknames: draft.nicknames ?? [],
    organizations: draft.organizations ?? [],
    titles: draft.titles ?? [],
    emails: draft.emails ?? [],
    phones: draft.phones ?? [],
    onlineServices: draft.onlineServices ?? [],
    addresses: draft.addresses ?? [],
    anniversaries: draft.anniversaries ?? [],
    notes: draft.notes ?? '',
    photoBlobId: draft.photoBlobId ?? null,
    isFavorite: draft.isFavorite ?? false,
    groupIds: draft.groupIds ?? [],
    pgpKey: draft.pgpKey ?? null,
    smimeCert: draft.smimeCert ?? null,
    etag: draft.etag ?? null,
  };
}

/** The display name a list row shows (full name → org → first email → "No name"). */
export function contactDisplayName(card: ContactCard): string {
  if (card.name.full.trim().length > 0) return card.name.full.trim();
  if (card.organizations[0] !== undefined && card.organizations[0].length > 0) return card.organizations[0];
  if (card.emails[0] !== undefined) return card.emails[0].value;
  return 'No name';
}

/** Case-insensitive substring test over a card's searchable fields. */
export function contactMatches(card: ContactCard, q: string): boolean {
  if (q.length === 0) return true;
  if (contactDisplayName(card).toLowerCase().includes(q)) return true;
  if (card.name.given.toLowerCase().includes(q) || card.name.surname.toLowerCase().includes(q)) return true;
  if (card.nicknames.some((n) => n.toLowerCase().includes(q))) return true;
  if (card.organizations.some((o) => o.toLowerCase().includes(q))) return true;
  if (card.emails.some((e) => e.value.toLowerCase().includes(q))) return true;
  if (card.phones.some((p) => p.value.toLowerCase().includes(q))) return true;
  return false;
}

/** Sort key: full name, case-insensitive; unnamed cards fall to the end. */
export function sortContacts(list: readonly ContactCard[]): ContactCard[] {
  return [...list].sort((a, b) => contactDisplayName(a).toLowerCase().localeCompare(contactDisplayName(b).toLowerCase()));
}

/** The contacts portion of `AppState` (accessors + actions). */
export interface ContactsSlice {
  addressBooks: Accessor<AddressBook[]>;
  contacts: Accessor<ContactCard[]>;
  contactGroups: Accessor<ContactGroup[]>;
  contactsLoading: Accessor<boolean>;

  /** Focused address book, or `null` for "all books". */
  selectedAddressBookId: Accessor<Id | null>;
  /** Group / distribution-list filter, or `null` for "no group filter". */
  selectedGroupId: Accessor<Id | null>;
  /** Restrict the list to favorites when true. */
  favoritesOnly: Accessor<boolean>;
  /** Free-text search across name / org / email / phone. */
  contactSearch: Accessor<string>;
  /** The currently open contact (detail/edit), or `null`. */
  selectedContactId: Accessor<Id | null>;

  // ── derived views ──
  /** The cards matching the current book + group + favorites + search, sorted. */
  filteredContacts: Accessor<ContactCard[]>;
  /** The open contact's card, or `null`. */
  selectedContact: Accessor<ContactCard | null>;
  /** Look up a loaded card by id. */
  contactById(id: Id): ContactCard | undefined;

  // ── filter setters ──
  selectAddressBook(id: Id | null): void;
  setSelectedGroup(id: Id | null): void;
  setFavoritesOnly(on: boolean): void;
  setContactSearch(q: string): void;
  selectContact(id: Id | null): void;

  // ── actions ──
  /** Load the account's address books, groups, and contacts (e10 → engine). */
  loadContacts(): Promise<void>;
  autocompleteContacts(prefix:string):Promise<ContactSuggestion[]>;
  /** Create a contact; returns the new id (or null on failure). */
  createContact(draft: ContactDraft, addressBookId?: Id): Promise<Id | null>;
  /** Patch an existing contact. */
  updateContact(id: Id, patch: Partial<ContactCard>): Promise<boolean>;
  /** Delete a contact. */
  deleteContact(id: Id): Promise<void>;
  /** Flip a contact's favorite flag. */
  toggleFavorite(id: Id): Promise<void>;
  /** Import parsed vCard/CSV drafts into an address book; returns count created. */
  importContacts(drafts: ParsedContact[], addressBookId?: Id): Promise<number>;
  /** Merge duplicates into `keepId` (non-destructive: sources become tombstones). */
  mergeContacts(keepId: Id, mergeIds: Id[]): Promise<Id | null>;

  // ── groups / distribution lists ──
  createGroup(name: string, addressBookId?: Id, memberIds?: Id[]): Promise<Id | null>;
  updateGroup(id: Id, patch: Partial<ContactGroup>): Promise<boolean>;
  deleteGroup(id: Id): Promise<boolean>;
  /** Add/remove a contact to/from a group (updates both the group and the card). */
  setGroupMembership(contactId: Id, groupId: Id, member: boolean): Promise<void>;
}

export function createContactsSlice(ctx: SliceContext): ContactsSlice {
  const { client } = ctx;

  const [addressBooks, setAddressBooks] = createSignal<AddressBook[]>([]);
  const [contacts, setContacts] = createSignal<ContactCard[]>([]);
  const [contactGroups, setContactGroups] = createSignal<ContactGroup[]>([]);
  const [contactsLoading, setContactsLoading] = createSignal(false);

  const [selectedAddressBookId, setSelectedAddressBookId] = createSignal<Id | null>(null);
  const [selectedGroupId, setSelectedGroupId] = createSignal<Id | null>(null);
  const [favoritesOnly, setFavoritesOnly] = createSignal(false);
  const [contactSearch, setContactSearch] = createSignal('');
  const [selectedContactId, setSelectedContactId] = createSignal<Id | null>(null);

  let currentAccount: string | null = null;
  let authority = 0;
  let autocompleteUnavailable=false;
  onCleanup(observePreferenceAccount(() => {
    authority += 1;
    autocompleteUnavailable=false;
    currentAccount = null;
    batch(() => {
      setContacts([]); setAddressBooks([]); setContactGroups([]);
      setSelectedContactId(null); setSelectedAddressBookId(null); setSelectedGroupId(null);
      setContactsLoading(false);
    });
  }));

  async function resolveAccount(): Promise<string | null> {
    if (currentAccount !== null) return currentAccount;
    const version = authority;
    const session = await client.session();
    if (version !== authority) return null;
    const acct = session.primaryAccounts[CAP_CONTACTS] ?? Object.keys(session.accounts)[0] ?? null;
    currentAccount = acct;
    return acct;
  }

  function patchCard(id: Id, patch: Partial<ContactCard>): void {
    setContacts((cs) => cs.map((c) => (c.id === id ? { ...c, ...patch } : c)));
  }

  // ── derived views ──
  const filteredContacts = createMemo<ContactCard[]>(() => {
    const book = selectedAddressBookId();
    const group = selectedGroupId();
    const favOnly = favoritesOnly();
    const q = contactSearch().trim().toLowerCase();
    let list = contacts();
    if (book !== null) list = list.filter((c) => c.addressBookId === book);
    if (group !== null) {
      const g = contactGroups().find((x) => x.id === group);
      const members = new Set(g?.memberIds ?? []);
      list = list.filter((c) => members.has(c.id) || c.groupIds.includes(group));
    }
    if (favOnly) list = list.filter((c) => c.isFavorite);
    if (q.length > 0) list = list.filter((c) => contactMatches(c, q));
    return sortContacts(list);
  });

  const selectedContact = createMemo<ContactCard | null>(() => {
    const id = selectedContactId();
    if (id === null) return null;
    return contacts().find((c) => c.id === id) ?? null;
  });

  function contactById(id: Id): ContactCard | undefined {
    return contacts().find((c) => c.id === id);
  }

  // ── loading ──
  // Each `loadContacts()` takes a ticket when it is called. Calls overlap
  // (opening Contacts starts two loads; Compose and the keys module start
  // others), and one that started earlier can answer last; only the call
  // holding the latest ticket writes state and ends the loading state, so the
  // older answer is dropped instead of replacing the newer one (as `loadTicket`
  // in `modules/calendar/controller.ts`).
  let loadTicket = 0;

  async function loadContacts(): Promise<void> {
    const ticket = ++loadTicket;
    const version = authority;
    const stale = (): boolean => ticket !== loadTicket || version !== authority;
    const acct = await resolveAccount();
    if (stale()) return;
    if (acct === null) {
      batch(() => {
        setAddressBooks([]);
        setContacts([]);
        setContactGroups([]);
      });
      return;
    }
    setContactsLoading(true);
    try {
      const [bookRes, groupRes, cardRes] = await Promise.all([
        client.jmap(addressBooksGet(acct)),
        client.jmap(contactGroupsGet(acct)),
        client.jmap(contactsQueryGet(acct, selectedAddressBookId() ?? undefined)),
      ]);
      if (stale()) return;
      batch(() => {
        setAddressBooks(responseFor<AddressBookGetResponse>(bookRes, 'books').list);
        setContactGroups(responseFor<ContactGroupGetResponse>(groupRes, 'groups').list);
        setContacts(sortContacts(responseFor<ContactGetResponse>(cardRes, 'g').list));
      });
    } finally {
      // The latest call ends the loading state; a dropped one leaves it alone.
      if (!stale()) setContactsLoading(false);
    }
  }

  async function autocompleteContacts(prefix:string):Promise<ContactSuggestion[]>{
    const version=authority;
    if(!prefix.trim()||autocompleteUnavailable)return [];
    const session=await client.session();
    if(version!==authority)return [];
    const acct=session.primaryAccounts[CAP_CONTACTS];
    if(!acct)return [];
    const result=await client.jmap(contactAutocomplete(acct,prefix,8));
    if(version!==authority)return [];
    if(result.methodResponses.some(([method,args,id])=>id==='ac'&&method==='error'&&['unknownMethod','notImplemented'].includes(String((args as {type?:string}).type)))){
      autocompleteUnavailable=true;return [];
    }
    const response=responseFor<{accountId:string;list:{id:string;name:string;email:string;isFavorite:boolean}[]}>(result,'ac');
    if(response.accountId!==acct||!Array.isArray(response.list))return [];
    return response.list.filter(row=>typeof row.email==='string'&&row.email.length>0&&typeof row.name==='string').map(row=>({cardId:row.id,name:row.name,email:row.email,isFavorite:row.isFavorite,display:suggestionDisplay(row.name,row.email),score:0}));
  }

  function selectAddressBook(id: Id | null): void {
    setSelectedAddressBookId(id);
  }

  // ── card CRUD ──
  async function createContact(draft: ContactDraft, addressBookId?: Id): Promise<Id | null> {
    const version = authority;
    const acct = await resolveAccount();
    if (acct === null || version !== authority) return null;
    const book = addressBookId ?? selectedAddressBookId() ?? addressBooks()[0]?.id ?? 'default';
    const { id: _omit, addressBookId: _omitBook, ...create } = draftToCard('', book, draft);
    try {
      // t91: keyed; the editor's Save after a failure reuses the key (no duplicate).
      const request = contactSet(acct, { create: { new: create as ContactCreate } });
      const res = await keyedAttempt(`contacts:${JSON.stringify(request)}`, key => client.jmap(request, { idempotencyKey: key }));
      if (version !== authority) return null;
      const result = responseFor<ContactSetResponse>(res, 'set');
      const created = result.created?.['new'];
      if (result.accountId !== acct || result.notCreated?.['new'] || !created?.id) return null;
      setContacts(cs => sortContacts([...cs, draftToCard(created.id, book, { ...draft, ...created })]));
      ctx.broadcastChange?.();
      return created.id;
    } catch { return null; }
  }

  async function updateContact(id: Id, patch: Partial<ContactCard>): Promise<boolean> {
    const version = authority;
    const acct = await resolveAccount();
    if (acct === null || version !== authority) return false;
    try {
      const res = await client.jmap(contactSet(acct, { update: { [id]: { ...patch } } }));
      if (version !== authority) return false;
      const result = responseFor<ContactSetResponse>(res, 'set');
      if (result.accountId !== acct || result.notUpdated?.[id] || !result.updated || !Object.hasOwn(result.updated, id)) return false;
      patchCard(id, patch);
      setContacts(cs => sortContacts(cs));
      ctx.broadcastChange?.();
      return true;
    } catch { return false; }
  }

  async function deleteContact(id: Id): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) return;
    batch(() => {
      setContacts((cs) => cs.filter((c) => c.id !== id));
      if (selectedContactId() === id) setSelectedContactId(null);
      // Drop the tombstoned member from any group it belonged to.
      setContactGroups((gs) => gs.map((g) => ({ ...g, memberIds: g.memberIds.filter((m) => m !== id) })));
    });
    await client.jmap(contactSet(acct, { destroy: [id] }));
    ctx.broadcastChange?.();
  }

  async function toggleFavorite(id: Id): Promise<void> {
    const cur = contactById(id);
    if (cur === undefined) return;
    await updateContact(id, { isFavorite: !cur.isFavorite });
  }

  async function importContacts(drafts: ParsedContact[], addressBookId?: Id): Promise<number> {
    const acct = await resolveAccount();
    if (acct === null || drafts.length === 0) return 0;
    const book = addressBookId ?? selectedAddressBookId() ?? addressBooks()[0]?.id ?? 'default';
    const create: Record<string, ContactCreate> = {};
    const keys: string[] = [];
    drafts.forEach((d, i) => {
      const key = `import-${i}`;
      keys.push(key);
      const { id: _o, addressBookId: _b, ...c } = draftToCard('', book, d);
      create[key] = c as ContactCreate;
    });
    const res = await client.jmap(contactSet(acct, { create }));
    const setRes = responseFor<ContactSetResponse>(res, 'set');
    const added: ContactCard[] = [];
    keys.forEach((key, i) => {
      if (setRes.notCreated?.[key] !== undefined) return;
      const created = setRes.created?.[key];
      const id = created?.id ?? localId('contact');
      added.push(draftToCard(id, book, { ...drafts[i]!, ...created }));
    });
    if (added.length > 0) {
      setContacts((cs) => sortContacts([...cs, ...added]));
      ctx.broadcastChange?.();
    }
    return added.length;
  }

  async function mergeContacts(keepId: Id, mergeIds: Id[]): Promise<Id | null> {
    const acct = await resolveAccount();
    if (acct === null) return null;
    const primary = contactById(keepId);
    if (primary === undefined) return null;
    const others = mergeIds.map((id) => contactById(id)).filter((c): c is ContactCard => c !== undefined);
    if (others.length === 0) return keepId;
    // Compute the survivor client-side (drives the preview + optimistic state);
    // `ContactCard/merge` is the engine's authority at integration (e10).
    const merged = mergeCards(primary, others);
    const gone = new Set(mergeIds);
    batch(() => {
      setContacts((cs) => sortContacts(cs.filter((c) => !gone.has(c.id)).map((c) => (c.id === keepId ? merged : c))));
      // Re-home group memberships from the tombstones onto the survivor.
      setContactGroups((gs) =>
        gs.map((g) => {
          if (!g.memberIds.some((m) => gone.has(m))) return g;
          const kept = g.memberIds.filter((m) => !gone.has(m));
          return { ...g, memberIds: kept.includes(keepId) ? kept : [...kept, keepId] };
        }),
      );
      if (selectedContactId() !== null && gone.has(selectedContactId()!)) setSelectedContactId(keepId);
    });
    const res = await client.jmap(contactMerge(acct, keepId, mergeIds));
    // Prefer the engine's canonical survivor when it answers with one.
    try {
      const mergedRes = responseFor<ContactMergeResponse>(res, 'merge');
      if (mergedRes.merged !== undefined && mergedRes.merged !== null) patchCard(keepId, mergedRes.merged);
    } catch {
      // Mock/echo backends return no merge payload — the client-side survivor stands.
    }
    ctx.broadcastChange?.();
    return keepId;
  }

  // ── groups ──
  async function createGroup(name: string, addressBookId?: Id, memberIds: Id[] = []): Promise<Id | null> {
    const version = authority;
    const acct = await resolveAccount();
    if (acct === null || version !== authority) return null;
    const book = addressBookId ?? selectedAddressBookId() ?? addressBooks()[0]?.id ?? 'default';
    try {
      const request = contactGroupSet(acct, { create: { new: { addressBookId: book, name, memberIds } } });
      const res = await keyedAttempt(`contact-groups:${JSON.stringify(request)}`, key => client.jmap(request, { idempotencyKey: key }));
      if (version !== authority) return null;
      const result = responseFor<ContactGroupSetResponse>(res, 'set');
      const id = result.created?.['new']?.id;
      if (result.accountId !== acct || result.notCreated?.['new'] || !id) return null;
      setContactGroups((gs) => [...gs, { id, addressBookId: book, name, memberIds }]);
      ctx.broadcastChange?.();
      return id;
    } catch { return null; }
  }

  async function updateGroup(id: Id, patch: Partial<ContactGroup>): Promise<boolean> {
    const version = authority;
    const acct = await resolveAccount();
    if (acct === null || version !== authority) return false;
    try {
    const response = await client.jmap(contactGroupSet(acct, { update: { [id]: { ...patch } } }));
    if (version !== authority) return false;
    const result = responseFor<ContactGroupSetResponse>(response, 'set');
    if (result.accountId !== acct || result.notUpdated?.[id] || !Object.hasOwn(result.updated ?? {}, id)) return false;
    setContactGroups((gs) => gs.map((g) => (g.id === id ? { ...g, ...patch } : g)));
    ctx.broadcastChange?.();
    return true;
    } catch { return false; }
  }

  async function deleteGroup(id: Id): Promise<boolean> {
    const version = authority;
    const acct = await resolveAccount();
    if (acct === null || version !== authority) return false;
    try {
    const response = await client.jmap(contactGroupSet(acct, { destroy: [id] }));
    if (version !== authority) return false;
    const result = responseFor<ContactGroupSetResponse>(response, 'set');
    if (result.accountId !== acct || result.notDestroyed?.[id] || !result.destroyed?.includes(id)) return false;
    batch(() => {
      setContactGroups((gs) => gs.filter((g) => g.id !== id));
      if (selectedGroupId() === id) setSelectedGroupId(null);
      // A card's `groupIds` back-reference is dropped too.
      setContacts((cs) => cs.map((c) => (c.groupIds.includes(id) ? { ...c, groupIds: c.groupIds.filter((x) => x !== id) } : c)));
    });
    ctx.broadcastChange?.();
    return true;
    } catch { return false; }
  }

  async function setGroupMembership(contactId: Id, groupId: Id, member: boolean): Promise<void> {
    const group = contactGroups().find((g) => g.id === groupId);
    const card = contactById(contactId);
    if (group === undefined || card === undefined) return;
    const members = member
      ? group.memberIds.includes(contactId) ? group.memberIds : [...group.memberIds, contactId]
      : group.memberIds.filter((m) => m !== contactId);
    const groupIds = member
      ? card.groupIds.includes(groupId) ? card.groupIds : [...card.groupIds, groupId]
      : card.groupIds.filter((g) => g !== groupId);
    // Never apply an unconfirmed group mutation or cross an account replacement.
    const version = authority;
    if (!await updateGroup(groupId, { memberIds: members }) || version !== authority) return;
    await updateContact(contactId, { groupIds });
  }

  return {
    addressBooks,
    contacts,
    contactGroups,
    contactsLoading,
    selectedAddressBookId,
    selectedGroupId,
    favoritesOnly,
    contactSearch,
    selectedContactId,
    filteredContacts,
    selectedContact,
    contactById,
    selectAddressBook,
    setSelectedGroup: setSelectedGroupId,
    setFavoritesOnly,
    setContactSearch,
    selectContact: setSelectedContactId,
    loadContacts,
    autocompleteContacts,
    createContact,
    updateContact,
    deleteContact,
    toggleFavorite,
    importContacts,
    mergeContacts,
    createGroup,
    updateGroup,
    deleteGroup,
    setGroupMembership,
  };
}
