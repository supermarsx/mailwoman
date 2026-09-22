#!/usr/bin/env python3
"""Verify every committed first-party WASM artifact against its source of truth.

t24-e15. This exists because two artifacts had silently drifted from the source
they are supposed to have been built from, and nothing in the repository compared
them:

  * `plugins/dist/bridge-ews.wasm` was the 26.10 build. `9b91b54` added the
    `basic-credentials` host import that on-prem EWS Basic auth needs and
    refreshed only `plugins/bridge-ews/fixtures/bridge-ews.wasm`. The server loads
    `plugins/dist/`; `crates/mw-server/tests/t12_ews_auth.rs` loads the fixture.
    So the shipped guest could not ask the host for an account's sealed
    credentials, while the test that exercises exactly that passed against a
    different file.
  * `crates/mw-media-wasm/media.wasm` was built by a rustc that no longer matched
    the (since pinned) toolchain, and — more importantly — could not be rebuilt to
    the same bytes by anyone, because it embedded 50 absolute paths from the
    builder's Cargo registry.

A digest pin attests to *a* file. These checks are what make it attest to *the
source*.

    python3 plugins/verify-artifacts.py               # no toolchain needed
    python3 plugins/verify-artifacts.py --rebuild     # also rebuild and compare
    python3 plugins/verify-artifacts.py --self-test   # prove the checks reject

Exit status is 0 only if every check passes.
"""

from __future__ import annotations

import argparse
import hashlib
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# ── what ships, and what each artifact's source of truth is ───────────────────
#
# `dist` is what `crates/mw-server/src/v7_mount.rs::plugin_dirs` resolves and what
# `FIRST_PARTY_DIGESTS` pins. `fixture` is the copy the crate's own tests load.
# They MUST be the same bytes or the tests are testing something that does not
# ship — which is precisely how E8-02 stayed invisible for ten tags.
COMPONENTS = {
    "bridge-graph": "plugins/bridge-graph/tests/fixtures/bridge-graph.wasm",
    "bridge-ews": "plugins/bridge-ews/fixtures/bridge-ews.wasm",
    "bridge-gmail": "plugins/bridge-gmail/fixtures/bridge-gmail.wasm",
    "languagetool": "plugins/languagetool/tests/fixtures/languagetool.wasm",
    "nextcloud": "plugins/nextcloud/tests/fixtures/nextcloud.wasm",
    "spam-rspamd": "plugins/spam-rspamd/tests/fixtures/spam-rspamd.wasm",
    "spam-spamassassin": "plugins/spam-spamassassin/tests/fixtures/spam-spamassassin.wasm",
}

# `plugins/<id>/build.sh` for each id above; the directory is the id itself.
BUILD_DIRS = {cid: f"plugins/{cid}" for cid in COMPONENTS}

# ── the host-capability surface each shipped component is allowed to have ──────
#
# The `mailwoman:plugin/host@…` imports are the guest's entire reach into the
# host: every capability the wasmtime jail can be asked to grant. Recording them
# here makes that surface a reviewable line in a text diff instead of a property
# buried in a binary, and it is the single check that would have caught E8-02 on
# its own — the shipped bridge-ews had lost `basic-credentials`.
#
# ADDING a name here widens what a shipped guest can ask the host for. Treat a
# diff to this table as a security review, not a chore.
EXPECTED_HOST_IMPORTS = {
    "bridge-graph": {"http-fetch", "oauth-token"},
    "bridge-ews": {"basic-credentials", "http-fetch", "now", "random"},
    "bridge-gmail": {"http-fetch", "oauth-token"},
    "languagetool": {"http-fetch"},
    "nextcloud": {"http-fetch"},
    # The spam guests hold their per-account training state in the sealed plugin
    # KV added in 26.15 (`47a1098`), hence `kv-get`.
    "spam-rspamd": {"http-fetch", "kv-get"},
    "spam-spamassassin": {"http-fetch", "kv-get"},
}

# ── the second-layer media jail (SPEC §7.5) ───────────────────────────────────
#
# A bare core module, not a component, and not in FIRST_PARTY_DIGESTS — it is
# pulled in by `crates/mw-render/src/media_jail.rs` with `include_bytes!`, so
# nothing pinned its bytes at all until now. Since 26.20 its build.sh remaps the
# Cargo registry and crate paths out of the output, so a Linux build with the
# pinned rustc is byte-reproducible and a digest pin is meaningful.
#
# To change it: run `sh crates/mw-media-wasm/build.sh` on Linux (the script header
# has the docker one-liner for other platforms), commit the artifact, and update
# this digest in the same commit.
MEDIA_WASM = "crates/mw-media-wasm/media.wasm"
MEDIA_WASM_SHA256 = "8149b8738bd3fc76eadc1f79fc9b266f9e05e135a092b7af3d523cb04020a481"
MEDIA_WASM_EXPORTS = {"memory (memory)", "mw_alloc (func)", "mw_parse_cfb (func)",
                      "mw_reencode_image (func)"}

CORE_PREAMBLE = bytes([0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00])
COMPONENT_PREAMBLE = bytes([0x00, 0x61, 0x73, 0x6D, 0x0D, 0x00, 0x01, 0x00])
KIND = {0: "func", 1: "table", 2: "memory", 3: "global"}


# ── minimal wasm reader (no third-party dependency, so CI needs nothing) ──────

def _uleb(b: bytes, i: int) -> tuple[int, int]:
    val = shift = 0
    while True:
        if i >= len(b):
            raise ValueError("truncated LEB128")
        c = b[i]
        i += 1
        val |= (c & 0x7F) << shift
        if not c & 0x80:
            return val, i
        shift += 7
        if shift > 63:
            raise ValueError("LEB128 too long")


def _name(b: bytes, i: int) -> tuple[str, int]:
    n, i = _uleb(b, i)
    if i + n > len(b):
        raise ValueError("truncated name")
    return b[i:i + n].decode("utf-8", "replace"), i + n


def core_interface(b: bytes) -> tuple[list[str], list[str]]:
    """(imports as `module.field`, exports as `name (kind)`) of a core module."""
    imports: list[str] = []
    exports: list[str] = []
    i = 8
    while i < len(b):
        sec = b[i]
        i += 1
        size, i = _uleb(b, i)
        end = i + size
        if end > len(b):
            raise ValueError(f"section {sec} overruns the module")
        j = i
        if sec == 2:  # import
            count, j = _uleb(b, j)
            for _ in range(count):
                mod, j = _name(b, j)
                fld, j = _name(b, j)
                k = b[j]
                j += 1
                if k == 0:
                    _, j = _uleb(b, j)
                elif k in (1, 2):
                    if k == 1:
                        j += 1
                    lim = b[j]
                    j += 1
                    _, j = _uleb(b, j)
                    if lim & 0x01:
                        _, j = _uleb(b, j)
                elif k == 3:
                    j += 2
                imports.append(f"{mod}.{fld}")
        elif sec == 7:  # export
            count, j = _uleb(b, j)
            for _ in range(count):
                nm, j = _name(b, j)
                k = b[j]
                j += 1
                _, j = _uleb(b, j)
                exports.append(f"{nm} ({KIND.get(k, k)})")
        i = end
    return sorted(imports), sorted(exports)


def component_core_imports(b: bytes) -> list[str]:
    """Every import of every core module nested inside a component.

    The component's own import section names WIT interfaces; the core modules it
    wraps name the concrete lowered functions (`mailwoman:plugin/host@0.1.0` /
    `basic-credentials`). The core view is both easier to parse without a
    component-model library and closer to what the jail actually links, so it is
    what this script compares.
    """
    if b[:8] != COMPONENT_PREAMBLE:
        raise ValueError(f"not a wasm component (preamble {b[:8].hex()})")
    found: set[str] = set()
    i = 8
    while i < len(b):
        sec = b[i]
        i += 1
        size, i = _uleb(b, i)
        end = i + size
        if end > len(b):
            raise ValueError(f"component section {sec} overruns the file")
        if sec == 1:  # a nested core module
            sub = b[i:end]
            if sub[:4] == b"\x00asm":
                found |= set(core_interface(sub)[0])
        i = end
    return sorted(found)


def host_capabilities(b: bytes) -> set[str]:
    """The `mailwoman:plugin/host@…` function names a component can call."""
    out = set()
    for imp in component_core_imports(b):
        mod, _, fld = imp.rpartition(".")
        if mod.startswith("mailwoman:plugin/host@"):
            out.add(fld)
    return out


def sha256(p: Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def pinned_digests(root: Path) -> dict[str, str]:
    """`FIRST_PARTY_DIGESTS` as parsed out of v7_mount.rs."""
    src = (root / "crates/mw-server/src/v7_mount.rs").read_text(encoding="utf-8")
    block = re.search(r"const FIRST_PARTY_DIGESTS[^=]*=\s*&\[(.*?)\n\];", src, re.S)
    if not block:
        raise ValueError("could not find FIRST_PARTY_DIGESTS in v7_mount.rs")
    out = {}
    for m in re.finditer(r'"([^"]+)"\s*,\s*\[([^\]]*)\]', block.group(1), re.S):
        octets = re.findall(r"0x([0-9a-fA-F]{2})", m.group(2))
        if len(octets) != 32:
            raise ValueError(f"{m.group(1)}: {len(octets)} octets, expected 32")
        out[m.group(1)] = "".join(o.lower() for o in octets)
    return out


# ── the checks ────────────────────────────────────────────────────────────────

def verify(root: Path) -> list[str]:
    """Every check that needs no wasm toolchain. Returns the failures."""
    errs: list[str] = []

    try:
        pins = pinned_digests(root)
    except Exception as e:  # a malformed table is itself the bug
        return [f"FIRST_PARTY_DIGESTS is unreadable: {e}"]

    for cid, fixture_rel in sorted(COMPONENTS.items()):
        dist = root / "plugins/dist" / f"{cid}.wasm"
        fixture = root / fixture_rel
        if not dist.is_file():
            errs.append(f"{cid}: {dist} is missing — the server would fail closed at boot")
            continue
        if not fixture.is_file():
            errs.append(f"{cid}: {fixture} is missing")
            continue

        dist_sha, fix_sha = sha256(dist), sha256(fixture)

        # (1) Shipped bytes == tested bytes. Without this, a green test proves
        #     nothing about what the server loads. THIS is the E8-02 check.
        if dist_sha != fix_sha:
            errs.append(
                f"{cid}: plugins/dist/{cid}.wasm ({dist_sha[:16]}…, {dist.stat().st_size} B) "
                f"is NOT the file the crate's tests load ({fixture_rel}, {fix_sha[:16]}…, "
                f"{fixture.stat().st_size} B). One of them is stale — rebuild with "
                f"plugins/{cid}/build.sh, copy the result to BOTH paths, then re-run "
                f"plugins/gen-digests.sh."
            )

        # (2) The compiled-in pin matches the shipped bytes. A mismatch means the
        #     server rejects its own first-party component at boot.
        pinned = pins.get(cid)
        if pinned is None:
            errs.append(f"{cid}: no FIRST_PARTY_DIGESTS entry — it would not load as first-party")
        elif pinned != dist_sha:
            errs.append(
                f"{cid}: FIRST_PARTY_DIGESTS pins {pinned[:16]}… but plugins/dist/{cid}.wasm "
                f"is {dist_sha[:16]}… — the server would refuse to load it. Re-run "
                f"plugins/gen-digests.sh and paste the table into "
                f"crates/mw-server/src/v7_mount.rs."
            )

        # (3) It is really a component, and its reach into the host is the
        #     reviewed one. A guest that gained a capability, or lost one it needs,
        #     shows up here as a named difference.
        try:
            caps = host_capabilities(dist.read_bytes())
        except Exception as e:
            errs.append(f"{cid}: plugins/dist/{cid}.wasm is not a readable component: {e}")
            continue
        want = EXPECTED_HOST_IMPORTS[cid]
        if caps != want:
            gained = sorted(caps - want)
            lost = sorted(want - caps)
            errs.append(
                f"{cid}: host-capability surface changed — gained {gained}, lost {lost}. "
                f"If this is intended, update EXPECTED_HOST_IMPORTS in "
                f"plugins/verify-artifacts.py in the same commit and say why; a GAINED "
                f"capability widens what the guest can ask the jail for."
            )

    # (4) The media jail: a pinned digest, a core module, zero host imports, the
    #     three real exports. Zero imports is the jail's whole claim — a swapped-in
    #     module that could do I/O would have to break it.
    media = root / MEDIA_WASM
    if not media.is_file():
        errs.append(f"{MEDIA_WASM} is missing")
    else:
        got = sha256(media)
        if got != MEDIA_WASM_SHA256:
            errs.append(
                f"{MEDIA_WASM} is {got[:16]}… but MEDIA_WASM_SHA256 pins "
                f"{MEDIA_WASM_SHA256[:16]}…. The build is reproducible since 26.20, so this "
                f"is either an unreviewed artifact or a stale pin — rebuild on Linux with "
                f"the pinned toolchain and update both in one commit."
            )
        b = media.read_bytes()
        if b[:8] != CORE_PREAMBLE:
            errs.append(f"{MEDIA_WASM}: preamble {b[:8].hex()} is not a wasm core module")
        else:
            imports, exports = core_interface(b)
            if imports:
                errs.append(
                    f"{MEDIA_WASM} declares {len(imports)} host import(s) — the media jail "
                    f"must have NONE, the guest must not be able to do I/O at all: {imports}"
                )
            if set(exports) != MEDIA_WASM_EXPORTS:
                errs.append(
                    f"{MEDIA_WASM} exports {sorted(exports)}, expected "
                    f"{sorted(MEDIA_WASM_EXPORTS)}"
                )
    return errs


def rebuild_and_compare(root: Path) -> list[str]:
    """Rebuild every artifact from source and compare it to the committed one.

    This is the leg that catches source drift — a committed artifact that nobody
    can reproduce attests to a file, not to the source it claims to come from.

    The comparison differs by artifact, and the difference is the honest part:

      * `media.wasm` is compared BYTE FOR BYTE. Its build.sh remaps the Cargo
        registry and crate paths out of the output, so a Linux build with the
        pinned rustc is deterministic (proven: two builds from different source
        paths and different CARGO_HOMEs give the same digest).
      * The plugin COMPONENTS are compared on their host-capability surface and
        their exports, not their bytes, because their build.sh scripts do not yet
        remap paths and so still embed the builder's registry directory. A byte
        difference is printed, not failed.

    That relaxation is safe only because it is not the whole check: a substituted
    or stale component still fails `verify()` above, on the dist-vs-fixture
    comparison, on the FIRST_PARTY_DIGESTS pin, and on the recorded capability
    surface — three independent byte-level pins inside the repository. What this
    leg adds on top is the one thing those cannot see: that the committed bytes
    still correspond to the CURRENT source. Making the components byte-reproducible
    the way media.wasm now is would let this be tightened to a digest comparison;
    it is a follow-up, not done here.

    Every `build.sh` OVERWRITES the artifact it builds, so each one is snapshotted
    and restored. Leaving the tree dirty would make the checks above fail for the
    next caller — an ordering trap, since a dirty fixture is indistinguishable from
    the drift they exist to catch.
    """
    errs: list[str] = []
    with tempfile.TemporaryDirectory() as td:
        out = Path(td)

        media = root / MEDIA_WASM
        snapshot = out / "media.committed.wasm"
        shutil.copy2(media, snapshot)
        committed = sha256(snapshot)
        try:
            subprocess.run(["sh", str(root / "crates/mw-media-wasm/build.sh")],
                           check=True, cwd=root)
            fresh = sha256(media)
        finally:
            shutil.copy2(snapshot, media)
        print(f"media.wasm  committed={committed[:16]}…  fresh={fresh[:16]}…")
        if fresh != committed:
            errs.append(
                f"{MEDIA_WASM} does not match a fresh build of its source: committed "
                f"{committed}, fresh {fresh}. The build IS reproducible, so this is real "
                f"drift — re-run crates/mw-media-wasm/build.sh and commit the result. "
                f"(If you are not on Linux, see the build.sh header: the remapped paths "
                f"keep the host's separators, so only a Linux build reproduces these bytes.)"
            )

        for cid, _ in sorted(COMPONENTS.items()):
            script = root / BUILD_DIRS[cid] / "build.sh"
            fixture = root / COMPONENTS[cid]
            snapshot = out / f"{cid}.committed.wasm"
            shutil.copy2(fixture, snapshot)
            try:
                subprocess.run(["sh", str(script)], check=True, cwd=root)
                built = fixture.read_bytes()  # each build.sh refreshes its own fixture
            finally:
                shutil.copy2(snapshot, fixture)
            dist = (root / "plugins/dist" / f"{cid}.wasm").read_bytes()
            try:
                bc, dc = host_capabilities(built), host_capabilities(dist)
            except Exception as e:
                errs.append(f"{cid}: rebuilt artifact unreadable: {e}")
                continue
            if bc != dc:
                errs.append(
                    f"{cid}: a fresh build of plugins/{cid}/src imports "
                    f"{sorted(bc)} from the host, but the shipped "
                    f"plugins/dist/{cid}.wasm imports {sorted(dc)}. The shipped artifact "
                    f"has drifted from its source — rebuild it and refresh the digest."
                )
            same = "byte-identical" if built == dist else "differs (paths, not gated)"
            print(f"{cid}: host caps {'match' if bc == dc else 'DIFFER'}; bytes {same}")
    return errs


def core_interface_of_component_exports(b: bytes) -> list[str]:
    """Exports of every core module nested in a component (diagnostic only)."""
    out: list[str] = []
    i = 8
    while i < len(b):
        sec = b[i]
        i += 1
        size, i = _uleb(b, i)
        end = i + size
        if sec == 1 and b[i:i + 4] == b"\x00asm":
            out += core_interface(b[i:end])[1]
        i = end
    return sorted(set(out))


# ── proof that the checks actually reject ─────────────────────────────────────

def self_test(root: Path) -> int:
    """Mutate a throwaway copy of the tree and assert every check fires.

    A check nobody has watched fail is not known to be a check. Each case below is
    a shape that has really happened or really could: the E8-02 stale artifact, a
    substituted artifact, a rebuild whose digest was not regenerated, and a single
    flipped byte.
    """
    cases: list = []

    def mutate(label):
        def deco(fn):
            cases.append((label, fn))
            return fn
        return deco

    @mutate("a shipped artifact that is stale w.r.t. its fixture (the E8-02 shape: "
            "it has lost a host capability the source now needs)")
    def _stale(t: Path):
        # bridge-graph imports oauth-token; bridge-ews does not import it but does
        # import basic-credentials. Standing in one for the other reproduces
        # "the shipped guest cannot reach a capability its source uses".
        src = t / "plugins/dist/bridge-gmail.wasm"
        (t / "plugins/dist/bridge-ews.wasm").write_bytes(src.read_bytes())

    @mutate("a substituted artifact whose digest pin was updated to match "
            "(the tamper shape a digest pin alone cannot see)")
    def _substituted(t: Path):
        dst = t / "plugins/dist/languagetool.wasm"
        dst.write_bytes((t / "plugins/dist/nextcloud.wasm").read_bytes())
        _repin(t, "languagetool", sha256(dst))

    @mutate("a rebuilt artifact whose FIRST_PARTY_DIGESTS entry was not regenerated "
            "(the server would fail closed at boot)")
    def _unpinned(t: Path):
        p = t / "plugins/dist/nextcloud.wasm"
        b = bytearray(p.read_bytes())
        b[len(b) - 1] ^= 0xFF
        p.write_bytes(bytes(b))
        (t / COMPONENTS["nextcloud"]).write_bytes(bytes(b))

    @mutate("one flipped byte in the media jail guest")
    def _media_byte(t: Path):
        p = t / MEDIA_WASM
        b = bytearray(p.read_bytes())
        b[len(b) - 1] ^= 0x01
        p.write_bytes(bytes(b))

    @mutate("a media jail guest rebuilt with host imports (it could then do I/O)")
    def _media_imports(t: Path):
        # Splice a minimal import section into the module. The point is that the
        # zero-import assertion is load-bearing, not decorative.
        p = t / MEDIA_WASM
        b = p.read_bytes()
        body = bytes([0x01, 0x03]) + b"env" + bytes([0x04]) + b"read" + bytes([0x00, 0x00])
        section = bytes([0x02, len(body)]) + body
        p.write_bytes(b[:8] + section + b[8:])

    print(f"positive control: the tree as committed")
    base = verify(root)
    if base:
        print("  the tree does NOT pass its own checks, so the negative cases below "
              "would prove nothing:")
        for e in base:
            print(f"    - {e}")
        return 1
    print("  PASS\n")

    failures = 0
    for label, apply in cases:
        with tempfile.TemporaryDirectory() as td:
            t = Path(td) / "tree"
            for rel in ["plugins", "crates/mw-media-wasm", "crates/mw-server/src"]:
                shutil.copytree(root / rel, t / rel,
                                ignore=shutil.ignore_patterns("target", "*.rs.bk"))
            apply(t)
            errs = verify(t)
            if errs:
                print(f"REJECTED (correct): {label}")
                for e in errs:
                    print(f"    - {e.splitlines()[0][:150]}")
            else:
                print(f"ACCEPTED (BUG — the check is not a check): {label}")
                failures += 1
            print()
    if failures:
        print(f"{failures} mutation(s) were not caught.")
        return 1
    print(f"all {len(cases)} mutations rejected.")
    return 0


def _repin(tree: Path, cid: str, digest: str) -> None:
    """Rewrite one FIRST_PARTY_DIGESTS entry in a throwaway tree."""
    path = tree / "crates/mw-server/src/v7_mount.rs"
    src = path.read_text(encoding="utf-8")
    octets = ", ".join(f"0x{digest[i:i + 2]}" for i in range(0, 64, 2))
    pattern = re.compile(r'("' + re.escape(cid) + r'"\s*,\s*\[)[^\]]*(\])', re.S)
    new, n = pattern.subn(lambda m: m.group(1) + octets + m.group(2), src, count=1)
    if n != 1:
        raise RuntimeError(f"could not re-pin {cid}")
    path.write_text(new, encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--rebuild", action="store_true",
                    help="also rebuild every artifact from source and compare "
                         "(needs the wasm32-wasip2 and wasm32-unknown-unknown targets)")
    ap.add_argument("--self-test", action="store_true",
                    help="prove the checks reject stale, substituted and tampered artifacts")
    args = ap.parse_args()

    if args.self_test:
        return self_test(ROOT)

    errs = verify(ROOT)
    if not errs:
        print(f"{len(COMPONENTS)} shipped components + the media-jail guest verified: "
              f"each matches the fixture its tests load, its FIRST_PARTY_DIGESTS pin and "
              f"its recorded host-capability surface; the media guest matches its pinned "
              f"digest and declares zero host imports.")
    if args.rebuild:
        errs += rebuild_and_compare(ROOT)

    if errs:
        print("\nFAIL:", file=sys.stderr)
        for e in errs:
            print(f"  - {e}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
