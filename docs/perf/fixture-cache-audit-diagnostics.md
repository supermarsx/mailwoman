# Fixture cache audit failure evidence

This diagnostic-only change preserves the strict full serving-handle audit.
A partial, empty, unavailable or timed-out audit remains a failed campaign with
no capacity qualification; no new requests or mutation retries are introduced.

The authenticated private fixture route may return503 with exactly
`cache-audit-unavailable` or `cache-audit-quiescence-timeout`. Both generators
retain and parse at most32KiB of this streamed JSON, validate and retain only the fixed code and
bounded numeric/boolean wait outcomes. Unknown status/type/code, malformed UTF8
or JSON, oversize input and malformed shapes retain no server body. No raw
error strings, headers, cookies, paths, handle IDs or private handoff are copied.

At most100 attempt outcomes are allowed, with <=32 handles per outcome and the
fixed1s cooperative helper budget/10ms retry interval. Elapsed time remains a nonnegative safe integer, including observations above
60s after a cooperative deadline overshoot. No numeric value is treated as proof
that synchronous SQLite or OS stalls can be killed at a wall-clock deadline.

The FAILED JSON/Markdown report remains unqualified with no successful stage
claim; the existing CLI catch still exits nonzero. This is failure provenance,
not a new performance result. Six focused response/shape/privacy tests are
source-authored and must be executed independently before verification claims.
