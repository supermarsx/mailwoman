// The crypto Web Worker entry (plan §2.3 / §2.5). Hosts the wasm-pack `mw-crypto`
// module and runs the RPC loop for `worker.ts`'s `spawnCryptoWorker` proxy. ALL
// private-key operations (keygen / decrypt / private sign / PKCS#12 import and
// export / certification request / unlock) run HERE, off the main thread; plaintext + private material never enter
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
import initSanitize, { sanitizeEmailHtml, sanitizeEmailHtmlReader } from '../wasm/mw-sanitize/mw_sanitize.js';
import type { CryptoWorkerRequest, CryptoWorkerResponse } from './worker.ts';
import { sanitizeDecryptResult, type RawDecryptResult } from './sanitize.ts';

// wasm-bindgen exports are synchronous (return a value or throw). `init()` must
// resolve before any of them is called.
type WasmFn = (options: unknown) => unknown;
const METHODS: Record<string, WasmFn> = {
  verifySmime: mw.verifySmime,
  smimeRootSetInfo: mw.smimeRootSetInfo,
  generateKey: mw.generateKey,
  encrypt: mw.encrypt,
  decrypt: mw.decrypt,
  sign: mw.sign,
  verify: mw.verify,
  importPkcs12: mw.importPkcs12,
  exportPkcs12: mw.exportPkcs12,
  certificateRequest: mw.certificateRequest,
  attachIssuedCert: mw.attachIssuedCert,
  importArmored: mw.importArmored,
  exportPublic: mw.exportPublic,
  exportBackup: mw.exportBackup,
  unlockKey: mw.unlockKey,
  lockKey: mw.lockKey,
  inspectKey: mw.inspectKey,
  verifyPgp: mw.verifyPgp,
  revokeKey: mw.revokeKey,
  revocationCertificate: mw.revocationCertificate,
  applyRevocationCertificate: mw.applyRevocationCertificate,
  setKeyExpiry: mw.setKeyExpiry,
  addSubkey: mw.addSubkey,
  revokeSubkey: mw.revokeSubkey,
  changePassphrase: mw.changePassphrase,
  exportBackupFull: mw.exportBackupFull,
  restoreBackup: mw.restoreBackup,
  pgpMimeEncrypt: mw.pgpMimeEncrypt,
  pgpMimeSign: mw.pgpMimeSign,
  pgpMimeOpen: mw.pgpMimeOpen,
  pgpMimeVerify: mw.pgpMimeVerify,
  analyzeMessage: mw.analyzeMessage,
  autocryptHeader: mw.autocryptHeader,
  parseAutocrypt: mw.parseAutocrypt,
  autocryptRecommend: mw.autocryptRecommend,
  detectAttachedKeys: mw.detectAttachedKeys,
  wkdUrls: mw.wkdUrls,
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
  if (method === 'sanitizeReader') {
    const args = rawArgs as { html?: unknown; blockTrackers?: unknown; cleanLinks?: unknown };
    if (typeof args?.html !== 'string' || args.html.length > 8 * 1024 * 1024 || typeof args.blockTrackers !== 'boolean' || typeof args.cleanLinks !== 'boolean') throw new Error('Invalid reader sanitizer request');
    await ensureSanitize();
    return sanitizeEmailHtmlReader(args.html, args.blockTrackers, args.cleanLinks);
  }
  const fn = METHODS[method];
  if (fn === undefined) throw new Error(`unknown crypto method: ${method}`);
  const args = normalizeArgs(method, (rawArgs ?? {}) as Record<string, unknown>);
  const { readerPolicy, ...cryptoArgs } = args;
  const value = fn(cryptoArgs);
  if (method === 'decrypt') {
    await ensureSanitize();
    const policy = readerPolicy as { blockTrackers?: unknown; cleanLinks?: unknown } | undefined;
    const sanitize = policy === undefined ? sanitizeEmailHtml : (html: string) => sanitizeEmailHtmlReader(html, policy.blockTrackers !== false, policy.cleanLinks !== false);
    return sanitizeDecryptResult(value as RawDecryptResult, sanitize);
  }
  return normalizeResult(method, value);
}

self.onmessage = (ev: MessageEvent<CryptoWorkerRequest>): void => {
  const { id, method } = ev.data;
  void (method === 'sanitizeReader' ? Promise.resolve() : ensureCrypto())
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
