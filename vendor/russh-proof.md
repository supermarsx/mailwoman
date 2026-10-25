# russh provenance and P-521 feature patch

Validated 2026-10-25. The isolated proof below established this patch before its integration into `mw-ssh-tunnel`. Proof artifacts mentioned by filename are retained under `.orchestration/prototypes/russh-no-p521`; the patch and license are retained here with the vendored source. The application selects `default-features = false` with `ring,rsa`, and excludes this vendor directory from workspace membership.

Upstream russh 0.64.1 is the newest non-yanked release published at least fourteen days before this review, according to the primary crates.io API. Published 2026-10-05T06:53:32.454537Z. Registry metadata is retained in `registry-metadata.json`; the downloaded crate SHA256 matches its registry checksum `ba61e87b9ec9a39a59a6bbed4c0b8ff7fb07073405b24a767a1f2c8560ac8f58`. Upstream commit: `e5d80a07dc554480b6ea3b2e14adfc190407b627`. Apache-2.0 license text was fetched from that commit and retained as `russh-0.64.1/LICENSE.txt` (the registry archive omitted the root license).

Primary provenance URLs:

- https://crates.io/api/v1/crates/russh/0.64.1
- https://static.crates.io/crates/russh/russh-0.64.1.crate
- https://raw.githubusercontent.com/Eugeny/russh/e5d80a07dc554480b6ea3b2e14adfc190407b627/LICENSE.txt

`russh-no-p521.patch` is the complete eight-file diff against the unmodified registry archive. It makes P-521 optional and gates key exchange, advertisements, PKCS8 conversions, and positive P-521 tests. The default enables the new feature to preserve upstream defaults. The selected consumer enables only `ring,rsa`; Ed25519/P256/P384 remain supported. No changes to cryptographic arithmetic or ssh-key are needed. PKCS8's generic conversion helper has separate feature configurations to avoid unstable conditional where predicates. Unsupported P-521 encoding returns an explicit error.

The vendor MUST be excluded from workspace members, and its P-521/backend features must not be forwarded by application features. Otherwise Cargo workspace resolution can include optional prohibited packages even when the consumer disables them. The final isolated `Cargo.lock` and `cargo tree --all-features -e features` both contain no p521, aws-lc-rs, or aws-lc-sys. `artifact-hashes.json` records the exact proof files and changed upstream paths. `patch.py` applies the patch to a fresh archive extraction; it is not idempotent. `artifacts.py` regenerates the diff and hashes.

Verified gates (logs under `.orchestration/logs`):

- `codex-ssh-prototype-check.txt`: ring+rsa compile PASS.
- `codex-ssh-prototype-enabled.txt`: explicit `--features russh/p521` compile PASS, establishing both cfg branches compile. Full upstream default/aws-lc mode was deliberately not built.
- `codex-ssh-prototype-final.txt`: six tests PASS, 34.80 seconds. Four independent russh client/server TCP tests exercise Ed25519/P256/P384/RSA host and user keys, wrong user key refusal, wrong host pin refusal, and actual direct-tcpip echo. P256/P384 tests force their respective ECDH. All four key types round-trip PKCS8 and OpenSSH containers. Negative test checks P-521 advertisements/generation/PKCS8 decoding and encoding from a parsed P-521 wire representation.
- `codex-ssh-openssh.txt` and final suite: independent OpenSSH 10.3p1 interoperability PASS for all four key types, forced P256/P384 ECDH, wrong host pin/user key refusal, denied forwarding destination, and actual direct-tcpip HTTP body read from a separate HTTP service. Host pins compare key material, not comments. Certificates are refused by the proof handler.
- `codex-ssh-prototype-deny.txt`: `cargo deny --all-features check --config deny.toml` PASS: advisories, bans, licenses, sources. Expected unmatched project allowances and an upstream unused compression constant warning are present.
- `codex-ssh-prototype-tree.txt`, `codex-ssh-prototype-forbidden.txt`: no prohibited package in selected all-features graph or final lockfile.

Commands use `CARGO_TARGET_DIR=F:/mw-t32-h7-target`, sccache, incremental disabled, jobs12. From the repository root:

```powershell
$env:MW_SSH_OPENSSH_PROOF='127.0.0.1:38451'
cargo test --manifest-path .orchestration/prototypes/russh-no-p521/Cargo.toml -- --nocapture
cargo tree --manifest-path .orchestration/prototypes/russh-no-p521/Cargo.toml --all-features -e features
cargo deny --manifest-path .orchestration/prototypes/russh-no-p521/Cargo.toml --all-features check --config deny.toml
```

Independent fixture: owned disposable container `codex-ssh-proof`, loopback38451, Alpine3.24.1 digest `sha256:28bd5fe8b56d1bd048e5babf5b10710ebe0bae67db86916198a6eec434943f8b`. `openssh.sh` installs OpenSSH10.3_p1-r1 and busybox-extras only inside this container, generates synthetic test keys, permits only public-key root login, and restricts forwarding to its own127.0.0.1:8080. No host install, application image build, or shared container mutation. OpenSSL exists only inside this independent test server; it is not linked into the prototype. Server debug output is retained in `codex-ssh-openssh-server.txt`. Keys under `openssh/` and the two P-521 PEM fixtures are public test material, never deployment credentials.

The integrated runtime provides guarded pooled SSH channels, saved tunnel configuration, and hosting-panel routing; its scope and limits are documented in `docs/admin/ssh-tunnels.md`. The selected SSH dependency graph contains no P-521 or aws-lc. This is not a claim about the entire application lockfile: existing unrelated OpenPGP dependencies also use a different P-521 generation. The prototype's tests are additional interoperability evidence, not a substitute for the application's store, HTTP, and browser acceptance tests.
