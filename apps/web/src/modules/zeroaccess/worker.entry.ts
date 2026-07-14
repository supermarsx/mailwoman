// The zero-access crypto Worker entry (plan §3 e8). Hosts the wasm-pack `mw-crypto`
// module and runs the RPC loop for `worker.ts`'s `spawnZeroAccessWorker` proxy. ALL
// key derivation / seal / open / device-pairing runs HERE via the `za*` wasm exports
// (built by e6, `crates/mw-crypto/src/zeroaccess.rs`), off the main thread; derived
// keys stay in the worker session and no plaintext key crosses back to the app.
//
// The wasm bundle is produced by `scripts/build-wasm.*` into `src/wasm/mw-crypto`. Its
// `init()` loads the `.wasm` lazily on the first RPC, so the ~4.5 MB module stays off
// the login→inbox critical path. This entry imports the SAME bundle the V4 crypto
// worker uses — no duplicated crypto.

import init, * as mw from '../../wasm/mw-crypto/mw_crypto.js';
import { ZA_METHODS } from './crypto.ts';
import type { ZeroAccessCrypto } from './crypto.ts';
import type { ZaWorkerRequest, ZaWorkerResponse } from './worker.ts';

type WasmFn = (options: unknown) => unknown;
type WasmNoArg = () => unknown;
const mwAny = mw as unknown as Record<string, WasmFn & WasmNoArg>;

// Map each RPC method → its `za*` wasm export (see `ZA_METHODS`).
function invoke(method: keyof ZeroAccessCrypto, args: unknown): unknown {
  const exportName = ZA_METHODS[method];
  const fn = mwAny[exportName];
  if (fn === undefined) throw new Error(`unknown zero-access method: ${method}`);
  // `zaGenerateDataKey` / `zaPairGenerate` / `zaLockAll` take no argument.
  if (method === 'generateDataKey' || method === 'pairGenerate' || method === 'lockAll') {
    return (fn as WasmNoArg)();
  }
  return fn(args ?? {});
}

let cryptoReady: Promise<unknown> | null = null;
function ensureCrypto(): Promise<unknown> {
  cryptoReady ??= init();
  return cryptoReady;
}

self.onmessage = (ev: MessageEvent<ZaWorkerRequest>): void => {
  const { id, method, args } = ev.data;
  void ensureCrypto()
    .then(() => invoke(method, args))
    .then((value) => {
      const reply: ZaWorkerResponse = { id, ok: true, value };
      self.postMessage(reply);
    })
    .catch((err: unknown) => {
      const reply: ZaWorkerResponse = {
        id,
        ok: false,
        error: err instanceof Error ? err.message : String(err),
      };
      self.postMessage(reply);
    });
};
