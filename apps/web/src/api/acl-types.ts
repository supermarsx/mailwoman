// Mailbox ACL (RFC 4314) + server/mailbox METADATA (RFC 5464) web contract
// (t13 26.13, plan §Workstream-2 E8). These ride the existing JMAP method-call
// surface — like `SecurityVerdict/get` (plan §6 E7), NOT a new admin endpoint —
// so there is NO new frozen JMAP type: the engine serialises the E0 frozen
// structs (`AclEntry`/`MetadataEntry`, `crates/mw-engine/src/backend.rs`) straight
// onto the wire, and this file mirrors them field-for-field.
//
// Read-through only (no client cache): the upstream IMAP server is the authority
// (mw-imap issues GETACL/SETACL/GETMETADATA/… against it). The `a` (admin) right
// gate on the editor is UX honesty, not enforcement — the server rejects an
// unauthorised SETACL regardless.
//
// E7's exact `MailboxRights/*` + `ServerMetadata/*` request/response shapes are in
// `.orchestration/logs/t13-E7.md`. E9 (mount) RECONCILED this file against E7's
// ACTUAL engine contract (`crates/mw-engine/src/acl.rs`):
//   • method names + arg shapes (`mailboxId`/`identifier`/`rights?`,
//     `entries[]`/`entry`/`value?`) and get-response fields (`myRights`+`acl`;
//     `list`) all matched the plan §Contract shapes — no change.
//   • REVOKE fix: E7's `mailbox_rights_set` reads `rights` with `as_str`, so an
//     EMPTY string `""` is `Some("")` → SETACL-empty, NOT DELETEACL. Only a null
//     /absent `rights` takes the DELETEACL arm. So `revoke` now sends `rights:
//     null` (below), and `mailboxRightsSet` accepts `string | null`.
//   • `using` URN: E7's dispatch (and mw-server `/jmap/api`) do NOT validate
//     `using` at all, and the session never advertises `urn:mailwoman:acl`. These
//     methods ride the existing session method surface by name. So `using` carries
//     only the always-advertised JMAP core capability — no invented URN.

import { CAP_CORE, type Id, type Invocation, type JmapRequest, type JmapResponse } from './jmap-types.ts';
import { responseFor } from './jmap.ts';

/**
 * `using` for the ACL/METADATA method calls. The engine dispatches these by method
 * name and checks nothing in `using` (§reconcile above), so this carries only the
 * base JMAP core capability — the server advertises no ACL-specific URN.
 */
const ACL_USING = [CAP_CORE];

// ── shapes (mirror the E0 frozen serde structs, byte-for-byte) ───────────────

/** One ACL grant: an identifier (user / group / `anyone`) + its RFC 4314 rights. */
export interface AclEntry {
  identifier: string;
  rights: string;
}

/** One METADATA annotation: an entry path + its value (`null` = not set / removed). */
export interface MetadataEntry {
  entry: string;
  value: string | null;
}

/** The `MailboxRights/get` result: the caller's own rights + the full ACL. */
export interface MailboxRights {
  /** The current user's rights on this mailbox (RFC 4314 `MYRIGHTS`). */
  myRights: string;
  /** Every identifier's grant on this mailbox (`GETACL`). */
  acl: AclEntry[];
}

// ── RFC 4314 rights bits ─────────────────────────────────────────────────────

/**
 * The eleven RFC 4314 standard rights, in canonical order. Each maps to a
 * labelled checkbox with a plain-language description in the editor
 * (`sharing-right-<bit>-label` / `-desc`).
 */
export const ACL_RIGHTS = ['l', 'r', 's', 'w', 'i', 'p', 'k', 'x', 't', 'e', 'a'] as const;
export type AclRight = (typeof ACL_RIGHTS)[number];

/** The `a` (administer) right — holding it is what gates SETACL/DELETEACL in the UI. */
export const ACL_ADMIN_RIGHT: AclRight = 'a';

const RIGHT_SET = new Set<string>(ACL_RIGHTS);

/** Parse a rights string into the set of recognised RFC 4314 bits it contains. */
export function parseRights(rights: string): Set<AclRight> {
  const out = new Set<AclRight>();
  for (const ch of rights) {
    if (RIGHT_SET.has(ch)) out.add(ch as AclRight);
  }
  return out;
}

/** Serialise a set of rights bits into the canonical RFC 4314 order string. */
export function serializeRights(bits: Iterable<AclRight>): string {
  const held = new Set(bits);
  return ACL_RIGHTS.filter((r) => held.has(r)).join('');
}

/** Whether a rights string grants a particular bit. */
export function hasRight(rights: string, right: AclRight): boolean {
  return rights.includes(right);
}

/** Return a new canonical rights string with `right` set on or off. */
export function toggleRight(rights: string, right: AclRight, on: boolean): string {
  const bits = parseRights(rights);
  if (on) bits.add(right);
  else bits.delete(right);
  return serializeRights(bits);
}

/** Whether the current user may edit the ACL (holds the `a` administer right). */
export function canAdminister(myRights: string): boolean {
  return hasRight(myRights, ACL_ADMIN_RIGHT);
}

// ── request builders (rides the JMAP method-call surface) ────────────────────

/** `MailboxRights/get` → the caller's `myRights` + the full `acl` for a mailbox. */
export function mailboxRightsGet(accountId: Id, mailboxId: Id, callId = 'mr'): JmapRequest {
  return { using: ACL_USING, methodCalls: [['MailboxRights/get', { accountId, mailboxId }, callId]] };
}

/**
 * `MailboxRights/set` → grant (`identifier` + a `rights` string, SETACL) or revoke
 * (`identifier` + `rights: null`, DELETEACL). E7's engine takes the DELETEACL arm
 * ONLY on a null/absent `rights`; an empty string is a SETACL to empty rights, so
 * `revoke` MUST pass `null` here. One grant per call, matching the E0 `set_acl` /
 * `delete_acl` backend seam.
 */
export function mailboxRightsSet(
  accountId: Id,
  mailboxId: Id,
  identifier: string,
  rights: string | null,
  callId = 'mrs',
): JmapRequest {
  return {
    using: ACL_USING,
    methodCalls: [['MailboxRights/set', { accountId, mailboxId, identifier, rights }, callId]],
  };
}

/**
 * `ServerMetadata/get` → the METADATA entries for a mailbox, or server-level when
 * `mailboxId` is `null` (RFC 5464 empty-mailbox scope). `entries` optionally
 * narrows to specific paths; omitted = whatever the server returns.
 */
export function serverMetadataGet(
  accountId: Id,
  mailboxId: Id | null,
  entries?: string[],
  callId = 'sm',
): JmapRequest {
  const args: Record<string, unknown> = { accountId, mailboxId };
  if (entries !== undefined) args['entries'] = entries;
  return { using: ACL_USING, methodCalls: [['ServerMetadata/get', args, callId]] };
}

/**
 * `ServerMetadata/set` → write (`value` non-null) or remove (`value` null, RFC
 * 5464 NIL) one annotation. `mailboxId` null = server-level.
 */
export function serverMetadataSet(
  accountId: Id,
  mailboxId: Id | null,
  entry: string,
  value: string | null,
  callId = 'sms',
): JmapRequest {
  return {
    using: ACL_USING,
    methodCalls: [['ServerMetadata/set', { accountId, mailboxId, entry, value }, callId]],
  };
}

// ── response shapes ──────────────────────────────────────────────────────────

interface MailboxRightsGetResponse {
  accountId: Id;
  myRights: string;
  acl: AclEntry[];
}

interface ServerMetadataGetResponse {
  accountId: Id;
  list: MetadataEntry[];
}

// ── client (takes a `jmap` fn so it is trivially unit-testable) ──────────────

/** The one JMAP transport call this client needs (`Client.jmap`). */
export type JmapFn = (body: JmapRequest) => Promise<JmapResponse>;

/** The ACL + server-metadata client the editor / metadata view consume. */
export interface AclClient {
  /** `MYRIGHTS` + full `GETACL` for a mailbox. */
  getMailboxRights(mailboxId: Id): Promise<MailboxRights>;
  /** SETACL: grant `identifier` the given rights (canonical string). */
  grant(mailboxId: Id, identifier: string, rights: string): Promise<void>;
  /** DELETEACL: remove `identifier`'s grant entirely. */
  revoke(mailboxId: Id, identifier: string): Promise<void>;
  /** GETMETADATA for a mailbox, or server-level when `mailboxId` is null. */
  getServerMetadata(mailboxId: Id | null): Promise<MetadataEntry[]>;
  /** SETMETADATA: write one annotation. */
  setServerMetadata(mailboxId: Id | null, entry: string, value: string): Promise<void>;
  /** SETMETADATA NIL: remove one annotation. */
  removeServerMetadata(mailboxId: Id | null, entry: string): Promise<void>;
}

/**
 * Build an {@link AclClient} bound to `accountId`, driving the JMAP surface via
 * `jmap` (in the app: `createConfiguredClient().jmap`; in tests: a fake). E9
 * wires the production instance into the mounted modules.
 */
export function createAclClient(accountId: Id, jmap: JmapFn): AclClient {
  return {
    async getMailboxRights(mailboxId) {
      const res = await jmap(mailboxRightsGet(accountId, mailboxId));
      const out = responseFor<MailboxRightsGetResponse>(res, 'mr');
      return { myRights: out.myRights ?? '', acl: out.acl ?? [] };
    },
    async grant(mailboxId, identifier, rights) {
      const res = await jmap(mailboxRightsSet(accountId, mailboxId, identifier, rights));
      responseFor<unknown>(res, 'mrs');
    },
    async revoke(mailboxId, identifier) {
      // `null` (not '') → E7's DELETEACL arm; an empty string would be a SETACL to
      // empty rights, leaving a zero-rights entry rather than removing it.
      const res = await jmap(mailboxRightsSet(accountId, mailboxId, identifier, null));
      responseFor<unknown>(res, 'mrs');
    },
    async getServerMetadata(mailboxId) {
      const res = await jmap(serverMetadataGet(accountId, mailboxId));
      return responseFor<ServerMetadataGetResponse>(res, 'sm').list ?? [];
    },
    async setServerMetadata(mailboxId, entry, value) {
      const res = await jmap(serverMetadataSet(accountId, mailboxId, entry, value));
      responseFor<unknown>(res, 'sms');
    },
    async removeServerMetadata(mailboxId, entry) {
      const res = await jmap(serverMetadataSet(accountId, mailboxId, entry, null));
      responseFor<unknown>(res, 'sms');
    },
  };
}

/** Re-export for the mount layer / callers that build the `Invocation` directly. */
export type { Invocation };
