#!/usr/bin/env python3
"""Copy completed deploy.sh backups to explicitly configured destinations.

Credentials remain in an operator-owned rclone configuration. This tool never
executes command strings, mounts filesystems, deletes remote backups or restores
live data. A completion manifest is published only after copy verification.
"""
import argparse
import configparser
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys
import tempfile

KINDS = {"local", "s3", "sftp", "webdav", "smb", "nfs"}
NAME = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,159}$")


class BackupError(Exception):
    pass


def load_config(path):
    if Path(path).stat().st_size > 65536:
        raise BackupError("Destination configuration exceeds 64 KiB")
    value = json.loads(Path(path).read_text(encoding="utf-8"))
    if not isinstance(value, dict) or set(value) - {"version", "destinations"} or value.get("version") != 1:
        raise BackupError("Expected destination configuration version 1")
    rows = value.get("destinations")
    if not isinstance(rows, list) or not 1 <= len(rows) <= 16:
        raise BackupError("Configure between 1 and 16 destinations")
    ids = set()
    for row in rows:
        if not isinstance(row, dict) or set(row) - {"id", "type", "path", "remote", "rcloneConfig", "enabled", "timeoutSeconds"}:
            raise BackupError("Unknown destination field")
        if not NAME.fullmatch(row.get("id", "")) or row["id"] in ids:
            raise BackupError("Destination IDs must be unique, simple names")
        ids.add(row["id"])
        if row.get("type") not in KINDS or type(row.get("enabled", True)) is not bool:
            raise BackupError("Unsupported destination type or enabled flag")
        timeout = row.get("timeoutSeconds", 900)
        if type(timeout) is not int or not 10 <= timeout <= 7200:
            raise BackupError("Destination timeout must be 10–7200 seconds")
        location = row.get("path", "")
        if not isinstance(location, str) or not location or len(location) > 2048 or any(ord(c) < 32 for c in location):
            raise BackupError("A bounded destination path is required")
        if row["type"] in {"local", "nfs"}:
            if not Path(location).is_absolute():
                raise BackupError("Local and NFS paths must be absolute")
        else:
            if location.startswith(("/", "-")) or any(p in {".", ".."} for p in location.split("/")) or ":" in location or "\\" in location:
                raise BackupError("Remote path must be relative without traversal or a remote prefix")
            if not re.fullmatch(r"[A-Za-z][A-Za-z0-9_-]{0,63}", row.get("remote", "")):
                raise BackupError("Select a named rclone remote")
            if not Path(row.get("rcloneConfig", "")).is_absolute():
                raise BackupError("rcloneConfig must be an absolute file path")
    return [row for row in rows if row.get("enabled", True)]


def run(args, timeout, output=False):
    try:
        result = subprocess.run(args, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE if output else subprocess.DEVNULL,
                                stderr=subprocess.PIPE, timeout=timeout, check=False)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise BackupError(f"{Path(args[0]).name} unavailable or timed out") from error
    if result.returncode:
        # Provider errors may contain credentials or signed URLs; do not echo them.
        raise BackupError(f"{Path(args[0]).name} failed with exit {result.returncode}")
    return result.stdout if output else None


def preflight(row):
    kind = row["type"]
    if kind in {"local", "nfs"}:
        root = Path(row["path"])
        if not root.is_dir() or root.is_symlink():
            raise BackupError("Destination must be an existing directory, not a symbolic link")
        if kind == "nfs":
            fs = run(["findmnt", "-n", "-o", "FSTYPE", "--target", str(root)], 10, True).decode().strip()
            if fs not in {"nfs", "nfs4"}:
                raise BackupError("NFS destination is not mounted as NFS; refusing local-disk fallback")
        return
    if not shutil.which("rclone"):
        raise BackupError("Install rclone for remote destinations")
    config = configparser.ConfigParser(interpolation=None)
    config.read(row["rcloneConfig"], encoding="utf-8")
    if not config.has_section(row["remote"]) or config.get(row["remote"], "type", fallback="") != kind:
        raise BackupError("Named rclone remote is missing or has a different storage type")
    remote = config[row["remote"]]
    if kind == "sftp":
        external = remote.get("ssh", "")
        if external:
            options = shlex.split(external)
            trust = [arg.removeprefix("-o").lower() for arg in options if arg.removeprefix("-o").lower().startswith("stricthostkeychecking=")]
            batch = [arg.removeprefix("-o").lower() for arg in options if arg.removeprefix("-o").lower().startswith("batchmode=")]
            if not options or Path(options[0]).name not in {"ssh", "ssh.exe"} or trust != ["stricthostkeychecking=yes"] or batch != ["batchmode=yes"]:
                raise BackupError("External SSH requires ssh -o StrictHostKeyChecking=yes -o BatchMode=yes; jump hosts must also enforce host-key verification")
        elif not remote.get("known_hosts_file") or remote.get("known_hosts_file", "").lower() == "none":
            raise BackupError("SFTP requires known_hosts_file with pinned host keys")
        if remote.get("pin_host_key", "false").lower() == "true":
            raise BackupError("Backups may not automatically accept new SSH host keys")
    if kind == "webdav" and not remote.get("url", "").startswith("https://"):
        raise BackupError("WebDAV requires HTTPS")


def digest(path):
    value = hashlib.sha256()
    with open(path, "rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def remote_path(row, name):
    return f"{row['remote']}:{row['path'].rstrip('/')}/{name}"


def copy_file(row, source, name):
    if not NAME.fullmatch(name):
        raise BackupError("Unsafe backup filename")
    if row["type"] in {"local", "nfs"}:
        target = Path(row["path"]) / name
        if target.exists() or target.is_symlink():
            if target.is_file() and not target.is_symlink() and digest(target) == digest(source):
                return
            raise BackupError("Destination already contains a different backup with this name")
        with tempfile.NamedTemporaryFile(prefix=".mailwoman-", dir=row["path"], delete=False) as temp:
            temporary = Path(temp.name)
        try:
            shutil.copyfile(source, temporary)
            os.chmod(temporary, 0o600)
            if digest(temporary) != digest(source):
                raise BackupError("Local backup checksum verification failed")
            # link creates the final name exclusively; it cannot replace an existing backup.
            os.link(temporary, target)
        finally:
            temporary.unlink(missing_ok=True)
    else:
        args = ["rclone", "--config", row["rcloneConfig"], "--retries", "2", "--low-level-retries", "2", "--contimeout", "15s", "--timeout", "60s"]
        target = remote_path(row, name)
        run(args + ["copyto", "--immutable", str(source), target], row.get("timeoutSeconds", 900))
        # Download verification also works on SFTP/WebDAV servers without hash support.
        with tempfile.TemporaryDirectory(prefix="mw-backup-verify-") as work:
            copied = Path(work) / name
            run(args + ["copyto", "--max-size", str(source.stat().st_size), "--max-transfer", str(source.stat().st_size + 65536), "--cutoff-mode", "HARD", target, str(copied)], row.get("timeoutSeconds", 900))
            if copied.stat().st_size != source.stat().st_size or digest(copied) != digest(source):
                raise BackupError("Remote backup checksum verification failed")


def publish(row, source):
    preflight(row)
    if not source.is_file() or source.is_symlink() or not NAME.fullmatch(source.name):
        raise BackupError("Publish a completed encrypted backup archive, not a directory or link")
    if not source.name.endswith((".tar.age", ".tar.gpg")):
        raise BackupError("Destination copies require an encrypted .tar.age or .tar.gpg backup")
    release = source.with_name(source.name.rsplit(".tar.", 1)[0] + ".release.env")
    if not release.is_file() or release.is_symlink():
        raise BackupError("Backup release metadata is missing")
    entries = []
    for file in [source, release]:
        entries.append({"name": file.name, "bytes": file.stat().st_size, "sha256": digest(file)})
        copy_file(row, file, file.name)
    with tempfile.TemporaryDirectory(prefix="mw-backup-manifest-") as work:
        manifest = Path(work) / (source.name + ".complete.json")
        manifest.write_text(json.dumps({"format": "mailwoman-backup", "version": 1, "files": entries}, indent=2) + "\n", encoding="utf-8")
        copy_file(row, manifest, manifest.name)


def read_copy(row, name, output, limit):
    if not NAME.fullmatch(name):
        raise BackupError("Unsafe backup filename")
    if row["type"] in {"local", "nfs"}:
        source = Path(row["path"]) / name
        if source.is_symlink() or not source.is_file() or source.stat().st_size > limit:
            raise BackupError("Backup file is missing, oversized or a link")
        shutil.copyfile(source, output)
    else:
        run(["rclone", "--config", row["rcloneConfig"], "copyto", "--max-size", str(limit),
             "--max-transfer", str(limit + 65536), "--cutoff-mode", "HARD", "--retries", "1",
             "--contimeout", "15s", "--timeout", "60s", remote_path(row, name), str(output)], row.get("timeoutSeconds", 900))
    if not output.is_file() or output.stat().st_size > limit:
        raise BackupError("Downloaded backup exceeds its advertised size")


def fetch(row, name, output):
    preflight(row)
    if not NAME.fullmatch(name) or not name.endswith((".tar.age", ".tar.gpg")):
        raise BackupError("Select an encrypted backup archive name")
    local = {"type": "local", "path": str(output)}
    preflight(local)
    with tempfile.TemporaryDirectory(prefix="mw-backup-fetch-", dir=output) as work:
        directory = Path(work)
        marker = directory / (name + ".complete.json")
        read_copy(row, marker.name, marker, 65536)
        manifest = json.loads(marker.read_text(encoding="utf-8"))
        if not isinstance(manifest, dict):
            raise BackupError("Invalid backup completion manifest")
        files = manifest.get("files", [])
        if not isinstance(files, list) or any(not isinstance(file, dict) for file in files):
            raise BackupError("Invalid backup completion manifest")
        expected = {name, name.rsplit(".tar.", 1)[0] + ".release.env"}
        if manifest.get("format") != "mailwoman-backup" or manifest.get("version") != 1 or len(files) != 2 or {file.get("name") for file in files} != expected:
            raise BackupError("Invalid backup completion manifest")
        for file in files:
            size = file.get("bytes")
            if type(size) is not int or not 0 <= size <= 512 * 1024 ** 3 or not re.fullmatch(r"[a-f0-9]{64}", file.get("sha256", "")):
                raise BackupError("Invalid backup file size or checksum")
            target = directory / file["name"]
            read_copy(row, file["name"], target, size)
            if target.stat().st_size != size or digest(target) != file["sha256"]:
                raise BackupError("Downloaded backup checksum mismatch; nothing restored")
        publish(local, directory / name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["check", "publish", "fetch"])
    parser.add_argument("--config", required=True)
    parser.add_argument("--source")
    parser.add_argument("--destination")
    parser.add_argument("--name")
    parser.add_argument("--output")
    args = parser.parse_args()
    try:
        rows = load_config(args.config)
        if args.command == "fetch":
            rows = [row for row in rows if row["id"] == args.destination]
            if not args.name or not args.output or len(rows) != 1:
                raise BackupError("Fetch requires an enabled --destination, --name and existing --output directory")
        if not rows:
            raise BackupError("No enabled backup destinations")
        failed = False
        for row in rows:
            try:
                if args.command == "check":
                    preflight(row)
                elif args.command == "fetch":
                    fetch(row, args.name, Path(args.output))
                else:
                    if not args.source:
                        raise BackupError("--source is required")
                    publish(row, Path(args.source))
                print(f"[backup] {row['id']}: {args.command} complete")
            except BackupError as error:
                failed = True
                print(f"[backup] {row['id']}: {error}", file=sys.stderr)
        return 1 if failed else 0
    except BackupError as error:
        print(f"[backup] {error}", file=sys.stderr)
        return 1
    except (OSError, ValueError, TypeError, configparser.Error) as error:
        print(f"[backup] Configuration or filesystem error: {type(error).__name__}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
