#!/usr/bin/env python3
"""Linux-only, content-aware source mirror for an exclusively owned Cargo pair.

No Cargo fingerprint is edited. Byte-identical files keep inode/mtime. Changed
files receive current wall time newer than observed Cargo1.98 freshness anchors.
The advisory lock covers sync AND child completion; callers must also use one
BuildKit sharing=locked cache namespace for this whole source/target pair.
"""
import argparse
import contextlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import time
import uuid
if sys.platform == "linux":
    import fcntl
else:
    fcntl = None

MAX_FILES = 50000
MAX_DIRS = 10000
MAX_NAMES = 8 * 1024 * 1024
MAX_BYTES = 512 * 1024 * 1024
MAX_FILE_BYTES = 64 * 1024 * 1024
MAX_ANCHOR_STATS = 100000
CHUNK = 65536
OWNER = b'{"schema":1,"source":"source","target":"target","cargoLayout":"1.98"}\n'
FLAGS = getattr(os, "O_NOFOLLOW", 0) | os.O_CLOEXEC
DIR_FLAGS = os.O_RDONLY | os.O_DIRECTORY | FLAGS

class MirrorError(RuntimeError):
    pass

class MirrorBusy(MirrorError):
    pass

def require_linux():
    # Windows reparse tags are not fully exposed by portable stat/dir_fd APIs.
    # Reject that execution platform rather than silently trusting junctions.
    if fcntl is None or not getattr(os, "O_NOFOLLOW", 0):
        raise MirrorError("Linux fd-relative/no-follow mirror required")

def checked(info, directory=False, device=None, ancestor=False):
    if getattr(info, "st_file_attributes", 0) & 0x400:
        raise MirrorError("reparse point refused")
    valid = stat.S_ISDIR(info.st_mode) if directory else stat.S_ISREG(info.st_mode)
    if not valid or (not ancestor and info.st_mode & (stat.S_ISUID | stat.S_ISGID | stat.S_ISVTX)):
        raise MirrorError("nonregular or special-mode source/cache entry refused")
    if not directory and info.st_nlink != 1:
        raise MirrorError("hard-linked file refused")
    if device is not None and info.st_dev != device:
        raise MirrorError("cross-device source/mirror entry refused")
    return info

def name(value):
    if not value or value in (".", "..") or any(c in value for c in ("/", "\\", ":", "\0")):
        raise MirrorError("unsafe path component")
    if len(value.encode("utf-8", "strict")) > 255:
        raise MirrorError("source filename limit")
    return value

def absolute(value):
    raw = os.fspath(value)
    if not raw.startswith("/") or "\\" in raw or ":" in raw or "\0" in raw or any(p in (".", "..") for p in raw.split("/")):
        raise MirrorError("absolute normalized Linux path required")
    path = Path(os.path.normpath(raw))
    if path == Path("/"):
        raise MirrorError("filesystem root refused")
    return path

def open_root(path):
    require_linux()
    path = absolute(path)
    fd = os.open("/", DIR_FLAGS)
    try:
        for part in path.parts[1:]:
            child = os.open(name(part), DIR_FLAGS, dir_fd=fd)
            try:
                checked(os.fstat(child), True, ancestor=True)
            except BaseException:
                os.close(child)
                raise
            os.close(fd)
            fd = child
        checked(os.fstat(fd), True)
        return fd
    except BaseException:
        os.close(fd)
        raise

@contextlib.contextmanager
def parent(root, relative):
    parts = relative.split("/")
    if len(parts) > 64 or len(relative.encode("utf-8")) > 4096:
        raise MirrorError("source path limit")
    fd = os.dup(root)
    device = os.fstat(root).st_dev
    try:
        for part in parts[:-1]:
            child = os.open(name(part), DIR_FLAGS, dir_fd=fd)
            try:
                checked(os.fstat(child), True, device)
            except BaseException:
                os.close(child)
                raise
            os.close(fd)
            fd = child
        yield fd, name(parts[-1])
    finally:
        os.close(fd)

def signature(info):
    return (info.st_dev, info.st_ino, info.st_size, info.st_mode, info.st_mtime_ns, info.st_ctime_ns)

def scan(root, cache=False):
    entries = {}
    total = files = directories = names = 0
    device = os.fstat(root).st_dev
    def visit(fd, prefix, depth):
        nonlocal total, files, directories, names
        if depth > 64:
            raise MirrorError("directory depth limit")
        before = os.fstat(fd)
        with os.scandir(fd) as iterator:
            for entry in iterator:
                piece = name(entry.name)
                relative = prefix + piece
                names += len(relative.encode("utf-8"))
                if names > MAX_NAMES or len(relative.encode("utf-8")) > 4096:
                    raise MirrorError("source name budget")
                info = os.stat(piece, dir_fd=fd, follow_symlinks=False)
                directory = stat.S_ISDIR(info.st_mode)
                checked(info, directory, device)
                if directory:
                    directories += 1
                    if directories > MAX_DIRS:
                        raise MirrorError("directory count limit")
                    entries[relative] = (True, info)
                    child = os.open(piece, DIR_FLAGS, dir_fd=fd)
                    try:
                        checked(os.fstat(child), True, device)
                        visit(child, relative + "/", depth + 1)
                    finally:
                        os.close(child)
                else:
                    files += 1
                    total += info.st_size
                    if files > MAX_FILES + int(cache) or info.st_size > MAX_FILE_BYTES or total > MAX_BYTES + (MAX_FILE_BYTES if cache else 0):
                        raise MirrorError("source file/byte budget")
                    entries[relative] = (False, info)
        if signature(os.fstat(fd)) != signature(before):
            raise MirrorError("source/cache directory changed during scan")
    visit(root, "", 0)
    return entries, total

@contextlib.contextmanager
def file_at(root, relative, expected):
    with parent(root, relative) as (directory, leaf):
        fd = os.open(leaf, os.O_RDONLY | FLAGS, dir_fd=directory)
    try:
        info = checked(os.fstat(fd), False, os.fstat(root).st_dev)
        if signature(info) != signature(expected):
            raise MirrorError("source/cache changed after scan")
        yield fd
        if signature(os.fstat(fd)) != signature(info):
            raise MirrorError("source/cache file changed while reading")
    finally:
        os.close(fd)

def equal_files(source, destination, path, a, b, stats):
    if a.st_size != b.st_size:
        return False
    with file_at(source, path, a) as src, file_at(destination, path, b) as dst:
        while True:
            left, right = os.read(src, CHUNK), os.read(dst, CHUNK)
            stats["bytesCompared"] += len(left) + len(right)
            if left != right:
                return False
            if not left:
                return True

def write_all(fd, data):
    while data:
        size = os.write(fd, data)
        if size <= 0:
            raise MirrorError("short source write")
        data = data[size:]

def fresh_stamp(fd, floor):
    deadline = time.monotonic() + 2
    while True:
        now = time.time_ns()
        if now > floor:
            os.utime(fd, ns=(now, now))
            if os.fstat(fd).st_mtime_ns > floor:
                return
        if time.monotonic() >= deadline:
            raise MirrorError("clock/resolution cannot provide fresh source timestamp")
        time.sleep(.01)

class MirrorSession:
    def __init__(self, cache_root):
        self.path = absolute(cache_root)
        self.fd = self.lock = self.source = self.target = None

    def __enter__(self):
        self.fd = open_root(self.path)
        try:
            initial = set(bounded_names(self.fd,5))
            allowed = {".mirror.lock", ".mirror-owner.json", ".mirror-dirty", "source", "target"}
            if not initial <= allowed or (".mirror-owner.json" not in initial and not initial <= {".mirror.lock"}):
                raise MirrorError("nonempty unowned cache refused before mutation")
            self.lock = os.open(".mirror.lock", os.O_CREAT | os.O_RDWR | FLAGS, 0o600, dir_fd=self.fd)
            checked(os.fstat(self.lock))
            try:
                fcntl.flock(self.lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError as error:
                raise MirrorBusy("source/target cache pair busy") from error
            names = set(bounded_names(self.fd,5))
            allowed = {".mirror.lock", ".mirror-owner.json", ".mirror-dirty", "source", "target"}
            if not names <= allowed:
                raise MirrorError("unowned cache root contents refused")
            if ".mirror-owner.json" not in names:
                if names != {".mirror.lock"}:
                    raise MirrorError("nonempty unowned cache pair refused; choose a fresh namespace")
                owner = os.open(".mirror-owner.json", os.O_WRONLY | os.O_CREAT | os.O_EXCL | FLAGS, 0o600, dir_fd=self.fd)
                try:
                    write_all(owner, OWNER)
                    os.fsync(owner)
                finally:
                    os.close(owner)
                os.fsync(self.fd)
            else:
                owner = os.open(".mirror-owner.json", os.O_RDONLY | FLAGS, dir_fd=self.fd)
                try:
                    checked(os.fstat(owner))
                    if os.read(owner, len(OWNER) + 1) != OWNER:
                        raise MirrorError("cache owner protocol mismatch")
                finally:
                    os.close(owner)
            for part in ("source", "target"):
                try:
                    os.mkdir(part, 0o700, dir_fd=self.fd)
                except FileExistsError:
                    pass
            self.source = os.open("source", DIR_FLAGS, dir_fd=self.fd)
            self.target = os.open("target", DIR_FLAGS, dir_fd=self.fd)
            checked(os.fstat(self.source), True)
            checked(os.fstat(self.target), True)
            return self
        except BaseException:
            self.__exit__(None, None, None)
            raise

    def __exit__(self, *_):
        # Child Cargo inherits the SAME open-description lock. Killing this
        # observer cannot release ownership while its accepted child still runs.
        for attribute in ("target", "source", "lock", "fd"):
            fd = getattr(self, attribute)
            if fd is not None:
                os.close(fd)
                setattr(self, attribute, None)

    def anchor_floor(self):
        """Read bounded Cargo1.98 metadata only; never modify fingerprints."""
        count, floor = 0, 0
        def budget():
            nonlocal count
            count += 1
            if count > MAX_ANCHOR_STATS:
                raise MirrorError("Cargo anchor metadata budget")
        def directory_info(fd):
            budget()
            return checked(os.fstat(fd), True)
        def observe(fd, piece):
            nonlocal floor
            budget()
            info = os.stat(piece, dir_fd=fd, follow_symlinks=False)
            checked(info, False)
            floor = max(floor, info.st_mtime_ns)
        def maybe_dir(fd, part):
            try:
                child = os.open(part, DIR_FLAGS, dir_fd=fd)
            except FileNotFoundError:
                return None
            try:
                directory_info(child)
                return child
            except BaseException:
                os.close(child)
                raise
        def profile(fd):
            fingerprint = maybe_dir(fd, ".fingerprint")
            if fingerprint is not None:
                try:
                    for unit in bounded_names(fingerprint,MAX_ANCHOR_STATS):
                        name(unit)
                        directory = os.open(unit, DIR_FLAGS, dir_fd=fingerprint)
                        try:
                            directory_info(directory)
                            for item in bounded_names(directory,MAX_ANCHOR_STATS):
                                name(item)
                                observe(directory, item)
                        finally:
                            os.close(directory)
                finally:
                    os.close(fingerprint)
            build = maybe_dir(fd, "build")
            if build is not None:
                try:
                    for unit in bounded_names(build,MAX_ANCHOR_STATS):
                        name(unit)
                        directory = os.open(unit, DIR_FLAGS, dir_fd=build)
                        try:
                            directory_info(directory)
                            for item in ("invoked.timestamp", "output", "stderr", "root-output"):
                                try:
                                    observe(directory, item)
                                except FileNotFoundError:
                                    pass
                        finally:
                            os.close(directory)
                finally:
                    os.close(build)
        top = bounded_names(self.target,128)
        if len(top) > 128:
            raise MirrorError("unsupported target layout")
        for part in top:
            name(part)
            budget()
            info = os.stat(part, dir_fd=self.target, follow_symlinks=False)
            if not stat.S_ISDIR(info.st_mode):
                checked(info, False)
                continue
            checked(info, True)
            directory = os.open(part, DIR_FLAGS, dir_fd=self.target)
            try:
                if part in ("debug", "release"):
                    profile(directory)
                elif re.fullmatch(r"[a-z0-9_]+-[a-z0-9_.-]+", part):
                    for mode in ("debug", "release"):
                        nested = maybe_dir(directory, mode)
                        if nested is not None:
                            try:
                                profile(nested)
                            finally:
                                os.close(nested)
                elif part not in ("tmp",):
                    raise MirrorError("unsupported target directory; use the pinned build layout")
            finally:
                os.close(directory)
        return floor, count

    def sync(self, input_root):
        input_path = absolute(input_root)
        if input_path == self.path or input_path in self.path.parents or self.path in input_path.parents:
            raise MirrorError("source input and owned cache must be disjoint")
        source = open_root(input_path)
        try:
            incoming, size = scan(source)
            existing, _ = scan(self.source, cache=True)
            floor, anchor_stats = self.anchor_floor()
            # Future timestamps can make changed files appear older than Cargo's
            # anchors. Refuse clock skew rather than backdate or edit fingerprints.
            floor = max([floor] + [info.st_mtime_ns for _, info in existing.values()])
            if floor > time.time_ns() + 2_000_000_000:
                raise MirrorError("future cache timestamp; clock repair/new cache namespace required")
            dirty = ".mirror-dirty" in bounded_names(self.fd, 5)
            if dirty:
                info = os.stat(".mirror-dirty", dir_fd=self.fd, follow_symlinks=False)
                checked(info)
            stats = {"schema":1,"sourceFiles":sum(not d for d, _ in incoming.values()),
                     "totalSourceBytes":size,"unchangedFiles":0,"writtenFiles":0,
                     "removedFiles":0,"createdDirectories":0,"removedDirectories":0,
                     "modeChanges":0,"bytesCompared":0,"bytesWritten":0,
                     "anchorStatCalls":anchor_stats,"anchorFloorNs":str(floor),
                     "recoveredInterruptedSync":dirty}
            copies = []
            for path, (directory, info) in incoming.items():
                old = existing.get(path)
                if directory:
                    continue
                same = bool(old and not old[0] and equal_files(source,self.source,path,info,old[1],stats))
                mode_same = bool(old and stat.S_IMODE(info.st_mode)==stat.S_IMODE(old[1].st_mode))
                if same and mode_same and not dirty:
                    stats["unchangedFiles"] += 1
                else:
                    copies.append(path)
                    if old and not mode_same:
                        stats["modeChanges"] += 1
            removals = [path for path, old in existing.items() if path not in incoming or old[0] != incoming[path][0]]
            directories = [path for path,(d,_) in incoming.items() if d and (path not in existing or not existing[path][0])]
            changed_dirs = [path for path,(d,info) in incoming.items() if d and path in existing and existing[path][0] and stat.S_IMODE(info.st_mode)!=stat.S_IMODE(existing[path][1].st_mode)]
            if not (copies or removals or directories or changed_dirs or dirty):
                verified, _ = scan(source)
                if {p:(d,signature(i)) for p,(d,i) in verified.items()} != {p:(d,signature(i)) for p,(d,i) in incoming.items()}:
                    raise MirrorError("input snapshot changed during comparison")
                return stats
            marker = os.open(".mirror-dirty", os.O_WRONLY | os.O_CREAT | FLAGS, 0o600, dir_fd=self.fd)
            try:
                checked(os.fstat(marker))
                os.fsync(marker)
            finally:
                os.close(marker)
            os.fsync(self.fd)
            touched = set()
            for path in sorted(removals,key=lambda x:(-x.count("/"),x)):
                directory, _ = existing[path]
                with parent(self.source,path) as (fd,leaf):
                    checked(os.stat(leaf,dir_fd=fd,follow_symlinks=False),directory)
                    if directory:
                        os.rmdir(leaf,dir_fd=fd)
                        stats["removedDirectories"] += 1
                    else:
                        os.unlink(leaf,dir_fd=fd)
                        stats["removedFiles"] += 1
                touched.add(path.rpartition("/")[0])
            for path in sorted(directories,key=lambda x:(x.count("/"),x)):
                with parent(self.source,path) as (fd,leaf):
                    os.mkdir(leaf,stat.S_IMODE(incoming[path][1].st_mode),dir_fd=fd)
                    created = os.open(leaf,DIR_FLAGS,dir_fd=fd)
                    try:
                        os.fchmod(created,stat.S_IMODE(incoming[path][1].st_mode))
                    finally:
                        os.close(created)
                stats["createdDirectories"] += 1
                touched.update((path,path.rpartition("/")[0]))
            for path in changed_dirs:
                with parent(self.source,path) as (fd,leaf):
                    directory = os.open(leaf,DIR_FLAGS,dir_fd=fd)
                    try:
                        os.fchmod(directory,stat.S_IMODE(incoming[path][1].st_mode))
                    finally:
                        os.close(directory)
                stats["modeChanges"] += 1
                touched.add(path)
            for path in copies:
                temporary = ".mirror-tmp-" + uuid.uuid4().hex
                with parent(self.source,path) as (fd,leaf):
                    output = os.open(temporary,os.O_WRONLY|os.O_CREAT|os.O_EXCL|FLAGS,0o600,dir_fd=fd)
                    try:
                        with file_at(source,path,incoming[path][1]) as src:
                            copied = 0
                            while True:
                                block = os.read(src,CHUNK)
                                if not block:
                                    break
                                copied += len(block)
                                if copied > incoming[path][1].st_size or copied > MAX_FILE_BYTES or stats["bytesWritten"] + len(block) > MAX_BYTES:
                                    raise MirrorError("input grew beyond scanned copy budget")
                                write_all(output,block)
                                stats["bytesWritten"] += len(block)
                        os.fchmod(output,stat.S_IMODE(incoming[path][1].st_mode))
                        fresh_stamp(output,floor)
                        os.fsync(output)
                        os.replace(temporary,leaf,src_dir_fd=fd,dst_dir_fd=fd)
                        os.fsync(fd)
                    finally:
                        os.close(output)
                        try:
                            os.unlink(temporary,dir_fd=fd)
                        except FileNotFoundError:
                            pass
                stats["writtenFiles"] += 1
                touched.add(path.rpartition("/")[0])
            # Watched directories must notice additions/removals/mode changes.
            # Do not touch any directory in a genuinely unchanged sync.
            if dirty:
                touched.update(path for path,(d,_) in incoming.items() if d)
                touched.add("")
            for path in sorted(touched):
                if path and (path not in incoming or not incoming[path][0]):
                    continue
                if path:
                    with parent(self.source,path) as (fd,leaf):
                        directory = os.open(leaf,DIR_FLAGS,dir_fd=fd)
                else:
                    directory = os.dup(self.source)
                try:
                    fresh_stamp(directory,floor)
                    os.fsync(directory)
                finally:
                    os.close(directory)
            # The immutable input may not change during planning/copying. A
            # failed check leaves the dirty marker; retry rewrites all source
            # bytes with fresh times before a child may use the cache again.
            verified, _ = scan(source)
            if {p:(d,signature(i)) for p,(d,i) in verified.items()} != {p:(d,signature(i)) for p,(d,i) in incoming.items()}:
                raise MirrorError("input snapshot changed during synchronization")
            os.unlink(".mirror-dirty",dir_fd=self.fd)
            os.fsync(self.fd)
            return stats
        finally:
            os.close(source)

    def run(self, command, **streams):
        if not command:
            raise MirrorError("explicit child command required")
        if any(arg == "--target-dir" or arg.startswith("--target-dir=") or arg == "--build-dir" or arg.startswith("--build-dir=") or arg == "--manifest-path" or arg.startswith("--manifest-path=") or "target-dir" in arg or "build-dir" in arg for arg in command):
            raise MirrorError("child target must stay in its owned cache pair")
        # Cargo CLI config has higher precedence than environment. Accept only
        # this exact existing build-profile override, never TOML files/escaped
        # keys or arbitrary config expressions. Toolchain selection must not
        # evade the separately verified Cargo1.98 binary/version.
        for index, arg in enumerate(command[1:], 1):
            if arg.startswith("+"):
                raise MirrorError("toolchain selectors are outside pinned layout")
            if arg == "--config":
                if index + 1 >= len(command) or command[index + 1] != "profile.release.package.mw-server.opt-level=2":
                    raise MirrorError("only the exact supported inline profile config is allowed")
            elif arg.startswith("--config="):
                raise MirrorError("config files and complex inline config are not supported")
        environment = dict(os.environ)
        environment["CARGO_TARGET_DIR"] = str(self.path / "target")
        # Environment overrides both source-local and inherited Cargo-home
        # config files. CLI config/flags containing build-dir are rejected above.
        # Keep build/fingerprint state under the same observed target anchors.
        environment["CARGO_BUILD_BUILD_DIR"] = str(self.path / "target")
        return subprocess.run(command,cwd=self.path/"source",env=environment,pass_fds=(self.lock,),**streams)

def bounded_names(fd, maximum):
    result = []
    with os.scandir(fd) as iterator:
        for entry in iterator:
            result.append(name(entry.name))
            if len(result) > maximum:
                raise MirrorError("directory entry budget")
    return result

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input",required=True)
    parser.add_argument("--cache-root",required=True)
    parser.add_argument("command",nargs=argparse.REMAINDER)
    arguments=parser.parse_args()
    command=arguments.command
    if command and command[0]=="--":
        command=command[1:]
    if not command or Path(command[0]).name!="cargo":
        raise MirrorError("CLI requires an explicit pinned cargo command")
    version=subprocess.run([command[0],"--version"],capture_output=True,text=True,check=True,timeout=10).stdout
    if not re.fullmatch(r"cargo 1\.98\.[0-9]+ \([^\n]+\)\n?",version):
        raise MirrorError("only the pinned Cargo1.98 metadata layout is supported")
    input_path,cache_path=absolute(arguments.input),absolute(arguments.cache_root)
    if input_path==cache_path or input_path in cache_path.parents or cache_path in input_path.parents:
        raise MirrorError("source input and cache must be disjoint")
    with MirrorSession(cache_path) as pair:
        result=pair.sync(arguments.input)
        print(json.dumps({"sourceMirror":result},sort_keys=True),file=sys.stderr,flush=True)
        completed=pair.run(command)
        return completed.returncode

if __name__=="__main__":
    try:
        sys.exit(main())
    except (MirrorError,OSError,ValueError,subprocess.SubprocessError) as error:
        # Fixed class only; do not echo command args/env or arbitrary FS paths.
        print("source mirror failed: "+type(error).__name__,file=sys.stderr)
        sys.exit(75 if isinstance(error,MirrorBusy) else 1)
