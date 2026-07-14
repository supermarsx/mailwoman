# Mailwoman — security surface strings (source locale: en).
# Covers the Reader Security panel (SecurityPanel) AND the zero-access storage +
# device-pairing UX. Ids are kebab-case, prefixed `security-`.
#
# HONESTY: the zero-access disclosure copy below (security-za-protects,
# security-za-sees-*, security-za-active-server-caveat, security-za-no-search-claim,
# security-za-recovery-tradeoff) is a deliberate, narrow, factual statement. It is
# byte-identical to the canonical source in `src/modules/zeroaccess/disclosure.ts`
# and the honesty unit tests enforce that. Do NOT soften it into marketing wording
# ("ultra secure", "unbreakable", …) when translating — keep it accurate.

# -- Security panel: sections + empty states --------------------------------
security-panel-region = Message security details
security-section-auth = Authentication
security-section-received = Delivery path
security-section-signature = Signature
security-section-attachments = Attachments
security-section-warnings = Warnings
security-section-controls = Sender controls
security-received-empty = No Received chain available.
security-attachments-empty = No attachments.
security-hop-unknown = unknown
security-unknown = unknown

# -- Authentication (DKIM/SPF/DMARC/ARC) results ----------------------------
security-auth-pass = passed
security-auth-fail = failed
security-auth-none = not present
security-auth-neutral = neutral
security-auth-temperror = temporary error
security-auth-permerror = permanent error

# Expert alignment detail field labels + values.
security-detail-domain = domain
security-detail-selector = selector
security-detail-policy = policy
security-detail-alignment = alignment
security-detail-chain-length = chain length
security-aligned = aligned
security-not-aligned = not aligned

# -- Signature / certificate ------------------------------------------------
security-sig-verified = Signature verified
security-sig-unverified-key = Signed — signer key not verified
security-sig-invalid = Signature is invalid
security-sig-none = Not signed
security-chain-trusted = Trusted chain
security-chain-untrusted = Untrusted chain
security-chain-expired = Chain expired
security-chain-unknown = Chain unknown
security-revocation-good = Not revoked
security-revocation-revoked = Key revoked
security-revocation-unknown = Revocation unknown
security-key-changed = Signer key changed since last seen
security-fact-signer-key = Signer key
security-fact-algorithm = Algorithm
security-fact-key-created = Key created
security-fact-key-expires = Key expires
security-fact-chain = Chain
security-fact-revocation = Revocation
security-fact-key-change = Key change

# -- Attachment risk --------------------------------------------------------
security-attach-none = No known risk
security-attach-macro = Contains macros
security-attach-executable = Executable file
security-attach-encrypted-archive = Encrypted archive
security-attach-double-extension = Double file extension
security-attach-mismatch = type mismatch (declared { $declared }, detected { $detected })

# -- Anomalies --------------------------------------------------------------
security-anomaly-replyToMismatch = Reply-To address differs from the sender
security-anomaly-envelopeFromDivergence = Envelope sender differs from the From address
security-anomaly-messageIdDomainAnomaly = Message-ID domain doesn't match the sender
security-anomaly-dateSkew = Send date looks skewed
security-anomaly-punycodeSender = Sender uses punycode (possible look-alike domain)

# -- Sender controls --------------------------------------------------------
security-control-block = Block sender
security-control-silence = Silence sender
security-control-ignore-conversation = Ignore conversation
security-control-report-phishing = Report phishing
security-control-report-junk = Report junk
security-control-done-block = Sender blocked
security-control-done-silence = Sender silenced
security-control-done-ignore-conversation = Conversation ignored
security-control-done-report-phishing = Reported as phishing
security-control-done-report-junk = Reported as junk

# ===========================================================================
# Zero-access storage
# ===========================================================================
security-za-title = Zero-access storage
security-za-on = On
security-za-off = Off
security-za-protects = Zero-access encrypts your stored mail, notes, and PIM data so the hosting server keeps only ciphertext it cannot read. It defends the contents of your stored data against a curious operator or a breach of the storage host: anyone who reads the database, the on-disk files, or a stolen backup sees XChaCha20-Poly1305 ciphertext and cannot recover message bodies, subjects, attachments, note text, or the search index.

security-za-server-sees-title = What the server still sees
security-za-sees-1 = Ciphertext blobs — the encrypted rows themselves.
security-za-sees-2 = Opaque row IDs — the identifiers used to store and fetch rows.
security-za-sees-3 = Sizes — the length of each ciphertext (approximate message/attachment size).
security-za-sees-4 = Timestamps — when rows are written and updated.
security-za-sees-5 = Envelope routing metadata needed to proxy IMAP/SMTP — the server still connects to your upstream mail provider on your behalf, so connection metadata and the routing envelope required to send and receive mail pass through it.

security-za-active-server-caveat = Zero-access protects data AT REST against a curious or breached host. A fully malicious active server that proxies your live IMAP/SMTP traffic is a stronger adversary, and zero-access does NOT defend against it: such a server sits on the live connection to your mail provider and could observe or tamper with mail as it flows through, regardless of how the stored copy is encrypted. Choose zero-access when your concern is "I do not want whoever runs (or breaches) the storage host to read my stored mail." It is not a defense against an operator who subverts the live mail path.

security-za-no-search-claim = There is no server-side searchable encryption here, and no such claim is made. Search runs entirely on the client over content it has decrypted locally; the server never holds a searchable form of your plaintext.

security-za-recovery-tradeoff = Your keys are derived and held only on your devices — the server never receives a plaintext key. If you lose your passphrase (or passkey) AND every paired device AND your recovery phrase, the encrypted data cannot be recovered. Save the recovery phrase offline and treat it as equivalent to your account master secret.

security-za-setup-key = Set up your key
security-za-key-source = Key source
security-za-passphrase = Passphrase
security-za-passkey = Passkey (passwordless)
security-za-passkey-unavailable = passkeys are not available in this browser
security-za-passphrase-label = Zero-access passphrase
security-za-enable = Enable zero-access
security-za-recovery-title = Recovery phrase
security-za-recovery-heading = Recovery phrase — save this offline now
security-za-recovery-note = This is the only copy. Anyone who has it can read your data; without it (and without a paired device) your data cannot be recovered.
security-za-disable-section = Disable zero-access
security-za-disable-heading = Turn off zero-access
security-za-disable-note = New data will be stored unencrypted again. Existing encrypted data stays readable only while you can still derive your key.
security-za-disable-btn = Disable zero-access
security-za-err-passphrase-len = use a passphrase of at least 8 characters
security-za-err-enable = could not enable zero-access
security-za-err-disable = could not disable zero-access

# ===========================================================================
# Device pairing (QR + SAS)
# ===========================================================================
security-pair-title = Device pairing
security-pair-heading = Pair a device
security-pair-intro = Move your keys to another device without sending them through the server — it relays only an opaque sealed envelope. Compare the six words on both screens before trusting the pairing.
security-pair-role = Pairing role
security-pair-new-role = This is the new device
security-pair-existing-role = Pair another device from here
security-pair-show-qr = Show pairing QR
security-pair-scan = Scan this on your existing device
security-pair-qr-label = Device pairing QR code
security-pair-qr-default = Pairing QR code
security-pair-copy-code = Or copy this pairing code
security-pair-paste-envelope = Paste the sealed envelope from the other device
security-pair-envelope-placeholder = envelope…
security-pair-envelope-label = Sealed envelope
security-pair-complete = Complete pairing
security-pair-code-from-new = Pairing code from the new device
security-pair-code-placeholder = pairing code…
security-pair-code-label = Pairing code
security-pair-seal = Seal my keys for that device
security-pair-envelope-out = Sealed envelope — paste this on the new device
security-pair-compare = Compare these words on both devices
security-pair-sas-label = Short authentication string
security-pair-match = The words match
security-pair-abort = They differ — abort
security-pair-confirmed = Pairing confirmed. This channel is authenticated.
security-pair-err-start = could not start pairing
security-pair-err-complete = could not complete pairing
security-pair-err-unlock = unlock zero-access first to pair another device
security-pair-err-seal = could not seal the root key
