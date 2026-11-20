# Bounded DLP CPU execution candidate

DLP parsing, scanning, policy JSON decoding and regex validation run on a
process-shared dedicated CPU executor. It lazily starts one or two threads
according to available parallelism and admits at most 16 jobs / 256 MiB of charged
owned input. Admission fails promptly when full; it never queues an unbounded
set of waiting futures retaining raw mail. Charges include Vec/String capacity,
recipient/attachment descriptors and rule data, not merely visible input length.

One shared 256-MiB input working-space allowance spans asynchronous Store/policy
waits. Its Arc-owned admission token travels with CPU and file-I/O jobs and their
results; cancellation cannot release that allowance while owned buffers still
execute. CPU queue byte admission is an additional bound, not an independent
allowance permitting another 256 MiB of abandoned DLP inputs. Stored-draft
inspection reserves the existing 64-MiB raw maximum before fetching it. Policy allowances are 48 MiB per active inspection, with environment
policy limited to 2 MiB, stored policy source 8 MiB (checked in SQL before value
materialization) and decoded rule data 16 MiB.
These are explicit fail-closed resource ceilings; over-limit input is unavailable,
not silently uninspected. Configuration file retrieval uses a separate two-slot
bounded blocking-I/O gate, never the CPU threads. CPU jobs perform no Store,
network or DNS work. Explicit synchronous admin validation APIs remain synchronous.

Outgoing inspection takes and returns the same owned SMTP Outgoing value. The
exact Bcc-stripped raw Vec and actual envelope recipients move through the CPU
job without cloning the raw message. A fresh, unchanged policy with no enabled
rules preserves the original fast path: it returns the outgoing bytes without
MIME parsing or applying inspection-only size/capacity limits. Active DLP still
enforces those limits and fails closed. Draft parse jobs likewise consume owned
body buffers. Index-publication parsing already has its own dedicated owner;
this candidate does not send that work through a second executor.

Every inspection reads current environment/file and stored policy sources. It
validates/compiles them on CPU threads and rereads the authoritative sources after
inspection. A change retries the exact same owned input at most twice; continuous
changes, malformed/unavailable policy, parser failure, queue saturation, worker
panic or timeout return the existing DLP error. Neither JMAP nor MDN submission
continues on that error. There is no TTL policy cache or stale allow fallback.
The unchanged-source check is optimistic, not a lease spanning SMTP/network I/O.

These limits bound admitted input and job counts, not a hard native allocator or
CPU-time sandbox. A parser/regex can allocate intermediate decoded structures
before an output-size check. A caller timeout stops waiting after 30 seconds but
cannot kill an in-process CPU operation. Its job count and byte permits remain
owned until actual completion, including when the caller is canceled. Hard
preemption and strict heap isolation require an OS process/sandbox boundary.

This is a source candidate. Correctness gates include actual buffer identity,
detector parity, late policy change/continuous change, capacity-based rejection,
cancellation-held admission, saturation/panic and asynchronous heartbeat. No
throughput improvement is established without subsequent controlled measurement;
the existing representative workload does not measure sending/DLP enforcement.
