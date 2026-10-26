# Incoming delivery status reports

The reader presents received delivery-status notifications separately from read receipts. It shows the reporting server and each recipient's reported action, enhanced status code, and optional diagnostic, original recipient, remote server and retry dates. These are claims made by the message sender, not independently authenticated delivery or read state. Reading a report never sends a notification or changes the state of another message.

The parser follows the report structure in [RFC 3464](https://www.rfc-editor.org/rfc/rfc3464.html) and the UTF-8 report type in [RFC 6533](https://www.rfc-editor.org/rfc/rfc6533.html). It recognizes failed, delayed, delivered, relayed and expanded actions. A failed action can retain a temporary 4.x.x status after retries are abandoned; the action is not inferred from the numeric code. Dates and diagnostics remain bounded display text.

Engine mode supplies the explicitly requested `mailwomanDsnReport` Email/get property from the cached raw message. Proxy mode checks root MIME headers through the existing authenticated blob download route, then downloads candidate reports with a streaming byte limit. The browser sends those bytes to `POST /api/mail/delivery-status`; the server accepts no remote URL and uses the same parser. Cookie authentication requires the normal CSRF proof; native session bearers use the existing authentication path. Responses containing parsed reports use `private, no-store`.

Limits are 8 MiB for the complete message, 256 KiB for the decoded status part, 64 recipients and 4 KiB per field. The proxy endpoint allows two concurrent parser tasks with a five-second response deadline; a task retains its permit if its request is cancelled. The browser aborts the proxy operation after fifteen seconds and cancels it when its reader is disposed or replaced. Ordinary messages only require the bounded header check, not a complete body download.

Only a direct matching machine-readable part of a root multipart/report is interpreted. Forwarded reports, MDNs, ambiguous duplicate fields, oversized reports, malformed required fields, and informal textual bounce formats are left as ordinary email. Their original message body remains available. No sender-controlled HTML or links are rendered from status fields. This feature does not implement outbound DSN requests, automatic bounce processing, sent-message correlation, or read receipts.

Acceptance evidence is tracked in `.orchestration/logs/codex-dsn-*`; an implementation checkpoint alone is not browser acceptance.
