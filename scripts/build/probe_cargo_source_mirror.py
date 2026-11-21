#!/usr/bin/env python3
"""Owned offline3-package Cargo correctness probe; never a production benchmark.
Run ONLY after the compiler lane is released, on pinned Cargo1.98/Linux.
Retains all artifacts/logs; no Docker, host cache purge or automatic cache delete.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
from source_mirror import MirrorSession,MirrorError,require_linux,open_root,bounded_names

def seed(root):
    files={
      ".cargo/config.toml":'[build]\nbuild-dir="../../escaped-build-config"\n',
      "Cargo.toml":'[workspace]\nmembers=["stable","changing","app"]\nresolver="2"\n',
      "Cargo.lock":'# mirror-token: 1\nversion = 4\n\n[[package]]\nname = "changing-lib"\nversion = "0.1.0"\n\n[[package]]\nname = "mirror-probe"\nversion = "0.1.0"\ndependencies = ["changing-lib", "stable-lib"]\n\n[[package]]\nname = "stable-lib"\nversion = "0.1.0"\n',
      "stable/Cargo.toml":'[package]\nname="stable-lib"\nversion="0.1.0"\nedition="2024"\n',
      "stable/src/lib.rs":'pub fn value()->u32{7}\n',
      "changing/Cargo.toml":'[package]\nname="changing-lib"\nversion="0.1.0"\nedition="2024"\n',
      "changing/src/lib.rs":'mod obsolete; pub fn value()->u32{11+obsolete::ZERO}\n',
      "changing/src/obsolete.rs":'pub const ZERO:u32=0;\n',
      "app/Cargo.toml":'[package]\nname="mirror-probe"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nstable-lib={path="../stable"}\nchanging-lib={path="../changing"}\n',
      "app/src/main.rs":'include!(concat!(env!("OUT_DIR"),"/token.rs")); fn main(){println!("{} {}",stable_lib::value()+changing_lib::value(),TOKEN);}\n',
      "app/build.rs":'fn main(){println!("cargo:rerun-if-changed=../Cargo.lock");let t=std::fs::read_to_string("../Cargo.lock").unwrap();let v=t.lines().find_map(|s|s.strip_prefix("# mirror-token: ")).unwrap();std::fs::write(std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("token.rs"),format!("const TOKEN:u32={};",v)).unwrap();}\n'
    }
    for relative,value in files.items():
        path=root/relative;path.parent.mkdir(parents=True,exist_ok=True);path.write_text(value)

def main():
    require_linux()
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument("--work-root",required=True);parser.add_argument("--cargo",default="cargo");args=parser.parse_args()
    root=Path(args.work_root)
    if not root.is_absolute():raise MirrorError("explicit empty owned work directory required")
    fd=open_root(root)
    try:
        if bounded_names(fd,1):raise MirrorError("empty owned probe directory required")
    finally:os.close(fd)
    version=subprocess.run([args.cargo,"--version"],capture_output=True,text=True,check=True,timeout=10).stdout.strip()
    if not version.startswith("cargo 1.98."):raise MirrorError("probe requires pinned Cargo1.98")
    source=root/"input";source.mkdir();cache=root/"cache";cache.mkdir();reports=root/"reports";reports.mkdir();seed(source)
    # Actual owned CLI-file escape input must be refused before Cargo starts;
    # the normal phases separately prove source-local config cannot relocate it.
    cli_config=root/"cli-escape.toml"
    cli_config.write_text('[build]\nbuild-dir="'+str(root/"escaped-cli")+'"\n')
    with MirrorSession(cache) as pair:
        pair.sync(source)
        try:pair.run([args.cargo,"--config",str(cli_config),"build","--offline","--locked"],timeout=10)
        except MirrorError:pass
        else:raise MirrorError("CLI-file escape was accepted")
    if (root/"escaped-cli").exists():raise MirrorError("CLI escape created outside state")
    phases=[]
    def run(label,expected,expected_fresh=None):
        output=reports/(label+".jsonl");errors=reports/(label+".stderr.txt")
        with MirrorSession(cache) as pair,output.open("xb") as stdout,errors.open("xb") as stderr:
            sync=pair.sync(source)
            environment_log=os.environ.get("CARGO_LOG")
            os.environ["CARGO_LOG"]="cargo::core::compiler::fingerprint=trace"
            try:result=pair.run([args.cargo,"build","--offline","--locked","--message-format=json","-p","mirror-probe"],stdout=stdout,stderr=stderr,timeout=120)
            finally:
                if environment_log is None:os.environ.pop("CARGO_LOG",None)
                else:os.environ["CARGO_LOG"]=environment_log
        if (root/"escaped-build-config").exists():raise MirrorError("Cargo config escaped owned build directory")
        if output.stat().st_size>8*1024*1024 or errors.stat().st_size>8*1024*1024:raise MirrorError("probe log budget exceeded")
        rows=[json.loads(line) for line in output.read_text().splitlines() if line]
        artifacts={row["target"]["name"]:row["fresh"] for row in rows if row.get("reason")=="compiler-artifact" and row["target"]["kind"] in (["lib"],["bin"])}
        if expected is None:
            if result.returncode==0:raise MirrorError("removed referenced source falsely reused stale code")
            actual=None
        else:
            if result.returncode!=0:raise MirrorError("probe compile failed; retained owned logs")
            actual=subprocess.run([str(cache/"target/debug/mirror-probe")],capture_output=True,text=True,check=True,timeout=10).stdout.strip()
            if actual!=expected:raise MirrorError("probe runtime output is stale")
            if expected_fresh is not None:
                for key,value in expected_fresh.items():
                    if artifacts.get(key) is not value:raise MirrorError("unexpected actual compiler-artifact freshness")
        phases.append({"phase":label,"exitCode":result.returncode,"runtime":actual,"compilerArtifactsFresh":artifacts,"sourceMirror":sync})
        (reports/"summary.json").write_text(json.dumps({"verified":False,"cargo":version,"phases":phases},indent=2)+"\n")
    run("first","18 1")
    for path in source.rglob("*"):
        if path.is_file():os.utime(path,ns=(1100000000000000000,)*2)
    run("identical-new-archive-mtimes","18 1",{"stable_lib":True,"changing_lib":True,"mirror-probe":True})
    changed=source/"changing/src/lib.rs";changed.write_text(changed.read_text().replace("11+","12+"));os.utime(changed,ns=(1000000000,1000000000))
    run("same-length-old-input-time-change","19 1",{"stable_lib":True,"changing_lib":False,"mirror-probe":False})
    (source/"changing/src/obsolete.rs").unlink();run("referenced-source-removed",None)
    (source/"changing/src/obsolete.rs").write_text("pub const ZERO:u32=1;\n");run("source-restored","20 1")
    lock=source/"Cargo.lock";lock.write_text(lock.read_text().replace("mirror-token: 1","mirror-token: 2"));run("lock-byte-change-build-script","20 2",{"stable_lib":True,"changing_lib":True,"mirror-probe":False})
    (reports/"summary.json").write_text(json.dumps({"verified":True,"cargo":version,"scope":"tiny offline correctness/freshness probe, not application build speed","phases":phases},indent=2)+"\n")

if __name__=="__main__":main()
