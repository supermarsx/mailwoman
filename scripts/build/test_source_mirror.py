"""Linux correctness gates; source only until the owned execution lane is free."""
import contextlib
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch
import source_mirror as mirror

@unittest.skipUnless(sys.platform == "linux", "requires real Linux dir_fd/flock; skips are not proof")
class MirrorContracts(unittest.TestCase):
    def setUp(self):
        self.owned=tempfile.TemporaryDirectory(prefix="mw-source-mirror-")
        self.root=Path(self.owned.name)
        self.input=self.root/"input";self.input.mkdir()
        self.cache=self.root/"cache";self.cache.mkdir()
    def tearDown(self):
        self.owned.cleanup()
    def put(self,path,data=b"old",mode=0o644):
        file=self.input/path;file.parent.mkdir(parents=True,exist_ok=True);file.write_bytes(data);file.chmod(mode)
        return file
    def sync(self):
        with mirror.MirrorSession(self.cache) as session:
            return session.sync(self.input)
    def output(self,path):
        return self.cache/"source"/path
    def test_identical_bytes_keep_inode_and_nanosecond_mtime_despite_new_archive_mtimes(self):
        file=self.put("crate/src/lib.rs")
        self.sync();before=self.output("crate/src/lib.rs").stat()
        os.utime(file,ns=(time.time_ns()+10000000000,)*2)
        result=self.sync();after=self.output("crate/src/lib.rs").stat()
        self.assertEqual((before.st_ino,before.st_mtime_ns),(after.st_ino,after.st_mtime_ns))
        self.assertEqual(result["writtenFiles"],0);self.assertEqual(result["unchangedFiles"],1)
    def test_same_length_change_with_old_input_mtime_is_written_newer_than_real_anchor(self):
        file=self.put("src.rs",b"old");self.sync()
        anchor=self.cache/"target/debug/.fingerprint/unit/dep-lib-unit";anchor.parent.mkdir(parents=True);anchor.write_bytes(b"owned anchor")
        file.write_bytes(b"new");os.utime(file,ns=(1000000000,1000000000))
        result=self.sync();self.assertEqual(self.output("src.rs").read_bytes(),b"new")
        self.assertGreater(self.output("src.rs").stat().st_mtime_ns,anchor.stat().st_mtime_ns)
        self.assertEqual(result["writtenFiles"],1);self.assertEqual(anchor.read_bytes(),b"owned anchor")
    def test_removals_and_type_replacement_do_not_delete_target_artifacts(self):
        self.put("removed/nested.rs");self.put("to-directory");self.put("to-file/old.rs");self.sync()
        artifact=self.cache/"target/keep";artifact.write_bytes(b"target")
        (self.input/"removed/nested.rs").unlink();(self.input/"removed").rmdir()
        (self.input/"to-directory").unlink();self.put("to-directory/new.rs")
        (self.input/"to-file/old.rs").unlink();(self.input/"to-file").rmdir();self.put("to-file",b"file")
        self.sync();self.assertFalse(self.output("removed").exists());self.assertTrue(self.output("to-directory/new.rs").is_file());self.assertEqual(self.output("to-file").read_bytes(),b"file");self.assertEqual(artifact.read_bytes(),b"target")
    def test_mode_change_gets_fresh_timestamp_and_exact_permissions(self):
        self.put("run.sh");self.sync();old=self.output("run.sh").stat().st_mtime_ns
        (self.input/"run.sh").chmod(0o755);self.sync()
        self.assertEqual(self.output("run.sh").stat().st_mode&0o777,0o755);self.assertGreater(self.output("run.sh").stat().st_mtime_ns,old)
    def test_input_symlink_is_rejected_without_following_or_copying(self):
        outside=self.root/"outside";outside.write_bytes(b"private outside")
        (self.input/"link").symlink_to(outside)
        with self.assertRaises((mirror.MirrorError,OSError)):self.sync()
        self.assertEqual(outside.read_bytes(),b"private outside");self.assertFalse(self.output("link").exists())
    def test_cache_symlink_is_rejected_without_escaped_write(self):
        self.put("src.rs");self.sync();outside=self.root/"outside";outside.write_bytes(b"outside")
        self.output("src.rs").unlink();self.output("src.rs").symlink_to(outside)
        with self.assertRaises((mirror.MirrorError,OSError)):self.sync()
        self.assertEqual(outside.read_bytes(),b"outside")
    def test_hardlink_and_fifo_are_rejected_before_mutation(self):
        outside=self.root/"outside";outside.write_bytes(b"outside");os.link(outside,self.input/"hard")
        with self.assertRaises(mirror.MirrorError):self.sync()
        (self.input/"hard").unlink();os.mkfifo(self.input/"pipe")
        with self.assertRaises(mirror.MirrorError):self.sync()
    def test_unowned_nonempty_cache_is_not_mutated(self):
        untouched=self.cache/"unowned";untouched.write_bytes(b"data")
        with self.assertRaises(mirror.MirrorError):self.sync()
        self.assertEqual(set(p.name for p in self.cache.iterdir()),{"unowned"})
    def test_input_cache_overlap_and_unsafe_components_are_refused(self):
        self.put("src.rs")
        with mirror.MirrorSession(self.cache) as session:
            with self.assertRaises(mirror.MirrorError):session.sync(self.root)
        for value in ["../escape","a/../b","a\\b","C:drive"]:
            with self.assertRaises(mirror.MirrorError):mirror.name(value)
    def test_future_anchor_fails_closed_without_backdating_or_changing_bytes(self):
        file=self.put("src.rs",b"old");self.sync();file.write_bytes(b"new")
        anchor=self.cache/"target/release/.fingerprint/unit/dep-lib-unit";anchor.parent.mkdir(parents=True);anchor.write_bytes(b"anchor")
        os.utime(anchor,ns=(time.time_ns()+10000000000,)*2)
        with self.assertRaises(mirror.MirrorError):self.sync()
        self.assertEqual(self.output("src.rs").read_bytes(),b"old")
    def test_interrupted_sync_rewrites_even_equal_bytes_and_discards_stale_temp(self):
        self.put("src.rs");self.sync();old=self.output("src.rs").stat().st_mtime_ns
        (self.cache/".mirror-dirty").write_bytes(b"");self.output(".mirror-tmp-interrupted").write_bytes(b"partial")
        result=self.sync();self.assertTrue(result["recoveredInterruptedSync"]);self.assertEqual(result["writtenFiles"],1);self.assertGreater(self.output("src.rs").stat().st_mtime_ns,old);self.assertFalse(self.output(".mirror-tmp-interrupted").exists())
    def test_byte_limit_failure_does_not_replace_previous_source(self):
        file=self.put("src.rs",b"old");self.sync();file.write_bytes(b"too large")
        with patch.object(mirror,"MAX_FILE_BYTES",8):
            with self.assertRaises(mirror.MirrorError):self.sync()
        self.assertEqual(self.output("src.rs").read_bytes(),b"old")
    def test_concurrent_source_and_target_owner_is_rejected(self):
        self.put("src.rs")
        with mirror.MirrorSession(self.cache) as first:
            first.sync(self.input)
            with self.assertRaises(mirror.MirrorBusy):
                with mirror.MirrorSession(self.cache):pass
        self.assertEqual(self.sync()["unchangedFiles"],1)
    def test_anchor_metadata_budget_fails_closed_before_source_replacement(self):
        file=self.put("src.rs",b"old");self.sync();file.write_bytes(b"new")
        anchor=self.cache/"target/debug/.fingerprint/unit/dep-lib-unit";anchor.parent.mkdir(parents=True);anchor.write_bytes(b"anchor")
        with patch.object(mirror,"MAX_ANCHOR_STATS",2):
            with self.assertRaises(mirror.MirrorError):self.sync()
        self.assertEqual(self.output("src.rs").read_bytes(),b"old")
    def test_symlinked_input_or_cache_root_is_never_followed(self):
        self.put("src.rs")
        source_link=self.root/"input-link";source_link.symlink_to(self.input,target_is_directory=True)
        with mirror.MirrorSession(self.cache) as session:
            with self.assertRaises((mirror.MirrorError,OSError)):session.sync(source_link)
        cache_link=self.root/"cache-link";cache_link.symlink_to(self.cache,target_is_directory=True)
        with self.assertRaises((mirror.MirrorError,OSError)):
            with mirror.MirrorSession(cache_link):pass
    def test_metadata_visible_input_change_after_comparison_is_rejected(self):
        file=self.put("src.rs",b"old");self.sync();original=mirror.equal_files
        def changed(*args,**kwargs):
            result=original(*args,**kwargs);file.write_bytes(b"new-length");return result
        with patch.object(mirror,"equal_files",changed):
            with self.assertRaises(mirror.MirrorError):self.sync()
        self.assertEqual(self.output("src.rs").read_bytes(),b"old")
    def test_cli_config_files_complex_keys_and_toolchain_selectors_are_rejected(self):
        self.put("src.rs");self.sync()
        denied=[
            ["cargo","--config","/tmp/extra.toml","build"],
            ["cargo","--config=/tmp/extra.toml","build"],
            ["cargo","--config",'build."build-dir"="/outside"',"build"],
            ["cargo","--config",'"bu\\u0069ld"."build-dir"="/outside"',"build"],
            ["cargo","--config"],
            ["cargo","+nightly","build"],
        ]
        with mirror.MirrorSession(self.cache) as session:
            for command in denied:
                with self.subTest(command=command):
                    with patch.object(mirror.subprocess,"run") as child:
                        with self.assertRaises(mirror.MirrorError):session.run(command)
                        child.assert_not_called()
    def test_only_exact_inline_profile_override_preserves_owned_child_environment(self):
        self.put("src.rs");self.sync()
        command=["cargo","build","--config","profile.release.package.mw-server.opt-level=2"]
        with mirror.MirrorSession(self.cache) as session:
            with patch.object(mirror.subprocess,"run") as child:
                session.run(command)
                self.assertEqual(child.call_args.args[0],command)
                self.assertEqual(child.call_args.kwargs["env"]["CARGO_BUILD_BUILD_DIR"],str(self.cache/"target"))
    def test_effective_build_dir_overrides_source_home_and_inherited_environment(self):
        self.put(".cargo/config.toml",b'[build]\nbuild-dir="/outside-source"\n')
        home=self.root/"cargo-home";home.mkdir()
        (home/"config.toml").write_text('[build]\nbuild-dir="/outside-home"\n')
        code="import os;print(os.environ['CARGO_BUILD_BUILD_DIR']);print(os.environ['CARGO_TARGET_DIR'])"
        with patch.dict(os.environ,{"CARGO_HOME":str(home),"CARGO_BUILD_BUILD_DIR":"/outside-env"}):
            with mirror.MirrorSession(self.cache) as session:
                session.sync(self.input)
                result=session.run([sys.executable,"-c",code],capture_output=True,text=True,check=True,timeout=5)
        self.assertEqual(result.stdout.splitlines(),[str(self.cache/"target")]*2)
    def test_input_growth_during_streaming_cannot_replace_or_exceed_scanned_budget(self):
        file=self.put("src.rs",b"old");self.sync();file.write_bytes(b"newlong")
        original=mirror.os.read;grew=False
        def growing(fd,count):
            nonlocal grew
            result=original(fd,count)
            if not grew and result==b"newlong":
                grew=True
                with file.open("ab") as stream:stream.write(b"growth")
            return result
        # Different lengths skip equality reads; mutation occurs in the copy loop.
        (self.cache/".mirror-dirty").write_bytes(b"")
        with patch.object(mirror.os,"read",growing):
            with self.assertRaisesRegex(mirror.MirrorError,"copy budget"):
                self.sync()
        self.assertTrue(grew)
        self.assertEqual(self.output("src.rs").read_bytes(),b"old")
        self.assertTrue((self.cache/".mirror-dirty").exists())
        self.assertFalse(any(self.output("").glob(".mirror-tmp-*")))
    def test_accepted_child_keeps_pair_lock_after_parent_observer_terminates(self):
        self.put("src.rs")
        ready=self.root/"ready";release=self.root/"release";done=self.root/"done"
        child_code="import os,pathlib,time,sys;p=pathlib.Path(sys.argv[1]);p.write_text(str(os.getpid()));end=time.monotonic()+5;\nwhile not pathlib.Path(sys.argv[2]).exists() and time.monotonic()<end: time.sleep(.01)\npathlib.Path(sys.argv[3]).write_text('done')"
        parent_code="import sys;sys.path.insert(0,sys.argv[1]);import source_mirror as m;\nwith m.MirrorSession(sys.argv[2]) as s: s.sync(sys.argv[3]);s.run([sys.executable,'-c',sys.argv[4],*sys.argv[5:]])"
        observer=subprocess.Popen([sys.executable,"-c",parent_code,str(Path(mirror.__file__).parent),str(self.cache),str(self.input),child_code,str(ready),str(release),str(done)],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        child_pid=None
        try:
            deadline=time.monotonic()+3
            while not ready.exists() and time.monotonic()<deadline:time.sleep(.01)
            self.assertTrue(ready.exists());child_pid=int(ready.read_text())
            observer.terminate();observer.wait(timeout=2)
            with self.assertRaises(mirror.MirrorBusy):
                with mirror.MirrorSession(self.cache):pass
            release.write_text("release")
            deadline=time.monotonic()+3
            while time.monotonic()<deadline:
                try:
                    with mirror.MirrorSession(self.cache):break
                except mirror.MirrorBusy:time.sleep(.01)
            else:self.fail("owned child failed to release the cache pair")
            self.assertTrue(done.exists())
        finally:
            release.touch()
            if observer.poll() is None:observer.kill();observer.wait(timeout=2)
            if child_pid and not done.exists():
                with contextlib.suppress(ProcessLookupError):os.kill(child_pid,signal.SIGKILL)

if __name__=="__main__":unittest.main()
