// In-worker sanitize of decrypted E2EE mail (plan §1.3 / risk #5).
//
// After `mw-crypto` decrypts a message CLIENT-SIDE, its plaintext may be HTML (mail
// composed by Thunderbird/Outlook/etc.) or plain text. Decrypted E2EE plaintext MUST
// NOT round-trip to the server sanitizer — that would defeat end-to-end encryption
// and pre-stage the V6 zero-access break. So the crypto Web Worker sanitizes any
// decrypted HTML HERE, in-browser, via the `mw-sanitize` wasm build (`sanitizeEmailHtml`,
// the SAME ammonia allowlist as the server) before it reaches the sandboxed body
// iframe. Non-HTML plaintext is returned as text and rendered escaped (Reader.tsx).
//
// The HTML-detection + routing below is pulled out of `worker.entry.ts` so it is unit-
// testable against the real wasm sanitizer without spinning up a Worker (which jsdom
// cannot host) — see `sanitize.test.ts` for the torture assertion (a decrypted
// `<script>`/onclick body is stripped by the in-worker wasm sanitize).

import type { DecryptResult } from '../contracts/crypto.ts';
import type { SignatureVerdict } from '../api/security-types.ts';

/** The raw shape `mw-crypto`'s wasm `decrypt` returns (plaintext not yet routed). */
export interface RawDecryptResult {
  plaintextText?: string;
  plaintextHtml?: string;
  subject?: string | null;
  signature: SignatureVerdict;
}

/**
 * Whether a decrypted plaintext body should be treated as HTML. True if it contains
 * an HTML-ish opening or closing tag (`<p>`, `<div …>`, `<script>`, `</p>`); a plain
 * body like `1 < 2 and 3 > 4` has no letter after `<`, so it stays text.
 */
export function looksLikeHtml(s: string): boolean {
  return /<[a-z][a-z0-9]*\b[^>]*>/i.test(s) || /<\/[a-z][a-z0-9]*\s*>/i.test(s);
}

/**
 * Route a raw decrypt result into the frozen [`DecryptResult`] (§2.3): HTML plaintext
 * is sanitized via `sanitizeHtml` (the `mw-sanitize` wasm) and returned as
 * `plaintextHtml`; non-HTML plaintext is returned as `plaintextText`. Exactly one of
 * the two is set. The sanitizer runs IN-WORKER — the decrypted plaintext never leaves
 * the client (plan §1.3).
 */
export function sanitizeDecryptResult(
  raw: RawDecryptResult,
  sanitizeHtml: (html: string) => string,
): DecryptResult {
  const base: { subject?: string; signature: SignatureVerdict } =
    raw.subject != null ? { subject: raw.subject, signature: raw.signature } : { signature: raw.signature };

  // If the worker already produced HTML, sanitize it; otherwise inspect the text.
  if (raw.plaintextHtml !== undefined) {
    return { ...base, plaintextHtml: sanitizeHtml(raw.plaintextHtml) };
  }
  const text = raw.plaintextText ?? '';
  if (looksLikeHtml(text)) {
    return { ...base, plaintextHtml: sanitizeHtml(text) };
  }
  return { ...base, plaintextText: text };
}
