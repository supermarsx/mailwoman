#!/usr/bin/env python3
"""Real S3/WebDAV/SFTP backup round trip; requires rclone, openssl and ssh-keygen."""
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
parser.add_argument('--method', choices=['webdav', 'sftp', 's3'], default='webdav')
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
    if args.method == 'webdav':
        config.write_text(f'[fixture]\ntype = webdav\nurl = {url}\nvendor = other\n')
        server_args = ['webdav', str(remote), '--addr', f'127.0.0.1:{port}', '--cert', str(cert), '--key', str(key)]
    elif args.method == 's3':
        config.write_text(f'[fixture]\ntype = s3\nprovider = Other\naccess_key_id = fixture\nsecret_access_key = fixture-secret\nendpoint = {url}\nforce_path_style = true\n')
        server_args = ['s3', str(remote), '--addr', f'127.0.0.1:{port}', '--cert', str(cert), '--key', str(key), '--auth-key', 'fixture,fixture-secret']
    else:
        host_key = root / 'host-key'
        subprocess.run(['ssh-keygen', '-t', 'ed25519', '-N', '', '-f', str(host_key), '-q'], check=True)
        known_hosts = root / 'known_hosts'
        known_hosts.write_text(f'[127.0.0.1]:{port} ' + host_key.with_suffix('.pub').read_text())
        password = subprocess.check_output([str(rclone), 'obscure', 'fixture-password'], text=True).strip()
        config.write_text(f'[fixture]\ntype = sftp\nhost = 127.0.0.1\nport = {port}\nuser = fixture\npass = {password}\nknown_hosts_file = {known_hosts}\n')
        server_args = ['sftp', str(remote), '--addr', f'127.0.0.1:{port}', '--key', str(host_key), '--user', 'fixture', '--pass', 'fixture-password']
    targets = root / 'targets.json'
    targets.write_text(json.dumps({'version': 1, 'destinations': [{
        'id': 'fixture', 'type': args.method, 'remote': 'fixture', 'path': 'mailwoman',
        'rcloneConfig': str(config), 'timeoutSeconds': 30}]}))
    archive = root / 'fixture.tar.age'
    # Opaque fixture bytes: this test covers transfer, not archive encryption.
    archive.write_bytes(b'opaque encrypted-archive fixture\n' * 4096)
    archive.with_name('fixture.release.env').write_text('IMAGE_ID=fixture\n')
    env = dict(os.environ, PATH=str(rclone.parent) + os.pathsep + os.environ.get('PATH', ''),
               RCLONE_CA_CERT=str(cert))
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    with open(root / 'server.log', 'wb') as log:
        server = subprocess.Popen([str(rclone), '--config', str(config), 'serve'] + server_args,
                                  stdin=subprocess.DEVNULL, stdout=log, stderr=log,
                                  creationflags=flags)
        try:
            context = ssl.create_default_context(cafile=str(cert))
            for _ in range(100):
                try:
                    if args.method == 'webdav':
                        with urllib.request.urlopen(url, context=context, timeout=1):
                            break
                    else:
                        with socket.create_connection(('127.0.0.1', port), timeout=1):
                            break
                except OSError:
                    if server.poll() is not None:
                        raise RuntimeError('Transport fixture exited')
                    time.sleep(.1)
            else:
                raise RuntimeError('Transport fixture did not become ready')
            command = [sys.executable, str(helper)]
            subprocess.run(command + ['publish', '--config', str(targets), '--source', str(archive)], env=env, check=True)
            subprocess.run(command + ['fetch', '--config', str(targets), '--destination', 'fixture',
                                      '--name', archive.name, '--output', str(recovered)], env=env, check=True)
            assert hashlib.sha256(archive.read_bytes()).digest() == hashlib.sha256((recovered / archive.name).read_bytes()).digest()
            assert (recovered / 'fixture.release.env').read_bytes() == archive.with_name('fixture.release.env').read_bytes()
            # Tampering must fail without overwriting the known-good recovery copy.
            (remote / 'mailwoman' / archive.name).write_bytes(b'tampered')
            result = subprocess.run(command + ['fetch', '--config', str(targets), '--destination', 'fixture',
                                               '--name', archive.name, '--output', str(recovered)], env=env)
            assert result.returncode != 0
            assert (recovered / archive.name).read_bytes() == archive.read_bytes()
            if args.method == 'sftp':
                (remote / 'mailwoman' / archive.name).write_bytes(archive.read_bytes())
                wrong_key = root / 'different-host-key'
                subprocess.run(['ssh-keygen', '-t', 'ed25519', '-N', '', '-f', str(wrong_key), '-q'], check=True)
                known_hosts.write_text(f'[127.0.0.1]:{port} ' + wrong_key.with_suffix('.pub').read_text())
                refused = subprocess.run(command + ['fetch', '--config', str(targets), '--destination', 'fixture',
                                                    '--name', archive.name, '--output', str(recovered)], env=env)
                assert refused.returncode != 0
                assert (recovered / archive.name).read_bytes() == archive.read_bytes()
                print('PASS: mismatched SSH host key refused')
            print(f'PASS: real {args.method} publication, verified retrieval, and tamper refusal')
        finally:
            server.terminate()
            try:
                server.wait(timeout=10)
            except subprocess.TimeoutExpired:
                server.kill()
                server.wait(timeout=10)
