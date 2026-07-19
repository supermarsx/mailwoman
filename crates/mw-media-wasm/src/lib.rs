//! Second-layer media jail guest (SPEC §7.5, plan t16 S5 / DQ5).
//!
//! A pure `wasm32-unknown-unknown` **core module** with NO host imports: it can
//! parse a hostile CFB/MS-OXMSG (`.msg`/`.oft`) compound file and re-encode a
//! remote image, and it can do *nothing else* — no filesystem, no network, no
//! clock, no randomness. The render child (`mw-render`) runs this inside a
//! wasmtime jail (fuel-metered, memory-capped) so the OLE2/CFB and image codecs —
//! historically rich attack surface — never execute as native Rust.
//!
//! # ABI (raw linear memory, host-driven)
//! * `mw_alloc(len) -> ptr` — reserve `len` bytes; the host writes the input there.
//! * `mw_parse_cfb(ptr, len) -> u64` — parse the CFB at `[ptr, ptr+len)`.
//! * `mw_reencode_image(ptr, len) -> u64` — decode + re-encode the image there.
//!
//! Each entry returns a packed pointer/length: `(out_ptr << 32) | out_len`. The
//! output buffer begins with a status byte — `1` = ok, `0` = error — followed by
//! the payload:
//! * `mw_parse_cfb` ok: `[1][u32 subject_len LE][subject][u32 body_len LE][body]`
//! * `mw_reencode_image` ok: `[1][normalised PNG bytes]`
//! * either, error: `[0][utf-8 message]`
//!
//! Buffers are intentionally leaked: one call runs in one disposable wasmtime
//! store, so the entire linear memory is reclaimed when the host drops the store.

use std::io::Cursor;

/// Whole-container / whole-stream read ceiling — mirrors the render child's
/// `MAX_INPUT_BYTES`. A corrupt length field can never drive an unbounded read.
const MAX_READ_BYTES: usize = 4 * 1024 * 1024;

/// Image-decode guards against decompression bombs (a tiny file that expands to a
/// gigantic bitmap). The wasmtime store also caps total linear memory as a backstop.
const MAX_IMAGE_DIM: u32 = 16_384;
const MAX_IMAGE_ALLOC: u64 = 256 * 1024 * 1024;

// ── ABI plumbing ───────────────────────────────────────────────────────────────

/// Reserve `len` bytes of guest linear memory and hand the host the pointer. The
/// buffer is leaked (see module note); it lives for the whole disposable instance.
#[unsafe(no_mangle)]
pub extern "C" fn mw_alloc(len: u32) -> u32 {
    let mut buf = vec![0u8; len as usize];
    let ptr = buf.as_mut_ptr() as u32;
    std::mem::forget(buf);
    ptr
}

/// Read the host-written input region `[ptr, ptr+len)`.
fn input(ptr: u32, len: u32) -> &'static [u8] {
    // SAFETY: `ptr`/`len` name a region the host reserved via `mw_alloc` and filled
    // with exactly `len` bytes; it lives for the whole (single-call) instance and
    // is only read here.
    unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) }
}

/// Leak `buf` and return its packed pointer/length. `len` fits in 32 bits (the
/// wasm32 address space); the store's memory cap keeps outputs well below that.
fn emit(buf: Vec<u8>) -> u64 {
    let mut buf = std::mem::ManuallyDrop::new(buf);
    let ptr = buf.as_mut_ptr() as u64;
    let len = buf.len() as u64;
    (ptr << 32) | len
}

fn ok_frame(payload: Vec<u8>) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 1);
    out.push(1);
    out.extend_from_slice(&payload);
    out
}

fn err_frame(msg: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(msg.len() + 1);
    out.push(0);
    out.extend_from_slice(msg.as_bytes());
    out
}

/// Append a `[u32 len LE][bytes]` field.
fn put_field(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}

// ── Entry points ───────────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn mw_parse_cfb(ptr: u32, len: u32) -> u64 {
    let out = match parse_cfb(input(ptr, len)) {
        Ok(payload) => ok_frame(payload),
        Err(e) => err_frame(&e),
    };
    emit(out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mw_reencode_image(ptr: u32, len: u32) -> u64 {
    let out = match reencode_image(input(ptr, len)) {
        Ok(png) => ok_frame(png),
        Err(e) => err_frame(&e),
    };
    emit(out)
}

// ── CFB / MS-OXMSG parse (hostile) ──────────────────────────────────────────────

/// Parse the untrusted `.msg`/`.oft` compound file and return the framed
/// `subject` + `body` the render child needs. Ports the essential top-level
/// property reads from `mw-export::msg::read_msg`; the render child sanitises the
/// returned body. Never panics on arbitrary bytes: malformed streams read as empty.
fn parse_cfb(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.len() > MAX_READ_BYTES {
        return Err("cfb exceeds size limit".into());
    }
    let mut comp = cfb::CompoundFile::open(Cursor::new(bytes.to_vec()))
        .map_err(|e| format!("not a CFB container: {e}"))?;

    // Enumerate stream paths first (the walk borrows immutably), normalised to
    // `/`-separated with a leading slash, then read only the ones we need.
    let paths: Vec<String> = comp
        .walk()
        .filter(|e| e.is_stream())
        .map(|e| {
            let p = e.path().to_string_lossy().replace('\\', "/");
            if p.starts_with('/') {
                p
            } else {
                format!("/{p}")
            }
        })
        .collect();

    // MS-OXMSG top-level Unicode property streams: subject 0x0037, body 0x1000.
    let subject = read_unicode(&mut comp, &paths, "__substg1.0_0037001F").unwrap_or_default();
    let body = read_unicode(&mut comp, &paths, "__substg1.0_1000001F").unwrap_or_default();

    let mut payload = Vec::new();
    put_field(&mut payload, subject.as_bytes());
    put_field(&mut payload, body.as_bytes());
    Ok(payload)
}

/// Read the root-level stream whose base name is `base` as a NUL-trimmed UTF-16LE
/// string. Best-effort: any inconsistency yields `None`.
fn read_unicode(
    comp: &mut cfb::CompoundFile<Cursor<Vec<u8>>>,
    paths: &[String],
    base: &str,
) -> Option<String> {
    let target = format!("/{base}");
    let path = paths.iter().find(|p| p.as_str() == target)?;
    let bytes = read_stream_bytes(comp, path)?;
    let mut units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    while units.last() == Some(&0) {
        units.pop();
    }
    Some(String::from_utf16_lossy(&units))
}

fn read_stream_bytes(comp: &mut cfb::CompoundFile<Cursor<Vec<u8>>>, path: &str) -> Option<Vec<u8>> {
    use std::io::Read;
    let mut stream = comp.open_stream(path).ok()?;
    let mut buf = Vec::new();
    // Cap per-stream reads so a corrupt length can't drive an unbounded allocation.
    Read::by_ref(&mut stream)
        .take(MAX_READ_BYTES as u64)
        .read_to_end(&mut buf)
        .ok()?;
    Some(buf)
}

// ── Image re-encode (hostile) ───────────────────────────────────────────────────

/// Decode an untrusted image and re-encode it to PNG. Re-encoding keeps only the
/// pixels, so every ancillary chunk (EXIF/GPS/ICC/comments) is dropped and the
/// output format is normalised. Decode limits guard against decompression bombs.
fn reencode_image(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIM);
    limits.max_image_height = Some(MAX_IMAGE_DIM);
    limits.max_alloc = Some(MAX_IMAGE_ALLOC);

    let mut reader = image::ImageReader::new(Cursor::new(bytes));
    reader.limits(limits);
    let reader = reader
        .with_guessed_format()
        .map_err(|e| format!("format guess failed: {e}"))?;
    let img = reader.decode().map_err(|e| format!("decode failed: {e}"))?;

    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png)
        .map_err(|e| format!("re-encode failed: {e}"))?;
    Ok(out.into_inner())
}
