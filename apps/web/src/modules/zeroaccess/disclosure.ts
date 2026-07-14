// The HONEST zero-access tradeoff disclosure (plan §1.4 frozen wording, SPEC §9.2).
// This copy mirrors `docs/security/zero-access.md` verbatim in substance: it states
// exactly what the mode protects, what the server still sees, the malicious-active-
// server caveat, and the no-searchable-encryption claim. Do NOT soften this into
// marketing ("ultra secure" etc.) — it is a deliberate, narrow, factual statement
// shown at the point the user enables the mode.

/** What zero-access DOES protect (data at rest). */
export const ZA_PROTECTS =
  'Zero-access encrypts your stored mail, notes, and PIM data so the hosting server ' +
  'keeps only ciphertext it cannot read. It defends the contents of your stored data ' +
  'against a curious operator or a breach of the storage host: anyone who reads the ' +
  'database, the on-disk files, or a stolen backup sees XChaCha20-Poly1305 ciphertext ' +
  'and cannot recover message bodies, subjects, attachments, note text, or the search index.';

/** The exhaustive list of what the server still observes (SPEC §9.2). */
export const ZA_SERVER_STILL_SEES: readonly string[] = [
  'Ciphertext blobs — the encrypted rows themselves.',
  'Opaque row IDs — the identifiers used to store and fetch rows.',
  'Sizes — the length of each ciphertext (approximate message/attachment size).',
  'Timestamps — when rows are written and updated.',
  'Envelope routing metadata needed to proxy IMAP/SMTP — the server still connects to ' +
    'your upstream mail provider on your behalf, so connection metadata and the routing ' +
    'envelope required to send and receive mail pass through it.',
];

/** The honest caveat: a malicious ACTIVE server is out of scope (plan §1.4 / R5). */
export const ZA_ACTIVE_SERVER_CAVEAT =
  'Zero-access protects data AT REST against a curious or breached host. A fully ' +
  'malicious active server that proxies your live IMAP/SMTP traffic is a stronger ' +
  'adversary, and zero-access does NOT defend against it: such a server sits on the ' +
  'live connection to your mail provider and could observe or tamper with mail as it ' +
  'flows through, regardless of how the stored copy is encrypted. Choose zero-access ' +
  'when your concern is "I do not want whoever runs (or breaches) the storage host to ' +
  'read my stored mail." It is not a defense against an operator who subverts the live ' +
  'mail path.';

/** No searchable-encryption claim is made (plan §1.4). */
export const ZA_NO_SEARCH_CLAIM =
  'There is no server-side searchable encryption here, and no such claim is made. ' +
  'Search runs entirely on the client over content it has decrypted locally; the ' +
  'server never holds a searchable form of your plaintext.';

/** The recovery-phrase / lost-key tradeoff, stated at enable time. */
export const ZA_RECOVERY_TRADEOFF =
  'Your keys are derived and held only on your devices — the server never receives a ' +
  'plaintext key. If you lose your passphrase (or passkey) AND every paired device AND ' +
  'your recovery phrase, the encrypted data cannot be recovered. Save the recovery ' +
  'phrase offline and treat it as equivalent to your account master secret.';
