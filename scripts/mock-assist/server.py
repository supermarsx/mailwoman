#!/usr/bin/env python3
"""Deterministic, offline mock of an OpenAI-compatible (+ Anthropic) endpoint.

Used by the `assist-mock` CI job (plan §5, §14) so the Assist gateway
(`mw-assist`) can be exercised end-to-end without any real AI provider or network
dependency. Every response is canned and stable, so the E2E is reproducible.

Routes:
  GET  /healthz                    -> 200 "ok" (compose healthcheck)
  POST /v1/chat/completions        -> OpenAI chat completion (SSE if stream=true)
  POST /v1/embeddings              -> OpenAI embeddings (content-dependent, see below)
  POST /v1/audio/transcriptions    -> OpenAI Whisper-style transcription
  POST /v1/messages                -> Anthropic Messages API (SSE if stream=true)

This server holds NO state and logs NOTHING about request bodies — it mirrors the
"content-free" posture the real gateway audits under. Stdlib only (no deps), so the
container is a plain `python:3-slim`.
"""
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

PORT = 8199

CHAT_TEXT = "This is a deterministic mock Assist reply for CI."
TRANSCRIPT = "deterministic mock transcription for ci"

# ── Embeddings (A8 semantic re-rank) ────────────────────────────────────────
#
# These MUST stay content-DEPENDENT. Until 26.19 this file returned one fixed
# 8-element vector for every input, which is a trap: every document then embeds
# identically, every cosine is equal, and the re-rank's (correct, tie-stable)
# result is the lexical order. An end-to-end test against such a mock passes
# while proving only that the plumbing runs — it asserts the wiring and reads as
# asserting the feature. If you are tempted to simplify this back to a constant,
# that is the bug you are re-introducing.
#
# The model is a hashed bag of words: each token lands in one bucket of a
# fixed-width vector, so texts sharing vocabulary sit near each other under
# cosine and texts that do not, do not. It mirrors `HashEmbedder` in
# `crates/mw-engine/src/search_semantic.rs`, which the Rust unit tests use.
#
# Determinism is load-bearing, in two directions:
#   * across processes — hence FNV-1a rather than Python's `hash()`, which is
#     randomised per interpreter for `str` unless PYTHONHASHSEED is pinned;
#   * across runs — the same text must always give the same vector, because
#     vectors are CACHED in the 0022 `message_embeddings` table and a second
#     query must reproduce the first query's ordering.
#
# EMBED_DIM must not change casually either: stored vectors of a different width
# are skipped (degrading to lexical), so changing it invalidates any cache a
# previous run left behind. That is safe by design, just wasteful.
EMBED_DIM = 64


def _embed(text):
    """A deterministic, content-dependent unit-ish vector for `text`."""
    vec = [0.0] * EMBED_DIM
    token = []
    for ch in str(text).lower():
        if ch.isalnum():
            token.append(ch)
            continue
        if token:
            _accumulate(vec, "".join(token))
            token = []
    if token:
        _accumulate(vec, "".join(token))
    # Never return a zero vector: it has no direction, so the consumer would
    # correctly refuse to rank it and the caller would see an unexplained
    # degradation instead of a mock that simply had nothing to say.
    if not any(vec):
        vec[0] = 1.0
    return vec


def _accumulate(vec, token):
    """FNV-1a 64-bit → one bucket. Stable across interpreters and platforms."""
    h = 0xCBF29CE484222325
    for b in token.encode("utf-8"):
        h ^= b
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    vec[h % EMBED_DIM] += 1.0


class Handler(BaseHTTPRequestHandler):
    # Silence the default per-request stderr logging (keep CI output clean and,
    # more importantly, never echo request lines that could carry a prompt).
    def log_message(self, *_a):  # noqa: D401
        return

    def _send(self, code, body, ctype="application/json"):
        payload = body.encode() if isinstance(body, str) else body
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def _sse(self, chunks):
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.end_headers()
        for c in chunks:
            self.wfile.write(f"data: {json.dumps(c)}\n\n".encode())
        self.wfile.write(b"data: [DONE]\n\n")
        self.wfile.flush()

    def do_GET(self):
        if self.path.rstrip("/") in ("/healthz", "/health"):
            self._send(200, "ok", "text/plain")
        else:
            self._send(404, json.dumps({"error": "not found"}))

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0) or 0)
        raw = self.rfile.read(length) if length else b""
        try:
            body = json.loads(raw) if raw else {}
        except ValueError:
            body = {}
        stream = bool(body.get("stream"))
        path = self.path.split("?", 1)[0].rstrip("/")

        if path == "/v1/chat/completions":
            if stream:
                self._sse([
                    {"choices": [{"index": 0, "delta": {"role": "assistant"}}]},
                    {"choices": [{"index": 0, "delta": {"content": CHAT_TEXT}}]},
                    {"choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}]},
                ])
            else:
                self._send(200, json.dumps({
                    "id": "chatcmpl-mock",
                    "object": "chat.completion",
                    "model": body.get("model", "mock"),
                    "choices": [{
                        "index": 0,
                        "message": {"role": "assistant", "content": CHAT_TEXT},
                        "finish_reason": "stop",
                    }],
                    "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2},
                }))
        elif path == "/v1/embeddings":
            # `input` is a string or a list of strings (OpenAI accepts both). The
            # in-tree adapter sends a string and reads `data[0].embedding`; the
            # list form is honoured so the mock stays faithful to the real API.
            inputs = body.get("input", "")
            if not isinstance(inputs, list):
                inputs = [inputs]
            self._send(200, json.dumps({
                "object": "list",
                "model": body.get("model", "mock-embed"),
                "data": [
                    {"object": "embedding", "index": i, "embedding": _embed(t)}
                    for i, t in enumerate(inputs)
                ],
                "usage": {"prompt_tokens": 1, "total_tokens": 1},
            }))
        elif path == "/v1/audio/transcriptions":
            self._send(200, json.dumps({"text": TRANSCRIPT}))
        elif path == "/v1/messages":
            # Anthropic Messages API.
            if stream:
                self._sse([
                    {"type": "message_start", "message": {"role": "assistant", "content": []}},
                    {"type": "content_block_delta", "index": 0,
                     "delta": {"type": "text_delta", "text": CHAT_TEXT}},
                    {"type": "message_stop"},
                ])
            else:
                self._send(200, json.dumps({
                    "id": "msg-mock",
                    "type": "message",
                    "role": "assistant",
                    "model": body.get("model", "mock"),
                    "content": [{"type": "text", "text": CHAT_TEXT}],
                    "stop_reason": "end_turn",
                    "usage": {"input_tokens": 1, "output_tokens": 1},
                }))
        else:
            self._send(404, json.dumps({"error": "not found"}))


if __name__ == "__main__":
    ThreadingHTTPServer(("0.0.0.0", PORT), Handler).serve_forever()
