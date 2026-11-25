# syntax=docker/dockerfile:1.7
# Isolated Linux multi-user fixture. Build from the repository root, then run with
# explicit CPU/memory limits and a bind mount for its synthetic reports.
ARG MW_BUILD_REVISION
ARG MW_BUILD_RELEASE_TAG=
FROM scratch AS rust-source
COPY Cargo.toml Cargo.lock rust-toolchain.toml SECURITY.md /src/
COPY .cargo /src/.cargo
COPY vendor /src/vendor
COPY crates /src/crates
COPY plugins /src/plugins
COPY apps/desktop/src-tauri /src/apps/desktop/src-tauri
COPY apps/mobile/src-tauri /src/apps/mobile/src-tauri
COPY apps/web/pnpm-lock.yaml /src/apps/web/pnpm-lock.yaml
# Include the inert placeholder in the immutable input, never mutate mirrored sources.
COPY apps/web/dist/.gitkeep /src/apps/web/dist/.gitkeep
COPY apps/web/public/workspace-popup.html /src/apps/web/public/workspace-popup.html
COPY apps/web/public/pdfjs /src/apps/web/public/pdfjs
COPY apps/web/public/pdf.worker.mjs /src/apps/web/public/pdf.worker.mjs
COPY apps/web/src/api/linked-scope.json /src/apps/web/src/api/linked-scope.json
COPY apps/web/src/api/native-read-intent-contract.json /src/apps/web/src/api/native-read-intent-contract.json
COPY apps/web/src/api/websocket-action-contract.json /src/apps/web/src/api/websocket-action-contract.json
COPY apps/web/src/viewers/previewTools.json /src/apps/web/src/viewers/previewTools.json
# Fixture include_str! inputs must participate in the same immutable snapshot.
COPY scripts/perf /src/scripts/perf

FROM node:24.20.0-bookworm@sha256:be23f54a88d34e8824c741b19b91064094f92c1c97b194144bfc8b50d67258e2 AS node
FROM rust:1.98.1-bookworm@sha256:93ce27a88655056a51dbdd8f5f2d7ddc071c7b0070fb288a37b5a285fc83971e AS compile-fixture
ARG MW_BUILD_REVISION
ARG MW_BUILD_RELEASE_TAG
ENV MW_BUILD_REVISION=${MW_BUILD_REVISION} MW_BUILD_RELEASE_TAG=${MW_BUILD_RELEASE_TAG}
ARG MW_BUILD_JOBS=8
ARG MW_FIXTURE_KIND=native
ARG MW_FIXTURE_PROFILE=release
ARG MW_BUILD_PROFILE=release
# 1 builds the native fixture with the sql-shape-diagnostics feature (statement attribution by phase and template;
# bench rig R1 diagnostics leg). Its numbers are counts only, never throughput. 0 (default) is the ordinary build.
ARG MW_FIXTURE_DIAGNOSTICS=0
ARG TARGETPLATFORM
ENV RUSTUP_TOOLCHAIN=1.98.1 CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
WORKDIR /src
COPY --from=rust-source /src/ /src/
COPY scripts/build/source_mirror.py /usr/local/lib/mailwoman-build/source_mirror.py
COPY --from=node /usr/local/bin/node /usr/local/bin/node
RUN --mount=type=bind,from=rust-source,source=/src,target=/mw-input,ro \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,id=mailwoman-source-target-gnu-v1-${TARGETPLATFORM},target=/mw-build-cache,sharing=locked \
    printf '%s' "$MW_BUILD_REVISION" | grep -Eq '^[0-9a-f]{40}$' \
    && python3 --version \
    && mirror_cargo() { python3 /usr/local/lib/mailwoman-build/source_mirror.py --input /mw-input --cache-root /mw-build-cache -- "$@"; } \
    && mkdir -p /out \
    && case "$MW_FIXTURE_PROFILE" in release) set -- --release ;; debug) set -- ;; *) echo 'Choose release or debug profile' >&2; exit 1 ;; esac \
    && case "$MW_BUILD_PROFILE" in release) export MW_CARGO_BUILD_MODE="$MW_FIXTURE_PROFILE" ;; fast) [ "$MW_FIXTURE_PROFILE" = release ] || exit 1; set -- "$@" --config 'profile.release.package.mw-server-core.opt-level=2' --config 'profile.release.package.mw-server.opt-level=2'; export MW_CARGO_BUILD_MODE=fast ;; *) echo 'Choose release or fast build profile' >&2; exit 1 ;; esac \
    && case "$MW_FIXTURE_DIAGNOSTICS" in 0) features=load-test-fixtures ;; 1) [ "$MW_FIXTURE_KIND" = native ] || exit 1; features=sql-shape-diagnostics ;; *) echo 'Choose MW_FIXTURE_DIAGNOSTICS 0 or 1' >&2; exit 1 ;; esac     && case "$MW_FIXTURE_KIND" in native) fixture=native_load_fixture; set -- "$@" --features "$features" ;; proxy) fixture=request_load_fixture ;; *) echo 'Choose native or proxy fixture' >&2; exit 1 ;; esac \
    && export CARGO_BUILD_JOBS="${MW_BUILD_JOBS:-8}" \
    && { mirror_cargo cargo test --locked "$@" -p mw-server --test "$fixture" --no-run --timings --message-format=json > /out/artifacts.jsonl; status=$?; \
         if [ -f /mw-build-cache/target/cargo-timings/cargo-timing.html ]; then cp /mw-build-cache/target/cargo-timings/cargo-timing.html /out/native-cargo-timing.html; fi; \
         if [ "$status" = 0 ] && [ "$MW_FIXTURE_KIND" = native ]; then \
           mirror_cargo cargo build --locked "$@" -p mw-server --example websocket-load --message-format=json >> /out/artifacts.jsonl; status=$?; \
           if [ "$status" = 0 ]; then mirror_cargo cargo test --locked "$@" -p mw-server --example websocket-load -- --test-threads=1 > /out/websocket-tests.log 2>&1; status=$?; cat /out/websocket-tests.log; fi; \
         fi; echo "$status" > /out/build-status; } \
    && node -e 'const fs=require("fs");const rows=fs.readFileSync("/out/artifacts.jsonl","utf8").trim().split("\n").filter(Boolean).map(JSON.parse);for(const row of rows)if(row.reason==="compiler-message"&&row.message.level==="error")console.error(row.message.rendered);if(fs.readFileSync("/out/build-status","utf8").trim()==="0"){const copy=(artifact,path)=>{if(!artifact)throw Error("Missing fixture executable");fs.copyFileSync(artifact.executable,path);fs.chmodSync(path,0o755);};copy(rows.findLast(r=>r.reason==="compiler-artifact"&&r.target.name===process.argv[1]&&r.executable),"/out/request-load-fixture");if(process.argv[1]==="native_load_fixture")copy(rows.findLast(r=>r.reason==="compiler-artifact"&&r.target.name==="websocket-load"&&r.profile.test===false&&r.executable),"/out/websocket-load");}' "$fixture"
# CARGO_MANIFEST_DIR refers to the stable mirror path. Outside the cache mount,
# retain that runtime lookup for the fixture's actual internal Node driver.
# This RUN occurs only after the owned pair has been unmounted; profiling mounts
# hide this compatibility link temporarily and never install it inside the pair.
RUN mkdir -p /mw-build-cache && ln -s /src /mw-build-cache/source
# Keep the compile/cache/diagnostic layer even when compilation fails, while the
# public fixture target still fails the Docker build rather than shipping it.
FROM compile-fixture AS fixture
LABEL org.opencontainers.image.revision=${MW_BUILD_REVISION}
RUN test "$(cat /out/build-status)" = 0
RUN run_logged_test() { log_path="$1"; shift; "$@" > "$log_path" 2>&1; test_status=$?; cat "$log_path"; return "$test_status"; }; \
    if [ -x /out/websocket-load ]; then \
      run_logged_test /out/native-readiness-tests.log /out/request-load-fixture readiness_ --nocapture --test-threads=1 \
      && grep -q 'test result: ok. 2 passed;' /out/native-readiness-tests.log \
      && run_logged_test /out/native-transport-tests.log /out/request-load-fixture representative_transport_mutations --nocapture --test-threads=1 \
      && grep -q 'test result: ok. 1 passed;' /out/native-transport-tests.log \
      && run_logged_test /out/native-writer-options-tests.log /out/request-load-fixture diagnostic_index_worker_override_is_strict_and_memory_bounded --exact --nocapture --test-threads=1 \
      && grep -q 'test result: ok. 1 passed;' /out/native-writer-options-tests.log \
      && run_logged_test /out/native-storage-options-tests.log /out/request-load-fixture diagnostic_index_storage_defaults_sealed_and_rejects_unknown_modes --exact --nocapture --test-threads=1 \
      && grep -q 'test result: ok. 1 passed;' /out/native-storage-options-tests.log \
      && run_logged_test /out/native-index-timing-tests.log /out/request-load-fixture index_timing_switch_is_explicit_and_bounded --exact --nocapture --test-threads=1 \
      && grep -q 'test result: ok. 1 passed;' /out/native-index-timing-tests.log \
      && run_logged_test /out/native-cache-expectation-tests.log /out/request-load-fixture cache_experiment_expectation_is_explicit_and_bounded --nocapture --test-threads=1 \
      && grep -q 'test result: ok. 1 passed;' /out/native-cache-expectation-tests.log \
      && run_logged_test /out/native-cache-audit-tests.log /out/request-load-fixture diagnostics_hook::cache_audit_tests:: --nocapture --test-threads=1 \
      && grep -q 'test result: ok. 3 passed;' /out/native-cache-audit-tests.log \
      && run_logged_test /out/native-runtime-driver-tests.log /out/request-load-fixture fixture_runtime::internal_node_driver_uses_runtime_source_root_after_build_mount_release --exact --nocapture --test-threads=1 \
      && grep -q 'test result: ok. 1 passed;' /out/native-runtime-driver-tests.log; \
    fi
ENV MW_RUN_HTTP_LOAD_FIXTURE=1 MW_RUN_NATIVE_HTTP_LOAD_FIXTURE=1
CMD ["/out/request-load-fixture", "--ignored", "--nocapture", "--test-threads=1"]

# Optional profiling image: reuse the completed, gated release and its dependency
# cache, then retain symbols only in a separate diagnostic executable. The
# ordinary release artifact remains byte-identical and is still the default.
FROM fixture AS fixture-profile
ARG MW_BUILD_REVISION
ARG MW_BUILD_RELEASE_TAG
ENV MW_BUILD_REVISION=${MW_BUILD_REVISION} MW_BUILD_RELEASE_TAG=${MW_BUILD_RELEASE_TAG}
ARG MW_BUILD_JOBS=8
ARG MW_FIXTURE_KIND=native
ARG MW_FIXTURE_PROFILE=release
ARG MW_BUILD_PROFILE=release
ARG TARGETPLATFORM
RUN --mount=type=bind,from=rust-source,source=/src,target=/mw-input,ro \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,id=mailwoman-source-target-gnu-v1-${TARGETPLATFORM},target=/mw-build-cache,sharing=locked \
    printf '%s' "$MW_BUILD_REVISION" | grep -Eq '^[0-9a-f]{40}$' \
    && python3 --version \
    && mirror_cargo() { python3 /usr/local/lib/mailwoman-build/source_mirror.py --input /mw-input --cache-root /mw-build-cache -- "$@"; } \
    && test "$MW_FIXTURE_KIND" = native \
    && test "$MW_FIXTURE_PROFILE" = release \
    && test "$MW_BUILD_PROFILE" = release \
    && sha256sum /out/request-load-fixture > /out/release-before-profile.sha256 \
    && export CARGO_BUILD_JOBS="${MW_BUILD_JOBS:-8}" MW_CARGO_BUILD_MODE=release \
    && mirror_cargo cargo rustc --locked --release --features load-test-fixtures -p mw-server \
         --test native_load_fixture --message-format=json -- -Cstrip=none > /out/profile-artifacts.jsonl \
    && node -e 'const fs=require("fs");const rows=fs.readFileSync("/out/profile-artifacts.jsonl","utf8").trim().split("\n").filter(Boolean).map(JSON.parse);const row=rows.findLast(r=>r.reason==="compiler-artifact"&&r.target.name==="native_load_fixture"&&r.executable);if(!row)throw Error("Missing symbol fixture executable");fs.copyFileSync(row.executable,"/out/request-load-fixture-profile");fs.chmodSync("/out/request-load-fixture-profile",0o755);' \
    && sha256sum --check /out/release-before-profile.sha256 \
    && sha256sum /out/request-load-fixture /out/request-load-fixture-profile > /out/fixture-executables.sha256

# Optional "as shipped" fixture (t62-n12): the native fixture linked the way the shipped image is
# (root Dockerfile `build-musl` / `runtime-static`): Alpine rust image, x86_64-unknown-linux-musl,
# RUSTFLAGS="-C target-feature=+crt-static", so musl libc and musl's allocator, statically linked.
# The image is the ordinary fixture image (Node for the generator, scripts, gates) with only the
# fixture executable replaced; build it with `--target fixture-static`.
FROM rust:1.98.1-alpine@sha256:7cc1c22d77d9432f7fe012a70e6d3e555af54c2a6832700ed7d553f1769ae89f AS compile-fixture-static
ARG MW_BUILD_REVISION
ARG MW_BUILD_RELEASE_TAG
ARG MW_BUILD_JOBS=8
ARG MW_FIXTURE_DIAGNOSTICS=0
RUN apk add --no-cache build-base ca-certificates python3
ENV RUSTFLAGS="-C target-feature=+crt-static" RUSTUP_TOOLCHAIN=1.98.1 CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 \
    MW_BUILD_REVISION=${MW_BUILD_REVISION} MW_BUILD_RELEASE_TAG=${MW_BUILD_RELEASE_TAG} MW_CARGO_BUILD_MODE=release
WORKDIR /src
COPY --from=rust-source /src/ /src/
COPY scripts/build/source_mirror.py /usr/local/lib/mailwoman-build/source_mirror.py
RUN --mount=type=bind,from=rust-source,source=/src,target=/mw-input,ro \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,id=mailwoman-source-target-musl-fixture-v1,target=/mw-build-cache,sharing=locked \
    printf '%s' "$MW_BUILD_REVISION" | grep -Eq '^[0-9a-f]{40}$' \
    && mirror_cargo() { python3 /usr/local/lib/mailwoman-build/source_mirror.py --input /mw-input --cache-root /mw-build-cache -- "$@"; } \
    && case "$MW_FIXTURE_DIAGNOSTICS" in 0) features=load-test-fixtures ;; 1) features=sql-shape-diagnostics ;; *) echo 'Choose MW_FIXTURE_DIAGNOSTICS 0 or 1' >&2; exit 1 ;; esac \
    && export CARGO_BUILD_JOBS="${MW_BUILD_JOBS:-8}" \
    && mkdir -p /out-static \
    && mirror_cargo cargo test --locked --release -p mw-server --test native_load_fixture --no-run --features "$features" \
         --target x86_64-unknown-linux-musl --message-format=json > /out-static/artifacts.jsonl \
    && python3 -c 'import json,shutil,os,sys;rows=[json.loads(l) for l in open("/out-static/artifacts.jsonl") if l.strip()];row=[r for r in rows if r.get("reason")=="compiler-artifact" and r["target"]["name"]=="native_load_fixture" and r.get("executable")][-1];shutil.copyfile(row["executable"],"/out-static/request-load-fixture");os.chmod("/out-static/request-load-fixture",0o755)'
FROM fixture AS fixture-static
LABEL org.opencontainers.image.revision=${MW_BUILD_REVISION} mw.fixture.link="static musl (+crt-static, x86_64-unknown-linux-musl)"
COPY --from=compile-fixture-static /out-static/request-load-fixture /out/request-load-fixture
RUN ! ldd /out/request-load-fixture 2>/dev/null | grep -q 'libc.so' \
    && /out/request-load-fixture readiness_ --nocapture --test-threads=1 > /out/static-readiness-tests.log 2>&1; status=$?; cat /out/static-readiness-tests.log; \
    test "$status" = 0 && grep -q 'test result: ok. 2 passed;' /out/static-readiness-tests.log
