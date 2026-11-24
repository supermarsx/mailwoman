#![recursion_limit = "512"]
//! mw-loadgen: native counterpart of the scripts/perf Node load generator.
//! Driven by `native-measure.mjs` (one process per measured stage); see
//! docs/perf/native-generator.md. Only private plan/stage/output file paths are
//! arguments, never credentials.
mod engine;
mod http1;
mod http2;
mod js;
mod jv;
mod measure;
mod plan;
mod serve;
mod sys;
mod templates;
mod validate;

use serde_json::Value;
use std::collections::HashMap;

fn args() -> Result<(String, HashMap<String, String>), String> {
    let mut it = std::env::args().skip(1);
    let command = it
        .next()
        .ok_or("usage: mw-loadgen <stage|templates|validate|serve|sysinfo> --name=value ...")?;
    let mut map = HashMap::new();
    for arg in it {
        let body = arg.strip_prefix("--").ok_or("Use --name=value options")?;
        let (k, v) = body.split_once('=').ok_or("Use --name=value options")?;
        map.insert(k.to_owned(), v.to_owned());
    }
    Ok((command, map))
}

const FILE_LIMIT: u64 = 256 * 1024 * 1024;

fn read_json(path: &str) -> Result<Value, String> {
    let meta = std::fs::metadata(path).map_err(|_| format!("cannot read {path}"))?;
    if !meta.is_file() || meta.len() > FILE_LIMIT {
        return Err("input must be a regular bounded file".into());
    }
    let bytes = std::fs::read(path).map_err(|_| format!("cannot read {path}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("invalid JSON in input: {e}"))
}

/// Writes a new private file (0600 on Unix).
pub fn write_private(path: &str, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|_| format!("cannot write {path}"))?;
    file.write_all(bytes)
        .map_err(|_| format!("cannot write {path}"))
}

pub fn base64_decode(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in text.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' | b'\n' | b'\r' => continue,
            _ => return None,
        } as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}

fn plan_of(map: &HashMap<String, String>) -> Result<Option<plan::PlanFile>, String> {
    match map.get("plan") {
        Some(path) => Ok(Some(
            serde_json::from_value(read_json(path)?).map_err(|e| format!("invalid plan: {e}"))?,
        )),
        None => Ok(None),
    }
}

fn run() -> Result<(), String> {
    let (command, map) = args()?;
    let out = |value: &Value| -> Result<(), String> {
        let bytes = serde_json::to_vec(value).unwrap();
        match map.get("out") {
            Some(path) => write_private(path, &bytes),
            None => {
                println!("{}", String::from_utf8_lossy(&bytes));
                Ok(())
            }
        }
    };
    match command.as_str() {
        "stage" => {
            let plan = plan_of(&map)?;
            let stage: engine::StageSpec =
                serde_json::from_value(read_json(map.get("stage").ok_or("--stage= required")?)?)
                    .map_err(|e| format!("invalid stage: {e}"))?;
            out(&engine::run(plan, stage)?)
        }
        "templates" => {
            let plan = plan_of(&map)?.ok_or("--plan= required")?;
            let count: usize = map
                .get("count")
                .map_or(Ok(2), |v| v.parse())
                .map_err(|_| "invalid --count")?;
            out(&engine::dump_templates(
                &plan,
                map.get("endpoint").ok_or("--endpoint= required")?,
                count,
                map.get("host").map_or("127.0.0.1:1", String::as_str),
            )?)
        }
        "validate" => {
            let plan = plan_of(&map)?.ok_or("--plan= required")?;
            let corpus = read_json(map.get("corpus").ok_or("--corpus= required")?)?;
            out(&engine::validate_corpus(
                &plan,
                corpus.as_array().ok_or("corpus must be an array")?,
            )?)
        }
        "bench-validate" => {
            let plan = plan_of(&map)?.ok_or("--plan= required")?;
            let corpus = read_json(map.get("corpus").ok_or("--corpus= required")?)?;
            out(&engine::bench_validate(
                &plan,
                corpus.as_array().ok_or("corpus must be an array")?,
                200,
            )?)
        }
        "serve" => serve::run(&map),
        "sysinfo" => out(&sys::processors()),
        _ => Err("unknown command".into()),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
