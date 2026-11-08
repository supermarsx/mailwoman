#!/usr/bin/env python3
"""Reproduce the self-hosted, pinned Mozilla PDF.js generic viewer (stdlib only).

Run from any directory: python apps/web/scripts/vendor-pdf-viewer.py [--check].
The archive is authenticated by its pinned SHA-256 before any file is extracted.
Our bootstrap/theme are separate files, never patches to upstream JavaScript.
"""
import hashlib
import io
import json
from pathlib import Path
import re
import sys
import urllib.request
import zipfile

VERSION = "6.1.200"
SHA256 = "9e1584d768ed099aa4be27ea423f89a038c2005f1ee417ea4f35ba4591ec1846"
WORKER_SHA256 = "2ab9e09667296dab1a618868b3ce6e6c23d5b8f48120ae7c5b34e7e335ed01fa"
URL = f"https://github.com/mozilla/pdf.js/releases/download/v{VERSION}/pdfjs-{VERSION}-dist.zip"
ROOT = Path(__file__).resolve().parents[1] / "public" / "pdfjs"
MANIFEST = ROOT / "manifest.json"
FRAME_CSP = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self' data:; style-src 'self' 'unsafe-inline'; img-src 'self' blob: data:; font-src 'self' blob: data:; connect-src 'self' blob: data:; media-src blob:; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'; require-trusted-types-for 'script'; trusted-types default"


def frame_document():
    html = (ROOT / "web/viewer.html").read_text()
    html = re.sub(r'(http-equiv="Content-Security-Policy"\s+content=")[^"]+', lambda match: match[1] + FRAME_CSP, html)
    # Stylesheets first, then the module script: a script ahead of a pending stylesheet makes Firefox log
    # "Layout was forced before the page was fully loaded" for the layout reads the viewer makes on start.
    html = html.replace('<script src="../build/pdf.mjs" type="module"></script>', '<link rel="stylesheet" href="../mailwoman.css" />')
    html = html.replace('<link rel="stylesheet" href="viewer.css" />', '<link rel="stylesheet" href="viewer.css" />\n<script src="../bootstrap.mjs" type="module"></script>')
    return html.replace('<script src="viewer.mjs" type="module"></script>', '')


def digest(data):
    return hashlib.sha256(data).hexdigest()


def selected(name):
    if name in ("LICENSE", "build/pdf.mjs", "web/viewer.html", "web/viewer.mjs", "web/viewer.css"):
        return True
    return name.startswith(("web/images/", "web/cmaps/", "web/standard_fonts/", "web/iccs/", "web/locale/", "web/wasm/")) and not name.endswith("/") and "quickjs" not in name


def main():
    if "--check" in sys.argv:
        manifest = json.loads(MANIFEST.read_text())
        assert manifest["version"] == VERSION and manifest["archiveSha256"] == SHA256
        assert digest((ROOT.parent / "pdf.worker.mjs").read_bytes()) == WORKER_SHA256, "Self-hosted PDF worker must match the pinned viewer version"
        for name, expected in manifest["files"].items():
            assert digest((ROOT / name).read_bytes()) == expected, f"PDF.js asset changed: {name}"
        assert (ROOT / "web/mailwoman.html").read_text() == frame_document(), "PDF viewer entry must match reproducible transformation"
        print(f"PDF.js {VERSION}: {len(manifest['files'])} pinned upstream assets verified")
        return
    # A local archive is convenient for offline rebuilds; it is verified identically.
    archive = Path(sys.argv[1]).read_bytes() if len(sys.argv) > 1 else urllib.request.urlopen(URL).read()
    assert digest(archive) == SHA256, "PDF.js archive checksum mismatch"
    files = {}
    with zipfile.ZipFile(io.BytesIO(archive)) as source:
        worker = source.read("build/pdf.worker.mjs")
        assert digest(worker) == WORKER_SHA256
        (ROOT.parent / "pdf.worker.mjs").write_bytes(worker)
        for name in source.namelist():
            if not selected(name):
                continue
            assert ".." not in Path(name).parts and not name.startswith("/")
            data = source.read(name)
            target = ROOT / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            files[name] = digest(data)
    MANIFEST.write_text(json.dumps({"version": VERSION, "source": URL, "archiveSha256": SHA256, "license": "Apache-2.0 (additional font/CMap/codec notices retained)", "files": files}, indent=2) + "\n")
    (ROOT / "web/mailwoman.html").write_text(frame_document())
    print(f"Vendored {len(files)} PDF.js {VERSION} assets; no maps, sample PDF, PDF scripting or AI runtime")


if __name__ == "__main__":
    main()
