# SpamAssassin `spamd` for the spam-classify live-E2E (t10-e14, SPEC §10.8).
#
# A real SpamAssassin daemon listening on the raw-TCP SPAMC port 783. The jail cannot
# reach it directly (spamd is not HTTP); the `spamassassin` relay sidecar
# (scripts/spamc-relay) fronts it and bridges HTTP<->raw-TCP. GTUBE detection is built
# into SpamAssassin, so no rule training is needed for the deterministic spam fixture.
#
# SpamAssassin is Apache-2.0. This is a NETWORK-ONLY service (mere aggregation, same
# posture as Dovecot/Radicale/rspamd) — never linked into any crate/binary, OUT of
# cargo-deny scope by design (documented in deny.toml).
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends spamassassin \
    && rm -rf /var/lib/apt/lists/*
# Compile the stock rules once at build time so first-scan latency is bounded.
RUN sa-update || true
EXPOSE 783
# -x  : disable per-user config (single shared config)
# -A  : allow connections from any address (CI network); -i binds all interfaces
# --allowed-ips keeps the ACL permissive inside the isolated compose network only.
# -p 783 is the SPAMC port the relay forwards to.
CMD ["spamd", "--nouser-config", "-x", "-i", "0.0.0.0", "-p", "783", \
     "-A", "0.0.0.0/0", "--allowed-ips=0.0.0.0/0", "--max-children=2"]
