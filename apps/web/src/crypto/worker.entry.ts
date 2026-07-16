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
//
// §1.3 (in-worker sanitize): after `decrypt`, HTML plaintext is sanitized HERE via
// the `mw-sanitize` wasm build (`sanitizeEmailHtml`) before it is returned — decrypted
// end-to-end-encrypted plaintext is NEVER round-tripped to the server sanitizer. The
// mw-sanitize wasm loads lazily too (only when a message is decrypted), so it adds
// nothing to the mail critical path.

import init, * as mw from '../wasm/mw-crypto/mw_crypto.js';
import initSanitize, { sanitizeEmailHtml } from '../wasm/mw-sanitize/mw_sanitize.js';
import type { CryptoWorkerRequest, CryptoWorkerResponse } from './worker.ts';
import { sanitizeDecryptResult, type RawDecryptResult } from './sanitize.ts';

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

// Each wasm module loads exactly once, lazily, on first use. `mw-crypto` inits on the
// first RPC (its `start` hook installs the panic→console handler); `mw-sanitize` inits
// only when a `decrypt` result needs HTML sanitizing, so it never touches the mail
// critical path.
let cryptoReady: Promise<unknown> | null = null;
function ensureCrypto(): Promise<unknown> {
  cryptoReady ??= init();
  return cryptoReady;
}
let sanitizeReady: Promise<unknown> | null = null;
function ensureSanitize(): Promise<unknown> {
  sanitizeReady ??= initSanitize();
  return sanitizeReady;
}

// Some frozen §2.3 args omit a field the wasm DTO requires; normalize here so the
// app-side interface (`contracts/crypto.ts`) stays untouched.
function normalizeArgs(method: string, args: Record<string, unknown>): Record<string, unknown> {
  // wasm `unlockKey` requires a `kind`; the app's `UnlockKeyRequest` omits it.
  if (method === 'unlockKey' && args['kind'] === undefined) return { ...args, kind: 'pgp' };
  return args;
}

// The wasm boundary shapes some results differently from the frozen §2.3 return
// types; unwrap them here so the app-side interface (`contracts/crypto.ts`) stays
// untouched. Currently just `unlockKey`: the wasm `UnlockOut` marshals to a
// `{ keyRef }` object, but the frozen `CryptoWorkerApi.unlockKey` resolves to the
// bare `KeyRef` (string). Without this, `Compose` would pass the whole object as
// `signWithKeyRef` and the wasm `encrypt` (`sign_with_key_ref: Option<String>`)
// panics `expected a string`, breaking encrypt+sign.
function normalizeResult(method: string, value: unknown): unknown {
  if (
    method === 'unlockKey' &&
    typeof value === 'object' &&
    value !== null &&
    'keyRef' in value
  ) {
    return (value as { keyRef: unknown }).keyRef;
  }
  return value;
}

// Run one RPC method: dispatch to the wasm export, then — for `decrypt` — route the
// plaintext through the in-worker mw-sanitize wasm (HTML sanitized before it leaves
// the worker; non-HTML kept as escaped text). See `sanitize.ts` (plan §1.3).
async function runMethod(method: string, rawArgs: unknown): Promise<unknown> {
  const fn = METHODS[method];
  if (fn === undefined) throw new Error(`unknown crypto method: ${method}`);
  const args = normalizeArgs(method, (rawArgs ?? {}) as Record<string, unknown>);
  const value = fn(args);
  if (method === 'decrypt') {
    await ensureSanitize();
    return sanitizeDecryptResult(value as RawDecryptResult, sanitizeEmailHtml);
  }
  return normalizeResult(method, value);
}

self.onmessage = (ev: MessageEvent<CryptoWorkerRequest>): void => {
  const { id, method } = ev.data;
  void ensureCrypto()
    .then(() => runMethod(method, ev.data.args))
    .then((value) => {
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
