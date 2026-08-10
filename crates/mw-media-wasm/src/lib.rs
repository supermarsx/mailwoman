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

// ── tests ──────────────────────────────────────────────────────────────────────
//
// ⚠️ These do NOT run in `cargo test --workspace`. This crate carries its own
// `[workspace]` table (see `Cargo.toml`), so it is excluded from the parent
// workspace on purpose — the parent must never try to build a
// `wasm32-unknown-unknown` cdylib. Run them explicitly:
//
//     cargo test --manifest-path crates/mw-media-wasm/Cargo.toml
//
// `parse_cfb` and `reencode_image` are the whole jail payload and are pure over
// `cfb` + `image`, both of which build for the host, so they can be exercised
// natively. The ABI plumbing (`mw_alloc` / `input` / `emit`) is wasm32-only —
// it packs pointers into `u32`/`u64` — and is covered by `mw-render`'s
// `media_jail` tests driving the committed `media.wasm` through wasmtime.
//
// What these pin is the property the jail exists for: **arbitrary bytes must
// produce an error, never a panic and never an unbounded read.** A panic here is
// a wasm trap rather than a native crash, but a trap on well-formed-but-hostile
// input is still a denial of service on the render child.
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// A UTF-16LE encoding of `s` with the trailing NUL MS-OXMSG writes.
    fn utf16le_nul(s: &str) -> Vec<u8> {
        let mut out: Vec<u8> = s.encode_utf16().flat_map(u16::to_le_bytes).collect();
        out.extend_from_slice(&[0, 0]);
        out
    }

    /// Build a CFB container holding the given `(stream path, bytes)` entries.
    fn cfb_with(streams: &[(&str, Vec<u8>)]) -> Vec<u8> {
        let mut comp = cfb::CompoundFile::create(Cursor::new(Vec::new())).unwrap();
        for (path, bytes) in streams {
            let mut s = comp.create_stream(path).unwrap();
            s.write_all(bytes).unwrap();
            s.flush().unwrap();
        }
        comp.flush().unwrap();
        comp.into_inner().into_inner()
    }

    /// Read back the `[u32 len LE][bytes]` fields `parse_cfb` emits.
    fn fields(payload: &[u8]) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        let mut i = 0usize;
        while i + 4 <= payload.len() {
            let len = u32::from_le_bytes(payload[i..i + 4].try_into().unwrap()) as usize;
            i += 4;
            out.push(payload[i..i + len].to_vec());
            i += len;
        }
        out
    }

    fn png_bytes(w: u32, h: u32) -> Vec<u8> {
        let img = image::DynamicImage::ImageRgb8(image::RgbImage::new(w, h));
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        buf.into_inner()
    }

    // ── framing ────────────────────────────────────────────────────────────────

    /// The status byte is what the host branches on, so `1` and `0` must never be
    /// confusable: an ok frame with an EMPTY payload is still one byte long.
    #[test]
    fn frames_carry_their_status_byte_even_when_empty() {
        assert_eq!(ok_frame(vec![]), vec![1]);
        assert_eq!(err_frame(""), vec![0]);
        assert_eq!(ok_frame(vec![9, 8]), vec![1, 9, 8]);
        assert_eq!(err_frame("no"), vec![0, b'n', b'o']);
    }

    #[test]
    fn put_field_prefixes_a_little_endian_length() {
        let mut out = Vec::new();
        put_field(&mut out, b"abc");
        put_field(&mut out, b"");
        assert_eq!(out, vec![3, 0, 0, 0, b'a', b'b', b'c', 0, 0, 0, 0]);
        assert_eq!(fields(&out), vec![b"abc".to_vec(), Vec::new()]);
    }

    // ── CFB parse ──────────────────────────────────────────────────────────────

    #[test]
    fn parses_subject_and_body_from_an_oxmsg_container() {
        let bytes = cfb_with(&[
            ("/__substg1.0_0037001F", utf16le_nul("Quarterly report")),
            (
                "/__substg1.0_1000001F",
                utf16le_nul("Body — with an em dash"),
            ),
        ]);
        let got = fields(&parse_cfb(&bytes).unwrap());

        assert_eq!(got.len(), 2);
        assert_eq!(
            String::from_utf8(got[0].clone()).unwrap(),
            "Quarterly report"
        );
        assert_eq!(
            String::from_utf8(got[1].clone()).unwrap(),
            "Body — with an em dash",
            "the trailing NUL is trimmed and non-BMP-safe UTF-16 survives"
        );
    }

    /// A CFB that is a valid container but carries none of the MS-OXMSG property
    /// streams yields two EMPTY fields, not an error. A `.msg` with an unexpected
    /// layout should render blank, not fail the whole import.
    #[test]
    fn a_container_without_the_expected_streams_yields_empty_fields() {
        let bytes = cfb_with(&[("/Unrelated", b"hello".to_vec())]);
        assert_eq!(
            fields(&parse_cfb(&bytes).unwrap()),
            vec![Vec::new(), Vec::new()]
        );
    }

    /// Only the ROOT-level property streams are read. A stream with the right base
    /// name nested inside a storage is not the top-level property and is ignored.
    #[test]
    fn nested_streams_are_not_mistaken_for_top_level_properties() {
        let mut comp = cfb::CompoundFile::create(Cursor::new(Vec::new())).unwrap();
        comp.create_storage("/attach").unwrap();
        let mut s = comp.create_stream("/attach/__substg1.0_0037001F").unwrap();
        s.write_all(&utf16le_nul("nested subject")).unwrap();
        s.flush().unwrap();
        comp.flush().unwrap();
        let bytes = comp.into_inner().into_inner();

        assert_eq!(
            fields(&parse_cfb(&bytes).unwrap()),
            vec![Vec::new(), Vec::new()]
        );
    }

    /// An odd-length property stream cannot be whole UTF-16 units. The trailing
    /// byte is dropped rather than read past the end of the buffer.
    #[test]
    fn an_odd_length_property_stream_drops_its_trailing_byte() {
        let mut subject = utf16le_nul("hi");
        subject.push(0x41); // a stray byte, no pair
        let bytes = cfb_with(&[("/__substg1.0_0037001F", subject)]);
        assert_eq!(
            String::from_utf8(fields(&parse_cfb(&bytes).unwrap())[0].clone()).unwrap(),
            "hi"
        );
    }

    /// Unpaired surrogates decode lossily to U+FFFD instead of panicking — the
    /// property streams are attacker-controlled and are not validated UTF-16.
    #[test]
    fn unpaired_surrogates_decode_lossily() {
        // A lone high surrogate (0xD800) followed by 'A', then the NUL terminator.
        let subject = vec![0x00, 0xD8, b'A', 0x00, 0x00, 0x00];
        let bytes = cfb_with(&[("/__substg1.0_0037001F", subject)]);
        let got = String::from_utf8(fields(&parse_cfb(&bytes).unwrap())[0].clone()).unwrap();
        assert!(got.contains('\u{FFFD}'), "got {got:?}");
        assert!(got.ends_with('A'), "got {got:?}");
    }

    /// Arbitrary bytes are an error, never a panic. These are the shapes a hostile
    /// `.msg` attachment actually arrives as.
    #[test]
    fn arbitrary_bytes_are_rejected_without_panicking() {
        let magic = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
        let mut truncated = magic.to_vec();
        truncated.extend_from_slice(&[0u8; 32]);

        let cases: Vec<(&str, Vec<u8>)> = vec![
            ("empty", Vec::new()),
            ("plain text", b"not a compound file at all".to_vec()),
            ("CFB magic then nothing", magic.to_vec()),
            ("CFB magic then garbage", truncated),
            ("all zeroes", vec![0u8; 4096]),
            ("all 0xff", vec![0xffu8; 4096]),
        ];
        for (why, bytes) in cases {
            let err = parse_cfb(&bytes).unwrap_err();
            assert!(
                err.contains("not a CFB container"),
                "{why}: unexpected error {err}"
            );
        }
    }

    /// A container over the read ceiling is refused BEFORE it is parsed, so a
    /// declared-huge attachment cannot drive an unbounded allocation in the guest.
    #[test]
    fn oversized_input_is_refused_before_parsing() {
        let too_big = vec![0u8; MAX_READ_BYTES + 1];
        assert_eq!(
            parse_cfb(&too_big).unwrap_err(),
            "cfb exceeds size limit",
            "the ceiling is checked before any parse work"
        );
        // Exactly at the ceiling it is parsed (and then rejected as not a CFB),
        // so the boundary is inclusive rather than off by one.
        assert!(parse_cfb(&vec![0u8; MAX_READ_BYTES])
            .unwrap_err()
            .contains("not a CFB container"));
    }

    // ── image re-encode ────────────────────────────────────────────────────────

    /// Re-encoding normalises the format: whatever went in, a PNG comes out, with
    /// the pixel dimensions preserved.
    #[test]
    fn reencode_normalises_to_png_and_preserves_dimensions() {
        let out = reencode_image(&png_bytes(8, 5)).unwrap();
        assert_eq!(
            &out[..8],
            &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a],
            "output must carry the PNG signature"
        );

        let decoded = image::load_from_memory(&out).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (8, 5));
    }

    /// Ancillary chunks are dropped. A PNG carrying a text comment goes in; the
    /// re-encoded output does not contain it. This is the whole point of the
    /// re-encode — EXIF/GPS/ICC/comment metadata must not reach the reader.
    #[test]
    fn reencode_strips_ancillary_metadata() {
        let mut with_comment = png_bytes(4, 4);
        // Splice a `tEXt` chunk (length, type, data, CRC) before the IEND chunk.
        let iend = with_comment.len() - 12;
        let data = b"CommentSECRET-GPS-TAG";
        let mut chunk = Vec::new();
        chunk.extend_from_slice(&(data.len() as u32).to_be_bytes());
        chunk.extend_from_slice(b"tEXt");
        chunk.extend_from_slice(data);
        chunk.extend_from_slice(&[0, 0, 0, 0]); // CRC — decoders tolerate ancillary
        with_comment.splice(iend..iend, chunk);

        assert!(
            with_comment
                .windows(data.len())
                .any(|w| w == data.as_slice()),
            "fixture must actually carry the comment"
        );
        let out = reencode_image(&with_comment).unwrap();
        assert!(
            !out.windows(data.len()).any(|w| w == data.as_slice()),
            "the re-encode must not carry metadata through"
        );
    }

    /// A decompression bomb — a tiny file declaring a huge canvas — is refused by
    /// the dimension limit rather than allocating the bitmap.
    #[test]
    fn oversized_dimensions_are_refused() {
        let bomb = png_bytes(MAX_IMAGE_DIM + 1, 1);
        assert!(
            bomb.len() < 100_000,
            "a {}px-wide blank PNG should stay small, got {} bytes",
            MAX_IMAGE_DIM + 1,
            bomb.len()
        );
        let err = reencode_image(&bomb).unwrap_err();
        assert!(err.starts_with("decode failed"), "got {err}");

        // A hair under the limit still decodes, so the guard is a ceiling and not
        // a blanket refusal of large images.
        assert!(reencode_image(&png_bytes(MAX_IMAGE_DIM, 1)).is_ok());
    }

    /// Bytes that are not an image at all are an error, never a panic.
    #[test]
    fn non_images_are_rejected_without_panicking() {
        for (why, bytes) in [
            ("empty", Vec::new()),
            ("plain text", b"<svg><script/></svg>".to_vec()),
            ("truncated PNG", png_bytes(4, 4)[..20].to_vec()),
            (
                "PNG signature only",
                vec![0x89, b'P', b'N', b'G', 13, 10, 26, 10],
            ),
        ] {
            assert!(reencode_image(&bytes).is_err(), "{why} must be refused");
        }
    }
}
