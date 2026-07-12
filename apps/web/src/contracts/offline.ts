// FROZEN offline contract (plan §2.5, SPEC §10.3). Implemented by e5
// (offline/** + sw/** + state/slices/offline.ts).
//
// Device-at-rest protection, NOT zero-access: the OPFS cache is AES-256-GCM
// under a non-extractable per-profile key in IndexedDB; the server still sends
// plaintext in V2. V6 swaps the key source for the user-derived hierarchy.

import type { Id } from '../api/jmap-types.ts';

// ── OPFS layout: each blob is AES-256-GCM with a 12-byte IV prefix. ──
export const opfsMessagePath = (accountId: Id, stableId: Id): string =>
  `/${accountId}/messages/${stableId}.enc`;
export const opfsHeadersPath = (accountId: Id, mailboxId: Id): string =>
  `/${accountId}/headers/${mailboxId}.enc`;
export const opfsSearchSlicePath = (accountId: Id): string =>
  `/${accountId}/searchslice.enc`;

/** AES-GCM IV length (bytes) prefixed to every ciphertext blob. */
export const GCM_IV_BYTES = 12;

// ── IndexedDB stores. ──
/** Holds the non-extractable AES-GCM `CryptoKey` for this browser profile. */
export const IDB_KEYS_STORE = 'mw-keys';
/** The outbound (offline) queue. */
export const IDB_OUTBOX_STORE = 'mw-outbox';

export type OutboundType = 'send' | 'flag' | 'move' | 'draft';
export type OutboundState = 'queued' | 'sent' | 'failed';

/** One queued mutation, replayed FIFO on reconnect and reconciled vs newState. */
export interface OutboundItem {
  id: string;
  type: OutboundType;
  payload: unknown;
  createdAt: number;
  state: OutboundState;
}

// ── Service Worker. ──
/** Bump when the precached app shell changes. */
export const SHELL_CACHE_VERSION = 1;
/** Cache name `mw-shell-v{N}` (plan §2.5): precache shell; network-first for
 *  `/jmap/*`, cache-first for hashed assets/fonts. */
export const shellCacheName = (version: number = SHELL_CACHE_VERSION): string =>
  `mw-shell-v${version}`;
