# Administrator logs

The administrator Logs section separates authentication history from recent
application warnings and errors. Both require an unscoped `logs.read` grant;
a domain-scoped grant cannot inspect global records.

Authentication history includes successful sign-ins and rejected attempts,
including attempts without a known account. Search and filters identify the
surface, backend, result, and bounded reason code. Public sign-in responses remain
generic. A backend failure is classified only when the server has reliable typed
evidence; an unspecified failure does not prove that the password was wrong.

Repeated equivalent failures are grouped within a one-minute interval. Retention
and the maximum entry count are configurable using `logs.auth.retentionDays`
(default seven days) and `logs.auth.maxEntries` (default 50,000). Pruning history
does not remove the authenticated identity used for account security policy.

Application history captures warnings and errors from the running server's
tracing subscriber, subject to its configured log filter. It keeps at most 1,000
entries for 24 hours in memory and groups identical events within a minute.
The view supports severity, component, and text filters, with paginated results.
It is specific to one server process and resets at restart; it is not an archive
of Docker output or a combined view across replicas. External log collection is
still needed for historical infrastructure investigations.

Capture bounds field sizes and omits oversized fields. Credential and mail-content
fields are redacted; free text also passes through credential-pattern redaction
before entering the buffer. Debug/trace records and span fields are not captured.
Do not add passwords, tokens, or message contents to application log statements.

Use Diagnostics for active database, storage, cryptographic, and connectivity
checks. Authentication history describes sign-in outcomes; application history
helps identify other recent backend failures.
