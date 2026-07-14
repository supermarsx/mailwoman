# SPAMC/1.5 -> raw-TCP relay sidecar for the spam-classify live-E2E (t10-e14).
# Fronts the real `spamd` (SpamAssassin) so the jail can reach it over the host
# `http-fetch` (HTTP) even though spamd speaks the raw-TCP SPAMC protocol.
#
#   docker compose -f docker-compose.ci.yml up -d --wait spamd spamassassin
#
# Named `spamassassin` in compose so it matches the spam-spamassassin guest's compiled
# default endpoint (spamassassin:783) with zero plugin config. Stdlib-only Python — no
# deps, no pip install — same posture as scripts/mock-assist (mere aggregation, OUT of
# cargo-deny scope). See scripts/spamc-relay/relay.py.
FROM python:3.12-slim
WORKDIR /app
COPY scripts/spamc-relay/relay.py /app/relay.py
# The relay listens on 783 (the SPAMC default) and forwards to $SPAMD_ADDR.
ENV PYTHONUNBUFFERED=1
ENV SPAMD_ADDR=spamd:783
ENV RELAY_PORT=783
EXPOSE 783
CMD ["python", "/app/relay.py"]
