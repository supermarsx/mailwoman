# Native JMAP Core behavior

The native engine answers `Core/echo` and returns method failures under the standard `error` response name. A failed method does not stop later calls. Result references validate the response name and support RFC 8620 wildcard array projection, flattening and JSON Pointer escapes. Conflicting normal and referenced arguments are refused. Echo preserves arbitrary client properties, including `type`.

Native HTTP admission rejects malformed JSON, malformed invocation envelopes, unknown capabilities and oversized requests before connecting an account or executing a method. Errors use `application/problem+json` and the standard JMAP problem URNs. Unknown request extension properties are ignored. In-process engine callers may omit `using` for compatibility; HTTP callers must provide it.

The native Session advertises these enforced limits:

| Limit | Value |
|---|---:|
| Upload bytes | 50,000,000 |
| Request bytes | 2,097,152 |
| Calls per request | 64 |
| Objects per get/set | 5,000 |
| Concurrent requests | 4 |
| Concurrent uploads | 4 |

Concurrency uses conservative process-wide budgets, so all accounts share four request slots and four upload slots. Exhausted slots return a retryable HTTP 429 problem; permits release when processing completes or is cancelled. GET/SET object excess is a method-level `requestTooLarge` error. A GET without explicit IDs that would return too many objects is also refused.

`createdIds` is returned when the request supplies a map, including the supplied IDs and newly created objects. `Email/set` checks a string `ifInState` before mutation and returns `stateMismatch` when stale; absent or null means no precondition. This is a local JMAP state check, not an atomic compare-and-mutate guarantee from an IMAP server.

Upstream-proxy mode retains the upstream capability and method contract. These Core improvements are not a claim that every published JMAP RFC or every mail-property edge case is implemented. Native contact/calendar/task extensions remain distinct from standard RFC capabilities.

References: [RFC 8620](https://www.rfc-editor.org/rfc/rfc8620.html), [RFC 8621](https://www.rfc-editor.org/rfc/rfc8621.html).
