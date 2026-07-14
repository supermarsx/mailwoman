#!/usr/bin/env python3
"""Deterministic, offline mock of an OpenAI-compatible (+ Anthropic) endpoint.

Used by the `assist-mock` CI job (plan §5, §14) so the Assist gateway
(`mw-assist`) can be exercised end-to-end without any real AI provider or network
dependency. Every response is canned and stable, so the E2E is reproducible.

Routes:
  GET  /healthz                    -> 200 "ok" (compose healthcheck)
  POST /v1/chat/completions        -> OpenAI chat completion (SSE if stream=true)
  POST /v1/embeddings              -> OpenAI embeddings (fixed 8-dim vector)
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
EMBEDDING = [0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875]
TRANSCRIPT = "deterministic mock transcription for ci"


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
            self._send(200, json.dumps({
                "object": "list",
                "model": body.get("model", "mock-embed"),
                "data": [{"object": "embedding", "index": 0, "embedding": EMBEDDING}],
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
