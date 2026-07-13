// The WASM crypto Web Worker host (plan §2.5 / §2.3 / risk #12) — spawns the
// wasm-pack `mw-crypto` module in a dedicated Worker so private-key operations
// (keygen, decrypt, private sign, PKCS#12 import) NEVER block the main thread and
// the plaintext + private material NEVER enter the main app state (plan §1.2).
// The decrypted plaintext is sanitized IN-WORKER via the mw-sanitize wasm build
// before it is handed back for the sandboxed iframe (plan §1.3 / risk #5).
//
// e0 authors the host SEAM (the RPC message contract below + a not-yet-wired
// factory); e8 builds `apps/web/src/wasm/` via `scripts/build-wasm.*`, loads it in
// the Worker, and points `crypto/index.ts#getCryptoWorker` here. Until then the
// app uses the stub in `index.ts`.

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

/** The Worker script URL (the wasm-pack glue + this RPC loop; built by e8). */
export const CRYPTO_WORKER_URL = new URL('./worker.entry.ts', import.meta.url);

/**
 * Spawn the real wasm-pack-backed crypto worker and return a [`CryptoWorkerApi`]
 * proxy over the RPC contract above. NOT wired until e8 (the wasm bundle +
 * `worker.entry.ts` do not exist yet); calling it now throws so a premature use
 * is loud rather than silently wrong. `crypto/index.ts` uses the stub until e8
 * flips `getCryptoWorker()` to this factory.
 */
export function spawnCryptoWorker(): CryptoWorkerApi {
  throw new Error(
    'crypto worker is not wired yet (e8 builds the wasm bundle + worker RPC loop); use the stub from crypto/index.ts',
  );
}
