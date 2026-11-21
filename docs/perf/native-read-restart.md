# One bounded pre-dispatch metadata-read restart

Source-only concern based437ec764; tests authored, not executed. A live single503
is not attributed to this cause without its actual response evidence. The new
regression deterministically models the due-touch class by aging only last_seen
AFTER preparation while the request actually waits behind a held JMAP permit.
It uses the unchanged old late_refusal branch as a control; no15-second sleep.

Only the already validated single native-cookie metadata Email/get body can mint
a private request-owned restart token. SHA256 binds the exact body and consumption
rechecks metadata eligibility. A forged mutation/mixed-body token fails even if
its body hash is injected by an internal test. No cached Session/allow is added.

Before Engine dispatch, a late native source mismatch releases the old connector
and permits, then falls through once to the existing original JMAP handler. It
recomputes COMPLETE unpruned Needs from the actual body, obtains a NEW revision
and protocol-presence observation, then performs the existing feature check,
complete ordinary authentication and final primary witness/session comparison.
It does not reuse an old absent-policy decision or old authenticated source.

The original handler reacquires the SAME concurrency gate using its current
ordinary knobs and preserves all helpers/ensure_account checks. A request-only
observer compares those knobs and the authenticated Session/witness against one
LAST bounded full native composite after those waits (at most two shaping reads).
No primary-only SELECT follows that wider observation. Pure source/namespace,
active parent, legacy flags, current credential/configuration, target, clock and
process checks run before Engine dispatch. Failure-only expiry cleanup remains
terminal. A second change or unsupported final observation refuses without any
second restart. No transaction/connector lock spans the whole fallback.

Healthy native fast hits retain the original request path and read count.
Ordinary non-restarted methods keep their original behavior; observer is empty.
No mutation, mixed-method, result-reference or raw-body request is replayed. The
rare restarted read costs extra fresh work; this is correctness/availability,
not a throughput claim.

Authored gates: all existing native metadata8 and feature boundary13 router cases
remain required, plus native_read_restart_tests4 real DEFAULT36 router cases and
2 private token controls. The shared primary witness factory is extracted without
changing its bytes/error order; existing Store primary-policy and native snapshot
contracts must run too. PostgreSQL/source claims remain pending actual execution.
