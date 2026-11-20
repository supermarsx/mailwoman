# syntax=docker/dockerfile:1.7
# Isolated Linux multi-user fixture. Build from the repository root, then run with
# explicit CPU/memory limits and a bind mount for its synthetic reports.
FROM scratch AS rust-source
COPY Cargo.toml Cargo.lock rust-toolchain.toml SECURITY.md /src/
COPY .cargo /src/.cargo
COPY vendor /src/vendor
COPY crates /src/crates
COPY plugins /src/plugins
COPY apps/desktop/src-tauri /src/apps/desktop/src-tauri
COPY apps/mobile/src-tauri /src/apps/mobile/src-tauri
COPY apps/web/pnpm-lock.yaml /src/apps/web/pnpm-lock.yaml
COPY apps/web/public/workspace-popup.html /src/apps/web/public/workspace-popup.html
COPY apps/web/public/pdfjs /src/apps/web/public/pdfjs
COPY apps/web/public/pdf.worker.mjs /src/apps/web/public/pdf.worker.mjs
COPY apps/web/src/api/linked-scope.json /src/apps/web/src/api/linked-scope.json
COPY apps/web/src/viewers/previewTools.json /src/apps/web/src/viewers/previewTools.json

FROM node:24.20.0-bookworm@sha256:be23f54a88d34e8824c741b19b91064094f92c1c97b194144bfc8b50d67258e2 AS node
FROM rust:1.98.1-bookworm@sha256:93ce27a88655056a51dbdd8f5f2d7ddc071c7b0070fb288a37b5a285fc83971e AS compile-fixture
ARG MW_BUILD_JOBS=8
ARG MW_FIXTURE_KIND=native
ARG MW_FIXTURE_PROFILE=release
ARG MW_BUILD_PROFILE=release
ARG TARGETPLATFORM
ENV RUSTUP_TOOLCHAIN=1.98.1 CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
WORKDIR /src
COPY --from=rust-source /src/ /src/
COPY --from=node /usr/local/bin/node /usr/local/bin/node
COPY scripts/perf /src/scripts/perf
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,id=mailwoman-target-gnu-ni-${TARGETPLATFORM},target=/src/target,sharing=locked \
    mkdir -p /out /src/apps/web/dist \
    && touch -t 200001010000 /src/apps/web/dist/.gitkeep \
    && case "$MW_FIXTURE_PROFILE" in release) set -- --release ;; debug) set -- ;; *) echo 'Choose release or debug profile' >&2; exit 1 ;; esac \
    && case "$MW_BUILD_PROFILE" in release) export MW_CARGO_BUILD_MODE="$MW_FIXTURE_PROFILE" ;; fast) [ "$MW_FIXTURE_PROFILE" = release ] || exit 1; set -- "$@" --config 'profile.release.package.mw-server.opt-level=2'; export MW_CARGO_BUILD_MODE=fast ;; *) echo 'Choose release or fast build profile' >&2; exit 1 ;; esac \
    && case "$MW_FIXTURE_KIND" in native) fixture=native_load_fixture; set -- "$@" --features load-test-fixtures ;; proxy) fixture=request_load_fixture ;; *) echo 'Choose native or proxy fixture' >&2; exit 1 ;; esac \
    && export CARGO_BUILD_JOBS="${MW_BUILD_JOBS:-8}" \
    && { cargo test --locked "$@" -p mw-server --test "$fixture" --no-run --timings --message-format=json > /out/artifacts.jsonl; status=$?; \
         if [ -f target/cargo-timings/cargo-timing.html ]; then cp target/cargo-timings/cargo-timing.html /out/native-cargo-timing.html; fi; \
         if [ "$status" = 0 ] && [ "$MW_FIXTURE_KIND" = native ]; then \
           cargo build --locked "$@" -p mw-server --example websocket-load --message-format=json >> /out/artifacts.jsonl; status=$?; \
           if [ "$status" = 0 ]; then cargo test --locked "$@" -p mw-server --example websocket-load -- --test-threads=1 > /out/websocket-tests.log 2>&1; status=$?; cat /out/websocket-tests.log; fi; \
         fi; echo "$status" > /out/build-status; } \
    && node -e 'const fs=require("fs");const rows=fs.readFileSync("/out/artifacts.jsonl","utf8").trim().split("\n").filter(Boolean).map(JSON.parse);for(const row of rows)if(row.reason==="compiler-message"&&row.message.level==="error")console.error(row.message.rendered);if(fs.readFileSync("/out/build-status","utf8").trim()==="0"){const copy=(artifact,path)=>{if(!artifact)throw Error("Missing fixture executable");fs.copyFileSync(artifact.executable,path);fs.chmodSync(path,0o755);};copy(rows.findLast(r=>r.reason==="compiler-artifact"&&r.target.name===process.argv[1]&&r.executable),"/out/request-load-fixture");if(process.argv[1]==="native_load_fixture")copy(rows.findLast(r=>r.reason==="compiler-artifact"&&r.target.name==="websocket-load"&&r.profile.test===false&&r.executable),"/out/websocket-load");}' "$fixture"
# Keep the compile/cache/diagnostic layer even when compilation fails, while the
# public fixture target still fails the Docker build rather than shipping it.
FROM compile-fixture AS fixture
RUN test "$(cat /out/build-status)" = 0
RUN run_logged_test() { log_path="$1"; shift; "$@" > "$log_path" 2>&1; test_status=$?; cat "$log_path"; return "$test_status"; }; \
    if [ -x /out/websocket-load ]; then \
      run_logged_test /out/native-readiness-tests.log /out/request-load-fixture readiness_ --nocapture --test-threads=1 \
      && grep -q 'test result: ok. 2 passed;' /out/native-readiness-tests.log \
      && run_logged_test /out/native-transport-tests.log /out/request-load-fixture representative_transport_mutations --nocapture --test-threads=1 \
      && grep -q 'test result: ok. 1 passed;' /out/native-transport-tests.log; \
    fi
ENV MW_RUN_HTTP_LOAD_FIXTURE=1 MW_RUN_NATIVE_HTTP_LOAD_FIXTURE=1
CMD ["/out/request-load-fixture", "--ignored", "--nocapture", "--test-threads=1"]
