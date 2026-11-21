# Content-aware Cargo source mirror prototype

The helper passed root-owned Linux21-contract and six-phase pinned Cargo1.98.1
correctness gates. Docker wiring below is a separate source-only candidate and
has not executed. It introduces versioned paired caches while leaving old cache
IDs/data intact; deployment targets, compiler flags and security behavior remain
unchanged. No nightly/bootstrap/checksum-freshness mode or speed claim is added.

## Evidence and remaining causality check

Root artifact `.orchestration/reports/perf-investigation/archive-source-freshness-v1.json`
compares the exact d71e06d3/437ec764 archives. Unchanged local sources and Cargo.lock
have equal SHA256 bytes but a new commit-wide archive mtime. Local reproduction
also confirmed engine.rs/store lib.rs byte identity while timestamps moved from
1795215182 to1795219868. Only two fixture Rust files and the Docker gate count
changed; build logs nevertheless show many unchanged local crates compiling.

This supports, but does not fully trace, the cause. The next owned build must
retain actual `CARGO_LOG=cargo::core::compiler::fingerprint=trace` dirty reasons,
relevant file/anchor timestamps and compiler-artifact fresh fields. No speedup is
claimed from source inspection.

Pinned [Cargo1.98 fingerprint source](https://raw.githubusercontent.com/rust-lang/cargo/rust-1.98.0/src/cargo/core/compiler/fingerprint/mod.rs)
describes local input mtimes versus fingerprint dep-info anchors and propagation
of dependency changes. This prototype reads that layout's metadata; it never
edits Cargo fingerprints. It rejects unsupported tool versions/layouts.
[Docker cache rules](https://docs.docker.com/build/cache/invalidation/) exclude
mtime alone from COPY checksums. The hypothesis concerns what happens when a
real included file change invalidates COPY and the rebuilt source layer carries
new archive timestamps, not a claim that mtime-only changes invalidate Docker.

Actual deployment differs: a maintained checkout may preserve unchanged input
mtimes; a fresh checkout/archive can reset them. Do not generalize fixture archive
evidence into a measured deployed-build result. Profile/features, build-script
inputs, toolchain, source/target absolute paths and environment can independently
invalidate Cargo and remain part of future evidence.

## Prototype contract

`scripts/build/source_mirror.py` uses Python3 standard library only on Linux.
Windows execution is rejected rather than pretending portable stat exposes every
reparse tag. Linux directory-fd/no-follow traversal rejects symbolic links,
hardlinked files, nonregular nodes, special permissions, escaped/dot/backslash/
colon paths and cross-device source subtrees. Input and cache must be disjoint.
Immutable read-only build input and an exclusively owned cache namespace are
required; this is not a sandbox against a malicious build script/root process.

A fresh cache root must be empty. The helper establishes a versioned owner marker
and a separate `source/target` pair. It never adopts an existing unowned Cargo
cache or recursively deletes the target. One flock protects synchronization AND
child completion. The child inherits the same open-description lock, so a killed
observer does not release ownership while accepted child work still runs. Use a
single BuildKit `sharing=locked` cache mount for the entire pair as well; don't
pair independently mutable source and target caches.

Normal synchronization streams byte comparisons in64KiB chunks. Equal bytes and
permissions retain inode/mtime; archive source mtime is not treated as content.
New/content/mode changes are atomic replacement files with current wall time
strictly newer than observed Cargo anchors. Removed source files/directories are
removed within fd-anchored owned mirror paths; parent-directory timestamps notice
add/remove changes. Source metadata is rechecked for concurrent modification.
The helper never assigns one constant timestamp to every source.

Clock/resolution trouble fails closed rather than backdating changed sources.
Bounded metadata-only scans cover1.98 debug/release host/cross-target fingerprint
files and fixed build-script timestamp/output anchors. There are caps on source
files50k, directories10k, path bytes8MiB, individual file64MiB, aggregate source
512MiB, path depth64 and anchor stats100k. These bounds are prototype limits,
not measured repository limits or hard process RSS promises. A failed/interrupted
sync leaves an owned dirty marker; retry rewrites source files with fresh times
before reuse, rather than trusting half-synchronized equal-looking files.

The normal CLI requires pinned cargo1.98, forces CARGO_TARGET_DIR to its owned
target and CARGO_BUILD_BUILD_DIR to that same directory (overriding source-local,
Cargo-home and inherited environment configuration), and refuses target/build-dir/manifest escape flags. Other inherited build/security settings remain unchanged.
Logs/receipts contain aggregate counts, not command environments. Python
availability in the exact frozen Rust image is NOT verified yet; if absent,
review an explicit pinned tool dependency before wiring. No unpinned installer or
nightly fallback is permitted.

## Proposed wiring, deliberately NOT implemented

1.Prepare the exact immutable Rust input snapshot, including BOTH rust-source
  files and fixture scripts used by include_str. Put the inert empty
  apps/web/dist/.gitkeep into that snapshot before synchronization. Do not create
  it after sync only to delete/recreate the generated directory next build.
2.Mount a NEW versioned source+target pair at one stable absolute location, with
  `sharing=locked`. The first migration build is cold; moving absolute paths or
  cache namespaces cannot promise reuse of the old /src/target artifacts.
3.Run the helper with the unchanged locked Cargo/profile/feature command while
  retaining the pair lock. Registry cache remains separate under Cargo's own
  locking. Copy successful executables/logs OUT of the cache mount as today.
4.Only after actual correctness/freshness and image-gate evidence, consider fixture
  wiring first; production deploy is a separate reviewed/measured change.

Example shape, not an instruction to run during another compile/measurement:

```
python3 /tool/source_mirror.py --input /snapshot --cache-root /owned-build-cache -- \
  cargo test --locked --release --features load-test-fixtures -p mw-server \
  --test native_load_fixture --no-run --message-format=json
```

## Deferred gates

`scripts/build/test_source_mirror.py` authors21 actual Linux filesystem/process
contracts: equal-byte mtime/inode preservation, same-length/backdated content,
removed/type-changed sources, mode changes, symlink/hardlink/FIFO refusal, unowned
cache, overlap/path guards, future anchors, interrupted-sync recovery, byte caps,
concurrent owner rejection, symlinked roots/anchor metadata caps, changing input and child-lock survival after parent
termination. Windows skips must NEVER be counted as passing these contracts.

`scripts/build/probe_cargo_source_mirror.py` authors an explicit empty-owned-root,
offline/locked3-package probe. It retains six phases of compiler JSON/dirty logs
and actual runtime outputs: first build; same bytes with new archive times (all
fresh); same-length old-mtime source change (only affected dependency/app rebuild);
removed referenced source (must fail, never run stale executable); restoration;
and real Cargo.lock byte change (actual build-script-generated token changes).
The probe is correctness/freshness evidence, not server build speed. It has no
network/ports or application data; timeouts/failures retain owned artifacts and
cannot establish success. Child compiler work may outlive a timed-out observer;
its inherited cache lock prevents unsafe reuse until the owned processes finish.

Future actual image validation must check unchanged dependency reuse, expected
fixture-only rebuild, real changed executable behavior, deletion failure, unchanged
production binary/gates, source/hash/profile provenance, and cold/warm elapsed/CPU
and I/O. Raw Docker build timing before those controls is insufficient.

CLI configuration is deliberately narrow: only the exact separate arguments
`--config profile.release.package.mw-server.opt-level=2` are supported. Config
files, other inline/quoted/escaped TOML expressions, `--config=...` and rustup
`+toolchain` selectors are rejected before child launch. The environment override
therefore does not claim to outrank arbitrary Cargo CLI configuration.

Input is an immutable snapshot throughout synchronization. Metadata revalidation
catches observable concurrent changes; it is not a filesystem snapshot and does
not detect every same-size mutation with coalesced/restored inode timestamps.
The race regression deliberately changes length. Across builds, streamed byte
comparison always runs, even when input mtime is identical; changed bytes never
qualify as unchanged merely because metadata matches. The retained initial Linux
19-case result (18 pass/1 failed same-tick mutable-input test) is not a passing
verification of the corrected 21-case source.


## Docker wiring candidate (source-only)

The production GNU/musl builds and fixture normal/symbol stages now bind their
`rust-source` snapshot read-only at `/mw-input`, launch Cargo through this helper,
and mount one `sharing=locked` source/target pair at `/mw-build-cache`. GNU uses
`mailwoman-source-target-gnu-v1-${TARGETPLATFORM}`; musl has its separate
`mailwoman-source-target-musl-v1` namespace. The outer BuildKit lock covers all
Cargo commands and artifact copies in a RUN; the inherited inner flock covers
individual accepted child lifetimes. Registry caching is unchanged.

The tracked `apps/web/dist/.gitkeep` is copied into the immutable snapshot,
per the existing exact `.dockerignore` exception. It is never generated or
constant-timestamped in the mirror. Fixture `scripts/perf` include inputs are
part of that snapshot. Existing `/src` remains available for diagnostic inspection
and fixture scripts, but is not the Cargo working tree. JSON executable paths
and timing/binary copies use actual pair target paths while the mount is held.
Normal/symbol fixture hash comparison and all seventeen fixture gates remain.
An eighteenth gate runs the actual internal Node driver via its shared command
builder and compiled source-root resolution after the cache is unmounted. A
separate post-compile RUN adds `/mw-build-cache/source -> /src` in the image,
never inside the mounted pair. Profiling temporarily overlays this image-only
compatibility link with the owned cache and restores its view on unmount.

GNU explicitly requires the pinned image's `python3` (actual availability was
observed as 3.11.2 in the frozen image). Alpine adds `python3` only to its existing
build tool package installation; scratch/distroless runtime contents do not
change. Alpine availability, package installation and both full Docker paths are
still unexecuted. The current first-use cache is cold; old target-only cache
contents are not migrated or claimed as reused. New absolute build paths can
change binary hashes and require a fresh release receipt; no bit-identical build
or application performance claim is made.

The isolated helper's twenty-one Linux contracts and six-phase real Cargo1.98.1
probe passed in root-owned runs before this wiring. They do not prove Docker
stage/mount/copy correctness. Required follow-up is actual Docker validation,
then an owned cold build, identical-byte/new-archive-mtime warm build, and changed
fixture/source build with retained Cargo dirty/fresh trace and executable/runtime
gates. GNU and musl must each be verified; profiler tools that invoke Cargo
directly against `/src` must adopt this same paired runner before claiming reuse.

CI cache-dance now exports/restores the whole musl pair under a new host-cache
subdirectory and `docker-mounts-source-pair-v2` key prefix including helper bytes.
It does not import the old target-only snapshot or remove old host caches.
Deploy-profile source contracts require both actual mirrored command launches,
both read-only input mounts and paired locks, and the tracked placeholder;
existing release/fast argument execution checks remain unchanged.
