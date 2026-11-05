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
pub const IMAGE_MEMORY_MAX_BYTES: usize = 512 * 1024 * 1024;
/// What one image decode typically needs, used only to size how many decodes may
/// run at once. The limiter above is the hard ceiling; reserving that ceiling for
/// every decoder (as sizing once did) left a 2 GiB instance with a single decoder
/// and made image bursts queue until they timed out. A 12 MP photo is ~48 MB of
/// bitmap, so 96 MiB covers the working set of the images mail actually carries.
pub const IMAGE_DECODE_TYPICAL_BYTES: usize = 96 * 1024 * 1024;
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
        Ok(desired <= IMAGE_MEMORY_MAX_BYTES)
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

/// Compile the guest module now rather than inside the first image request.
/// Compilation took seconds on a cold process, and every request arriving meanwhile
/// waited on it (and was answered "busy" once its deadline passed). Blocking; call
/// from a background thread at start-up.
pub fn warm() {
    use base64::Engine as _;
    const PIXEL: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";
    if let Ok(png) = base64::engine::general_purpose::STANDARD.decode(PIXEL) {
        let _ = reencode_image_classified(&png);
    } else {
        let _ = module();
    }
}

/// The shared jail engine, built once. Pulley pure-interpreter target: no JIT,
/// no W^X page — safe under the render child's kernel jail and systemd MDWE.
fn engine() -> &'static Engine {
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE.get_or_init(|| {
        let mut cfg = Config::new();
        cfg.epoch_interruption(true);
        // A disposable child recompiles this already optimized guest per job.
        // Skip optimizing the Pulley lowering again: cold-start must fit the
        // parent's bounded worker lifetime, not only a warm in-process test.
        cfg.cranelift_opt_level(wasmtime::OptLevel::None);
        // No copy-on-write memory images. On Linux wasmtime backs them with
        // `memfd_create`, which the render child's seccomp allowlist does not
        // permit, so every CFB job was SIGSYS-killed inside the kernel jail from
        // 26.16 until t24-e10. Initial linear memory is copied instead of mapped: a
        // per-instantiation cost, with the seccomp allowlist and rlimits unchanged.
        // (Allowlisting memfd would not have worked either: writes to it are
        // bounded by the jail's `RLIMIT_FSIZE=0`.)
        cfg.memory_init_cow(false);
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

/// One ticker per shared engine: concurrent requests must not multiply the clock
/// and shorten each other's deadlines. It lives for the process lifetime.
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
#[derive(Debug)]
enum JailFailure {
    Host(String),
    Guest(String),
    Rejected(String),
}
impl std::fmt::Display for JailFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Host(value) | Self::Guest(value) | Self::Rejected(value) => f.write_str(value),
        }
    }
}
impl From<String> for JailFailure {
    fn from(value: String) -> Self {
        Self::Host(value)
    }
}
impl From<&str> for JailFailure {
    fn from(value: &str) -> Self {
        Self::Host(value.into())
    }
}

fn call(entry: &str, input: &[u8]) -> Result<Vec<u8>, JailFailure> {
    call_with_budget(entry, input, Duration::from_millis(DEADLINE_MS))
}

fn call_with_budget(entry: &str, input: &[u8], budget: Duration) -> Result<Vec<u8>, JailFailure> {
    let budget = budget.clamp(Duration::from_millis(1), Duration::from_secs(60));
    let ticks = (budget.as_millis() as u64)
        .div_ceil(EPOCH_TICK.as_millis() as u64)
        .max(1);
    if input.len() > crate::MAX_INPUT_BYTES {
        return Err(JailFailure::Rejected(
            "media-jail: input exceeds limit".into(),
        ));
    }
    let engine = engine();
    let compiled = module();
    static TICKER: OnceLock<Ticker> = OnceLock::new();
    TICKER.get_or_init(|| Ticker::spawn(engine.clone()));
    let mut store = Store::new(engine, Limiter);
    store.limiter(|s| s);
    store.set_epoch_deadline(ticks);
    store.epoch_deadline_trap();

    // Empty linker: the guest imports nothing, so nothing is granted.
    let linker: Linker<Limiter> = Linker::new(engine);
    let instance = linker
        .instantiate(&mut store, compiled)
        .map_err(|e| JailFailure::Host(format!("media-jail instantiate: {e}")))?;

    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or("media-jail: no memory export")?;
    let alloc = instance
        .get_typed_func::<u32, u32>(&mut store, "mw_alloc")
        .map_err(|e| JailFailure::Host(format!("media-jail: mw_alloc: {e}")))?;
    let func = instance
        .get_typed_func::<(u32, u32), u64>(&mut store, entry)
        .map_err(|e| JailFailure::Host(format!("media-jail: {entry}: {e}")))?;

    let in_len = u32::try_from(input.len()).map_err(|_| "media-jail: input too large")?;

    // Compilation/instantiation may be cold. Start this request's execution
    // budget immediately before entering guest code, on the single shared clock.
    store.set_epoch_deadline(ticks);
    store.epoch_deadline_trap();
    let ptr = alloc
        .call(&mut store, in_len)
        .map_err(|e| JailFailure::Host(format!("media-jail: alloc trapped: {e}")))?;
    memory
        .write(&mut store, ptr as usize, input)
        .map_err(|e| JailFailure::Host(format!("media-jail: write input: {e}")))?;
    let packed = func
        .call(&mut store, (ptr, in_len))
        .map_err(|e| JailFailure::Host(format!("media-jail: {entry} trapped: {e:#}")))?;

    let out_ptr = (packed >> 32) as usize;
    let out_len = (packed & 0xffff_ffff) as usize;
    if out_len == 0 {
        return Err("media-jail: empty output".into());
    }
    if out_len > MAX_OUTPUT_BYTES {
        return Err(JailFailure::Rejected(
            "media-jail: output exceeds limit".into(),
        ));
    }
    let mut buf = vec![0u8; out_len];
    memory
        .read(&mut store, out_ptr, &mut buf)
        .map_err(|e| JailFailure::Host(format!("media-jail: read output: {e}")))?;

    match buf.first() {
        Some(1) => Ok(buf[1..].to_vec()),
        Some(0) => Err(JailFailure::Guest(
            String::from_utf8_lossy(&buf[1..]).into_owned(),
        )),
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
    let payload = call("mw_parse_cfb", bytes).map_err(|error| error.to_string())?;
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
    call("mw_reencode_image", bytes).map_err(|error| error.to_string())
}

/// Guest codec rejection is permanent; host/epoch/resource failure may recover
/// after bounded admission. Never infer this distinction from codec error text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageDecodeFailure {
    InvalidImage,
    JailUnavailable,
}
pub fn reencode_image_classified(bytes: &[u8]) -> Result<Vec<u8>, ImageDecodeFailure> {
    reencode_image_with_budget(bytes, Duration::from_millis(DEADLINE_MS))
}

/// A bounded administrator-selected budget for the authenticated image proxy.
pub fn reencode_image_with_budget(
    bytes: &[u8],
    budget: Duration,
) -> Result<Vec<u8>, ImageDecodeFailure> {
    call_with_budget("mw_reencode_image", bytes, budget).map_err(|error| match error {
        JailFailure::Guest(_) | JailFailure::Rejected(_) => ImageDecodeFailure::InvalidImage,
        JailFailure::Host(_) => ImageDecodeFailure::JailUnavailable,
    })
}

/// Guest JSON remains untrusted; attachment::preview validates and sanitizes it.
pub(crate) fn preview_payload(
    format: crate::attachment::PreviewFormat,
    bytes: &[u8],
) -> Result<Vec<u8>, String> {
    call(
        match format {
            crate::attachment::PreviewFormat::Tnef => "mw_preview_tnef",
            crate::attachment::PreviewFormat::Eml => "mw_preview_eml",
        },
        bytes,
    )
    .map_err(|error| error.to_string())
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
    #[test]
    fn malformed_image_is_a_permanent_codec_rejection() {
        assert_eq!(
            super::reencode_image_classified(b"not an image"),
            Err(super::ImageDecodeFailure::InvalidImage)
        );
    }
    #[test]
    fn installed_app_icons_fit_the_image_jail() {
        for (name, bytes) in [
            (
                "192px",
                include_bytes!("../../../apps/web/public/icons/icon-192.png").as_slice(),
            ),
            (
                "512px",
                include_bytes!("../../../apps/web/public/icons/icon-512.png").as_slice(),
            ),
        ] {
            let image = super::reencode_image(bytes)
                .unwrap_or_else(|error| panic!("{name} installed app icon: {error}"));
            assert!(image.starts_with(b"\x89PNG"));
        }
    }
    use super::*;

    #[test]
    fn canonical_media_guest_has_no_host_imports() {
        assert_eq!(module().imports().count(), 0);
    }

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

    #[test]
    fn advertised_webp_decodes_inside_the_committed_jail() {
        // A one-pixel lossless WebP. CDNs may negotiate WebP even for a .png URL.
        use base64::Engine;
        let webp = base64::engine::general_purpose::STANDARD
            .decode("UklGRh4AAABXRUJQVlA4TBEAAAAvAAAAAAfQ//73v/+BiOh/AAA=")
            .unwrap();
        let png = reencode_image(&webp).expect("advertised WebP must be decodable");
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert_eq!(&png[16..24], &[0, 0, 0, 1, 0, 0, 0, 1]);
        assert!(reencode_image(&webp[..20]).is_err());
    }
}

/// Throughput probe for the image re-encode path (run by hand:
/// `cargo test -p mw-render --release --lib image_reencode_bench -- --ignored --nocapture`).
/// Not an assertion test: it prints cold/warm latency and parallel scaling.
#[cfg(test)]
mod bench {
    fn crc32(data: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for &b in data {
            crc ^= u32::from(b);
            for _ in 0..8 {
                crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
            }
        }
        !crc
    }
    fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        out.extend((data.len() as u32).to_be_bytes());
        let mut body = kind.to_vec();
        body.extend(data);
        out.extend(&body);
        out.extend(crc32(&body).to_be_bytes());
    }
    /// An RGB PNG with deterministic noise, deflated with stored blocks.
    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut raw = Vec::new();
        let mut seed = 0x1234_5678u32;
        for _ in 0..height {
            raw.push(0);
            for _ in 0..width * 3 {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                raw.push((seed >> 24) as u8 & 0xE0);
            }
        }
        let mut z = vec![0x78, 0x01];
        let mut chunks = raw.chunks(65_535).peekable();
        while let Some(c) = chunks.next() {
            z.push(u8::from(chunks.peek().is_none()));
            z.extend((c.len() as u16).to_le_bytes());
            z.extend((!(c.len() as u16)).to_le_bytes());
            z.extend(c);
        }
        let (mut a, mut b) = (1u32, 0u32);
        for &x in &raw {
            a = (a + u32::from(x)) % 65_521;
            b = (b + a) % 65_521;
        }
        z.extend(((b << 16) | a).to_be_bytes());
        let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut ihdr = Vec::new();
        ihdr.extend(width.to_be_bytes());
        ihdr.extend(height.to_be_bytes());
        ihdr.extend([8, 2, 0, 0, 0]);
        chunk(&mut out, b"IHDR", &ihdr);
        chunk(&mut out, b"IDAT", &z);
        chunk(&mut out, b"IEND", &[]);
        out
    }
    #[test]
    #[ignore = "timing probe, run by hand"]
    fn image_reencode_bench() {
        use std::time::Instant;
        let small = png(200, 120);
        let medium = png(640, 400);
        let t = Instant::now();
        super::reencode_image(&small).expect("small");
        println!("cold first call (module compile + 200x120): {:?}", t.elapsed());
        for (name, image) in [("200x120", &small), ("640x400", &medium)] {
            let t = Instant::now();
            let mut out = 0;
            for _ in 0..5 {
                out = super::reencode_image(image).expect("reencode").len();
            }
            println!("warm {name}: in {} B out {out} B, {:?} per image", image.len(), t.elapsed() / 5);
        }
        for threads in [1usize, 2, 4, 8] {
            let t = Instant::now();
            let handles: Vec<_> = (0..threads * 4)
                .map(|_| {
                    let image = medium.clone();
                    std::thread::spawn(move || super::reencode_image(&image).expect("reencode").len())
                })
                .collect();
            // Run `threads` at a time by joining sequentially in waves.
            for h in handles {
                h.join().unwrap();
            }
            println!("{} jobs of 640x400 with {threads}-wide waves: {:?}", threads * 4, t.elapsed());
        }
    }
}
