# Server build metadata boundary prototype

Status: source prototype. No compiler, runtime or build-speed result is claimed.

`mw-server` retains its package name, `mw_server` public host API, `mailwoman`
binary and integration-test directory. Its library target is now the small
`src/facade.rs`. It owns the revision-sensitive build script and compiled public
metadata, backend workspace inventory and SECURITY.md bytes.

`mw-server-core` compiles the existing `crates/mw-server/src/lib.rs` and modules
through an explicit library path. Source files stay in place, so later security
and asset-cache changes retain their paths. This private workspace crate has no
build script. It must not acquire a production edge back to the facade or the
metadata producer. Its current library name remains `mw_server` under the
facade's `server_core` dependency alias to preserve tracing module targets.
Actual Cargo alias/output-filename compatibility is a required gate, not an
assumption: the tiny build-profile probe rejects filename collisions. If it
fails, the prototype needs a reviewed log-filter compatibility design before
acceptance; warnings must not be ignored to obtain a faster-looking build.

`mw-build-metadata` contains pure generation functions. Inputs are explicit
bytes/scalars; the library has no build script, filesystem, environment or Git
discovery. It is a facade build dependency and a **dev-only** core dependency.
Core unit fixtures use it at test execution against the current owned checkout,
including real source hashes, provenance and executable-compatible target.
Missing provenance or a cross-target mismatch is an error, not an empty or
invented inventory. `MW_TEST_BUILD_TARGET` names the exact target when the
runner's rustc host differs from the executable.

The facade preserves all five `build_app*` constructor signatures. Each supplies
a validated immutable `ProductBuildInfo`; direct implementation-crate builders
require it explicitly. Startup checks the version, revision, policy/SBOM hashes,
component identities and resolved dependency references. AppState retains shared
immutable bytes, and the public routes keep their content types, download names
and cache behavior. No request reads Git, source files or the database for these
facts. The dashboard's build SHA uses the same injected revision instead of a
separate `option_env!` in the large crate.

The existing generator scope stays honest: it inventories the workspace lock,
including development/platform dependencies; it does not claim exact shipped
membership or conformity assessment. All source hashes and support-date fields
remain available. The exporter still selects the exact `mw-server` package's
build-script output and now also verifies/exports its compiled policy bytes.

## Source archive provenance

Docker's Rust source stage has no `.git`. Supply a full immutable source commit:

```sh
docker build --build-arg MW_BUILD_REVISION="$(git rev-parse HEAD)" --target runtime-static .
```

The same argument supplies the OCI revision label and compiled facts. The normal
deployment script derives/verifies it from the checkout; Compose accepts
`MW_BUILD_REVISION` (or CI's `GITHUB_SHA`). A source archive without explicit
provenance fails. The legacy `MW_BUILD_SHA` alias must agree with the full commit;
partial hashes and conflicting provenance are rejected. `MW_BUILD_RELEASE_TAG`
is explicit for an archive, otherwise an exact Git tag or undeclared.
`SOURCE_DATE_EPOCH` remains optional, validated Unix seconds. A support-end date
is never inferred from the rolling-release policy.

## Gate mapping

| Before | Prototype |
| --- | --- |
| `cargo test -p mw-server --lib <historical-filter>` | `cargo test -p mw-server-core --lib <historical-filter>` |
| Server implementation unit tests | Core unit tests, at their original module paths |
| `cargo test -p mw-server --test <target>` | Unchanged integration target and binary environment |
| `cargo build -p mw-server` / `--bin mailwoman` | Unchanged host package/binary |
| Native fixture targets/features | Unchanged host targets; features forwarded to core |
| Workspace unit/integration gates | Include both workspace members; do not substitute facade-only units |

New focused source-authored cases: six pure producer cases, three core metadata
cases, one facade compiled-data case, one actual router integration case, and one
additional Node exporter case. Existing standalone restart/vacuum/search process
tests retain `CARGO_BIN_EXE_mailwoman`. Actual counts and skips must be recorded
when run; none were executed while authoring this prototype.

The optional `scripts/test-deploy-build-profile.sh --cargo-probe` first checks
the proposed two-crate alias/external-source layout with tiny actual crates,
legacy log targets, both fast-profile overrides, overflow/unwind behavior and a
metadata-only edit leaving the core fresh. Its result is a prerequisite; it is
not a measurement of the full application.

## Measurement required

Capture all core/facade/binary timing units. Renaming the old large unit does not
remove cold compiler work and is not a speedup. Use the same source, jobs,
toolchain, features, inputs and codegen flags for matched builds, with other
compilers/load stopped. Record cold work, a repeat, and the same small handler
edit. Separately change only the frontend lock, SECURITY.md or revision in an
owned checkout: actual Cargo artifacts must show core reuse while the served
metadata/hash changes correctly. A lock change that changes effective dependency
versions, or a package-version/compiler/profile change, can legitimately rebuild
the core. Release defaults, LTO, codegen units and CPU target flags are unchanged.
