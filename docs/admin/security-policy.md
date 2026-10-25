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
US social security numbers), a custom regular expression, recipient domains (inside or outside the
list), attachment types and a largest attachment size in bytes. Dictionaries and a classification set
outside the console are kept as stored. A rule from `MW_DLP_RULES` with the same id wins over a stored
rule; the page lists those rules read-only.
