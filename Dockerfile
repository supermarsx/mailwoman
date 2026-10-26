# syntax=docker/dockerfile:1.7
#
# Mailwoman multi-stage build (SPEC §7.5: minimal, non-root runtime).
#
#   web     -> builds the SolidJS SPA (apps/web/dist)
#   build   -> compiles the Rust workspace (release), independently of the SPA
#   runtime -> production image: mailwoman + mw-render only, distroless non-root
#   mock    -> dev/E2E image: mw-mock-jmap (+ mailwoman for its healthcheck)
#
# The deployment script selects `runtime-static`. The dev compose stack selects
# `mock` explicitly; `runtime` remains available for distroless deployments.
#
# Every base image is named by tag AND digest (`name:tag@sha256:<index digest>`),
# so a rebuild uses the same bytes until the digest is changed here.
# scripts/check-image-pins.sh fails on a FROM without one; Dependabot
# (.github/dependabot.yml) opens a pull request when a tag's digest moves.

# ---------------------------------------------------------------------------
# Stage 1 — web: build the SPA. apps/web is a self-contained pnpm project
# (its own pnpm-workspace.yaml + pnpm-lock.yaml live under apps/web/).
# ---------------------------------------------------------------------------
FROM node:22-alpine@sha256:fe4f6d74e20370acb04ad889077d14d0f64a9c8ea6a42bc60cddf1e844699914 AS web
RUN corepack enable
WORKDIR /web
# Copy manifests first so `pnpm install` layer caches on unchanged deps.
COPY apps/web/package.json apps/web/pnpm-lock.yaml apps/web/pnpm-workspace.yaml ./
RUN --mount=type=cache,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile
COPY apps/web/ ./
RUN pnpm build
# -> /web/dist

# ---------------------------------------------------------------------------
# Rust inputs are isolated from web/docs/deploy changes. Docker images serve the
# separately packaged SPA through the existing MW_WEB_DIR path; standalone Rust
# builds retain their embedded-asset support. UI-only deploys reuse the Rust layer.
# Both build stages need every workspace manifest, even for unselected members.
# ---------------------------------------------------------------------------
FROM scratch AS rust-source
COPY Cargo.toml Cargo.lock rust-toolchain.toml /src/
COPY .cargo /src/.cargo
COPY vendor /src/vendor
COPY crates /src/crates
COPY plugins /src/plugins
COPY apps/desktop/src-tauri /src/apps/desktop/src-tauri
COPY apps/mobile/src-tauri /src/apps/mobile/src-tauri
COPY apps/web/dist/.gitkeep /src/apps/web/dist/.gitkeep

# ---------------------------------------------------------------------------
# Stage 2 — build: compile the Rust workspace independently of the web build.
# ---------------------------------------------------------------------------
# The image's rustc must match `rust-toolchain.toml` (rust-source brings that file
# in, and a mismatch makes rustup download the pinned toolchain inside the build).
# Bump both together.
FROM rust:1.98.1-bookworm@sha256:93ce27a88655056a51dbdd8f5f2d7ddc071c7b0070fb288a37b5a285fc83971e AS build
WORKDIR /src
# Keep optimized workspace intermediates in the existing persistent target cache.
# Explicit 16 units preserves the release profile's previous codegen partitioning.
ARG MW_CARGO_INCREMENTAL=1
ARG MW_BUILD_JOBS=
ENV CARGO_INCREMENTAL=${MW_CARGO_INCREMENTAL} \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
COPY --from=rust-source /src/ /src/
# Cache the cargo registry and target dir across builds; copy the finished
# binaries OUT of the (non-persisted) cache mount so later stages can COPY them.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target,sharing=locked \
    if [ -n "$MW_BUILD_JOBS" ]; then export CARGO_BUILD_JOBS="$MW_BUILD_JOBS"; fi \
    && cargo build --locked --release --bin mailwoman --bin mw-render --bin mw-mock-jmap \
    && mkdir -p /out /data-empty \
    && cp target/release/mailwoman target/release/mw-render target/release/mw-mock-jmap /out/

# ---------------------------------------------------------------------------
# Stage 3 — runtime: production server image. Non-root, no shell, no package
# manager. Ships only the server and its render worker.
# ---------------------------------------------------------------------------
FROM gcr.io/distroless/cc-debian12:nonroot@sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f AS runtime
COPY --from=build /out/mailwoman   /usr/local/bin/mailwoman
COPY --from=build /out/mw-render   /usr/local/bin/mw-render
COPY --from=web /web/dist /usr/lib/mailwoman/web
# First-party bridge/plugin `.wasm` components (26.9, t9-e5): no longer embedded in
# the binary — shipped as external, read-only data files and digest-pinned by the
# server (`v7_mount::FIRST_PARTY_DIGESTS`), which loads them from MW_PLUGIN_DIR and
# fails closed on a missing/tampered component. Canonical shipped layout: plugins/dist.
COPY --from=build /src/plugins/dist /usr/lib/mailwoman/plugins
# Writable data dir owned by the distroless `nonroot` user (uid 65532). The dir
# is created empty in the build stage; --chown sets ownership on copy. A named
# volume mounted here inherits this ownership, so the server can create the DB.
COPY --from=build --chown=65532:65532 /data-empty /data
ENV MW_BIND=0.0.0.0:8080 \
    MW_DB_PATH=/data/mailwoman.db \
    MW_RENDER_BIN=/usr/local/bin/mw-render \
    MW_PLUGIN_DIR=/usr/lib/mailwoman/plugins \
    MW_WEB_DIR=/usr/lib/mailwoman/web
EXPOSE 8080
VOLUME ["/data"]
USER nonroot
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["/usr/local/bin/mailwoman", "healthcheck"]
ENTRYPOINT ["/usr/local/bin/mailwoman"]
CMD ["serve"]

# ---------------------------------------------------------------------------
# Stage 4 — mock: in-repo JMAP mock for deterministic E2E. Bundles `mailwoman`
# solely so the compose healthcheck can probe the mock's /healthz without a
# shell or curl in the distroless image.
# ---------------------------------------------------------------------------
FROM gcr.io/distroless/cc-debian12:nonroot@sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f AS mock
COPY --from=build /out/mw-mock-jmap /usr/local/bin/mw-mock-jmap
COPY --from=build /out/mailwoman    /usr/local/bin/mailwoman
ENV MW_MOCK_PORT=8181
EXPOSE 8181
USER nonroot
HEALTHCHECK --interval=15s --timeout=5s --start-period=5s --retries=5 \
    CMD ["/usr/local/bin/mailwoman", "healthcheck", "--url", "http://127.0.0.1:8181/healthz"]
ENTRYPOINT ["/usr/local/bin/mw-mock-jmap"]

# ---------------------------------------------------------------------------
# Stage 5 — build-musl: statically link the server + render worker against musl
# libc, so the runtime image can be `FROM scratch` (no libc, no shell, no
# package manager, nothing to attack). The whole floor is pure-Rust/rustls, so
# no C/OpenSSL is pulled in and the static link is self-contained.
#
# The Alpine rust image targets musl natively; RUSTFLAGS static-links the
# crt so the binaries have zero shared-object dependencies.
# ---------------------------------------------------------------------------
# Same pin as the `build` stage above — keep it equal to `rust-toolchain.toml`.
FROM rust:1.98.1-alpine@sha256:7cc1c22d77d9432f7fe012a70e6d3e555af54c2a6832700ed7d553f1769ae89f AS build-musl
# build-base = gcc + make + musl-dev, needed to compile the C/asm in `ring` and
# `zstd-sys` against musl for a fully static link. ca-certificates provides the
# root bundle copied into the scratch image for rustls upstream TLS.
RUN apk add --no-cache build-base ca-certificates
WORKDIR /src
ARG MW_CARGO_INCREMENTAL=1
ARG MW_BUILD_JOBS=
ENV RUSTFLAGS="-C target-feature=+crt-static" \
    CARGO_INCREMENTAL=${MW_CARGO_INCREMENTAL} \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
COPY --from=rust-source /src/ /src/
# Build a nonroot passwd/group and an empty data dir to carry into scratch.
RUN printf 'nonroot:x:65532:65532:nonroot:/data:/sbin/nologin\n' > /etc/mw-passwd \
    && printf 'nonroot:x:65532:\n' > /etc/mw-group \
    && mkdir -p /data-empty /tmp-empty
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target,sharing=locked \
    if [ -n "$MW_BUILD_JOBS" ]; then export CARGO_BUILD_JOBS="$MW_BUILD_JOBS"; fi \
    && cargo build --locked --release --target x86_64-unknown-linux-musl \
        --bin mailwoman --bin mw-render \
    && mkdir -p /out-musl \
    && cp target/x86_64-unknown-linux-musl/release/mailwoman \
          target/x86_64-unknown-linux-musl/release/mw-render /out-musl/

# ---------------------------------------------------------------------------
# Stage 6 — runtime-static: FROM scratch. The hardened production image (X3).
# Ships only the two static binaries, the CA bundle (for rustls upstream TLS),
# a single nonroot passwd/group entry, and the read-only first-party plugins.
# No libc, no shell, no coreutils — the attack surface is the binary alone.
#
# Build/select explicitly:  docker build --target runtime-static -t mailwoman:static .
# This final stage is also Docker's default when no target is specified.
# ---------------------------------------------------------------------------
FROM scratch AS runtime-static
COPY --from=build-musl /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build-musl /etc/mw-passwd /etc/passwd
COPY --from=build-musl /etc/mw-group  /etc/group
COPY --from=build-musl /out-musl/mailwoman /usr/local/bin/mailwoman
COPY --from=build-musl /out-musl/mw-render /usr/local/bin/mw-render
COPY --from=web /web/dist /usr/lib/mailwoman/web
# Plugins are committed, read-only data files — take them from the musl build
# context so the scratch image never depends on the glibc `build` stage.
COPY --from=build-musl /src/plugins/dist /usr/lib/mailwoman/plugins
# Writable data dir + /tmp, owned by the nonroot uid. A named volume mounted at
# /data inherits this ownership so the server can create its DB.
COPY --from=build-musl --chown=65532:65532 /data-empty /data
COPY --from=build-musl --chown=65532:65532 /tmp-empty  /tmp
ENV MW_BIND=0.0.0.0:8080 \
    MW_DB_PATH=/data/mailwoman.db \
    MW_RENDER_BIN=/usr/local/bin/mw-render \
    MW_PLUGIN_DIR=/usr/lib/mailwoman/plugins \
    MW_WEB_DIR=/usr/lib/mailwoman/web \
    SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
EXPOSE 8080
VOLUME ["/data"]
USER 65532:65532
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["/usr/local/bin/mailwoman", "healthcheck"]
ENTRYPOINT ["/usr/local/bin/mailwoman"]
CMD ["serve"]
