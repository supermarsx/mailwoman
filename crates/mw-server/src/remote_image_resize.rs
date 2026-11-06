//! Optional downscale of oversized remote images, after the media jail.
//!
//! The only decoder of hostile bytes stays the wasm jail. It hands back a PNG that
//! its own encoder wrote; this module reads that normalised PNG, scales it down to
//! the width the reader's frame asked for, and writes a smaller PNG. Nothing here
//! parses a codec the jail did not already vouch for.
//!
//! * Off unless `privacy.imageResizeEnabled`. The width is a hint from the frame
//!   (`w=`), accepted only when it is one of [`STEPS`], and capped by
//!   `privacy.imageResizeMaxWidth`. The cache key carries the width, so a resized
//!   copy never answers a request for the full image.
//! * Bomb guard: the pixel count is read from the PNG header before any decode and
//!   an image above `privacy.imageResizeMaxPixels` is passed through unscaled.
//! * Animated GIF, APNG and animated WebP are passed through unscaled (the jail
//!   already reduces them to one frame); SVG never reaches here (refused earlier).
//! * CMYK JPEGs are converted to RGB by the jail's decoder. EXIF orientation is
//!   read from the original JPEG (a bounded scan of its first segments) and applied
//!   to the resized copy.
//! * A result that is not smaller than its input is discarded.
use std::io::Cursor;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

/// The widths a frame may ask for, in CSS pixels.
pub(crate) const STEPS: [u32; 5] = [320, 640, 960, 1280, 1600];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    /// Pick the best available encoder. Today that is PNG, the only one in the
    /// build (there is no lossy WebP encoder to choose).
    Auto,
    Png,
}

impl Format {
    pub(crate) fn parse(s: &str) -> Self {
        if s == "png" { Self::Png } else { Self::Auto }
    }
    pub(crate) fn tag(self) -> &'static str {
        // Both resolve to PNG output; the tag keeps the cache key honest if that changes.
        match self {
            Self::Auto | Self::Png => "png",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ResizeConfig {
    pub enabled: bool,
    pub max_width: u32,
    pub max_pixels: u64,
    pub format: Format,
}

impl Default for ResizeConfig {
    fn default() -> Self {
        Self { enabled: false, max_width: 1600, max_pixels: 4_000_000, format: Format::Auto }
    }
}

impl ResizeConfig {
    /// The width to scale to for a request hint, or `None` for the full image: the
    /// feature is off, the hint is absent, or the hint is not one of [`STEPS`].
    pub(crate) fn target(&self, hint: Option<u32>) -> Option<u32> {
        if !self.enabled {
            return None;
        }
        let w = hint.filter(|w| STEPS.contains(w))?;
        Some(w.min(self.max_width.max(STEPS[0])))
    }
}

/// What the `w=` query value means: a number on one of the steps.
pub(crate) fn parse_hint(query: Option<&str>) -> Option<u32> {
    let w: u32 = query?
        .split('&')
        .find_map(|p| p.strip_prefix("w="))?
        .parse()
        .ok()?;
    STEPS.contains(&w).then_some(w)
}

/// The cache-key suffix of a resized copy.
pub(crate) fn key_suffix(width: u32, cfg: &ResizeConfig) -> String {
    format!("\u{1}w{width}.{}", cfg.format.tag())
}

// ── metrics ────────────────────────────────────────────────────────────────────

static RESIZED: AtomicU64 = AtomicU64::new(0);
static BYTES_SAVED: AtomicU64 = AtomicU64::new(0);
static RESIZE_MS: AtomicU64 = AtomicU64::new(0);
static PASSED_THROUGH: AtomicU64 = AtomicU64::new(0);

/// `(resized, bytes saved, milliseconds spent resizing, passed through)`.
pub(crate) fn counters() -> (u64, u64, u64, u64) {
    (
        RESIZED.load(Relaxed),
        BYTES_SAVED.load(Relaxed),
        RESIZE_MS.load(Relaxed),
        PASSED_THROUGH.load(Relaxed),
    )
}

// ── what to leave alone ────────────────────────────────────────────────────────

/// Whether `raw` is an animated GIF, APNG or animated WebP. Bounded: it looks at the
/// container structure only and never decodes pixels.
pub(crate) fn is_animated(raw: &[u8]) -> bool {
    if raw.starts_with(b"GIF8") {
        // Count image descriptors (0x2C) by walking the blocks; two frames animate.
        let Some(flags) = raw.get(10) else { return false };
        let mut i = 13usize;
        if flags & 0x80 != 0 {
            i += 3 * (1usize << ((flags & 7) + 1));
        }
        let mut frames = 0;
        while let Some(&b) = raw.get(i) {
            match b {
                0x21 => {
                    i += 2;
                    while let Some(&n) = raw.get(i) {
                        i += 1 + n as usize;
                        if n == 0 {
                            break;
                        }
                    }
                }
                0x2C => {
                    frames += 1;
                    if frames > 1 {
                        return true;
                    }
                    let Some(f) = raw.get(i + 9) else { return false };
                    i += 10;
                    if f & 0x80 != 0 {
                        i += 3 * (1usize << ((f & 7) + 1));
                    }
                    i += 1;
                    while let Some(&n) = raw.get(i) {
                        i += 1 + n as usize;
                        if n == 0 {
                            break;
                        }
                    }
                }
                _ => return false,
            }
        }
        return false;
    }
    if raw.starts_with(b"\x89PNG\r\n\x1a\n") {
        // `acTL` must precede the first IDAT.
        let mut i = 8usize;
        while let Some(head) = raw.get(i..i + 8) {
            let len = u32::from_be_bytes([head[0], head[1], head[2], head[3]]) as usize;
            match &head[4..8] {
                b"acTL" => return true,
                b"IDAT" | b"IEND" => return false,
                _ => {}
            }
            i = i.saturating_add(12).saturating_add(len);
        }
        return false;
    }
    if raw.len() > 16 && &raw[..4] == b"RIFF" && &raw[8..12] == b"WEBP" {
        // The first chunk of an extended WebP is VP8X; bit 1 (0x02) is "animation".
        return &raw[12..16] == b"VP8X" && raw.get(20).is_some_and(|f| f & 0x02 != 0);
    }
    false
}

/// The EXIF orientation (1..=8) of a JPEG, 1 when absent or unreadable. Scans at most
/// the first 64 KiB of segments.
pub(crate) fn exif_orientation(raw: &[u8]) -> u8 {
    if !raw.starts_with(&[0xFF, 0xD8]) {
        return 1;
    }
    let raw = &raw[..raw.len().min(64 * 1024)];
    let mut i = 2usize;
    while i + 4 <= raw.len() && raw[i] == 0xFF {
        let marker = raw[i + 1];
        if marker == 0xDA || marker == 0xD9 {
            break;
        }
        let len = u16::from_be_bytes([raw[i + 2], raw[i + 3]]) as usize;
        if len < 2 {
            break;
        }
        if marker == 0xE1
            && let Some(seg) = raw.get(i + 4..i + 2 + len)
            && seg.starts_with(b"Exif\0\0")
        {
            return tiff_orientation(&seg[6..]).unwrap_or(1);
        }
        i += 2 + len;
    }
    1
}

fn tiff_orientation(t: &[u8]) -> Option<u8> {
    let le = match t.get(..2)? {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let u16at = |o: usize| {
        let b = t.get(o..o + 2)?;
        Some(if le { u16::from_le_bytes([b[0], b[1]]) } else { u16::from_be_bytes([b[0], b[1]]) })
    };
    let u32at = |o: usize| {
        let b = t.get(o..o + 4)?;
        Some(if le {
            u32::from_le_bytes([b[0], b[1], b[2], b[3]])
        } else {
            u32::from_be_bytes([b[0], b[1], b[2], b[3]])
        })
    };
    let ifd = u32at(4)? as usize;
    let n = u16at(ifd)? as usize;
    for k in 0..n.min(64) {
        let e = ifd + 2 + k * 12;
        if u16at(e)? == 0x0112 {
            let v = u16at(e + 8)?;
            return (1..=8).contains(&v).then_some(v as u8);
        }
    }
    None
}

// ── the resize ─────────────────────────────────────────────────────────────────

/// Why an image was left as the jail wrote it.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Skip {
    Animated,
    AlreadySmall,
    OverPixelCap,
    Undecodable,
    NotSmaller,
}

pub(crate) enum Outcome {
    Resized { png: Vec<u8>, from: (u32, u32), to: (u32, u32) },
    Skipped(Skip),
}

/// Scale the jail's `png` to `width` (never up). `raw` is the original download,
/// consulted only for animation and EXIF orientation. Blocking: run it on the decoder
/// pool.
pub(crate) fn resize(raw: &[u8], png: &[u8], width: u32, cfg: &ResizeConfig) -> Outcome {
    if is_animated(raw) {
        PASSED_THROUGH.fetch_add(1, Relaxed);
        return Outcome::Skipped(Skip::Animated);
    }
    let started = std::time::Instant::now();
    let outcome = resize_inner(raw, png, width, cfg);
    match &outcome {
        Outcome::Resized { png: out, .. } => {
            RESIZED.fetch_add(1, Relaxed);
            BYTES_SAVED.fetch_add(png.len().saturating_sub(out.len()) as u64, Relaxed);
            RESIZE_MS.fetch_add(started.elapsed().as_millis() as u64, Relaxed);
        }
        Outcome::Skipped(_) => {
            PASSED_THROUGH.fetch_add(1, Relaxed);
        }
    }
    outcome
}

fn resize_inner(raw: &[u8], png: &[u8], width: u32, cfg: &ResizeConfig) -> Outcome {
    // The dimensions come straight from the IHDR bytes, before a decoder exists: an
    // image over the cap never reaches an allocation.
    let Some((w, h)) = ihdr_size(png) else {
        return Outcome::Skipped(Skip::Undecodable);
    };
    if u64::from(w) * u64::from(h) > cfg.max_pixels {
        return Outcome::Skipped(Skip::OverPixelCap);
    }
    let mut decoder = png::Decoder::new(Cursor::new(png));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let Ok(mut reader) = decoder.read_info() else {
        return Outcome::Skipped(Skip::Undecodable);
    };
    if w <= width || w == 0 || h == 0 {
        return Outcome::Skipped(Skip::AlreadySmall);
    }
    let (color, _) = reader.output_color_type();
    let channels = match color {
        png::ColorType::Grayscale => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        png::ColorType::Indexed => return Outcome::Skipped(Skip::Undecodable),
    };
    let mut buf = vec![0u8; reader.output_buffer_size().unwrap_or(0)];
    if buf.is_empty() || reader.next_frame(&mut buf).is_err() {
        return Outcome::Skipped(Skip::Undecodable);
    }
    let new_h = ((u64::from(h) * u64::from(width)).div_ceil(u64::from(w))).max(1) as u32;
    let mut pixels = scale(&buf, (w, h), (width, new_h), channels);
    let (mut ow, mut oh) = (width, new_h);
    let orientation = exif_orientation(raw);
    if orientation != 1 {
        (pixels, ow, oh) = orient(&pixels, ow, oh, channels, orientation);
    }
    let Some(out) = encode(&pixels, ow, oh, color) else {
        return Outcome::Skipped(Skip::Undecodable);
    };
    if out.len() >= png.len() {
        return Outcome::Skipped(Skip::NotSmaller);
    }
    Outcome::Resized { png: out, from: (w, h), to: (ow, oh) }
}

/// Width and height from a PNG signature and IHDR, without decoding anything.
fn ihdr_size(png: &[u8]) -> Option<(u32, u32)> {
    if png.len() < 24 || !png.starts_with(b"\x89PNG\r\n\x1a\n") || &png[12..16] != b"IHDR" {
        return None;
    }
    Some((
        u32::from_be_bytes(png[16..20].try_into().ok()?),
        u32::from_be_bytes(png[20..24].try_into().ok()?),
    ))
}

/// Area-average downscale, separable, in premultiplied alpha when there is alpha.
fn scale(src: &[u8], (sw, sh): (u32, u32), (dw, dh): (u32, u32), c: usize) -> Vec<u8> {
    let (sw, sh, dw, dh) = (sw as usize, sh as usize, dw as usize, dh as usize);
    let alpha = c == 2 || c == 4;
    // Horizontal pass into f32 rows of `dw * c`.
    let mut mid = vec![0f32; dw * sh * c];
    let xs = sw as f32 / dw as f32;
    for y in 0..sh {
        let row = &src[y * sw * c..(y + 1) * sw * c];
        for x in 0..dw {
            let (a, b) = (x as f32 * xs, (x + 1) as f32 * xs);
            let mut acc = [0f32; 4];
            let mut wsum = 0f32;
            let mut i = a.floor() as usize;
            while (i as f32) < b && i < sw {
                let wgt = (b.min((i + 1) as f32) - a.max(i as f32)).max(0.0);
                let px = &row[i * c..i * c + c];
                if alpha {
                    let al = f32::from(px[c - 1]) / 255.0;
                    for k in 0..c - 1 {
                        acc[k] += f32::from(px[k]) * al * wgt;
                    }
                    acc[c - 1] += f32::from(px[c - 1]) * wgt;
                } else {
                    for k in 0..c {
                        acc[k] += f32::from(px[k]) * wgt;
                    }
                }
                wsum += wgt;
                i += 1;
            }
            let o = (y * dw + x) * c;
            for k in 0..c {
                mid[o + k] = acc[k] / wsum.max(f32::MIN_POSITIVE);
            }
        }
    }
    // Vertical pass.
    let mut out = vec![0u8; dw * dh * c];
    let ys = sh as f32 / dh as f32;
    for y in 0..dh {
        let (a, b) = (y as f32 * ys, (y + 1) as f32 * ys);
        for x in 0..dw {
            let mut acc = [0f32; 4];
            let mut wsum = 0f32;
            let mut j = a.floor() as usize;
            while (j as f32) < b && j < sh {
                let wgt = (b.min((j + 1) as f32) - a.max(j as f32)).max(0.0);
                let px = &mid[(j * dw + x) * c..(j * dw + x) * c + c];
                for k in 0..c {
                    acc[k] += px[k] * wgt;
                }
                wsum += wgt;
                j += 1;
            }
            let o = (y * dw + x) * c;
            let wsum = wsum.max(f32::MIN_POSITIVE);
            if alpha {
                let al = acc[c - 1] / wsum;
                for k in 0..c - 1 {
                    let v = if al > 0.0 { acc[k] / wsum / (al / 255.0) } else { 0.0 };
                    out[o + k] = v.round().clamp(0.0, 255.0) as u8;
                }
                out[o + c - 1] = al.round().clamp(0.0, 255.0) as u8;
            } else {
                for k in 0..c {
                    out[o + k] = (acc[k] / wsum).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
    out
}

/// Apply an EXIF orientation (2..=8), returning the pixels and the new size.
fn orient(src: &[u8], w: u32, h: u32, c: usize, o: u8) -> (Vec<u8>, u32, u32) {
    let (w, h) = (w as usize, h as usize);
    let swap = o >= 5;
    let (nw, nh) = if swap { (h, w) } else { (w, h) };
    let mut out = vec![0u8; src.len()];
    for y in 0..h {
        for x in 0..w {
            // The destination of source pixel (x, y) per the EXIF definition.
            let (dx, dy) = match o {
                2 => (w - 1 - x, y),
                3 => (w - 1 - x, h - 1 - y),
                4 => (x, h - 1 - y),
                5 => (y, x),
                6 => (h - 1 - y, x),
                7 => (h - 1 - y, w - 1 - x),
                8 => (y, w - 1 - x),
                _ => (x, y),
            };
            let s = (y * w + x) * c;
            let d = (dy * nw + dx) * c;
            out[d..d + c].copy_from_slice(&src[s..s + c]);
        }
    }
    (out, nw as u32, nh as u32)
}

fn encode(pixels: &[u8], w: u32, h: u32, color: png::ColorType) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, w, h);
    enc.set_color(color);
    enc.set_depth(png::BitDepth::Eight);
    enc.set_compression(png::Compression::Balanced);
    let mut writer = enc.write_header().ok()?;
    writer.write_image_data(pixels).ok()?;
    writer.finish().ok()?;
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> ResizeConfig {
        ResizeConfig { enabled: true, ..ResizeConfig::default() }
    }

    /// A smooth gradient: compresses, and downscales to something smaller.
    fn gradient(w: u32, h: u32) -> Vec<u8> {
        let mut px = Vec::new();
        for y in 0..h {
            for x in 0..w {
                px.extend_from_slice(&[(x * 255 / w) as u8, (y * 255 / h) as u8, 90, 255]);
            }
        }
        let mut out = Vec::new();
        let mut e = png::Encoder::new(&mut out, w, h);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        let mut wr = e.write_header().unwrap();
        wr.write_image_data(&px).unwrap();
        wr.finish().unwrap();
        out
    }

    fn size_of(png: &[u8]) -> (u32, u32) {
        let r = png::Decoder::new(Cursor::new(png)).read_info().unwrap();
        (r.info().width, r.info().height)
    }

    #[test]
    fn the_hint_is_accepted_only_on_a_step_and_only_when_enabled() {
        assert_eq!(cfg().target(Some(640)), Some(640));
        assert_eq!(cfg().target(Some(641)), None);
        assert_eq!(cfg().target(None), None);
        assert_eq!(ResizeConfig::default().target(Some(640)), None);
        let capped = ResizeConfig { max_width: 960, ..cfg() };
        assert_eq!(capped.target(Some(1600)), Some(960));
        assert_eq!(parse_hint(Some("url=x&w=960")), Some(960));
        assert_eq!(parse_hint(Some("w=7")), None);
        assert_eq!(parse_hint(Some("w=abc")), None);
    }

    #[test]
    fn a_wide_image_is_scaled_to_the_width_keeping_its_aspect_and_is_smaller() {
        let src = gradient(1200, 600);
        let Outcome::Resized { png, from, to } = resize(&src, &src, 320, &cfg()) else {
            panic!("expected a resize");
        };
        assert_eq!(from, (1200, 600));
        assert_eq!(to, (320, 160));
        assert_eq!(size_of(&png), (320, 160));
        assert!(png.len() < src.len());
    }

    #[test]
    fn an_image_already_within_the_width_is_left_alone() {
        let src = gradient(300, 100);
        assert!(matches!(resize(&src, &src, 320, &cfg()), Outcome::Skipped(Skip::AlreadySmall)));
    }

    #[test]
    fn the_pixel_cap_is_checked_from_the_header_before_decoding() {
        let src = gradient(1200, 600); // 720_000 px
        let tight = ResizeConfig { max_pixels: 100_000, ..cfg() };
        assert!(matches!(resize(&src, &src, 320, &tight), Outcome::Skipped(Skip::OverPixelCap)));
        // A header that claims a huge image over truncated data never allocates it:
        // the cap refuses on the dimensions alone.
        let mut bomb = gradient(8, 8);
        bomb[16..20].copy_from_slice(&60_000u32.to_be_bytes());
        bomb[20..24].copy_from_slice(&60_000u32.to_be_bytes());
        let crc = crc32(&bomb[12..29]);
        bomb[29..33].copy_from_slice(&crc.to_be_bytes());
        assert!(matches!(resize(&bomb, &bomb, 320, &cfg()), Outcome::Skipped(Skip::OverPixelCap)));
    }

    fn crc32(data: &[u8]) -> u32 {
        let mut c = 0xFFFF_FFFFu32;
        for &b in data {
            c ^= u32::from(b);
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
        }
        !c
    }

    #[test]
    fn animated_images_pass_through() {
        // GIF89a with two image descriptors.
        let mut gif = b"GIF89a".to_vec();
        gif.extend_from_slice(&[2, 0, 2, 0, 0x00, 0, 0]);
        for _ in 0..2 {
            gif.extend_from_slice(&[0x2C, 0, 0, 0, 0, 2, 0, 2, 0, 0, 2, 2, 0x44, 0x01, 0]);
        }
        gif.push(0x3B);
        assert!(is_animated(&gif));
        let one: Vec<u8> = {
            let mut g = gif.clone();
            g.truncate(g.len() - 1 - 15);
            g.push(0x3B);
            g
        };
        assert!(!is_animated(&one));
        // APNG: an acTL chunk before IDAT.
        let mut apng = b"\x89PNG\r\n\x1a\n".to_vec();
        apng.extend_from_slice(&[0, 0, 0, 13]);
        apng.extend_from_slice(b"IHDR");
        apng.extend_from_slice(&[0; 13]);
        apng.extend_from_slice(&[0; 4]);
        apng.extend_from_slice(&[0, 0, 0, 8]);
        apng.extend_from_slice(b"acTL");
        apng.extend_from_slice(&[0; 12]);
        assert!(is_animated(&apng));
        assert!(!is_animated(&gradient(4, 4)));
        // Animated WebP: VP8X with the animation flag.
        let mut webp = b"RIFF\0\0\0\0WEBPVP8X".to_vec();
        webp.extend_from_slice(&[10, 0, 0, 0, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert!(is_animated(&webp));
        let src = gradient(1200, 600);
        assert!(matches!(resize(&gif, &src, 320, &cfg()), Outcome::Skipped(Skip::Animated)));
    }

    #[test]
    fn exif_orientation_is_read_and_applied() {
        // JPEG SOI + APP1 Exif (little endian) with orientation 6.
        let mut jpeg = vec![0xFF, 0xD8, 0xFF, 0xE1, 0, 0, b'E', b'x', b'i', b'f', 0, 0];
        jpeg.extend_from_slice(b"II*\0");
        jpeg.extend_from_slice(&8u32.to_le_bytes());
        jpeg.extend_from_slice(&1u16.to_le_bytes());
        jpeg.extend_from_slice(&0x0112u16.to_le_bytes());
        jpeg.extend_from_slice(&3u16.to_le_bytes());
        jpeg.extend_from_slice(&1u32.to_le_bytes());
        jpeg.extend_from_slice(&6u16.to_le_bytes());
        jpeg.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
        let len = (jpeg.len() - 4) as u16;
        jpeg[4..6].copy_from_slice(&len.to_be_bytes());
        assert_eq!(exif_orientation(&jpeg), 6);
        assert_eq!(exif_orientation(b"not a jpeg"), 1);
        // A 1200x600 image with orientation 6 comes out portrait.
        let src = gradient(1200, 600);
        let Outcome::Resized { png, to, .. } = resize(&jpeg, &src, 320, &cfg()) else {
            panic!("expected a resize");
        };
        assert_eq!(to, (160, 320));
        assert_eq!(size_of(&png), (160, 320));
    }

    #[test]
    fn transparency_survives_the_scale() {
        let (w, h) = (800u32, 400u32);
        let mut px = Vec::new();
        for y in 0..h {
            for _x in 0..w {
                px.extend_from_slice(&[200, 30, 30, if y < h / 2 { 0 } else { 255 }]);
            }
        }
        let mut src = Vec::new();
        let mut e = png::Encoder::new(&mut src, w, h);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        let mut wr = e.write_header().unwrap();
        wr.write_image_data(&px).unwrap();
        wr.finish().unwrap();
        let Outcome::Resized { png, .. } = resize(&src, &src, 320, &cfg()) else {
            panic!("expected a resize");
        };
        let mut r = png::Decoder::new(Cursor::new(&png)).read_info().unwrap();
        let mut buf = vec![0; r.output_buffer_size().unwrap()];
        r.next_frame(&mut buf).unwrap();
        assert_eq!(buf[3], 0, "top row stays transparent");
        assert_eq!(buf[buf.len() - 1], 255, "bottom row stays opaque");
        assert_eq!(&buf[buf.len() - 4..buf.len() - 1], &[200, 30, 30], "colour is not darkened by the transparent half");
    }
}
