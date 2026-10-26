#!/usr/bin/env python3
"""Optional real HTTPS WebDAV backup round trip; requires rclone and openssl."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import socket
import ssl
import subprocess
import sys
import tempfile
import time
import urllib.request

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--rclone', required=True, help='Absolute rclone executable path')
args = parser.parse_args()
rclone = Path(args.rclone).resolve(strict=True)
helper = Path(__file__).with_name('backup-destinations.py').resolve()
with tempfile.TemporaryDirectory(prefix='mailwoman-webdav-') as temporary:
    root = Path(temporary)
    remote, recovered = root / 'remote', root / 'recovered'
    remote.mkdir()
    recovered.mkdir()
    cert, key = root / 'cert.pem', root / 'key.pem'
    subprocess.run(['openssl', 'req', '-x509', '-newkey', 'rsa:2048', '-nodes',
                    '-keyout', str(key), '-out', str(cert), '-days', '1',
                    '-subj', '/CN=localhost', '-addext', 'subjectAltName=IP:127.0.0.1',
                    '-addext', 'basicConstraints=critical,CA:TRUE'],
                   check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        port = listener.getsockname()[1]
    url = f'https://127.0.0.1:{port}/'
    config = root / 'rclone.conf'
    config.write_text(f'[fixture]\ntype = webdav\nurl = {url}\nvendor = other\n')
    targets = root / 'targets.json'
    targets.write_text(json.dumps({'version': 1, 'destinations': [{
        'id': 'webdav', 'type': 'webdav', 'remote': 'fixture', 'path': 'mailwoman',
        'rcloneConfig': str(config), 'timeoutSeconds': 30}]}))
    archive = root / 'fixture.tar.age'
    # Opaque fixture bytes: this test covers transfer, not archive encryption.
    archive.write_bytes(b'opaque encrypted-archive fixture\n' * 4096)
    archive.with_name('fixture.release.env').write_text('IMAGE_ID=fixture\n')
    env = dict(os.environ, PATH=str(rclone.parent) + os.pathsep + os.environ.get('PATH', ''),
               RCLONE_CA_CERT=str(cert))
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    with open(root / 'server.log', 'wb') as log:
        server = subprocess.Popen([str(rclone), '--config', str(config), 'serve', 'webdav',
                                   str(remote), '--addr', f'127.0.0.1:{port}',
                                   '--cert', str(cert), '--key', str(key)],
                                  stdin=subprocess.DEVNULL, stdout=log, stderr=log,
                                  creationflags=flags)
        try:
            context = ssl.create_default_context(cafile=str(cert))
            for _ in range(100):
                try:
                    with urllib.request.urlopen(url, context=context, timeout=1):
                        break
                except OSError:
                    if server.poll() is not None:
                        raise RuntimeError('WebDAV fixture exited')
                    time.sleep(.1)
            else:
                raise RuntimeError('WebDAV fixture did not become ready')
            command = [sys.executable, str(helper)]
            subprocess.run(command + ['publish', '--config', str(targets), '--source', str(archive)], env=env, check=True)
            subprocess.run(command + ['fetch', '--config', str(targets), '--destination', 'webdav',
                                      '--name', archive.name, '--output', str(recovered)], env=env, check=True)
            assert hashlib.sha256(archive.read_bytes()).digest() == hashlib.sha256((recovered / archive.name).read_bytes()).digest()
            assert (recovered / 'fixture.release.env').read_bytes() == archive.with_name('fixture.release.env').read_bytes()
            # Tampering must fail without overwriting the known-good recovery copy.
            (remote / 'mailwoman' / archive.name).write_bytes(b'tampered')
            result = subprocess.run(command + ['fetch', '--config', str(targets), '--destination', 'webdav',
                                               '--name', archive.name, '--output', str(recovered)], env=env)
            assert result.returncode != 0
            assert (recovered / archive.name).read_bytes() == archive.read_bytes()
            print('PASS: real HTTPS WebDAV publication, verified retrieval, and tamper refusal')
        finally:
            server.terminate()
            try:
                server.wait(timeout=10)
            except subprocess.TimeoutExpired:
                server.kill()
                server.wait(timeout=10)
