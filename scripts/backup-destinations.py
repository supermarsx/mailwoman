#!/usr/bin/env python3
"""Copy completed deploy.sh backups to explicitly configured destinations.

Credentials remain in an operator-owned rclone configuration, or for Nextcloud in
an operator-owned app-password file. This tool never executes command strings,
mounts filesystems or restores live data. It deletes copies only when a
destination sets keep/keepDays, and then only complete sets of its own naming.
A completion manifest is published only after copy verification.
"""
import argparse
import base64
import configparser
import contextlib
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import secrets
import shutil
import ssl
import subprocess
import sys
import tempfile
import urllib.error
import urllib.parse
import urllib.request

KINDS = {"local", "s3", "sftp", "webdav", "smb", "nfs", "nextcloud"}
NAME = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,159}$")
# deploy.sh snapshot names (SNAPSHOT_NAME) as encrypted archives.
SNAPSHOT = re.compile(r"^([0-9]{8}T[0-9]{6}Z)-[0-9]+-[0-9]+\.tar\.(?:age|gpg)$")
COMMON_FIELDS = {"id", "type", "path", "enabled", "timeoutSeconds", "keep", "keepDays"}
RCLONE_FIELDS = {"remote", "rcloneConfig"}
NEXTCLOUD_FIELDS = {"url", "user", "appPasswordFile", "chunkSizeMiB"}
APP_PASSWORD_DOCS = "https://docs.nextcloud.com/server/latest/user_manual/en/session_management.html#managing-devices"


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
        if not isinstance(row, dict) or set(row) - COMMON_FIELDS - RCLONE_FIELDS - NEXTCLOUD_FIELDS:
            raise BackupError("Unknown destination field")
        if row.get("type") == "nextcloud" and set(row) & RCLONE_FIELDS:
            raise BackupError("Nextcloud destinations take url, user and appPasswordFile instead of an rclone remote")
        if row.get("type") != "nextcloud" and set(row) & NEXTCLOUD_FIELDS:
            raise BackupError("url, user, appPasswordFile and chunkSizeMiB apply only to Nextcloud destinations")
        if not NAME.fullmatch(row.get("id", "")) or row["id"] in ids:
            raise BackupError("Destination IDs must be unique, simple names")
        ids.add(row["id"])
        if row.get("type") not in KINDS or type(row.get("enabled", True)) is not bool:
            raise BackupError("Unsupported destination type or enabled flag")
        timeout = row.get("timeoutSeconds", 900)
        if type(timeout) is not int or not 10 <= timeout <= 7200:
            raise BackupError("Destination timeout must be 10–7200 seconds")
        for field, high in (("keep", 1000), ("keepDays", 36500)):
            if field in row and (type(row[field]) is not int or not 1 <= row[field] <= high):
                raise BackupError(f"{field} must be a whole number from 1 to {high}")
        location = row.get("path", "")
        if not isinstance(location, str) or not location or len(location) > 2048 or any(ord(c) < 32 for c in location):
            raise BackupError("A bounded destination path is required")
        if row["type"] in {"local", "nfs"}:
            if not Path(location).is_absolute():
                raise BackupError("Local and NFS paths must be absolute")
        else:
            if location.startswith(("/", "-")) or any(p in {".", ".."} for p in location.split("/")) or ":" in location or "\\" in location:
                raise BackupError("Remote path must be relative without traversal or a remote prefix")
            if row["type"] == "nextcloud":
                nextcloud_root(row)
                if not Path(row.get("appPasswordFile", "")).is_absolute():
                    raise BackupError("appPasswordFile must be an absolute file path")
                chunk = row.get("chunkSizeMiB", 10)
                if type(chunk) is not int or not 0 <= chunk <= 1024:
                    raise BackupError("chunkSizeMiB must be 0 (no chunking) to 1024")
                continue
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


def nextcloud_root(row):
    """The WebDAV files root of one Nextcloud user: <url>/remote.php/dav/files/<user>/."""
    url, user = row.get("url", ""), row.get("user", "")
    if not isinstance(url, str) or len(url) > 2048 or any(ord(c) < 33 for c in url) or "\\" in url or "%" in url:
        raise BackupError("Enter the Nextcloud server URL without spaces or encoded characters")
    parts = urllib.parse.urlsplit(url)
    if parts.scheme != "https" or not parts.hostname or parts.username or parts.password or parts.query or parts.fragment:
        raise BackupError("Nextcloud requires an HTTPS server URL without credentials, query or fragment")
    if "remote.php" in parts.path:
        raise BackupError("Enter the Nextcloud server URL only; the WebDAV path is added automatically")
    if (not isinstance(user, str) or not user or len(user) > 256 or user.strip() != user or user in {".", ".."}
            or any(ord(c) < 32 or c in "/\\:" for c in user)):
        raise BackupError("Enter a valid Nextcloud username")
    base = parts.path.rstrip("/")
    return urllib.parse.urlunsplit(("https", parts.netloc, f"{base}/remote.php/dav/files/{urllib.parse.quote(user, safe='')}/", "", ""))


def read_app_password(row):
    path = Path(row["appPasswordFile"])
    if path.is_symlink() or not path.is_file():
        raise BackupError("appPasswordFile must be an existing regular file, not a link")
    info = path.stat()
    if os.name != "nt" and info.st_mode & 0o077:
        raise BackupError("appPasswordFile must be readable only by its owner (chmod 600)")
    if info.st_size > 1024:
        raise BackupError("appPasswordFile is too large for an app password")
    value = path.read_text(encoding="utf-8").strip()
    if not value or any(ord(c) < 33 for c in value):
        raise BackupError(f"appPasswordFile must hold one Nextcloud app password; see {APP_PASSWORD_DOCS}")
    return value


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


TLS_FAILED = "Nextcloud TLS certificate verification failed; use a publicly trusted certificate or RCLONE_CA_CERT"


def dav(method, url, user, password, depth=None, timeout=30):
    """One credential-bearing request that never follows redirects: (status, bounded body)."""
    context = ssl.create_default_context(cafile=os.environ.get("RCLONE_CA_CERT") or None)
    opener = urllib.request.build_opener(NoRedirect, urllib.request.HTTPSHandler(context=context))
    request = urllib.request.Request(url, method=method)
    request.add_header("Authorization", "Basic " + base64.b64encode(f"{user}:{password}".encode()).decode())
    if depth is not None:
        request.add_header("Depth", str(depth))
    if method == "PROPFIND":
        request.data = b'<?xml version="1.0"?><d:propfind xmlns:d="DAV:"><d:prop><d:resourcetype/></d:prop></d:propfind>'
        request.add_header("Content-Type", "application/xml")
    try:
        with opener.open(request, timeout=timeout) as response:
            return response.status, response.read(65536)
    except urllib.error.HTTPError as error:
        return error.code, b""
    except ssl.SSLError as error:
        raise BackupError(TLS_FAILED) from None
    except (urllib.error.URLError, OSError) as error:
        if isinstance(getattr(error, "reason", None), ssl.SSLError):
            raise BackupError(TLS_FAILED) from None
        raise BackupError("Could not connect to the Nextcloud server") from None


def nextcloud_folder(row, password):
    """Check the credentials, then create each missing folder segment with MKCOL."""
    root = nextcloud_root(row)
    status, _ = dav("PROPFIND", root, row["user"], password, depth=0)
    if status == 401:
        raise BackupError(f"Nextcloud rejected the username or app password; create an app password: {APP_PASSWORD_DOCS}")
    if 300 <= status < 400:
        raise BackupError("Nextcloud redirected the request; enter the final HTTPS server URL")
    if status == 404:
        raise BackupError("Nextcloud has no files endpoint for this user; check the server URL and username")
    if status != 207:
        raise BackupError(f"Nextcloud refused the account check (HTTP {status})")
    created, current = False, root
    for segment in row["path"].strip("/").split("/"):
        current += urllib.parse.quote(segment, safe="") + "/"
        status, body = dav("PROPFIND", current, row["user"], password, depth=0)
        if status == 207:
            if b"collection" not in body:
                raise BackupError("The Nextcloud backup path exists but is a file, not a folder")
            continue
        if status != 404:
            raise BackupError(f"Nextcloud refused access to the backup folder (HTTP {status})")
        status, _ = dav("MKCOL", current, row["user"], password)
        if status not in {201, 405}:
            raise BackupError(f"Nextcloud could not create the backup folder (HTTP {status}); check the account's permissions and quota")
        created = True
    return created


@contextlib.contextmanager
def active(row):
    """Yield the row as the transfer code sees it. Nextcloud is a preset over the
    WebDAV route: a private, temporary rclone remote with vendor=nextcloud, which
    uploads large files in chunks."""
    if row["type"] != "nextcloud":
        yield row
        return
    if not shutil.which("rclone"):
        raise BackupError("Install rclone for remote destinations")
    password = read_app_password(row)
    nextcloud_folder(row, password)
    try:
        obscured = subprocess.run(["rclone", "obscure", "-"], input=(password + "\n").encode(), stdout=subprocess.PIPE,
                                  stderr=subprocess.DEVNULL, timeout=15, check=True).stdout.decode().strip()
    except (OSError, subprocess.SubprocessError):
        raise BackupError("rclone could not prepare the Nextcloud credential") from None
    with tempfile.TemporaryDirectory(prefix="mw-backup-nextcloud-") as work:
        os.chmod(work, 0o700)
        config = Path(work) / "rclone.conf"
        descriptor = os.open(config, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        chunk = row.get("chunkSizeMiB", 10)
        with os.fdopen(descriptor, "w", encoding="utf-8") as handle:
            handle.write(f"[nextcloud]\ntype = webdav\nurl = {nextcloud_root(row)}\nvendor = nextcloud\n"
                         f"user = {row['user']}\npass = {obscured}\nnextcloud_chunk_size = {chunk}Mi\n")
        yield dict(row, type="webdav", remote="nextcloud", rcloneConfig=str(config))


def rclone_args(row):
    return ["rclone", "--config", row["rcloneConfig"], "--retries", "2", "--low-level-retries", "2",
            "--contimeout", "15s", "--timeout", "60s"]


def probe(row):
    """Write, verify and delete one small file: proves write access end to end."""
    name = f"mailwoman-probe-{secrets.token_hex(8)}.txt"
    with tempfile.TemporaryDirectory(prefix="mw-backup-probe-") as work:
        source = Path(work) / name
        source.write_bytes(b"mailwoman backup destination probe\n")
        copy_file(row, source, name)
    if row["type"] in {"local", "nfs"}:
        (Path(row["path"]) / name).unlink()
    else:
        run(rclone_args(row) + ["deletefile", remote_path(row, name)], 60)


def list_names(row):
    if row["type"] in {"local", "nfs"}:
        return [entry.name for entry in Path(row["path"]).iterdir() if entry.is_file() and not entry.is_symlink()]
    listing = run(rclone_args(row) + ["lsjson", "--files-only", "--no-modtime", "--no-mimetype",
                                      f"{row['remote']}:{row['path'].rstrip('/')}"], row.get("timeoutSeconds", 900), True)
    if len(listing) > 16 * 1024 * 1024:
        raise BackupError("Destination listing is too large")
    rows = json.loads(listing.decode("utf-8"))
    return [entry["Name"] for entry in rows if isinstance(entry, dict) and isinstance(entry.get("Name"), str)]


def retention(row, current, now=None):
    """Apply keep/keepDays (BACKUP_KEEP / BACKUP_KEEP_DAYS semantics) to complete
    sets on this destination. Archives without a completion manifest, other
    files and the archive just published are never removed."""
    if "keep" not in row and "keepDays" not in row:
        return []
    keep = row.get("keep", 7)
    names = set(list_names(row))
    complete = sorted((name for name in names if SNAPSHOT.fullmatch(name) and name + ".complete.json" in names), reverse=True)
    cutoff = None
    if "keepDays" in row:
        moment = (now or datetime.datetime.now(datetime.timezone.utc)) - datetime.timedelta(days=row["keepDays"])
        cutoff = moment.strftime("%Y%m%dT%H%M%SZ")
    removed = []
    for position, name in enumerate(complete, start=1):
        stamp = SNAPSHOT.fullmatch(name).group(1)
        if name == current or position <= keep or (cutoff is not None and stamp >= cutoff):
            continue
        # The manifest goes first, so an interrupted prune never leaves a set that looks complete.
        for victim in [name + ".complete.json", name, name.rsplit(".tar.", 1)[0] + ".release.env"]:
            if victim not in names:
                continue
            if row["type"] in {"local", "nfs"}:
                (Path(row["path"]) / victim).unlink(missing_ok=True)
            else:
                run(rclone_args(row) + ["deletefile", remote_path(row, victim)], 120)
        removed.append(name)
    return removed


def external_ssh_is_pinned(command):
    """Accept required checks only as leading OpenSSH options, never command text."""
    options = shlex.split(command)
    if not options or Path(options[0]).name not in {"ssh", "ssh.exe"}:
        return False
    values, index = [], 1
    takes_value = set("BbcDEeFIiJLlmOoPpQRSWw")
    flags = set("46AaCfGgKkMNnqsTtVvXxYy")
    while index < len(options):
        item = options[index]
        if item == "--":
            index += 1
            break
        if not item.startswith("-") or item == "-":
            break
        if len(item) < 2:
            return False
        option = item[1]
        if option in takes_value:
            value = item[2:]
            if not value:
                index += 1
                if index >= len(options):
                    return False
                value = options[index]
            if option == "o":
                values.append(value.lower())
        elif not all(flag in flags for flag in item[1:]):
            return False
        index += 1
    # SFTP supplies its own subsystem command; extra command tokens are not options.
    if index != len(options) - 1 or options[index].startswith("-"):
        return False
    trust = [value for value in values if value.startswith("stricthostkeychecking=")]
    batch = [value for value in values if value.startswith("batchmode=")]
    return trust == ["stricthostkeychecking=yes"] and batch == ["batchmode=yes"]


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
    unsafe = sorted(name for name in os.environ
                    if name.upper().startswith("RCLONE_") and name.upper() != "RCLONE_CA_CERT")
    if unsafe:
        names = ", ".join(re.sub(r"[^A-Za-z0-9_]", "?", name)[:128] for name in unsafe[:16])
        raise BackupError(f"Use the configured rclone file; only RCLONE_CA_CERT is allowed. Refused environment names: {names}")
    config = configparser.ConfigParser(interpolation=None)
    config.read(row["rcloneConfig"], encoding="utf-8")
    if not config.has_section(row["remote"]) or config.get(row["remote"], "type", fallback="") != kind:
        raise BackupError("Named rclone remote is missing or has a different storage type")
    remote = config[row["remote"]]
    overrides = sorted(name for name in remote if name.startswith(("global.", "override.")))
    if overrides:
        names = ", ".join(re.sub(r"[^A-Za-z0-9_.]", "?", name)[:128] for name in overrides[:16])
        raise BackupError(f"Remote global/override options are not supported. Refused option names: {names}")
    if kind == "sftp":
        external = remote.get("ssh", "")
        if external:
            if not external_ssh_is_pinned(external):
                raise BackupError("External SSH requires ssh -o StrictHostKeyChecking=yes -o BatchMode=yes; jump hosts must also enforce host-key verification")
        elif not remote.get("known_hosts_file") or remote.get("known_hosts_file", "").lower() == "none":
            raise BackupError("SFTP requires known_hosts_file with pinned host keys")
        if remote.get("pin_host_key", "false").lower() == "true":
            raise BackupError("Backups may not automatically accept new SSH host keys")
    if kind == "webdav" and not remote.get("url", "").startswith("https://"):
        raise BackupError("WebDAV requires HTTPS")
    if kind == "s3" and remote.get("endpoint", "") and not remote.get("endpoint", "").startswith("https://"):
        raise BackupError("Custom S3 endpoints require HTTPS")


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
        args = rclone_args(row)
        target = remote_path(row, name)
        run(args + ["copyto", "--immutable", str(source), target], row.get("timeoutSeconds", 900))
        # Download verification also works on SFTP/WebDAV servers without hash support.
        with tempfile.TemporaryDirectory(prefix="mw-backup-verify-") as work:
            copied = Path(work) / name
            run(args + ["copyto", "--max-transfer", str(source.stat().st_size + 65536), "--cutoff-mode", "HARD", target, str(copied)], row.get("timeoutSeconds", 900))
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
        run(["rclone", "--config", row["rcloneConfig"], "copyto",
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
    parser.add_argument("command", choices=["check", "test", "publish", "fetch"])
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
        elif args.destination:
            rows = [row for row in rows if row["id"] == args.destination]
        if not rows:
            raise BackupError("No enabled backup destinations")
        failed = False
        for row in rows:
            try:
                if args.command == "check":
                    # Offline: configuration and prerequisites, no connection.
                    if row["type"] == "nextcloud":
                        read_app_password(row)
                        if not shutil.which("rclone"):
                            raise BackupError("Install rclone for remote destinations")
                    else:
                        preflight(row)
                elif args.command == "test":
                    with active(row) as live:
                        preflight(live)
                        probe(live)
                elif args.command == "fetch":
                    with active(row) as live:
                        fetch(live, args.name, Path(args.output))
                else:
                    if not args.source:
                        raise BackupError("--source is required")
                    with active(row) as live:
                        publish(live, Path(args.source))
                        try:
                            for name in retention(live, Path(args.source).name):
                                print(f"[backup] {row['id']}: removed {name} (keep/keepDays)")
                        except (BackupError, OSError, ValueError) as error:
                            # The verified copy stands; a failed prune only leaves extra copies.
                            message = str(error) if isinstance(error, BackupError) else type(error).__name__
                            print(f"[backup] {row['id']}: retention skipped: {message}", file=sys.stderr)
                print(f"[backup] {row['id']}: {args.command} complete")
            except BackupError as error:
                failed = True
                print(f"[backup] {row['id']}: {error}", file=sys.stderr)
            except (OSError, ValueError, TypeError, configparser.Error) as error:
                # One destination must not prevent independent recovery copies.
                # Exception text can include credentials, signed URLs or paths.
                failed = True
                print(f"[backup] {row['id']}: Configuration or filesystem error: {type(error).__name__}", file=sys.stderr)
        return 1 if failed else 0
    except BackupError as error:
        print(f"[backup] {error}", file=sys.stderr)
        return 1
    except (OSError, ValueError, TypeError, configparser.Error) as error:
        print(f"[backup] Configuration or filesystem error: {type(error).__name__}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
