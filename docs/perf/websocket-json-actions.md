# JSON WebSocket actions and cookie completion

This candidate dispatches JSON actions through the original HTTP router. It is
source-authored and independently reviewed only after the frozen manifest is
cleared; no runtime or all-operations-complete claim follows from this document.

## Authorities and guards

`/jmap/actions` authenticates a primary webmail cookie and records only a bounded
Store-bound identity witness. The dynamic console `/actions` endpoint authenticates
an admin cookie directly, without requiring a webmail Session. A connection never
retains AdminCtx, grants or credentials. Every frame receives fresh request
extensions containing only ConnectInfo plus private typed cookie capture where
applicable; handshake policy/auth/protection extensions are not copied. Source
headers and actual cookies stay frozen; selected account/link headers are per
frame and still pass original linked/account/capability guards.

Admin resolution is repeated per frame and at the original registrar. Native
webmail admission compares its already-fresh principal/binding/session source
against the socket anchor without adding an authority await after the final
snapshot. The ordinary authentication chain performs the additional bounded
actor-source check before linked-account selection. HTTP requests outside a socket
retain their original authentication inputs and queries.

## Contract and boundedness

The static JSON contract is bundled into both builds. It declares exact HTTP-only
bootstrap/cookie/credential/control endpoints and known stream/binary exclusions.
JSON settings including reset and admin settings use the broad JSON route class;
unknown admin endpoints are rejected before dispatch unless registered with a
signed-in/capability access gate. Percent-encoded path/query validation rejects
ambiguous separators, traversal, nested encodings and control bytes. Exact and
case-folded duplicate headers are rejected while decoding. Duplicate in-flight
IDs close the connection without a misleading per-ID notDispatched reply.

Per socket concurrency remains eight webmail or one admin. Both actors acquire
one of 256 process-local Service slots before creating dispatch tasks. A Service
has at most 256 reservations/receipts, no eviction of an active reservation, and
one active/retired reservation per old admin actor. At most 64 live Service
instances can register for shutdown; excess instances refuse socket admission.
Frames/bodies are at most 2MiB; raw responses and exactly counted encoded replies
are at most 64MiB each. These are finite object/concurrency caps, not an RSS cap:
request/router/parser scratch and simultaneous large responses remain separate.

## Typed cookie handoff

Every admin action reserves receipt capacity before original-router dispatch,
including reads. Only admin_rotation can issue a typed canonical admin+CSRF grant.
The random delivery token is 32 bytes encoded as 64 lowercase hexadecimal characters.
Only its digest indexes the registry. The grant is AEAD sealed with Store scope
and receipt binding; staging grant serialization/token buffers are zeroized.
Cookie header values never enter acknowledgement JSON or logs.

A successful rotation retires the old actor. Acknowledgements have explicit
notDispatched/completed/unknown outcome. Completed action plus cookieDelivery
means complete that transport step, never replay the action. An unexpected cookie,
non-JSON response or response-bound failure after dispatch is a completed transport
failure with no HTTP replay instruction. A valid typed receipt is retained even
if response-body transport fails. Receiver cancellation does not cancel a sent
admin action: its bounded owned task retains the reservation through rotation.
If the ACK/token itself is lost, the action is uncertain and may require re-login;
there is no durable action journal or promise to recover a lost action result.

POST to the trusted console `/actions/cookies` accepts only `{token}` on the exact
origin-form route, with no query or redirect. Before any translation it requires
the actual captured Origin, actual old or issued-new HttpOnly cookie, and actual
readable/header double-submit proof tied to that cookie. It authorizes only the
same actor's already-issued cookie delivery. The issued session and grants are
freshly resolved before translation; original request/source/session/CSRF guards
then run; the handler repeats fresh issued-session/grant checks and the monotonic
receipt/absolute expiry checks after awaits. Re-delivery of the same receipt is
idempotent and never extends 30s receipt expiry or absolute session expiry.

Delivery does not consult the WebSocket enabled/prefer flag: an authorized action
that disables actions must still complete its cookies. Revocation, changed role
or domain-scope fingerprint, expiry, or bad proof refuses delivery and requires
re-login. This conservative self-role-change behavior reports a completed action,
not a failed action to replay. Shared `mw_csrf` across admin/webmail tabs is not
silently forged or bypassed; a stale actor proof must refresh/reconnect.

## Lifecycle and staged limits

`shutdown_websocket_actions()` closes admission and waits at most two seconds for
both actor classes' request futures/owned admin work. False reports outstanding
uncertain work honestly. Existing backend-detached publication/telemetry tasks
still require their own shutdown lifecycle. Main must call this after stopping
HTTP acceptance and before exiting; this concern does not edit main.rs.

`/api/me`, login/logout/renew/remembered-device/cookie bootstraps, console session,
credential ceremonies, binary and stream routes remain explicit HTTP controls.
This does not deliver binary/native-bearer action transport or separate actor CSRF
cookies. There is no all-requests-on-WebSocket completion claim.

## Authored gates (not executed)

- `cargo test -p mw-store --lib websocket_cookie_handoff::tests --locked`: 2 cases.
- `cargo test -p mw-server-core --lib websocket_actions::tests --locked`: 10 cases,
  including actual admin-only WebSocket GET, original-router rotation crossing a
  real paused handler, action disables WS then cookie completion, repeated ACK
  delivery, Origin/CSRF/query/revocation/expiry/grant/session negatives, owned
  cancellation, duplicate header parsing, bounded reservations and shutdown.
- Existing admin_rotation 2, admin_router registry 1, native admission/original auth,
  route registry/security/static-source and client contract gates must run after
  integration under the coordinator's serial compiler slot.

The isolated base predates the facade split, so it names the original core source
paths. Root selects the integrated core package name for actual gates.
