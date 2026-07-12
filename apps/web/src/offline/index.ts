// Public surface of the offline module (plan §3 e5). The offline slice
// (state/slices/offline.ts) is the store-facing seam; these are the building
// blocks other executors (e6 realtime, e7 UX) reuse directly.

export {
  encryptBytes,
  decryptBytes,
  encryptJson,
  decryptJson,
  generateProfileKey,
  getOrCreateProfileKey,
  PROFILE_KEY_ID,
  type KeyStore,
} from './crypto.ts';
export {
  EncryptedCache,
  opfsBackend,
  memoryBackend,
  opfsAvailable,
  type BlobBackend,
} from './opfs.ts';
export {
  enqueueOutbound,
  drainOutbox,
  outboundToRequest,
  outboundApplied,
  memoryOutboxStore,
  type OutboxStore,
  type DrainResult,
  type FlagPayload,
  type MovePayload,
  type SendPayload,
  type DraftPayload,
} from './outbox.ts';
export { offlineSearch, matchesOffline, type OfflineQuery } from './search.ts';
export { idbAvailable, idbKeyStore, idbOutboxStore, openOfflineDb } from './idb.ts';
