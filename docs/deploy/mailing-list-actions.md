# Mailing-list command review

The Reader recognizes List-Help, List-Subscribe, List-Unsubscribe, List-Post,
List-Owner and List-Archive, including ordered alternatives and List-Post: NO.
It reuses the existing bounded message-header read; parsing and opening the
review panel do not contact the listed destination.

These headers are unverified claims made by the message. Each web action first
shows its full destination. Following it is an explicit browser navigation with
no referrer and no opener access. A mailto action opens an ordinary editable new
message; it never sends automatically or marks the original answered/forwarded.
Only one recipient, subject and plain-text body are accepted. Body text is HTML
escaped before entering the composer. Hidden recipient fields, attachment
parameters, duplicate query fields and subject/header injection are refused.

The implementation is a bounded safe subset of [RFC 2369](https://www.rfc-editor.org/rfc/rfc2369.html),
not a general URL launcher. Only HTTP, HTTPS and restricted mailto are supported.
Duplicate command headers, malformed separators, excessive alternatives/comment
nesting and oversized fields are ignored. Embedded URL whitespace is refused
rather than repaired. A reader/message/account change invalidates old header
results and destination reviews. Network failures do not invent commands.

RFC 8058 one-click unsubscribe POST is not implemented by this surface. A
List-Unsubscribe-Post header alone neither authenticates the sender nor grants
permission for an automatic request. No subscription state is asserted after
opening a URL or preparing a draft.

Acceptance: focused parser/component/Reader/Composer tests and actual
Chromium/Firefox/phone tests against authenticated message import and rendering.
The browser tests verify destination review and the editable draft without any
request to the external fixture domain or any EmailSubmission/set call.
