// Pure `AddressBook/*` / `ContactCard/*` / `ContactGroup/*` request builders +
// response shapes for the contacts module (plan §2.2). These mirror the frozen
// envelope machinery in `api/jmap.ts` (methodCalls array, `#`-result-references,
// `{accountId,state,list,notFound}` / `{created,updated,destroyed,...}` shapes)
// but for the Mailwoman PIM contacts families. No I/O here so they are trivially
// unit-testable; the slice runs them through the shared `Client.jmap` transport
// (mock until e10 swaps in the real engine surface).

import { request } from '../../api/jmap.ts';
import { CAP_CORE, type Id, type Invocation, type JmapRequest } from '../../api/jmap-types.ts';
import {
  CAP_CONTACTS,
  type AddressBook,
  type ContactCard,
  type ContactGroup,
} from '../../api/pim-types.ts';

/** `using` for every contacts request: core + the contacts capability. */
const CONTACTS_USING = [CAP_CORE, CAP_CONTACTS];

// ── Response shapes (frozen JMAP get/query/set) ──────────────────────────────

export interface AddressBookGetResponse {
  accountId: Id;
  state: string;
  list: AddressBook[];
  notFound: Id[];
}

export interface ContactGetResponse {
  accountId: Id;
  state: string;
  list: ContactCard[];
  notFound: Id[];
}

export interface ContactGroupGetResponse {
  accountId: Id;
  state: string;
  list: ContactGroup[];
  notFound: Id[];
}

export interface ContactSetResponse {
  accountId: Id;
  oldState: string | null;
  newState: string;
  created: Record<string, Partial<ContactCard> & { id: Id }> | null;
  updated: Record<Id, unknown> | null;
  destroyed: Id[] | null;
  notCreated: Record<string, { type: string; description?: string | null }> | null;
  notUpdated: Record<Id, { type: string; description?: string | null }> | null;
  notDestroyed: Record<Id, { type: string; description?: string | null }> | null;
}

export interface ContactGroupSetResponse {
  accountId: Id;
  created: Record<string, Partial<ContactGroup> & { id: Id }> | null;
  updated: Record<Id, unknown> | null;
  destroyed: Id[] | null;
  notCreated: Record<string, { type: string; description?: string | null }> | null;
}

/** `ContactCard/merge` result: the surviving card + the tombstoned source ids. */
export interface ContactMergeResponse {
  accountId: Id;
  merged: ContactCard;
  destroyed: Id[];
}

/** `ContactCard/autocomplete` result: server-ranked cards for the given prefix. */
export interface ContactAutocompleteResponse {
  accountId: Id;
  list: ContactCard[];
}

// ── Builders ─────────────────────────────────────────────────────────────────

/** Fetch the account's address books. */
export function addressBooksGet(accountId: Id, callId = 'books'): JmapRequest {
  return request(CONTACTS_USING, [['AddressBook/get', { accountId, ids: null }, callId]]);
}

/** Fetch the account's contact groups / distribution lists. */
export function contactGroupsGet(accountId: Id, callId = 'groups'): JmapRequest {
  return request(CONTACTS_USING, [['ContactGroup/get', { accountId, ids: null }, callId]]);
}

/**
 * List an address book's cards in one round-trip: `ContactCard/query` for the
 * ids (filtered to `addressBookId` when given), then `ContactCard/get` for
 * exactly those ids via a JMAP result reference (`#ids` from the query).
 */
export function contactsQueryGet(accountId: Id, addressBookId?: Id, limit = 1000): JmapRequest {
  const filter = addressBookId !== undefined ? { addressBookId } : {};
  return request(CONTACTS_USING, [
    ['ContactCard/query', { accountId, filter, sort: [{ property: 'name/full' }], limit }, 'q'],
    ['ContactCard/get', { accountId, '#ids': { resultOf: 'q', name: 'ContactCard/query', path: '/ids' } }, 'g'],
  ]);
}

/** A `ContactCard/set` create payload (a partial card; server assigns id/uid/etag). */
export type ContactCreate = Partial<ContactCard>;

/** Build a `ContactCard/set` request (any of create / update / destroy). */
export function contactSet(
  accountId: Id,
  ops: {
    create?: Record<string, ContactCreate>;
    update?: Record<Id, Record<string, unknown>>;
    destroy?: Id[];
  },
  callId = 'set',
): JmapRequest {
  const args: Record<string, unknown> = { accountId };
  if (ops.create !== undefined) args['create'] = ops.create;
  if (ops.update !== undefined) args['update'] = ops.update;
  if (ops.destroy !== undefined) args['destroy'] = ops.destroy;
  const call: Invocation = ['ContactCard/set', args, callId];
  return request(CONTACTS_USING, [call]);
}

/** Build a `ContactGroup/set` request (create / update / destroy groups). */
export function contactGroupSet(
  accountId: Id,
  ops: {
    create?: Record<string, Partial<ContactGroup>>;
    update?: Record<Id, Record<string, unknown>>;
    destroy?: Id[];
  },
  callId = 'set',
): JmapRequest {
  const args: Record<string, unknown> = { accountId };
  if (ops.create !== undefined) args['create'] = ops.create;
  if (ops.update !== undefined) args['update'] = ops.update;
  if (ops.destroy !== undefined) args['destroy'] = ops.destroy;
  const call: Invocation = ['ContactGroup/set', args, callId];
  return request(CONTACTS_USING, [call]);
}

/** `ContactCard/merge` — resolve duplicates into `keepId`, tombstoning the rest. */
export function contactMerge(accountId: Id, keepId: Id, mergeIds: Id[], callId = 'merge'): JmapRequest {
  return request(CONTACTS_USING, [['ContactCard/merge', { accountId, keepId, mergeIds }, callId]]);
}

/** `ContactCard/autocomplete` — server-side ranked recipient completion (e10). */
export function contactAutocomplete(accountId: Id, prefix: string, limit = 8, callId = 'ac'): JmapRequest {
  return request(CONTACTS_USING, [['ContactCard/autocomplete', { accountId, prefix, limit }, callId]]);
}
