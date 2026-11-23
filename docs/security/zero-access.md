# Zero-access storage

Zero-access storage is a per-account mode that a user turns on in their own settings. It sets up a key hierarchy that exists only on the user's devices, and it switches off several server features that would otherwise keep a readable form of the account's mail. This page states exactly what the mode does in the current code, and what it does not do.

::: warning What zero-access does not do today
Turning zero-access on does **not** make the server store the account's mail as ciphertext it cannot read. In engine mode the server still fetches the account's mail from the mail server and stores message bodies and list envelopes sealed with the **server key**, the same as for every other account (see [Encryption at rest](./encryption-at-rest.md)). Anyone who holds the server key, including the operator, can decrypt them. The client-side keys are created and stored, but nothing in the web app encrypts mail, notes or other data with them yet.

A copy of the database is therefore protected only when the server key is stored somewhere else; on a default SQLite installation the key file `<database>.server-key` sits next to the database. The key that zero-access creates (from its own passphrase or passkey; the server stores only a wrapped copy) encrypts no stored data yet.

The settings panel says the same: "It does not change how mail is stored: message bodies and envelopes are sealed with the server's key whether this mode is on or off, and the server can open them."
:::

## What turning it on does

![The top of the Zero-access storage panel, marked Off, stating that the mode keeps the account out of the search index and shared caches and that stored mail stays sealed with the server's key.](/screenshots/security/zero-access-panel.webp){.mw-crop}

When a user selects **Enable zero-access** under **Settings** > **Security centre** > **Encryption**, the following happens.

In the browser (inside a Web Worker running the WebAssembly build of `mw-crypto`):

1. A root key is derived from the chosen secret with Argon2id (19,456 KiB memory, 2 passes, 1 lane) and a random 16-byte salt. The secret is either a **Passphrase** of at least 8 characters, or the output of a passkey's WebAuthn PRF extension (**Passkey (passwordless)**).
2. A key-encryption key is derived from the root key, and a random 32-byte data key is generated.
3. The data key is wrapped (XChaCha20-Poly1305) under the key-encryption key.
4. The root key is shown once as a 33-word recovery phrase (32 words for the key bytes and one checksum word) under **Recovery phrase — save this offline now**.

The browser sends the server only the salt, the Argon2id parameters and the wrapped data key. The passphrase, the root key and the unwrapped data key are not sent.

On the server:

- The account is recorded as zero-access (table `zeroaccess_accounts`).
- The account's documents are removed from the full-text search index, and no new ones are written while the mode is on. Server-side search returns nothing for the account.
- Plaintext-derived values for the account (header windows, message bodies) are never placed in the memory cache or the Redis/Valkey cache; they are read from the store for each request.
- Sealed credentials for linked mailboxes are deleted, and new links that would store a credential are refused, for the account both as the owner and as the linked mailbox.
- Saving drafts to the server (account drafts) is refused for the account.
- AI Assist requests leave the account's content out, whatever the request asks for.
- Some security verdicts computed from stored mail are not cached for the account.

## What the server still holds and sees

With zero-access on, in engine mode the server still holds:

- Message bodies and list envelopes, sealed with the server key. An operator or an attacker with the server key can read them.
- Message metadata used for folders, threads and flags (sizes, dates, flags, folder membership, message and thread ids).
- Notes, contacts, calendars and other personal data, stored the same way as for any other account.
- The account's upstream mail credentials or tokens, as needed to sync and send.

The server also sees all mail as it passes through when it syncs from the mail server, sends, or proxies a request.

## Device pairing

**Device pairing** copies the root key to another device without giving it to the server:

1. The new device (**This is the new device**) creates a P-256 key pair and shows the public key as a QR code.
2. The existing device (**Pair another device from here**) reads the QR code, creates its own ephemeral P-256 key, derives a shared key by ECDH and seals the root key with it (XChaCha20-Poly1305). It uploads `ephemeral public key ‖ nonce ‖ ciphertext`.
3. The server stores that envelope and hands it to the new device, which derives the same shared key and opens it.
4. Both devices show six words derived (SHA-256) from both public keys and the ciphertext. The user compares them. Matching words mean no one replaced a key in between; if they differ, stop.

The server only relays the sealed envelope. A server that swaps keys during pairing is detected only if the user actually compares the six words.

## Recovery phrase

The recovery phrase is the root key itself, written as words. Anyone who has it can derive the account's keys. It is displayed once and not stored on the server. If the user loses the passphrase or passkey, every paired device and the recovery phrase, the client-side keys cannot be recovered. Because no stored mail is encrypted with those keys today, losing them does not make mail unreadable.

## Turning it off

**Disable zero-access** marks the account as standard again. Search indexing and caching resume for new and re-indexed mail. The wrapped key stays stored. Deleted linked-mailbox credentials are not restored; the links have to be signed in again.

## For administrators

The **Zero-access** column on the admin console's **Users** page is a record only. Ticking it does not encrypt anything and clearing it does not turn the user's mode off; only the user can turn zero-access on or off. The search index maintenance view reports how many zero-access accounts were left unindexed.

## Threat model summary

| Who | Can they read the account's stored mail? |
|---|---|
| Thief of the database file only (SQLCipher key and server key kept elsewhere) | No, as for any account (see [Encryption at rest](./encryption-at-rest.md)) |
| Thief of the database and the server key | Yes |
| Operator of the running server | Yes |
| Someone with access to the search index files | No index documents exist for the account |
| Someone with access to the Redis/Valkey cache | No plaintext-derived values are cached for the account |

## Related

- [Encryption at rest](./encryption-at-rest.md)
- [OpenPGP and S/MIME](./crypto.md) for end-to-end encrypted mail, which the server cannot read
- [Password change](./password-change.md)
- [Passkeys](./passkeys.md)
