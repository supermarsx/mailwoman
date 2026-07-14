# syntax=docker/dockerfile:1.7
#
# Mailwoman multi-stage build (SPEC §7.5: minimal, non-root runtime).
#
#   web     -> builds the SolidJS SPA (apps/web/dist)
#   build   -> compiles the Rust workspace (release), embedding the SPA
#   runtime -> production image: mailwoman + mw-render only, distroless non-root
#   mock    -> dev/E2E image: mw-mock-jmap (+ mailwoman for its healthcheck)
#
# Default target is `runtime`. The dev compose stack selects `mock` explicitly.
# Follow-up (SPEC §7.5): a FROM scratch + musl static build is a later
# hardening step; distroless/cc is used here to de-risk V0.

# ---------------------------------------------------------------------------
# Stage 1 — web: build the SPA. apps/web is a self-contained pnpm project
# (its own pnpm-workspace.yaml + pnpm-lock.yaml live under apps/web/).
# ---------------------------------------------------------------------------
FROM node:22-alpine AS web
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
# Stage 2 — build: compile the Rust workspace. The SPA must be in place at
# apps/web/dist BEFORE cargo runs so rust-embed bakes it into `mailwoman`.
# ---------------------------------------------------------------------------
FROM rust:1.95-bookworm AS build
WORKDIR /src
COPY . .
# Replace the committed dist placeholder with the freshly built SPA.
RUN rm -rf apps/web/dist
COPY --from=web /web/dist apps/web/dist
# Cache the cargo registry and target dir across builds; copy the finished
# binaries OUT of the (non-persisted) cache mount so later stages can COPY them.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --bin mailwoman --bin mw-render --bin mw-mock-jmap \
    && mkdir -p /out /data-empty \
    && cp target/release/mailwoman target/release/mw-render target/release/mw-mock-jmap /out/

# ---------------------------------------------------------------------------
# Stage 3 — runtime: production server image. Non-root, no shell, no package
# manager. Ships only the server and its render worker.
# ---------------------------------------------------------------------------
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
COPY --from=build /out/mailwoman   /usr/local/bin/mailwoman
COPY --from=build /out/mw-render   /usr/local/bin/mw-render
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
    MW_PLUGIN_DIR=/usr/lib/mailwoman/plugins
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
FROM gcr.io/distroless/cc-debian12:nonroot AS mock
COPY --from=build /out/mw-mock-jmap /usr/local/bin/mw-mock-jmap
COPY --from=build /out/mailwoman    /usr/local/bin/mailwoman
ENV MW_MOCK_PORT=8181
EXPOSE 8181
USER nonroot
HEALTHCHECK --interval=15s --timeout=5s --start-period=5s --retries=5 \
    CMD ["/usr/local/bin/mailwoman", "healthcheck", "--url", "http://127.0.0.1:8181/healthz"]
ENTRYPOINT ["/usr/local/bin/mw-mock-jmap"]
