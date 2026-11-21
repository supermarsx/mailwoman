//! Public static encodings only. Codec work runs inside the existing bounded
//! static-loader workers, never on a cache hit or on mail/API/SSE responses.
use axum::{
    body::Bytes,
    http::{HeaderMap, HeaderValue, header},
};
use std::{
    borrow::Cow,
    io::{self, Read, Write},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Coding {
    Identity,
    Gzip,
    Brotli,
}
impl Coding {
    pub(crate) fn name(self) -> Option<&'static str> {
        match self {
            Self::Identity => None,
            Self::Gzip => Some("gzip"),
            Self::Brotli => Some("br"),
        }
    }
    pub(crate) fn suffix(self) -> &'static str {
        match self {
            Self::Identity => "",
            Self::Gzip => ".gz",
            Self::Brotli => ".br",
        }
    }
}
#[derive(Clone)]
pub(crate) struct Representation {
    pub bytes: Bytes,
    pub etag: HeaderValue,
    pub charge: usize,
    pub generated: bool,
}
impl Representation {
    fn new(value: Cow<'static, [u8]>, generated: bool) -> Self {
        let charge = if matches!(&value, Cow::Owned(_)) {
            value.len()
        } else {
            0
        };
        let bytes = match value {
            Cow::Borrowed(v) => Bytes::from_static(v),
            Cow::Owned(v) => Bytes::from(v),
        };
        let etag =
            HeaderValue::from_str(&crate::asset_etag(&bytes)).expect("content hash is a header");
        Self {
            bytes,
            etag,
            charge,
            generated,
        }
    }
}
#[derive(Clone, Default)]
pub(crate) struct Encodings {
    pub gzip: Option<Representation>,
    pub brotli: Option<Representation>,
}
impl Encodings {
    pub(crate) fn charge(&self) -> usize {
        self.gzip.as_ref().map_or(0, |v| v.charge) + self.brotli.as_ref().map_or(0, |v| v.charge)
    }
    pub(crate) fn bytes(&self) -> usize {
        self.gzip.as_ref().map_or(0, |v| v.bytes.len())
            + self.brotli.as_ref().map_or(0, |v| v.bytes.len())
    }
    pub(crate) fn get(&self, coding: Coding) -> Option<&Representation> {
        match coding {
            Coding::Gzip => self.gzip.as_ref(),
            Coding::Brotli => self.brotli.as_ref(),
            Coding::Identity => None,
        }
    }
}
pub(crate) fn eligible(path: &str) -> bool {
    path.rsplit_once('.').is_some_and(|(_, ext)| {
        matches!(ext, "js" | "mjs" | "css" | "ftl" | "json" | "svg" | "wasm")
    })
}
fn qvalue(text: &str) -> Option<u16> {
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    if !matches!(whole, "0" | "1")
        || fraction.len() > 3
        || !fraction.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    if whole == "1" {
        return fraction.bytes().all(|b| b == b'0').then_some(1000);
    }
    let value = if fraction.is_empty() {
        0
    } else {
        fraction.parse::<u16>().ok()?
    };
    Some(value * 10u16.pow((3 - fraction.len()) as u32))
}
/// No arbitrary labels, allocation or permissive substring matching. Absent or
/// empty Accept-Encoding requests identity. q=0 is never overridden by '*'.
pub(crate) fn negotiate(headers: &HeaderMap, available: &Encodings) -> Result<Coding, ()> {
    let mut values = [None; 4]; // identity, gzip, br, wildcard
    let mut bytes = 0usize;
    let mut tokens = 0usize;
    let mut present = false;
    for header in headers.get_all(header::ACCEPT_ENCODING) {
        present = true;
        let text = header.to_str().map_err(|_| ())?;
        bytes = bytes.checked_add(text.len()).ok_or(())?;
        if bytes > 4096 {
            return Err(());
        }
        for token in text.split(',').map(str::trim).filter(|t| !t.is_empty()) {
            tokens += 1;
            if tokens > 16 {
                return Err(());
            }
            let mut parts = token.split(';');
            let name = parts.next().ok_or(())?.trim();
            if name.is_empty()
                || !name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
            {
                return Err(());
            }
            let mut q = 1000;
            if let Some(parameter) = parts.next() {
                let (key, value) = parameter.trim().split_once('=').ok_or(())?;
                if !key.trim().eq_ignore_ascii_case("q") {
                    return Err(());
                }
                q = qvalue(value.trim()).ok_or(())?;
            }
            if parts.next().is_some() {
                return Err(());
            }
            let index = if name.eq_ignore_ascii_case("identity") {
                Some(0)
            } else if name.eq_ignore_ascii_case("gzip") || name.eq_ignore_ascii_case("x-gzip") {
                Some(1)
            } else if name.eq_ignore_ascii_case("br") {
                Some(2)
            } else if name == "*" {
                Some(3)
            } else {
                None
            };
            if let Some(i) = index {
                if values[i].replace(q).is_some() {
                    return Err(());
                }
            }
        }
    }
    if !present || tokens == 0 {
        return Ok(Coding::Identity);
    }
    // An omitted identity token remains an acceptable fallback, rather than
    // defeating an explicitly requested coding's positive qvalue. An explicit
    // identity weight still competes normally with the requested codings.
    let identity_allowed = values[0].map_or(values[3] != Some(0), |q| q > 0);
    let identity = values[0].unwrap_or(0);
    let gzip = if available.gzip.is_some() {
        values[1].or(values[3]).unwrap_or(0)
    } else {
        0
    };
    let br = if available.brotli.is_some() {
        values[2].or(values[3]).unwrap_or(0)
    } else {
        0
    };
    if br > 0 && br >= gzip && br >= identity {
        Ok(Coding::Brotli)
    } else if gzip > 0 && gzip >= identity {
        Ok(Coding::Gzip)
    } else if identity_allowed {
        Ok(Coding::Identity)
    } else {
        Err(())
    }
}
fn read_matches(mut reader: impl Read, raw: &[u8]) -> bool {
    let mut at = 0;
    let mut buffer = [0u8; 8192];
    loop {
        let Ok(n) = reader.read(&mut buffer) else {
            return false;
        };
        if n == 0 {
            return at == raw.len();
        }
        if n > raw.len().saturating_sub(at) || raw[at..at + n] != buffer[..n] {
            return false;
        }
        at += n;
    }
}
/// Check both actual decoded content and complete encoded consumption. A raw
/// filename/mtime or a stale unsigned manifest is never a sidecar proof.
fn sidecar_matches(coding: Coding, encoded: &[u8], raw: &[u8]) -> bool {
    match coding {
        Coding::Identity => false,
        Coding::Gzip => read_matches(flate2::read::MultiGzDecoder::new(encoded), raw),
        Coding::Brotli => {
            let mut state = brotli::BrotliState::new(
                brotli::HeapAlloc::<u8>::default(),
                brotli::HeapAlloc::<u32>::default(),
                brotli::HeapAlloc::<brotli::HuffmanCode>::default(),
            );
            let mut input_left = encoded.len();
            let mut input_at = 0;
            let mut total = 0;
            let mut at = 0;
            let mut buffer = [0u8; 8192];
            // Full encoded input is present; only bounded output-buffer retries
            // are needed. Default Brotli large-window extension stays disabled.
            for _ in 0..raw.len() / buffer.len() + 3 {
                let mut output_left = buffer.len();
                let mut output_at = 0;
                let result = brotli::BrotliDecompressStream(
                    &mut input_left,
                    &mut input_at,
                    encoded,
                    &mut output_left,
                    &mut output_at,
                    &mut buffer,
                    &mut total,
                    &mut state,
                );
                if output_at > raw.len().saturating_sub(at)
                    || raw[at..at + output_at] != buffer[..output_at]
                {
                    return false;
                }
                at += output_at;
                match result {
                    brotli::BrotliResult::ResultSuccess => {
                        return at == raw.len() && input_at == encoded.len();
                    }
                    brotli::BrotliResult::NeedsMoreOutput => {}
                    _ => return false,
                }
            }
            false
        }
    }
}
struct LimitedOutput {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for LimitedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(io::Error::other("static encoding output bound"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn generate(coding: Coding, raw: &[u8], limit: usize) -> Option<Vec<u8>> {
    let mut output = LimitedOutput {
        bytes: Vec::new(),
        limit,
    };
    match coding {
        Coding::Gzip => {
            let mut codec = flate2::GzBuilder::new()
                .mtime(0)
                .write(output, flate2::Compression::new(6));
            codec.write_all(raw).ok()?;
            output = codec.finish().ok()?;
        }
        Coding::Brotli => {
            let params = brotli::enc::BrotliEncoderParams {
                quality: 5,
                lgwin: 22,
                ..Default::default()
            };
            brotli::BrotliCompress(&mut io::Cursor::new(raw), &mut output, &params).ok()?;
        }
        Coding::Identity => return None,
    }
    Some(output.bytes)
}
/// Called only by a bounded cache-MISS blocking job. Generation is also needed
/// for prefix-adapted CSS, where the build's sidecar correctly fails raw proof.
pub(crate) fn prepare(
    raw: &[u8],
    brotli_enabled: bool,
    mut budget: usize,
    read: impl Fn(Coding) -> Option<Cow<'static, [u8]>>,
) -> Encodings {
    let mut result = Encodings::default();
    let worthwhile = raw.len().saturating_mul(95) / 100;
    let limit = worthwhile.min(raw.len().saturating_sub(128));
    for coding in [Coding::Brotli, Coding::Gzip] {
        if coding == Coding::Brotli && !brotli_enabled {
            continue;
        }
        let candidate = read(coding)
            .filter(|bytes| bytes.len() <= limit && sidecar_matches(coding, bytes, raw))
            .map(|bytes| (bytes, false));
        let candidate = candidate.or_else(|| {
            (budget > 0)
                .then(|| generate(coding, raw, limit.min(budget)))
                .flatten()
                .map(|bytes| (Cow::Owned(bytes), true))
        });
        if let Some((bytes, generated)) = candidate {
            let repr = Representation::new(bytes, generated);
            if repr.charge > budget {
                continue;
            }
            budget -= repr.charge;
            match coding {
                Coding::Brotli => result.brotli = Some(repr),
                Coding::Gzip => result.gzip = Some(repr),
                Coding::Identity => {}
            }
        }
    }
    result
}

#[cfg(test)]
#[path = "public_asset_compression_tests.rs"]
mod tests;
