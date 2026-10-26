# Admin console: Security policy

The **Security policy** page (`#/security-policy`) edits the deployment-wide policy stored by
`PUT /admin/security-policy`. Every server process reads it again within 30 seconds. A field that an
environment variable sets is shown disabled with the variable's name; the server refuses a change to it
(409).

<a id="minTls"></a>
## Lowest TLS version

`1.2` (the default) or `1.3`. Applies to the HTTPS listener and to new outbound IMAP, SMTP, POP3,
ManageSieve and proxy-tunnel connections. Connections that are already open keep their version.
`MW_ADMIN_MIN_TLS` pins it.

<a id="require2fa"></a>
## Require a second factor for every account

The global two-factor rule (the global row of the two-factor policy). Accounts without a second factor
are asked to enrol at their next sign-in. Per-domain rules are on the **Two-factor** page.
`MW_ADMIN_REQUIRE_2FA` pins it.

<a id="argon2MCost"></a>
## Argon2 memory

Memory for each new password and recovery-code hash, in KiB: 19456 to 1048576 (the default is 19456).
With a single pass the server requires at least 47104 KiB. Existing hashes keep their cost until the
password changes.

<a id="argon2TCost"></a>
## Argon2 passes

Passes over the memory, 1 to 10 (default 2).

<a id="argon2PCost"></a>
## Argon2 parallelism

Threads per hash, 1 to 16 (default 1).

<a id="maxSecurityFloor"></a>
## Reading floor

The least restrictive view a Mailwoman client may use for a message: no floor (default), sanitised HTML
with media, sanitised HTML without remote media, or plain text. A person can choose a stricter view.
Clients read it from `GET /api/policy`; other mail clients do not apply it.

<a id="capturePolicy"></a>
## Screen capture protection

`off` (default) or `protect`. Mailwoman apps that can block screenshots and screen recording do so;
browsers cannot enforce it.

<a id="dlpRules"></a>
## Data loss prevention rules

Rules checked on outgoing mail sent through the server in engine mode, beside the rules in
`MW_DLP_RULES`. Each rule has a name, an on/off switch, a priority (higher first), an action (warn,
block, require encryption, or send and notify an administrator), an optional message for the sender,
and at least one condition: built-in detectors (payment card numbers, IBAN, national identity numbers,
US social security numbers, Portuguese NIF, NISS and Citizen Card/civil ID, and English/Portuguese
credential keywords), custom words or phrases, a custom regular expression, recipient domains (inside or outside the
list), attachment types and a largest attachment size in bytes. A classification set
outside the console is kept as stored. A rule from `MW_DLP_RULES` with the same id wins over a stored
rule; the page lists those rules read-only.

Portuguese detectors (`pt-nif`, `pt-niss`, `pt-citizen-card`) check local format and check digits;
they do not establish whether an identifier was issued. NIF and NISS accept common digit separators.
Citizen Card accepts the full document number, including its alphanumeric version, or the civil
number with its check digit. Full-document validation follows the
[AMA algorithm](https://www.autenticacao.gov.pt/documents/20126/0/Valida%C3%A7%C3%A3o%2Bde%2BN%C3%BAmero%2Bde%2BDocumento%2Bdo%2BCart%C3%A3o%2Bde%2BCidad%C3%A3o%2B%281%29.pdf/7d5745ba-2bcc-e861-3954-bafe9f7591a0).
Checks validate structure, not a government registry; identifiers must be delimited rather than
embedded in a longer word or number. The generic `national-id` detector remains broader.

The `credentials` detector matches whole words and phrases without case sensitivity, including
password/passwords, senha/senhas, palavra-passe, passphrase, API keys, access tokens and private keys.
It detects mentions, not proof that a secret is present. Start with a warning rule if ordinary
discussion of credentials should remain possible. Custom keyword entries are literal phrases;
spaces inside an entry are preserved, and matches use word boundaries. These detectors are opt-in
per rule, using the same action, priority and enabled controls as other DLP conditions.

Coverage is message subject/body and attachment metadata, not OCR or extraction of attachment
contents. Verdicts and audit records contain detector names rather than matched identity/secret
values. Enforcement applies to the engine submission path; this is not a claim that external
mail clients or independently sending third-party services are covered.

### Authoritative checks and proxy limitations

Unavailable or malformed configured rules refuse submission with `dlpPolicyUnavailable`;
missing, oversized or unavailable authoritative draft bodies refuse with `dlpInputUnavailable`.
No configured rules remains an empty policy. Native sends check again immediately before SMTP,
using the exact outgoing bytes and actual envelope recipients, including queued releases and
recipient overrides. A recipient-domain exclusion applies if **any** recipient is outside the list.
Draft scans require the authenticated account to own the draft.

Proxy-mode submissions with any enabled DLP rule are refused before any operation in the batch
is forwarded (`dlpProxyUnsupported`). Fetching an upstream draft and later submitting it does not
guarantee the inspected bytes were sent; same-request composition is not an upstream atomicity
guarantee. Cancellation-only updates remain available. This deliberately does not claim proxy
content inspection. Administrators must remove/disable those rules or use the native engine to
send; warnings and notify rules also require authoritative inspection.

A matching `require-encryption` verdict refuses server submission, just like a blocking verdict;
merely declaring an encrypted Content-Type does not grant an exemption. This is conservative:
a recipient-only encryption rule can also refuse a genuinely encrypted message until authoritative
cryptographic validation is supported. Content-based rules cannot inspect encrypted ciphertext;
this policy does not decrypt encrypted messages or attachments and does not certify encryption.
