// Fingerprint "safe words" (plan §3 e2 — trust/verify). A key fingerprint is 40
// hex characters — hard to compare aloud or over a call. We render it as a short
// sequence of pronounceable words so two people can *say* the fingerprint to each
// other to confirm a key out-of-band (the TOFU "verify" step).
//
// The encoding is Proquints (Wilkerson, "A Proposal for Proquint Identifiers"):
// each 16-bit group maps deterministically to one 5-letter pronounceable word
// (consonant-vowel-consonant-vowel-consonant). It is a published, reversible,
// language-neutral scheme — NOT ad-hoc — so the same fingerprint always yields the
// same words on every device, which is exactly what an out-of-band compare needs.

/** Proquint consonants (4 bits each) — index 0..15. */
const CONSONANTS = 'bdfghjklmnprstvz';
/** Proquint vowels (2 bits each) — index 0..3. */
const VOWELS = 'aiou';

/** Encode one 16-bit value as a 5-letter proquint (c v c v c). */
function proquint(word: number): string {
  const c1 = (word >> 12) & 0xf;
  const v1 = (word >> 10) & 0x3;
  const c2 = (word >> 6) & 0xf;
  const v2 = (word >> 4) & 0x3;
  const c3 = word & 0xf;
  return (
    CONSONANTS[c1]! + VOWELS[v1]! + CONSONANTS[c2]! + VOWELS[v2]! + CONSONANTS[c3]!
  );
}

/**
 * Turn a hex fingerprint into safe words — one proquint per 16 bits (4 hex
 * chars). Non-hex characters (spaces, colons) are ignored; a trailing partial
 * group is zero-padded so any-length input still yields stable words.
 */
export function fingerprintWords(fingerprint: string): string[] {
  const hex = fingerprint.replace(/[^0-9a-fA-F]/g, '');
  if (hex.length === 0) return [];
  const words: string[] = [];
  for (let i = 0; i < hex.length; i += 4) {
    const chunk = hex.slice(i, i + 4).padEnd(4, '0');
    words.push(proquint(Number.parseInt(chunk, 16)));
  }
  return words;
}

/** Group a fingerprint into readable 4-char blocks (e.g. `ABCD 1234 …`). */
export function groupFingerprint(fingerprint: string, group = 4): string {
  const hex = fingerprint.replace(/[^0-9a-fA-F]/g, '').toUpperCase();
  const parts: string[] = [];
  for (let i = 0; i < hex.length; i += group) parts.push(hex.slice(i, i + group));
  return parts.join(' ');
}
