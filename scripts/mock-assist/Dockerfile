# Mock OpenAI-compatible (+ Anthropic) endpoint for the assist-mock CI job
# (plan §5, §14). Deterministic + offline so the Assist E2E is reproducible; no
# real provider, no network egress. Stdlib-only Python — no pip install, no deps.
#
#   docker compose -f docker-compose.ci.yml up -d --wait mock-assist
#
# Listens on :8199. See scripts/mock-assist/server.py for the canned responses.
FROM python:3.12-slim
WORKDIR /app
COPY scripts/mock-assist/server.py /app/server.py
EXPOSE 8199
# Unbuffered so container logs flush immediately in CI.
ENV PYTHONUNBUFFERED=1
CMD ["python", "/app/server.py"]
