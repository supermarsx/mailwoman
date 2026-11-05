# libsqlite3-sys 0.30.1 - Mailwoman patch

Upstream: crates.io libsqlite3-sys 0.30.1 (rusqlite developers, MIT). The bundled SQLCipher
amalgamation under `sqlcipher/` (SQLCipher community edition, BSD-style licence, SQLite 3.45.3)
is unmodified on disk.

Why patched: upstream's `bundled-sqlcipher` links OpenSSL's libcrypto, which deny.toml bans
(SPEC 5.1, rustls only).

Changes (the only ones):
1. `build.rs`: the OpenSSL discovery/link block is removed. In a copy of the amalgamation
   written to OUT_DIR, the single line `#error "NO DEFAULT SQLCIPHER CRYPTO PROVIDER DEFINED"`
   is replaced by a call to `mw_sqlcipher_setup(p)`.
2. `src/provider.rs` (new, feature `bundled-sqlcipher`): that function, filling SQLCipher's
   crypto-provider table with RustCrypto AES-256-CBC, HMAC-SHA1/256/512, PBKDF2 and getrandom.
3. `Cargo.toml`: optional deps aes/cbc/hmac/sha1/sha2/pbkdf2/getrandom enabled by the feature
   (all already in Cargo.lock); `lib.rs` declares the module.
4. The non-SQLCipher bundled `sqlite3/` amalgamation is dropped (only its bindings remain).

Wired in through `[patch.crates-io]` in the workspace Cargo.toml. Tests live in
`crates/mw-store/tests/db_encryption.rs`.
