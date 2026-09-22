// What the app does when the crypto Worker CANNOT be constructed (t24-e13).
//
// This is not hypothetical. Between 26.17 (which shipped
// `require-trusted-types-for 'script'`) and 26.20 t24-e13 (which gave the default
// Trusted Types policy a `createScriptURL`), `new Worker(...)` was refused by the
// app's own CSP in every shipped release — so the crypto/sanitize worker never
// started at all. The question this file answers, and pins, is what happened to
// DECRYPTED E2EE mail in that window.
//
// The answer is FAIL CLOSED, and the reason is stronger than "the fallback was
// safe": there is no fallback, and no plaintext is ever produced. The throw
// happens at `new Worker()` — construction — which is BEFORE any decryption has
// run, so there is no plaintext in existence to leak or to render. Specifically:
//
//   1. `new Worker(new URL('./worker.entry.ts', import.meta.url))` throws
//      (crypto/worker.ts) — asserted below against the real Chrome message.
//   2. `createLazyCryptoWorker`'s `get()` (crypto/index.ts) has no try/catch, so
//      the throw propagates out of `getCryptoWorker().decrypt(...)`.
//   3. `Reader.tsx#decryptNow` awaits that call inside a `try`; its `catch` only
//      calls `setError(...)`, which renders a `role="alert"` message.
//   4. `props.onDecrypted(...)` sits AFTER the awaited decrypt inside the same
//      `try`, so it never runs — no decrypted content reaches `setDecryptedHtml`
//      / `setDecryptedText`, the only signals that render it.
//
// In particular decrypted plaintext was NOT routed to the server-side sanitizer.
// The app has exactly one `client.sanitize()` call (state/slices/mail.ts), and it
// is fed `extractHtmlBody(email)` — the body as fetched FROM the server, which the
// server already holds. The decrypt path never touches `client`.
//
// The tests below pin all of that, so a future CSP, policy or refactor change
// cannot silently reroute decrypted content to the server or render it raw.

import { afterEach, describe, expect, it, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnCryptoWorker } from './worker.ts';

/** The exact message Chrome raises when a TrustedScriptURL sink has no policy —
 *  the one t24-e12 recovered from the e2e-crypto Playwright trace. */
const TT_REFUSAL =
  "Failed to construct 'Worker': This document requires 'TrustedScriptURL' " +
  "assignment and no 'default' policy for 'TrustedScriptURL' has been defined.";

const realWorker = globalThis.Worker;
afterEach(() => {
  if (realWorker === undefined) delete (globalThis as { Worker?: unknown }).Worker;
  else globalThis.Worker = realWorker;
  vi.restoreAllMocks();
});

function src(path: string): string {
  // vitest runs with cwd = apps/web (the vite config dir).
  return readFileSync(resolve(process.cwd(), path), 'utf8');
}

describe('crypto worker refused by the CSP — fails closed', () => {
  it('spawnCryptoWorker throws at CONSTRUCTION, before any RPC is issued', () => {
    const ctor = vi.fn(() => {
      throw new TypeError(TT_REFUSAL);
    });
    globalThis.Worker = ctor as unknown as typeof Worker;

    expect(() => spawnCryptoWorker()).toThrow(/TrustedScriptURL/);
    // Construction was attempted exactly once and nothing else happened: no
    // postMessage, so no ciphertext was submitted and no plaintext came back.
    expect(ctor).toHaveBeenCalledTimes(1);
  });

  it('the lazy worker does NOT swallow the construction failure', () => {
    // crypto/index.ts memoizes `real ??= spawnCryptoWorker()` with no try/catch,
    // so the throw must reach the caller rather than yielding a degraded object
    // that silently does nothing (which is how a "render raw" path would appear).
    const text = src('src/crypto/index.ts');
    const lazy = text.slice(text.indexOf('function createLazyCryptoWorker'));
    const body = lazy.slice(0, lazy.indexOf('\n}'));
    expect(body).toContain('spawnCryptoWorker()');
    expect(body, 'a catch here would convert a hard failure into a silent one').not.toMatch(
      /catch\s*[({]/,
    );
  });
});

describe('decrypted plaintext never reaches the server sanitizer', () => {
  it('the decrypt path has no server-sanitize fallback', () => {
    const reader = src('src/components/Reader.tsx');
    const start = reader.indexOf('async function decryptNow');
    expect(start, 'decryptNow must still exist').toBeGreaterThan(-1);
    const decryptNow = reader.slice(start, reader.indexOf('\n  }', start));

    // The whole point: no route to the server from the decrypt path.
    expect(decryptNow).not.toMatch(/client\s*\.\s*sanitize/);
    expect(decryptNow).not.toContain('/api/sanitize');
    // And the success handoff is inside the try, after the awaited decrypt, so a
    // throw skips it entirely.
    const decryptAt = decryptNow.indexOf('.decrypt(');
    const handoffAt = decryptNow.indexOf('props.onDecrypted(');
    const catchAt = decryptNow.indexOf('} catch');
    expect(decryptAt).toBeGreaterThan(-1);
    expect(handoffAt).toBeGreaterThan(decryptAt);
    expect(handoffAt, 'onDecrypted must be inside the try, not the catch').toBeLessThan(catchAt);
    // The catch surfaces an error; it must not hand content onward.
    const catchBody = decryptNow.slice(catchAt);
    expect(catchBody).toContain('setError');
    expect(catchBody).not.toContain('onDecrypted');
  });

  it('the one client.sanitize() call is fed SERVER-fetched body HTML, not plaintext', () => {
    const mail = src('src/state/slices/mail.ts');
    const calls = [...mail.matchAll(/client\s*\.\s*sanitize\(([^)]*)\)/g)];
    expect(calls, 'exactly one server-sanitize call site in the app').toHaveLength(1);
    // Its argument is the body extracted from the JMAP Email the server returned.
    expect(calls[0]?.[1]).toBe('raw');
    const ctx = mail.slice(Math.max(0, (calls[0]?.index ?? 0) - 400), calls[0]?.index);
    expect(ctx).toContain('extractHtmlBody(email)');
    expect(ctx).toContain('client.jmap(');
  });
});
