# JSON action client completion

Source implementation, not a runtime or throughput verification. The server owns
`apps/web/src/api/websocket-action-contract.json`; client tests use those exact
bytes, including its lower-case hexadecimal cookie-delivery token shape.

The browser keeps distinct webmail and administrator sockets. Webmail allows eight
in-flight actions, with at most sixteen queued frames and 8 MiB of queued encoded
frames. Each frame remains subject to the shared 2 MiB frame limit. Administrators
serialize the entire action and cookie-delivery turn, with at most sixteen
admitted turns and 8 MiB of queued path/header/body encodings. The queue wait is
bounded to thirty seconds; an expired/cancelled turn retains its bounded place
until the prior turn releases it, but cannot dispatch. Socket handshake is bounded
to five seconds; an action waits at most 120 seconds by default. Retiring sockets
are limited to eight. These are retained-payload/admission bounds, not hard RSS or
browser scheduling/preemption guarantees.

There is no HTTP spill when the socket has eight busy actions. HTTP remains an
explicit transport for the contract's binary, streaming and credential controls.
Unavailable-before-send may use HTTP. After send, only the server's explicit
`actionOutcome: notDispatched` plus `httpRequired: true` permits that fallback.
Unknown/malformed/lost acknowledgements never cause application replay. Shared
worker and leader-channel replacement preserve the same terminal outcome.

A body below the body limit can exceed the independent frame limit after JSON
string escaping and envelope overhead. This locally unsupported representation
falls back before channel/queue admission, using the unchanged request through
ordinary HTTP and its authority assertions, CSRF and server guards. The private
prepared frame is never a caller-supplied override; administrator frames are
regenerated from the current CSRF after cookie handoff. This fallback neither
raises limits nor treats queue pressure or an unknown/completed outcome as a
reason to replay a request. The size test exercises actual escaped signature
JSON, not merely a body that was already over its limit.

Every queued frame invokes its caller's current authority assertion immediately
before `socket.send`, in addition to the existing fresh server authorization.
Admin session generations bind sockets; refreshed CSRF alone cannot adopt a new
actor. Clearing account state rejects unsent work. Worker `/api/me` remains a
fresh HTTP identity/account check because it may issue cookies. Eligible JMAP
session discovery can use WS; no authentication or account proof is cached here.

An administrator ACK may contain an opaque delivery receipt, never a cookie or
target URL. The client derives the exact same-origin console cookie-control URL,
POSTs only the receipt, and waits before releasing the next administrator turn.
At most two control attempts share a ten-second/receipt-expiry deadline. A lost
control ACK can use the newly delivered readable CSRF token on its second attempt;
the application action is never retried. On completion the old socket retires and
later unsent work establishes a fresh cookie-authenticated socket. Failed handoff
blocks that admin generation/proof until sign-in changes it.

`ActionCompletionError` keeps the known original response in the calling realm;
worker messages serialize only its fixed code/status and safe explanation.
`outcomeUnknown` distinguishes an unknown application outcome. Both bypass generic
automatic recovery, including keyed writes. The offline replay queue retains the
original queued payload as failed for explicit reconciliation and does not drain
it automatically again. A manual retry of an unknown create retains its prior
idempotency key; this is not a durable exactly-once journal or permission for
automatic replay. Server-confirmed completed transport failure still performs any
valid typed cookie handoff before returning its terminal error.

Authored tests cover shared routing/escape/header bounds, queue count/bytes,
authority changes at send, actor isolation, serialized rotation, lost control ACK,
known completed versus unknown errors, recovery, offline repeat-drain, leader
replacement and fresh worker metadata. They require actual execution before any
completion claim; real HTTP cookies/server guard tests remain server-lane work.
