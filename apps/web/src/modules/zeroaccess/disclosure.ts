// The zero-access disclosure (SPEC §9.2). It states what the mode does and does not
// do, as the code behaves: the mode keeps the account out of the server's search
// index and shared caches and deletes linked-mailbox passwords; stored mail is sealed
// with the SERVER's key whether the mode is on or off, and the key the mode creates
// encrypts no stored data yet. Do NOT soften this into marketing ("ultra secure"
// etc.) or claim protection the code does not give. `locales/en/security.ftl`
// carries the same English (security-za-*); the honesty tests compare the two.

/** What zero-access does, and what it does not change about storage. */
export const ZA_PROTECTS =
  "Zero-access keeps this account's mail out of the server's search index and keeps " +
  'text taken from it out of shared caches. Turning it on also deletes the stored ' +
  'passwords of linked mailboxes. It does not change how mail is stored: message ' +
  "bodies and envelopes are sealed with the server's key whether this mode is on or " +
  'off, and the server can open them. A copy of the database is protected only if the ' +
  'server key is kept separately from it; on a default SQLite installation the key ' +
  'file sits next to the database.';

/** What the server still holds and can read (SPEC §9.2). */
export const ZA_SERVER_STILL_SEES: readonly string[] = [
  "Message bodies and envelopes — sealed with the server's own key, which the server " +
    'can use to open them.',
  'Mail in transit — the server connects to your mail provider for you and handles ' +
    'every message it fetches or sends.',
  'Sizes — the size of each message and attachment.',
  'Timestamps — when rows are written and updated.',
  'Envelope routing metadata needed to proxy IMAP/SMTP — the server still connects to ' +
    'your upstream mail provider on your behalf, so connection metadata and the routing ' +
    'envelope required to send and receive mail pass through it.',
];

/** The caveat: the server, its operator, and a database copy with the key are not defended against. */
export const ZA_ACTIVE_SERVER_CAVEAT =
  'Zero-access does NOT defend your mail against the server or whoever runs it. The ' +
  'server holds the key that seals stored mail, so its operator, an attacker who ' +
  'breaches the server, or anyone with a copy of the database and the server key can ' +
  'read it. The server also sits on the live connection to your mail provider and ' +
  'could observe or change mail as it flows through. Choose zero-access to keep this ' +
  "account out of the server's search index; it is not encryption that the server " +
  'cannot read.';

/** What happens to the search index; no searchable-encryption claim is made. */
export const ZA_NO_SEARCH_CLAIM =
  "When zero-access is turned on, the server removes this account's documents from " +
  'its search index and writes no new ones while the mode stays on. There is no ' +
  'server-side searchable encryption, and no such claim is made.';

/** The key the mode creates, and what it does not yet protect. */
export const ZA_RECOVERY_TRADEOFF =
  'Turning zero-access on creates a key on this device from your passphrase or ' +
  'passkey; the server stores only a wrapped copy and never receives the key itself. ' +
  'Stored mail is not encrypted with this key yet, so the key does not decide who can ' +
  'read your mail. Keep the recovery phrase offline: anyone who has it can recreate ' +
  'the key.';
