//! Second-layer media jail host (SPEC §7.5, plan t16 S5 / DQ5).
//!
//! The render child parses hostile compound files (`.msg`/`.oft` CFB) and
//! re-encodes remote images **inside a wasmtime jail** running the committed
//! `crates/mw-media-wasm` guest — never as native Rust. The guest is a bare
//! `wasm32` core module with **zero host imports** (it cannot touch the
//! filesystem, network, clock, or randomness); this host bounds it further with a
//! linear-memory ceiling and a wall-clock deadline (epoch interruption). A trap
//! (malformed input, runaway loop, memory-ceiling trip) surfaces as a plain
//! `Err` — the hostile codec can crash the guest without touching this process.
//!
//! The engine targets the **Pulley pure-interpreter** (no JIT, no W^X page), so it
//! keeps working under the render child's seccomp/Landlock jail (t16-e4) and
//! systemd `MemoryDenyWriteExecute`.
//!
//! `reencode_image` is the entry the anonymizing image proxy (t16-e6) consumes to
//! strip/normalise fetched images.

use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use wasmtime::{Config, Engine, Linker, Module, ResourceLimiter, Store};

/// The committed media-jail guest, built from `crates/mw-media-wasm` (`build.sh`).
static MEDIA_WASM: &[u8] = include_bytes!("../../mw-media-wasm/media.wasm");

/// Linear-memory ceiling for one jail invocation. Generous enough for a decoded
/// bitmap of a legitimate image, a hard backstop against decompression bombs (the
/// guest also caps decode dimensions/alloc).
const MEMORY_MAX: usize = 512 * 1024 * 1024;
/// Wall-clock CPU budget per invocation, enforced via epoch interruption.
const DEADLINE_MS: u64 = 5_000;
/// Epoch ticker cadence (coarse but < the smallest useful deadline).
const EPOCH_TICK: Duration = Duration::from_millis(5);
/// Output ceiling: a re-encoded PNG can exceed the 4 MiB input, but not without bound.
const MAX_OUTPUT_BYTES: usize = 16 * 1024 * 1024;

/// Memory/table growth limiter for the disposable store.
struct Limiter;

impl ResourceLimiter for Limiter {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        Ok(desired <= MEMORY_MAX)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        Ok(desired <= 100_000)
    }
}

/// The shared jail engine, built once. Pulley pure-interpreter target: no JIT,
/// no W^X page — safe under the render child's kernel jail and systemd MDWE.
fn engine() -> &'static Engine {
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE.get_or_init(|| {
        let mut cfg = Config::new();
        cfg.epoch_interruption(true);
        cfg.target("pulley64")
            .expect("wasmtime pulley target unavailable");
        Engine::new(&cfg).expect("wasmtime media-jail engine init")
    })
}

/// The compiled guest module, built once from the committed bytes.
fn module() -> &'static Module {
    static MODULE: OnceLock<Module> = OnceLock::new();
    MODULE.get_or_init(|| Module::new(engine(), MEDIA_WASM).expect("compile media.wasm"))
}

/// Epoch ticker: advances the engine epoch so the per-store wall-clock deadline
/// fires. Scoped to one invocation; stopped + joined on drop (no thread leak).
struct Ticker {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Ticker {
    fn spawn(engine: Engine) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();
        let handle = std::thread::Builder::new()
            .name("mw-media-jail-epoch".into())
            .spawn(move || {
                while !stop_thread.load(Ordering::Relaxed) {
                    std::thread::sleep(EPOCH_TICK);
                    engine.increment_epoch();
                }
            })
            .expect("spawn media-jail epoch ticker");
        Self {
            stop,
            handle: Some(handle),
        }
    }

    /// Epoch ticks corresponding to `DEADLINE_MS` of wall-clock time.
    fn deadline_ticks() -> u64 {
        (DEADLINE_MS / EPOCH_TICK.as_millis() as u64).max(1)
    }
}

impl Drop for Ticker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// Run one jail entry (`mw_parse_cfb` / `mw_reencode_image`) over `input`, in a
/// fresh disposable store, and return the entry's payload (status byte stripped).
/// An error frame from the guest, or any trap, becomes an `Err`.
fn call(entry: &str, input: &[u8]) -> Result<Vec<u8>, String> {
    let engine = engine();
    let mut store = Store::new(engine, Limiter);
    store.limiter(|s| s);
    store.set_epoch_deadline(Ticker::deadline_ticks());
    store.epoch_deadline_trap();

    // Empty linker: the guest imports nothing, so nothing is granted.
    let linker: Linker<Limiter> = Linker::new(engine);
    let instance = linker
        .instantiate(&mut store, module())
        .map_err(|e| format!("media-jail instantiate: {e}"))?;

    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or("media-jail: no memory export")?;
    let alloc = instance
        .get_typed_func::<u32, u32>(&mut store, "mw_alloc")
        .map_err(|e| format!("media-jail: mw_alloc: {e}"))?;
    let func = instance
        .get_typed_func::<(u32, u32), u64>(&mut store, entry)
        .map_err(|e| format!("media-jail: {entry}: {e}"))?;

    let in_len = u32::try_from(input.len()).map_err(|_| "media-jail: input too large")?;

    // The ticker only needs to run while guest code executes.
    let ticker = Ticker::spawn(engine.clone());
    let ptr = alloc
        .call(&mut store, in_len)
        .map_err(|e| format!("media-jail: alloc trapped: {e}"))?;
    memory
        .write(&mut store, ptr as usize, input)
        .map_err(|e| format!("media-jail: write input: {e}"))?;
    let packed = func
        .call(&mut store, (ptr, in_len))
        .map_err(|e| format!("media-jail: {entry} trapped: {e}"))?;
    drop(ticker);

    let out_ptr = (packed >> 32) as usize;
    let out_len = (packed & 0xffff_ffff) as usize;
    if out_len == 0 {
        return Err("media-jail: empty output".into());
    }
    if out_len > MAX_OUTPUT_BYTES {
        return Err("media-jail: output exceeds limit".into());
    }
    let mut buf = vec![0u8; out_len];
    memory
        .read(&mut store, out_ptr, &mut buf)
        .map_err(|e| format!("media-jail: read output: {e}"))?;

    match buf.first() {
        Some(1) => Ok(buf[1..].to_vec()),
        Some(0) => Err(String::from_utf8_lossy(&buf[1..]).into_owned()),
        _ => Err("media-jail: malformed output frame".into()),
    }
}

/// What the CFB jail recovers for the render child: the template subject + body.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CfbParsed {
    pub subject: Option<String>,
    pub body: String,
}

/// Parse an untrusted `.msg`/`.oft` CFB container **inside the wasm jail**. The
/// OLE2/CFB parse and MS-OXMSG stream reads run in the sandbox; malformed input
/// returns `Err`, never a native panic or memory-safety issue in this process.
pub fn parse_cfb(bytes: &[u8]) -> Result<CfbParsed, String> {
    let payload = call("mw_parse_cfb", bytes)?;
    let mut off = 0;
    let subject = read_field(&payload, &mut off)?;
    let body = read_field(&payload, &mut off)?;

    let subject = String::from_utf8_lossy(subject);
    Ok(CfbParsed {
        subject: (!subject.is_empty()).then(|| subject.into_owned()),
        body: String::from_utf8_lossy(body).into_owned(),
    })
}

/// Decode an untrusted image and re-encode it to a normalised, metadata-stripped
/// PNG **inside the wasm jail**. Consumed by the anonymizing image proxy (t16-e6).
pub fn reencode_image(bytes: &[u8]) -> Result<Vec<u8>, String> {
    call("mw_reencode_image", bytes)
}

/// Read one `[u32 len LE][bytes]` field from the framed CFB payload.
fn read_field<'a>(buf: &'a [u8], off: &mut usize) -> Result<&'a [u8], String> {
    let hdr_end = off
        .checked_add(4)
        .ok_or("media-jail: truncated field header")?;
    let len_bytes = buf
        .get(*off..hdr_end)
        .ok_or("media-jail: truncated field header")?;
    let len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
    let data_end = hdr_end
        .checked_add(len)
        .ok_or("media-jail: field length overflow")?;
    let data = buf
        .get(hdr_end..data_end)
        .ok_or("media-jail: truncated field")?;
    *off = data_end;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real `.oft` (written by `mw-export`) round-trips THROUGH the wasm jail —
    /// the CFB parse ran in the sandbox, not native Rust.
    #[test]
    fn cfb_parses_in_the_jail() {
        let raw = b"Subject: Weekly status template\r\n\r\nFill me in.\r\n";
        let oft = mw_export::export_one(
            &mw_export::RawEmail::new(raw.to_vec()),
            mw_export::Format::Oft,
        )
        .expect("write .oft");
        let parsed = parse_cfb(&oft).expect("jail parse");
        assert_eq!(parsed.subject.as_deref(), Some("Weekly status template"));
        assert!(parsed.body.contains("Fill me in"));
    }

    /// Malformed CFB bytes cannot reach a native parser: the jail returns `Err`,
    /// this process survives.
    #[test]
    fn malformed_cfb_is_refused_not_crashed() {
        // Valid CFB magic then garbage — exercises the OLE2 parser on hostile input.
        let mut hostile = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
        hostile.extend(std::iter::repeat_n(0xFFu8, 512));
        assert!(parse_cfb(&hostile).is_err());
        // Non-CFB bytes are refused too.
        assert!(parse_cfb(b"not a compound file").is_err());
    }

    /// The image re-encode entry produces a normalised PNG from a valid input and
    /// refuses garbage.
    #[test]
    fn image_reencode_normalises_and_refuses_garbage() {
        // A 1x1 GIF (smallest real image) → re-encoded to PNG.
        let gif: &[u8] = &[
            0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x21, 0xF9, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2C,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00,
            0x3B,
        ];
        let png = reencode_image(gif).expect("re-encode");
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        assert!(reencode_image(b"not an image").is_err());
    }
}
