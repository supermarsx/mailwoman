# Mailwoman — compose crypto + DLP strings (source locale: en).
# Covers the ComposeCrypto subcomponents: the E2EE/TLS/mixed capability banner,
# the encrypt/sign toggles, and the DLP pre-send warnings. Ids prefixed `crypto-`.

# -- Capability banner ------------------------------------------------------
crypto-banner-e2ee-label = End-to-end encrypted
crypto-banner-e2ee-detail = Every recipient has a key — the message body is encrypted on this device.
crypto-banner-tls-label = Transport encryption (TLS)
crypto-banner-tls-detail = No recipient encryption keys were found; delivery is protected in transit only.
crypto-banner-mixed-label = Mixed protection
crypto-banner-mixed-detail = Some recipients can receive end-to-end encryption; others get TLS only.
crypto-banner-checking-label = Checking recipient keys
crypto-banner-checking = · checking…
crypto-banner-tls-only = TLS only for:

# -- Encrypt / sign toggles -------------------------------------------------
crypto-toggles-legend = Message security
crypto-encrypt-label = Encrypt (end-to-end)
crypto-encrypt-hint-default = Encrypt the body on this device before sending.
crypto-encrypt-drafted = Draft encrypted on this device.
crypto-encrypt-no-key = No recipient encryption key available.
crypto-sign-label = Sign (verify it's from you)
crypto-protect-subject = Also encrypt the subject line
crypto-reason-add-recipient = Add a recipient to check for encryption keys.
crypto-reason-tls = No recipient encryption key available — sending over TLS.

# -- DLP pre-send warnings --------------------------------------------------
crypto-dlp-aria = Data-loss prevention warnings
crypto-dlp-block = Sending blocked
crypto-dlp-require = Encryption required
crypto-dlp-warn = Heads up
crypto-dlp-notify = Administrator will be notified
crypto-dlp-matched = Matched: { $list }
