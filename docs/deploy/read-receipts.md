# Read-receipt authority and delivery outcomes

`mail.readReceipts` currently accepts `ask`, `never`, and `always`. A personal
preference controls automatic responses. An administrator-held `never` also
forbids manual `MDN/send` calls and raw disposition-notification submissions.
The native engine checks that authority before queueing and again immediately
before transmission, including delayed submissions. Policy read failures do
not grant permission.

An SMTP outcome can be uncertain after message data has left the client. In
that case the receipt claim remains consumed and a retry is refused. A definite
pre-transmission authentication or capability refusal releases the claim. This
favors avoiding duplicate receipts over silently retrying an uncertain send.

In proxy mode the whole JMAP request is checked before forwarding any mutation.
A held prohibition refuses explicit `MDN/send` and uninspectable submissions.
Normal immediate `Email/set` composition followed by a creation-reference
`EmailSubmission/set` is admitted only with explicit safe MIME structure. Opaque
raw imports, existing mutable upstream drafts, custom MIME header overrides,
creation-ID aliases, ambiguous patches, and deferred sends are refused under
that prohibition. Attachment-only convenience composition is also refused;
attachments require an explicit ordinary text/HTML body. Explicit `bodyStructure`
cannot be combined with convenience body or attachment fields. This follows the
unambiguous creation constraints in [RFC 8621, section 4.6](https://www.rfc-editor.org/rfc/rfc8621.html#section-4.6)
without assuming the MIME root an upstream would generate from attachments alone.
MIME-changing `Email/set` writes through this gateway are
checked independently, preventing another gateway request from replacing the
checked composition with a receipt. Ordinary typed draft edits, keyword changes,
and cancellation remain available. Clients connected directly to the upstream
service are outside this gateway's authority. These checks cannot recall a
message or scheduled send already accepted by an upstream service, and are not
a claim that the upstream server executes a JMAP batch transactionally.

`receipts.dsnAllowed` independently controls requests for SMTP delivery-status
notifications. Administrators may set it globally, by domain, or by address.
The engine checks it both when queueing and when dispatching; malformed held
values and lookup failures deny DSN requests. Proxy mode checks SMTP envelope
parameters before forwarding any batch operation. It refuses opaque result
references and release of existing upstream submissions when DSN is prohibited,
since the gateway cannot recheck those submissions at their transmission boundary.
Parameter-free composition and cancellation remain available.

Engine accounts support `never`, `ask`, `always`, `contactsOnly`, `orgOnly`, and
`orgAlwaysAskOthers`. Contacts are matched against parsed email addresses in
the same account's address books (at most64 candidate cards of64KiB each).
Organisation matching uses the signed-in account domain and administrator-defined
domain aliases, not a message-supplied identity. A return-path mismatch or list
message always requires confirmation, even when its sender otherwise qualifies
for automatic handling.

The Read receipts settings section has a dedicated policy editor and exact-address
sender exceptions. Sender edits require the current ETag and have an atomic500-row
account quota. A personal `never` suppresses prompts and automatic responses even
when an older `always` exception exists; any administrator-held policy ignores
personal exceptions. Explicit manual sends remain subject to administrator authority.

The Reader requests eligibility only after its body frame reports displaying the
current message. Automatic sending additionally requires a visible, active full
message view and a1.5-second dwell. Preview panes never automatically answer.
Background tabs, failed display acknowledgement and unavailable policy evidence
do not send. Uncertain outcomes have no automatic client retry. Proxy accounts
do not expose these engine-specific controls.

### Standard JMAP MDNs

Engine accounts expose `urn:ietf:params:jmap:mdn` in both the session and account
capabilities. Requests using it use the [RFC 9007](https://www.rfc-editor.org/rfc/rfc9007.html)
`MDN/send` creation map and `MDN/parse` blob map; the legacy custom
`MDN/send {emailId, automatic}` remains available without that capability.
Standard sends also require the mail capability and an account-owned Identity.

Every send entry must have an `onSuccessUpdateEmail` creation reference setting
`keywords/$mdnsent` to true. Missing or conflicting patches are rejected before
sending. The keyword must be recorded before SMTP; successful operations include
the implicit `Email/set` response. The durable duplicate ledger also survives an
uncertain transport result, which must not be retried automatically. An explicit
final recipient must match the selected identity and be visible in the original
To/Cc; an identity removed or changed before dispatch is refused.

Subject, plain-text body, Reporting-UA, all four dispositions, original-message
inclusion and extension fields are supported. Server-set gateway, original
recipient, original Message-ID and error properties are returned as applicable.
The server is not a foreign-MDN gateway, so it does not invent gateway/error data.
Including the original is explicit and may disclose its contents and headers to
the receipt recipient; no decrypted replacement is generated. Reports use the
null SMTP reverse path. Existing administrator and automatic-response policy
checks apply to standard sends too. The exact outgoing report bytes and SMTP
recipient also pass ordinary DLP enforcement before transmission; malformed
policy, unavailable inspection input and matching blocks refuse the send.

Limits: 32 records per method, 8 MiB original messages for sending, 9 MiB blobs
for parsing, 64 KiB report/plain-text fields, 4 KiB subjects, and 32 extension
fields when sending. Oversized/ambiguous report fields are refused rather than
silently truncated. Upload ownership and plaintext size are checked before
reading the sealed object, separately from encryption-envelope overhead. Reads
use bounded cached raw messages and parts; a missing cache object is unavailable
rather than causing an unbounded upstream fetch. `MDN/parse` leaves `forEmailId`
null instead of guessing a relationship or scanning every message in an account,
as permitted by RFC 9007. Duplicate field names are treated as ambiguous.
RFC 6533 internationalized receipt construction is not implemented; ASCII
recipient/report fields are enforced while human-readable content supports Unicode.

A received delivery report does not establish that a person read a message;
a disposition notification reports another mail program's claim rather than
independently proving human reading.

## Outgoing SMTP DSN support

Native engine session publication now lists `DSN` only after a recent successful
EHLO observation through the configured SMTP egress, TLS and authentication path,
and only when effective `receipts.dsnAllowed` permits it. Unknown, expired,
failed-authentication and unsupported observations do not advertise support.
The probe sends no MAIL, RCPT or DATA. Its network work is limited to two seconds;
concurrent callers share a probe. Positive observations last five minutes,
negative observations thirty seconds. Each immutable submitter owns its cache:
changing endpoint, credentials or dialer cannot reuse another authority's result.
Legacy `smtp_dsn:<account>` settings are no longer capability authority.

Submission checks actual EHLO support again. If a requested DSN becomes
unavailable, the submission fails before MAIL; the engine does not retry it as
an ordinary send. The masked-address wrapper preserves DSN options after the
same account/alias authorization used for ordinary submission. A successful
SMTP acceptance still does not prove delivery or reading, and downstream
report generation remains under the receiving mail systems' control.

This implements truthful [RFC 8621 submission extension publication](https://www.rfc-editor.org/rfc/rfc8621.html#section-1.3.2)
and the configured submission hop of [RFC 3461](https://www.rfc-editor.org/rfc/rfc3461.html).
It does not claim Mailwoman is a complete DSN-generating relay. Existing native
parameter limits remain: common NOTIFY across recipients, ASCII ENVID, RET
FULL/HDRS, and ORCPT restricted to each recipient's own address. Proxy sessions
retain their upstream capability authority; Mailwoman still enforces its own
administrative DSN restriction at admission.

Compose offers a separate delivery-notification checkbox only as an available
choice when the selected account advertises DSN and effective policy permits it.
The dedicated Read receipts settings editor saves `mail.requestDeliveryReceipt`
(default false) per account; administrators can supply and lock this default.
Locking the default does not prohibit a per-message choice: use
`receipts.dsnAllowed=false` to prohibit requests. Drafts retain their explicit
choice; losing capability or permission produces an error rather than silently
removing the request. The web control requests RET=HDRS and
NOTIFY=SUCCESS,FAILURE,DELAY for every envelope recipient, including Bcc.
Incoming DSN display is separate and always identifies reports as unverified
mail content.

## Uncertain SMTP acceptance

If DATA/BDAT content transmission begins but the server's final acceptance or
refusal is lost, Mailwoman records delivery as unknown. It does not automatically
resend that attempt. This addresses the duplicate-delivery risk described in
[RFC 5321 section 4.5.3.2.6](https://www.rfc-editor.org/rfc/rfc5321.html#section-4.5.3.2.6).
An explicit negative final reply remains a refusal; a confirmed acceptance is
recorded before filing the Sent copy.

Each queued attempt acquires a durable unique claim before SMTP. Competing
processes cannot claim the same row, stale workers cannot record another
attempt's result, and a crash leaves a claim excluded from the retry queue.
The Outbox conservatively labels both active/unreconciled claims and lost replies
as “Delivery status unknown”, with no release/resend action. Ordinary failures
known to precede acceptance retain the bounded retry behavior.

Check Sent or contact the recipient before deliberately creating a new send.
A missing Sent copy does not prove non-delivery: filing is separate from SMTP
acceptance. Mailwoman cannot recall an upstream-accepted message. This provides
per-submission retry safety, not global deduplication of separately created
messages or a complete SMTP relay retry implementation. Proxy submissions remain
subject to their upstream implementation's queue and outcome semantics.

Connected-provider adapters cannot infer the transport phase from opaque provider
errors. They therefore retain explicit authentication/unsupported-operation
refusals, but classify other send failures as uncertain rather than automatically
retrying a possibly accepted message. This does not provide provider-side
idempotency or recall.
