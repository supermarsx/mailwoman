# Outgoing email header privacy

`privacy.stripMailerHeaders` defaults to true. Administrators can set and lock it globally or by domain; users can override it in Privacy when permitted. The native engine resolves the current account policy for every send attempt, including queued messages. It removes only top-level `X-Mailer`, `User-Agent`, and `X-MimeOLE` fields, including folded continuation lines.

The shared native submitter wraps SMTP and EWS transports. Normal submissions, imported-message resend, queued/released mail, redirects, vacation replies, MDNs, calendar invitations and security reports using that submitter share this boundary. It does not govern arbitrary upstream JMAP proxy sends, HTTP User-Agent, or headers added later by a relay. Ordinary Mailwoman composition already does not add an X-Mailer field.

Other headers, the MIME separator, body, attachments and nested messages are byte-preserved. Message-ID and origin headers remain. The User-Agent inside an ARF feedback-report payload remains because it is report content, not an email header. Signed/encrypted MIME bodies are not rewritten.

A DKIM or ARC message signature covering a header that would actually be removed causes an explicit pre-signed-header privacy conflict before transport. Malformed signature header lists are refused conservatively. Disable this policy when permitted to preserve such a pre-signed message verbatim; ordinary newly composed messages are unaffected. Header parsing is bounded to 128 KiB and rejects malformed top-level fields. DSN options and uncertain-delivery errors pass through unchanged, without retries.

Verification covers exact folded-field removal, real MIME builder attachment bytes, signed-header conflicts and unaffected oversigning, per-account overrides and administrator locks, and native JMAP resend through the registered submitter after Bcc removal. These are controlled byte-capture tests, not a claim that external relays preserve the result.
