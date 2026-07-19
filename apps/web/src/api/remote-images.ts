// Thin client wrapper for the anonymizing image proxy's remote-image display
// grants (t16 §S8/S9, e14b UI ↔ e6 server). This is the ONE file that binds the
// UI to e6's wire shapes: the reader's grant bar (`RemoteContentBar.tsx`) and its
// tests import ONLY the interface + types below, never a raw JMAP call, so when e6
// (image proxy, `crates/mw-server/src/image_proxy.rs` + `mw-store/src/image_grants
// .rs`) finalizes the exact method names/args, this module is the single place
// that changes.
//
// The grant model is the 0016 `remote_image_grants` table's 4-scope shape
// (`crates/mw-store/src/image_grants.rs`): a remote image loads only when a
// matching, non-revoked grant exists — deny-by-default (SPEC §7.2). `single`
// grants one message, `per-sender` a sender address, `per-domain` a sender
// domain, `all` the whole account.
//
// The blocked-content REPORT (how many trackers/remote resources the sanitizer
// stripped, and from which hosts — S9) is derived CLIENT-SIDE from the already-
// sanitized body string, so it needs no extra round-trip. The contract with the
// sanitizer (e6 adds the classification without changing the strip default,
// `crates/mw-sanitize/src/lib.rs`): a stripped remote resource is marked with
// `data-mw-blocked-host="<host>"`, and one the sanitizer classified as a tracker
// (1×1 beacon / known-tracker host) additionally carries `data-mw-tracker`. When
// those markers are absent (a body with nothing blocked, or a pre-e6 sanitizer)
// the report is empty and the bar stays hidden — honest by construction.

import { responseFor } from './jmap.ts';
import { CAP_CORE } from './jmap-types.ts';
import { CAP_SECURITY } from './crypto-types.ts';
import type { Client } from './client.ts';
import type { Invocation } from './jmap-types.ts';

/** The 4 grant scopes (0016 `scope_kind`). */
export type GrantScopeKind = 'single' | 'all' | 'per-sender' | 'per-domain';

/** A grant scope: `kind` + the value it applies to (message id / sender / domain;
 *  `''` for the account-wide `all`). */
export interface GrantScope {
  kind: GrantScopeKind;
  value: string;
}

/** An active remote-image grant (0016 row, public fields). */
export interface RemoteImageGrant {
  scopeKind: GrantScopeKind;
  scopeValue: string;
  grantedAt: string;
}

/** What the sanitizer blocked in the open message's body (S9). */
export interface BlockedContentReport {
  /** Distinct remote hosts whose resources were stripped, deduped + sorted. */
  blockedHosts: string[];
  /** Total number of blocked remote resources (may exceed `blockedHosts.length`
   *  when several load from the same host). */
  blockedCount: number;
  /** How many of the blocked resources the sanitizer classified as trackers. */
  trackerCount: number;
}

const EMPTY_REPORT: BlockedContentReport = { blockedHosts: [], blockedCount: 0, trackerCount: 0 };

/** The remote-image grant surface the UI depends on. e6 provides the production
 *  implementation ({@link createRemoteImageApi}); tests pass a fake. */
export interface RemoteImageApi {
  /** Grant remote-image loading for a scope (idempotent; un-revokes). */
  grant(accountId: string, scope: GrantScope): Promise<void>;
  /** Soft-revoke a grant (remote images for that scope block again next load). */
  revoke(accountId: string, scope: GrantScope): Promise<void>;
  /** Active (non-revoked) grants for the account, newest first. */
  listGrants(accountId: string): Promise<RemoteImageGrant[]>;
}

// The JMAP capability set the grant methods ride. Kept local (not exported) so the
// exact `using` is part of the e6-localized seam, not a cross-module constant.
const REMOTE_IMAGE_USING = [CAP_CORE, CAP_SECURITY];

interface GrantGetResponse {
  accountId: string;
  list: RemoteImageGrant[];
}

/**
 * Build the production {@link RemoteImageApi} over a JMAP {@link Client} (the same
 * session the reader already uses). The three methods map onto the engine's
 * `RemoteImage/{set,get}` extension — grant and revoke are one `RemoteImage/set`
 * each (distinguished by the `grant` / `revoke` arg), and `listGrants` is
 * `RemoteImage/get`. This mapping is the whole e6 seam.
 */
export function createRemoteImageApi(client: Pick<Client, 'jmap'>): RemoteImageApi {
  async function set(accountId: string, key: 'grant' | 'revoke', scope: GrantScope): Promise<void> {
    const call: Invocation = [
      'RemoteImage/set',
      { accountId, [key]: { scopeKind: scope.kind, scopeValue: scope.value } },
      's',
    ];
    const res = await client.jmap({ using: REMOTE_IMAGE_USING, methodCalls: [call] });
    // Surface a method-level error (responseFor throws on an `error` tuple).
    responseFor<unknown>(res, 's');
  }
  return {
    grant: (accountId, scope) => set(accountId, 'grant', scope),
    revoke: (accountId, scope) => set(accountId, 'revoke', scope),
    async listGrants(accountId) {
      const res = await client.jmap({
        using: REMOTE_IMAGE_USING,
        methodCalls: [['RemoteImage/get', { accountId }, 'g']],
      });
      return responseFor<GrantGetResponse>(res, 'g').list ?? [];
    },
  };
}

/** The sender domain of an address (`a@b.example` → `b.example`), lower-cased;
 *  `''` when the address has no `@`. */
export function senderDomain(address: string): string {
  const at = address.lastIndexOf('@');
  return at >= 0 ? address.slice(at + 1).toLowerCase() : '';
}

/**
 * The grant scope a UI action maps to, for the open message. `single` uses the
 * message id, `per-sender` the sender address, `per-domain` its domain, `all` the
 * empty account-wide value — matching `image_grants.rs`.
 */
export function scopeFor(kind: GrantScopeKind, ctx: { emailId: string; sender: string }): GrantScope {
  switch (kind) {
    case 'single':
      return { kind, value: ctx.emailId };
    case 'per-sender':
      return { kind, value: ctx.sender.toLowerCase() };
    case 'per-domain':
      return { kind, value: senderDomain(ctx.sender) };
    case 'all':
      return { kind, value: '' };
  }
}

/**
 * Derive the {@link BlockedContentReport} from a sanitized body string by reading
 * the sanitizer's block markers (`data-mw-blocked-host`, `data-mw-tracker`). Pure
 * + client-side: no network, safe on any string (a parse failure yields the empty
 * report). Parsing the STRING (not the live sandboxed iframe) keeps the reader's
 * no-same-origin frame contract intact.
 */
export function analyzeBlockedContent(sanitizedHtml: string | null): BlockedContentReport {
  if (sanitizedHtml === null || sanitizedHtml === '') return EMPTY_REPORT;
  let doc: Document;
  try {
    doc = new DOMParser().parseFromString(sanitizedHtml, 'text/html');
  } catch {
    return EMPTY_REPORT;
  }
  const marked = Array.from(doc.querySelectorAll('[data-mw-blocked-host]'));
  if (marked.length === 0) return EMPTY_REPORT;
  const hosts = new Set<string>();
  let trackerCount = 0;
  for (const el of marked) {
    const host = (el.getAttribute('data-mw-blocked-host') ?? '').trim().toLowerCase();
    if (host !== '') hosts.add(host);
    if (el.hasAttribute('data-mw-tracker')) trackerCount += 1;
  }
  return {
    blockedHosts: Array.from(hosts).sort((a, b) => a.localeCompare(b)),
    blockedCount: marked.length,
    trackerCount,
  };
}

/** True when a report has anything to show (drives whether the bar renders). */
export function hasBlockedContent(report: BlockedContentReport): boolean {
  return report.blockedCount > 0;
}

/** Whether an active grant list already covers the open message's context, so the
 *  bar can offer "turn off" (revoke) instead of "load". A `single` grant matches
 *  the message id, `per-sender` the sender, `per-domain` the domain, `all` always. */
export function coveringGrant(
  grants: RemoteImageGrant[],
  ctx: { emailId: string; sender: string },
): RemoteImageGrant | null {
  const domain = senderDomain(ctx.sender);
  const sender = ctx.sender.toLowerCase();
  for (const g of grants) {
    const v = g.scopeValue.toLowerCase();
    if (g.scopeKind === 'all') return g;
    if (g.scopeKind === 'single' && g.scopeValue === ctx.emailId) return g;
    if (g.scopeKind === 'per-sender' && v === sender && sender !== '') return g;
    if (g.scopeKind === 'per-domain' && v === domain && domain !== '') return g;
  }
  return null;
}
