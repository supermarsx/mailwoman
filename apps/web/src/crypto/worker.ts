// The WASM crypto Web Worker host (plan §2.5 / §2.3 / risk #12) — spawns the
// wasm-pack `mw-crypto` module in a dedicated Worker so private-key operations
// (keygen, decrypt, private sign, PKCS#12 import) NEVER block the main thread and
// the plaintext + private material NEVER enter the main app state (plan §1.2).
//
// Decrypted E2EE plaintext is sanitized IN-WORKER via the `mw-sanitize` wasm build
// (plan §1.3, wired in `worker.entry.ts`): HTML decrypted mail is sanitized here and
// returned as `plaintextHtml` (rendered as sanitized HTML in the existing no-scripts /
// no-same-origin sandboxed iframe, Reader.tsx); non-HTML plaintext is returned as
// `plaintextText` and rendered escaped. Either way the decrypted plaintext NEVER
// round-trips to the server sanitizer (risk #5) — end-to-end encryption holds.
//
// e1 built the crypto wasm surface; e8 built + hosted it in `worker.entry.ts` and
// pointed `crypto/index.ts#getCryptoWorker` at [`spawnCryptoWorker`]; e8b added the
// mw-sanitize wasm surface + the in-worker sanitize of decrypted HTML.

import type { CryptoWorkerApi } from '../contracts/crypto.ts';

/** One RPC call posted to the crypto Worker (`{id, method, args}`). */
export interface CryptoWorkerRequest {
  id: number;
  method: keyof CryptoWorkerApi;
  args: unknown;
}

/** One RPC reply from the crypto Worker (`{id, ok, value|error}`). */
export interface CryptoWorkerResponse {
  id: number;
  ok: boolean;
  value?: unknown;
  error?: string;
}

interface Pending {
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
}

/**
 * Spawn the real wasm-pack-backed crypto worker and return a [`CryptoWorkerApi`]
 * proxy over the RPC contract above. Each call posts `{id, method, args}` and
 * resolves when the matching `{id, …}` reply arrives; a worker-level crash rejects
 * every in-flight call (loud, not silent). The wasm module itself loads lazily
 * INSIDE the worker on its first RPC (plan risk #12), so constructing the worker
 * is cheap and off the login→inbox critical path.
 */
export function spawnCryptoWorker(): CryptoWorkerApi {
  // The `new Worker(new URL(...), ...)` form (inline, not via a hoisted const) is
  // what Vite's worker plugin detects to bundle `worker.entry.ts` + its wasm as a
  // separate lazy chunk off the main bundle (plan risk #12).
  const worker = new Worker(new URL('./worker.entry.ts', import.meta.url), { type: 'module' });
  const pending = new Map<number, Pending>();
  let seq = 0;

  worker.onmessage = (ev: MessageEvent<CryptoWorkerResponse>): void => {
    const { id, ok, value, error } = ev.data;
    const p = pending.get(id);
    if (p === undefined) return;
    pending.delete(id);
    if (ok) p.resolve(value);
    else p.reject(new Error(error ?? 'crypto worker error'));
  };
  worker.onerror = (ev: ErrorEvent): void => {
    const err = new Error(ev.message !== '' ? ev.message : 'crypto worker crashed');
    for (const p of pending.values()) p.reject(err);
    pending.clear();
  };

  function call<T>(method: keyof CryptoWorkerApi, args: unknown): Promise<T> {
    const id = (seq += 1);
    return new Promise<T>((resolve, reject) => {
      pending.set(id, { resolve: resolve as (value: unknown) => void, reject });
      const req: CryptoWorkerRequest = { id, method, args };
      worker.postMessage(req);
    });
  }

  return {
    verifySmime: (req) => call('verifySmime', req),
    smimeRootSetInfo: (req) => call('smimeRootSetInfo', req),
    generateKey: (req) => call('generateKey', req),
    encrypt: (req) => call('encrypt', req),
    decrypt: (req) => call('decrypt', req),
    sign: (req) => call('sign', req),
    verify: (req) => call('verify', req),
    importPkcs12: (req) => call('importPkcs12', req),
    exportPkcs12: (req) => call('exportPkcs12', req),
    certificateRequest: (req) => call('certificateRequest', req),
    attachIssuedCert: (req) => call('attachIssuedCert', req),
    importArmored: (req) => call('importArmored', req),
    exportPublic: (req) => call('exportPublic', req),
    exportBackup: (req) => call('exportBackup', req),
    unlockKey: (req) => call('unlockKey', req),
    lockKey: (req) => call('lockKey', req),
    inspectKey: (req) => call('inspectKey', req),
    verifyPgp: (req) => call('verifyPgp', req),
    revokeKey: (req) => call('revokeKey', req),
    revocationCertificate: (req) => call('revocationCertificate', req),
    applyRevocationCertificate: (req) => call('applyRevocationCertificate', req),
    setKeyExpiry: (req) => call('setKeyExpiry', req),
    addSubkey: (req) => call('addSubkey', req),
    revokeSubkey: (req) => call('revokeSubkey', req),
    changePassphrase: (req) => call('changePassphrase', req),
    exportBackupFull: (req) => call('exportBackupFull', req),
    restoreBackup: (req) => call('restoreBackup', req),
    pgpMimeEncrypt: (req) => call('pgpMimeEncrypt', req),
    pgpMimeSign: (req) => call('pgpMimeSign', req),
    pgpMimeOpen: (req) => call('pgpMimeOpen', req),
    pgpMimeVerify: (req) => call('pgpMimeVerify', req),
    analyzeMessage: (req) => call('analyzeMessage', req),
    autocryptHeader: (req) => call('autocryptHeader', req),
    parseAutocrypt: (req) => call('parseAutocrypt', req),
    autocryptRecommend: (req) => call('autocryptRecommend', req),
    detectAttachedKeys: (req) => call('detectAttachedKeys', req),
    wkdUrls: (req) => call('wkdUrls', req),
  };
}
