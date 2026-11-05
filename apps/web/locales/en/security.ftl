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

# Fresh confirmation for sensitive changes.
security-stepup-title = Confirm it is you
security-stepup-body = Confirm your identity to continue this action. Confirmation lasts five minutes in this session.
security-stepup-password = Password
security-stepup-code = Authenticator code
security-stepup-confirm = Confirm and continue
security-stepup-passkey = Use a passkey
security-stepup-cancel = Cancel
security-stepup-failed = Confirmation failed. Please try again.
security-stepup-unavailable = This session has no available confirmation method.
security-center-title = Security centre
security-center-history = Sign-in history
security-center-result = Result
security-center-all = All results
security-center-success = Successful
security-center-failure = Failed
security-center-blocked = Blocked
security-center-refresh = Refresh
security-center-export = Export history
security-center-devices = Remembered devices
security-center-remember-help = If your administrator permits it, remember this device after verifying your identity. Sensitive changes still require fresh verification.
security-center-label = Device name
security-center-remember = Remember this device
security-center-revoke = Revoke
security-center-activity = Account activity
security-center-error = Security information could not be loaded or changed. Your administrator may have disabled this action.
security-center-first-seen = First seen: { $when }
security-center-passkey-mode = Passkeys can sign you in without a password in engine mode. In proxy mode, they verify a second factor after your mail password.
security-stepup-recovery = Recovery code (with your password, instead of an authenticator code)

security-center-signin = Sign-in & passkeys
security-center-sessions = Sessions & access
security-center-encryption = Encryption

signin-geo-title = Sign-in country policy
signin-geo-help = Optionally refuse new sign-ins to your account from countries you have not used before, or from any country outside your allowed list. It is off by default and never affects sessions that are already signed in. Allowed IPs and countries always pass this check; your password and second factor are still required.
signin-geo-accuracy = Country detection uses an IP geolocation database that can be wrong or out of date. VPNs, mobile carriers, proxies and travel can block a legitimate sign-in or let a foreign one through. Add trusted IP addresses or countries to the allowlist.
signin-geo-locked = An administrator controls this setting for your account.
signin-geo-mode = Policy
signin-geo-mode-off = Off
signin-geo-mode-block-unusual-countries = Block countries I have not signed in from
signin-geo-mode-block-foreign-countries = Block countries outside my allowed list
signin-geo-countries = Allowed countries (two-letter codes, comma separated)
signin-geo-ips = Allowed IP addresses or networks (one per line)
signin-geo-current = Your current address is { $ip } (detected country: { $country }).
signin-geo-no-context = Your current address could not be determined.
signin-geo-unknown-country = unknown
signin-geo-add-ip = Add my current IP
signin-geo-add-country = Add my current country
signin-geo-save = Save
signin-geo-saved = Saved.
signin-geo-invalid = Check the country codes and IP addresses.
signin-geo-self-lockout = This would block sign-in from your current address or country. Add your current IP address or country to the allowlist first.
signin-geo-error = Could not save the sign-in country policy.

session-alerts-title = Session security notices
session-alerts-help = Warnings about unusual use of your signed-in sessions. Each notice shows what the server saw and what it did.
session-alerts-none = No session security notices.
session-alerts-banner = { $count ->
    [one] You have 1 unread session security notice.
   *[other] You have { $count } unread session security notices.
}
session-alerts-banner-review = Review
session-alerts-kind-countryChange = A session was used from a different country ({ $from } to { $to }).
session-alerts-kind-impossibleTravel = A session appeared in another country ({ $from } to { $to }) sooner than travel allows.
session-alerts-kind-concurrentUse = One session was used from different countries or networks at the same time ({ $from } and { $to }).
session-alerts-kind-fingerprintMismatch = A session was used from a browser or device that does not match the one that signed in.
session-alerts-kind-rotationReuse = An old session identifier was presented again after the session had been renewed.
session-alerts-action-alerted = Action: you were alerted.
session-alerts-action-step-up = Action: the session was asked to confirm your password and second factor.
session-alerts-action-revoked = Action: the session was signed out.
session-alerts-countries = Countries: { $from } to { $to }
session-alerts-unknown-country = unknown
session-alerts-session = Session { $id }
session-alerts-caveat = Country detection may be inaccurate; VPNs and mobile networks can cause false changes.
session-alerts-not-me = This wasn't me
session-alerts-dismiss = Dismiss
session-alerts-dismiss-all = Dismiss all
session-alerts-revoked = Your other sessions were signed out. Change your password in the password section of Sign-in & passkeys now.
session-alerts-error = The session security notices could not be updated.
session-alerts-prefs = Alert preferences
session-alerts-country = Warn me when a session is used from a new country
session-alerts-country-help = If the administrator does not allow muting, the server keeps alerting even when this is off.
session-alerts-channels = Where to receive warnings
session-alerts-channel-app = In the app
session-alerts-channel-email = In my mailbox
session-alerts-channel-both = In the app and in my mailbox
session-alerts-sensitivity = Sensitivity
session-alerts-sensitivity-low = Low: location findings only, no forced sign-out
session-alerts-sensitivity-normal = Normal
session-alerts-sensitivity-high = High: stricter travel check, also compares networks
session-alerts-reset = Use administrator defaults
session-alerts-pref-error = The preference could not be saved.

signin-geo-history-title = Countries seen
signin-geo-history-help = The countries you have signed in from successfully, kept for { $days } days. "Unusual countries" mode uses this list. Only country codes, dates and counts are stored, never addresses.
signin-geo-history-empty = No countries are recorded yet.
signin-geo-history-entry = { $country }: first { $first }, last { $last }, { $count } sign-ins
signin-geo-history-forget = Forget countries seen

# -- Explanations: headline is the security-anomaly-* / security-auth-* key --
security-severity-info = Informational
security-severity-caution = Caution
security-severity-suspicious = Suspicious
security-explain-toggle = What does this mean?
security-explain-label-checked = What was checked
security-explain-label-legit = Common harmless causes
security-explain-label-risk = What it can indicate
security-explain-label-action = Suggested action
security-explain-messageIdDomainAnomaly-severity = { security-severity-caution }
security-explain-messageIdDomainAnomaly-meaning = The domain after the @ in the Message-ID header was compared with the domain in the From address. They are different.
security-explain-messageIdDomainAnomaly-legit = Mailing lists, marketing and newsletter platforms, forwarding services, corporate gateways and webmail hosts often generate the Message-ID with their own server name.
security-explain-messageIdDomainAnomaly-risk = A forged sender, or a message built by software that is not the sender's own mail system. On its own this is a weak signal.
security-explain-messageIdDomainAnomaly-action = Look at the SPF, DKIM and DMARC results above. If they passed for the sender's domain, this is most likely routine. If they failed or are missing, avoid links and attachments and confirm with the sender another way.
security-explain-replyToMismatch-severity = { security-severity-caution }
security-explain-replyToMismatch-meaning = The Reply-To address was compared with the From address. Replies would go to a different address.
security-explain-replyToMismatch-legit = Mailing lists, support desks, assistants writing on someone's behalf and contact forms set a different Reply-To on purpose.
security-explain-replyToMismatch-risk = Replies being redirected to an address the attacker controls, a common pattern in impersonation and invoice fraud.
security-explain-replyToMismatch-action = Check the Reply-To address before replying. If the message asks for money, credentials or a change of details, confirm through a channel you already trust.
security-explain-envelopeFromDivergence-severity = { security-severity-info }
security-explain-envelopeFromDivergence-meaning = The envelope sender (Return-Path), which is where bounces go, was compared with the From address. They are different.
security-explain-envelopeFromDivergence-legit = Bulk mail services, ticketing systems and mailing lists use their own bounce address while showing your contact's address in From.
security-explain-envelopeFromDivergence-risk = Sender spoofing, when SPF or DMARC also fails. Alone, it is common and usually harmless.
security-explain-envelopeFromDivergence-action = Check whether DMARC passed with alignment. If it did, no action is needed.
security-explain-dateSkew-severity = { security-severity-info }
security-explain-dateSkew-meaning = The Date header was compared with the time the message was received. They are far apart.
security-explain-dateSkew-legit = A sender computer with a wrong clock, a message held in a queue or retried for a long time, or a scheduled send.
security-explain-dateSkew-risk = A replayed or backdated message. This is uncommon and rarely harmful by itself.
security-explain-dateSkew-action = Treat time-sensitive requests in the message with care and confirm them if they matter.
security-explain-punycodeSender-severity = { security-severity-suspicious }
security-explain-punycodeSender-meaning = The sender domain contains punycode (an xn-- label), which is how non-Latin characters are written in domain names.
security-explain-punycodeSender-legit = Genuine domains in languages such as Arabic, Chinese, Cyrillic or Portuguese with accents are encoded this way.
security-explain-punycodeSender-risk = A look-alike domain that mixes characters from other alphabets to resemble a known brand or person.
security-explain-punycodeSender-action = Compare the sender domain character by character with the one you expect. Do not click links or open attachments until you have confirmed the sender another way, and report it if it is not genuine.
security-explain-auth-fail-severity = { security-severity-suspicious }
security-explain-auth-fail-meaning = The receiving server checked this method and the result did not match what the sender's domain publishes.
security-explain-auth-fail-legit = Forwarding and mailing lists can break SPF or DKIM, and a misconfigured sending service can fail either.
security-explain-auth-fail-risk = The message may not come from who it claims, or was changed on the way.
security-explain-auth-fail-action = Do not trust the sender on this evidence. Avoid links and attachments and verify through another channel.
security-explain-auth-permerror-severity = { security-severity-caution }
security-explain-auth-permerror-meaning = The check could not be completed because the domain's published record is invalid or too complex.
security-explain-auth-permerror-legit = A mistake in the sender's DNS records.
security-explain-auth-permerror-risk = Nothing about the message itself, but the sender's domain cannot be verified.
security-explain-auth-permerror-action = Treat it like a missing result and check the other methods.
security-explain-auth-none-severity = { security-severity-info }
security-explain-auth-none-meaning = No result for this method was found in the message headers.
security-explain-auth-none-legit = The sender's domain does not publish this method, or the receiving server does not record it.
security-explain-auth-none-risk = The domain gives no way to verify the sender, so spoofing cannot be ruled out.
security-explain-auth-none-action = Rely on the other methods and on whether you expect this message.
security-explain-auth-neutral-severity = { security-severity-info }
security-explain-auth-neutral-meaning = The domain published a record for this method but made no claim about this message.
security-explain-auth-neutral-legit = A domain that is still setting up its policy.
security-explain-auth-neutral-risk = Little. The sender could not be confirmed or rejected.
security-explain-auth-neutral-action = Rely on the other methods and on whether you expect this message.
security-explain-auth-temperror-severity = { security-severity-info }
security-explain-auth-temperror-meaning = The check could not finish because of a temporary problem, usually a DNS lookup failure.
security-explain-auth-temperror-legit = A short outage at the sender's or receiver's DNS.
security-explain-auth-temperror-risk = Nothing about the message itself.
security-explain-auth-temperror-action = No action is usually needed. Check again later if the message is important.
