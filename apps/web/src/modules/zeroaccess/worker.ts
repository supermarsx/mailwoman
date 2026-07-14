// The zero-access crypto Worker host (plan §3 e8). A dedicated Web Worker running the
// wasm-pack `mw-crypto` `za*` exports, so key derivation / seal / open / pairing never
// block the main thread and no plaintext key ever enters the main app state (plan §1.2).
//
// This mirrors the existing V4 crypto-worker RPC shape (`crypto/worker.ts`) but stays
// inside the zero-access module (ownership boundary, plan §3 e8): it reuses the SAME
// `src/wasm/mw-crypto` bundle rather than duplicating any crypto in JS. e11 MAY later
// fold these `za*` methods into the shared crypto worker; functionally this is complete.

import type { ZeroAccessCrypto } from './crypto.ts';

/** One RPC call posted to the worker (`{id, method, args}`). */
export interface ZaWorkerRequest {
  id: number;
  method: keyof ZeroAccessCrypto;
  args: unknown;
}

/** One RPC reply (`{id, ok, value|error}`). */
export interface ZaWorkerResponse {
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
 * Spawn the real wasm-backed zero-access worker and return a [`ZeroAccessCrypto`]
 * proxy. Each call posts `{id, method, args}` and resolves on the matching reply; a
 * worker crash rejects every in-flight call. The wasm module loads lazily inside the
 * worker on first use, so constructing the worker is cheap.
 */
export function spawnZeroAccessWorker(): ZeroAccessCrypto {
  const worker = new Worker(new URL('./worker.entry.ts', import.meta.url), { type: 'module' });
  const pending = new Map<number, Pending>();
  let seq = 0;

  worker.onmessage = (ev: MessageEvent<ZaWorkerResponse>): void => {
    const { id, ok, value, error } = ev.data;
    const p = pending.get(id);
    if (p === undefined) return;
    pending.delete(id);
    if (ok) p.resolve(value);
    else p.reject(new Error(error ?? 'zero-access worker error'));
  };
  worker.onerror = (ev: ErrorEvent): void => {
    const err = new Error(ev.message !== '' ? ev.message : 'zero-access worker crashed');
    for (const p of pending.values()) p.reject(err);
    pending.clear();
  };

  function call<T>(method: keyof ZeroAccessCrypto, args: unknown): Promise<T> {
    const id = (seq += 1);
    return new Promise<T>((resolve, reject) => {
      pending.set(id, { resolve: resolve as (value: unknown) => void, reject });
      const req: ZaWorkerRequest = { id, method, args };
      worker.postMessage(req);
    });
  }

  return {
    deriveRootKey: (i) => call('deriveRootKey', i),
    deriveKek: (i) => call('deriveKek', i),
    deriveSubkey: (i) => call('deriveSubkey', i),
    generateDataKey: () => call('generateDataKey', {}),
    wrapKey: (i) => call('wrapKey', i),
    unwrapKey: (i) => call('unwrapKey', i),
    sealRow: (i) => call('sealRow', i),
    openRow: (i) => call('openRow', i),
    recoveryPhrase: (i) => call('recoveryPhrase', i),
    restoreFromPhrase: (i) => call('restoreFromPhrase', i),
    pairGenerate: () => call('pairGenerate', {}),
    pairSeal: (i) => call('pairSeal', i),
    pairComplete: (i) => call('pairComplete', i),
    lock: (i) => call('lock', i),
    lockAll: () => call('lockAll', {}),
  };
}
