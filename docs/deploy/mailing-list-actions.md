# Mailing-list badge and command review

The Reader shows a mailing list badge after the spam score badge when a message has a `List-Id` or `List-Unsubscribe` header (setting `mail.badge.list`). It opens a dialog with the list's name and id, its archive, help, post and owner links as text (http, https and mailto only; each can be opened in a new tab or copied), how the message offers to unsubscribe, how many messages from the sender are in the loaded mailbox, and the actions. The old notice bar is gone.

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

RFC 8058 one-click unsubscribe is sent by the server, never by the browser:
`POST /api/mail/list-unsubscribe {"url"}` (session required) sends the fixed body
`List-Unsubscribe=One-Click` to an `https` URL through the egress plan's
`unsubscribe` class (address gate, pinned DNS, no redirects, no cookies or
referrer), so the list host sees the server and not the reader. The client offers
it only when `List-Unsubscribe-Post: List-Unsubscribe=One-Click` and an `https`
URL are both present, and `mail.list.unsubscribeMode` decides whether it asks
first (`ask`, the default), sends at once (`one-click-direct`) or never sends
(`open-link`). A `List-Unsubscribe-Post` header alone neither authenticates the
sender nor grants permission for an automatic request.

"Unsubscribed on <date>" is stored in the browser per account and List-Id after a
successful one-click request, or after the user says they finished a web page or
mail draft; opening a URL or preparing a draft alone asserts nothing. Mail from the
list received more than three days after that date shows a warning, because the
list is then ignoring the request. "Always keep" is also browser-local and only
turns off the clean up actions in this dialog.

Acceptance: focused parser/component/Reader/Composer tests and actual
Chromium/Firefox/phone tests against authenticated message import and rendering.
The browser tests verify destination review and the editable draft without any
request to the external fixture domain or any EmailSubmission/set call.
