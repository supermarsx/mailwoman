// The crypto Web Worker entry (plan §2.3 / §2.5). Hosts the wasm-pack `mw-crypto`
// module and runs the RPC loop for `worker.ts`'s `spawnCryptoWorker` proxy. ALL
// private-key operations (keygen / decrypt / private sign / PKCS#12 import /
// unlock) run HERE, off the main thread; plaintext + private material never enter
// the main app state or the server in plaintext (plan §1.2 / risk #4). Errors are
// marshalled back as strings and re-thrown on the app side.
//
// The `mw-crypto` wasm bundle is produced by `scripts/build-wasm.*` into
// `src/wasm/mw-crypto`. Its `init()` loads the `.wasm` lazily on the first RPC, so
// the heavy (~4.5 MB) module stays off the login→inbox critical path (risk #12).

import init, * as mw from '../wasm/mw-crypto/mw_crypto.js';
import type { CryptoWorkerRequest, CryptoWorkerResponse } from './worker.ts';

// wasm-bindgen exports are synchronous (return a value or throw). `init()` must
// resolve before any of them is called.
type WasmFn = (options: unknown) => unknown;
const METHODS: Record<string, WasmFn> = {
  generateKey: mw.generateKey,
  encrypt: mw.encrypt,
  decrypt: mw.decrypt,
  sign: mw.sign,
  verify: mw.verify,
  importPkcs12: mw.importPkcs12,
  importArmored: mw.importArmored,
  exportPublic: mw.exportPublic,
  exportBackup: mw.exportBackup,
  unlockKey: mw.unlockKey,
  lockKey: mw.lockKey,
};

// The wasm module loads exactly once, lazily, on the first RPC. Subsequent calls
// await the same promise (`__init` — the wasm-bindgen `start` hook — installs the
// panic→console handler during init).
let ready: Promise<unknown> | null = null;
function ensureReady(): Promise<unknown> {
  ready ??= init();
  return ready;
}

// Some frozen §2.3 args omit a field the wasm DTO requires; normalize here so the
// app-side interface (`contracts/crypto.ts`) stays untouched.
function normalizeArgs(method: string, args: Record<string, unknown>): Record<string, unknown> {
  // wasm `unlockKey` requires a `kind`; the app's `UnlockKeyRequest` omits it.
  if (method === 'unlockKey' && args['kind'] === undefined) return { ...args, kind: 'pgp' };
  return args;
}

self.onmessage = (ev: MessageEvent<CryptoWorkerRequest>): void => {
  const { id, method } = ev.data;
  void ensureReady()
    .then(() => {
      const fn = METHODS[method];
      if (fn === undefined) throw new Error(`unknown crypto method: ${method}`);
      const args = (ev.data.args ?? {}) as Record<string, unknown>;
      const value = fn(normalizeArgs(method, args));
      const reply: CryptoWorkerResponse = { id, ok: true, value };
      self.postMessage(reply);
    })
    .catch((err: unknown) => {
      const reply: CryptoWorkerResponse = {
        id,
        ok: false,
        error: err instanceof Error ? err.message : String(err),
      };
      self.postMessage(reply);
    });
};
