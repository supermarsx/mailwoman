// V6 zero-access storage module (SPEC §9, plan §2.6, §3 e6/e8). SCAFFOLD (t6-e0):
// inert placeholder types — importable + typecheck-green, NOT wired into any
// route yet. e6 fills the crypto/orchestration half (composing the existing
// `mw-crypto` WASM key stack into the §9.1 hierarchy: passphrase/passkey-PRF →
// root key → KEK → per-account data keys; row encrypt/decrypt; device-pairing
// QR + SAS). e8 fills the settings/UX half (enable/disable with the honest
// trade-off disclosure, passphrase/passkey setup, recovery phrase). No new client
// crypto — the existing crypto worker is reused.

/** Whether zero-access is enabled for an account, and what the server still sees. */
export interface ZeroAccessStatus {
  readonly enabled: boolean;
  /**
   * The HONEST list of what the server still sees at rest (SPEC §9.2, plan §1.4):
   * ciphertext blobs, opaque IDs, message sizes, timestamps, and the envelope
   * routing needed to proxy IMAP/SMTP. Zero-access protects data AT REST against
   * a curious/breached host; a malicious ACTIVE server proxying live traffic is a
   * stronger adversary and is NOT defended by this mode. No searchable-encryption
   * claim is made — search is a client-built encrypted slice (§9.3).
   */
  readonly serverVisibleMetadata: readonly string[];
}

/** Placeholder default status (disabled). e6/e8 replace this module wholesale. */
export const ZERO_ACCESS_DEFAULT: ZeroAccessStatus = {
  enabled: false,
  serverVisibleMetadata: [
    'ciphertext blobs',
    'opaque IDs',
    'message sizes',
    'timestamps',
    'envelope routing for IMAP/SMTP proxying',
  ],
};
