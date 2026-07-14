# Mailwoman — key-management module strings (source locale: en).
#
# Lazily loaded by the keys module (`loadCatalog('keys')`). Ids are kebab-case and
# module-prefixed (keys-*). Wording is intentionally verbatim with the pre-i18n
# UI — this is a security/keys surface, so strings are factual, not editorialised.
#
# Untrusted values (key owner emails, fingerprints) are wrapped in a bidi isolate
# at the call site via `isolate()`, NOT here (SPEC §24).

# -- Panel header ------------------------------------------------------------
keys-panel-label = Key management
keys-title = Keys & certificates
keys-subtitle = OpenPGP and S/MIME keys. Private keys stay on this device and never reach the server.
keys-generate = Generate key
keys-import = Import key

# -- Key list groups ---------------------------------------------------------
keys-your-keys = Your keys
keys-empty-own = No keys yet. Generate or import one to start.
keys-contact-keys = Contact keys
keys-empty-contact = No contact keys. Look one up below.
keys-loading = Loading keys…
keys-select-prompt = Select a key to view and verify it.
keys-untitled = { $kind } key

# -- Trust states ------------------------------------------------------------
keys-trust-unverified = unverified
keys-trust-tofu = tofu
keys-trust-verified = verified
keys-trust-revoked = revoked

# -- Key detail --------------------------------------------------------------
keys-key-card = Key { $name }
keys-fingerprint = Fingerprint
keys-safe-words = Safe words
keys-safe-words-help = Read these aloud with the contact to confirm the key out-of-band.
keys-safe-words-list = Fingerprint safe words
keys-scan-label = Scan to verify
keys-qr-label = Fingerprint QR code
keys-autocrypt = Autocrypt
keys-autocrypt-on = Advertised in Autocrypt headers
keys-autocrypt-off = Not advertised via Autocrypt
keys-trust-label = Trust
keys-trust-level = Trust level
keys-associate-label = Associate with a contact
keys-associate-select = Contact to associate
keys-choose-contact = Choose a contact…
keys-associate-button = Associate
keys-backup-label = Backup
keys-backup-help = Export an Autocrypt Setup Message to move this key to another device.
keys-export-backup = Export backup
keys-asm-label = Autocrypt Setup Message
keys-no-private-backup = No private key held on this device for backup

# -- Consent-gated lookup ----------------------------------------------------
keys-lookup-form = Look up a contact key
keys-lookup-heading = Look up a key
keys-lookup-address = Address to look up
keys-lookup-placeholder = name@example.org
keys-lookup-sources = Lookup sources
keys-source = Source { $source }
keys-consent-label = Consent to external lookup
keys-consent-text = Looking a key up contacts external directories (WKD/VKS). I have this person's consent to do so.
keys-lookup-button = Look up
keys-looking-up = Looking up…
keys-lookup-found = Found { $count } key(s) — added to Contact keys.
keys-lookup-none = No key found.

# -- Generate dialog ---------------------------------------------------------
keys-generate-title = Generate a key
keys-type = Type
keys-key-type = Key type
keys-openpgp = OpenPGP
keys-smime = S/MIME
keys-name = Name
keys-email = Email
keys-key-passphrase = Key passphrase
keys-passphrase-help = The passphrase wraps the private key on this device. It never leaves the browser.
keys-cancel = Cancel
keys-generate-submit = Generate
keys-generating = Generating…

# -- Import dialog -----------------------------------------------------------
keys-import-title = Import a key
keys-import-type = Import type
keys-tab-armored = Armored (PGP)
keys-tab-pkcs12 = PKCS#12 (S/MIME)
keys-armored-key = Armored key
keys-armored-placeholder = -----BEGIN PGP PUBLIC KEY BLOCK-----
keys-armored-passphrase-label = Passphrase (if the key is private)
keys-import-passphrase = Import passphrase
keys-pkcs12-file-label = PKCS#12 file (.p12 / .pfx)
keys-pkcs12-file = PKCS#12 file
keys-import-password-label = Import password
keys-pkcs12-password = PKCS#12 password
keys-import-preview = Import preview
keys-preview = Preview
keys-preview-type = Type: { $kind }
keys-preview-fingerprint-aria = Preview fingerprint
keys-preview-fingerprint = Fingerprint: { $fp }
keys-preview-has-private = Includes a private key (will be stored on this device)
keys-preview-public-only = Public key only
keys-import-submit = Import
