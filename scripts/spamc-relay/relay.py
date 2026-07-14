#!/usr/bin/env python3
"""SPAMC/1.5 -> raw-TCP relay (t10-e14 backend live-E2E).

The Mailwoman jail's ONLY egress is the host `http-fetch` import (HTTP). SpamAssassin's
`spamd` speaks the line-based SPAMC protocol over a raw TCP port (default 783), which is
NOT HTTP. The `spam-spamassassin` guest therefore emits the exact SPAMC request frame as
its HTTP POST body (see plugins/spam-spamassassin/src/component.rs) and parses the SPAMD
frame back out of the HTTP response body. This tiny sidecar bridges the two:

    guest --http POST /--> [ spamc-relay :783 ] --raw TCP--> spamd (SPAMD_ADDR)
          <----body------                       <----bytes---

The relay is deliberately named `spamassassin` in docker-compose so it matches the guest's
compiled default endpoint (`spamassassin:783`) with ZERO plugin config in a real
deployment. It forwards verbatim to the real spamd container at $SPAMD_ADDR.

Stdlib-only Python (no pip, no deps) — same posture as scripts/mock-assist/server.py:
a networked service, mere aggregation, OUT of cargo-deny / JS-license scope. The relay is
transport glue only; it neither parses nor mutates the SPAMC/SPAMD payload.
"""
import os
import socket
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

# host:port of the real spamd (the SpamAssassin daemon). In docker-compose this is the
# `spamd` service; overridable for local runs.
SPAMD_ADDR = os.environ.get("SPAMD_ADDR", "spamd:783")
LISTEN_HOST = os.environ.get("RELAY_HOST", "0.0.0.0")
LISTEN_PORT = int(os.environ.get("RELAY_PORT", "783"))
# Cap on a single exchange so a hostile/huge frame can never exhaust memory.
MAX_BYTES = 4 * 1024 * 1024


def _spamd_endpoint():
    host, _, port = SPAMD_ADDR.rpartition(":")
    return host or "spamd", int(port or "783")


def _relay(frame: bytes) -> bytes:
    """Send `frame` to spamd over raw TCP; return the full SPAMD response bytes."""
    host, port = _spamd_endpoint()
    with socket.create_connection((host, port), timeout=15) as sock:
        sock.sendall(frame)
        try:
            # SPAMC clients half-close the write side to signal end-of-request.
            sock.shutdown(socket.SHUT_WR)
        except OSError:
            pass
        chunks = []
        total = 0
        while total < MAX_BYTES:
            data = sock.recv(65536)
            if not data:
                break
            chunks.append(data)
            total += len(data)
        return b"".join(chunks)


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *_args):  # noqa: D401 - quiet by default
        pass

    def do_GET(self):
        if self.path == "/healthz":
            self._send(200, b"ok", "text/plain")
        else:
            self._send(404, b"", "text/plain")

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0") or "0")
        if length > MAX_BYTES:
            self._send(413, b"", "application/octet-stream")
            return
        frame = self.rfile.read(length) if length else b""
        try:
            reply = _relay(frame)
        except Exception as exc:  # noqa: BLE001 - fail-soft: guest maps non-2xx -> unknown
            sys.stderr.write(f"spamc-relay: spamd exchange failed: {exc}\n")
            self._send(502, b"", "application/octet-stream")
            return
        self._send(200, reply, "application/octet-stream")

    def _send(self, status, body, content_type):
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        if body:
            self.wfile.write(body)


def main():
    server = ThreadingHTTPServer((LISTEN_HOST, LISTEN_PORT), Handler)
    sys.stderr.write(
        f"spamc-relay listening on {LISTEN_HOST}:{LISTEN_PORT} -> spamd {SPAMD_ADDR}\n"
    )
    server.serve_forever()


if __name__ == "__main__":
    main()
