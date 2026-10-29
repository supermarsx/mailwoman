#!/usr/bin/env python3
"""Live Nextcloud round trip for backup-destinations.py; never uses configured storage.

Requires rclone on PATH and a disposable Nextcloud account (for example the
nextcloud:stable image with an app password made by `occ user:auth-tokens:add`).
RCLONE_CA_CERT may name the CA of a test certificate. Checks: connection test with
folder creation (MKCOL), publication with verified download, chunked upload of a
large archive, fetch with tamper refusal, keep-based pruning, a wrong app password
and a folder the account may not create.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import subprocess
import sys
import tempfile
import urllib.parse

sys.path.insert(0, str(Path(__file__).resolve().parent))
import importlib.util

spec = importlib.util.spec_from_file_location("destinations", Path(__file__).with_name("backup-destinations.py"))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--url", required=True, help="HTTPS Nextcloud server URL")
parser.add_argument("--user", required=True)
parser.add_argument("--app-password-file", required=True)
parser.add_argument("--readonly-folder", help="A folder shared read-only with the user; creating a child must fail")
parser.add_argument("--large-mib", type=int, default=24)
args = parser.parse_args()
helper = [sys.executable, str(Path(__file__).with_name("backup-destinations.py"))]
password = Path(args.app_password_file).read_text(encoding="utf-8").strip()
run_id = secrets.token_hex(4)
folder = f"mw-t64-{run_id}/nested/backups"


def call(*command, expect=0):
    result = subprocess.run(helper + list(command), capture_output=True, text=True)
    output = result.stdout + result.stderr
    assert password not in output, "app password leaked into helper output"
    if expect is not None and result.returncode != expect:
        raise AssertionError(f"exit {result.returncode}, expected {expect}: {output}")
    return output


def remote_names(path):
    status, body = module.dav("PROPFIND", module.nextcloud_root({"url": args.url, "user": args.user}) +
                              "/".join(urllib.parse.quote(p) for p in path.split("/")) + "/", args.user, password, depth=1)
    assert status == 207, status
    text = body.decode()
    return {urllib.parse.unquote(part.split("<", 1)[0]).rstrip("/").rsplit("/", 1)[-1]
            for part in text.split("href>")[1::2]}


def snapshot(work, stamp, size):
    archive = work / f"{stamp}-1-{secrets.randbelow(10**6)}.tar.age"
    with open(archive, "wb") as handle:
        handle.write(secrets.token_bytes(size))
    archive.with_name(archive.name.rsplit(".tar.", 1)[0] + ".release.env").write_text("IMAGE_ID=fixture\n")
    return archive


with tempfile.TemporaryDirectory(prefix="mw-t64-nextcloud-") as temporary:
    work = Path(temporary)
    secret = work / "app-password"
    secret.write_text(password + "\n")
    secret.chmod(0o600)
    row = {"id": "cloud", "type": "nextcloud", "url": args.url, "user": args.user,
           "appPasswordFile": str(secret), "path": folder, "chunkSizeMiB": 5, "keep": 2, "timeoutSeconds": 600}
    config = work / "targets.json"
    config.write_text(json.dumps({"version": 1, "destinations": [row]}))

    output = call("check", "--config", str(config))
    print("PASS check (offline):", output.strip())
    output = call("test", "--config", str(config))
    assert "cloud: test complete" in output
    names = remote_names(folder)
    assert not any(name.startswith("mailwoman-probe-") for name in names), names
    print("PASS test: account check, nested folder created with MKCOL, probe written, verified and deleted")

    archives = []
    for index, stamp in enumerate(["20261001T000000Z", "20261002T000000Z", "20261003T000000Z"]):
        archives.append(snapshot(work, stamp, 200_000 + index))
        call("publish", "--config", str(config), "--source", str(archives[-1]))
    names = remote_names(folder)
    oldest = archives[0].name
    assert oldest not in names and oldest + ".complete.json" not in names, names
    for archive in archives[1:]:
        assert {archive.name, archive.name + ".complete.json", archive.name.rsplit(".tar.", 1)[0] + ".release.env"} <= names, names
    print("PASS publish x3 with keep=2: newest two complete sets on Nextcloud, oldest set pruned")

    large = snapshot(work, "20261004T000000Z", args.large_mib * 1024 * 1024 + 12345)
    call("publish", "--config", str(config), "--source", str(large))
    assert large.name in remote_names(folder)
    print(f"PASS large archive ({large.stat().st_size} bytes, 5 MiB chunks) published and verified by download")

    recovered = work / "recovered"
    recovered.mkdir()
    call("fetch", "--config", str(config), "--destination", "cloud", "--name", large.name, "--output", str(recovered))
    assert hashlib.sha256((recovered / large.name).read_bytes()).digest() == hashlib.sha256(large.read_bytes()).digest()
    print("PASS fetch: manifest and SHA-256 checked, archive recovered")

    target = module.nextcloud_root(row) + "/".join(urllib.parse.quote(p) for p in folder.split("/")) + "/" + large.name
    tampered = work / "tampered"
    tampered.write_bytes(b"tampered")
    subprocess.run(["curl", "-sS", "-o", os.devnull, "--fail", "-u", f"{args.user}:{password}", "-T", str(tampered), target]
                   + (["--cacert", os.environ["RCLONE_CA_CERT"]] if os.environ.get("RCLONE_CA_CERT") else []), check=True)
    other = work / "recovered-2"
    other.mkdir()
    output = call("fetch", "--config", str(config), "--destination", "cloud", "--name", large.name, "--output", str(other), expect=1)
    assert not (other / large.name).exists()
    print("PASS tampered remote copy refused:", output.strip().splitlines()[-1])

    wrong = work / "wrong-password"
    wrong.write_text("Wrong-Wrong-Wrong-Wrong-Wrong\n")
    wrong.chmod(0o600)
    config.write_text(json.dumps({"version": 1, "destinations": [dict(row, appPasswordFile=str(wrong))]}))
    output = call("test", "--config", str(config), expect=1)
    assert "rejected the username or app password" in output, output
    print("PASS wrong app password:", output.strip())

    if args.readonly_folder:
        config.write_text(json.dumps({"version": 1, "destinations": [dict(row, path=f"{args.readonly_folder}/cannot-create-{run_id}")]}))
        output = call("test", "--config", str(config), expect=1)
        assert "could not create the backup folder" in output, output
        print("PASS uncreatable folder:", output.strip())

    # One failing destination never blocks another.
    local = work / "local-copy"
    local.mkdir()
    config.write_text(json.dumps({"version": 1, "destinations": [dict(row, appPasswordFile=str(wrong)),
                                                                {"id": "local", "type": "local", "path": str(local)}]}))
    small = snapshot(work, "20261005T000000Z", 1000)
    output = call("publish", "--config", str(config), "--source", str(small), expect=1)
    assert (local / small.name).is_file() and (local / (small.name + ".complete.json")).is_file(), output
    print("PASS failure isolation: Nextcloud refused, local destination still published")
print("ALL PASS")
