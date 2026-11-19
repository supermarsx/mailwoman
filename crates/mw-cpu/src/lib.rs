//! Safe runtime dispatch for small byte-oriented operations. Feature discovery
//! reports availability, not a claim that every feature is used by Mailwoman.
//! No global target-cpu flags or unsupported instructions reach portable callers.
#![deny(unsafe_op_in_unsafe_fn)]
use serde::Serialize;
use std::sync::LazyLock;

pub const AVX2_MIN_BYTES: usize = 256;
pub const AVX512_MIN_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Backend {
    Scalar,
    Avx2,
    Avx512,
}
impl Backend {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Scalar => "portable",
            Self::Avx2 => "avx2",
            Self::Avx512 => "avx512f+bw",
        }
    }
    pub fn is_supported(self) -> bool {
        match self {
            Self::Scalar => true,
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Self::Avx2 => std::is_x86_feature_detected!("avx2"),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Self::Avx512 => {
                std::is_x86_feature_detected!("avx512f")
                    && std::is_x86_feature_detected!("avx512bw")
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            Self::Avx2 | Self::Avx512 => false,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnsupportedBackend(pub Backend);
impl std::fmt::Display for UnsupportedBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CPU backend {} is unavailable on this device",
            self.0.label()
        )
    }
}
impl std::error::Error for UnsupportedBackend {}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuCapabilities {
    pub architecture: &'static str,
    pub available_features: Vec<&'static str>,
    /// Highest available implemented lowercase kernel. Short calls can use the
    /// portable implementation according to the documented dispatch thresholds.
    pub selected_backend: Backend,
    pub avx2_min_bytes: usize,
    pub avx512_min_bytes: usize,
}
pub fn capabilities() -> &'static CpuCapabilities {
    static CAPABILITIES: LazyLock<CpuCapabilities> = LazyLock::new(|| {
        let mut features = Vec::new();
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            macro_rules! feature {
                ($name:tt) => {
                    if std::is_x86_feature_detected!($name) {
                        features.push($name);
                    }
                };
            }
            feature!("sse2");
            feature!("ssse3");
            feature!("sse4.1");
            feature!("sse4.2");
            feature!("avx");
            feature!("avx2");
            feature!("avx512f");
            feature!("avx512bw");
            feature!("avx512vl");
            feature!("avx512dq");
            feature!("avx512cd");
            feature!("aes");
            feature!("pclmulqdq");
            feature!("sha");
            feature!("bmi1");
            feature!("bmi2");
        }
        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("neon") {
                features.push("neon");
            }
            if std::arch::is_aarch64_feature_detected!("aes") {
                features.push("aes");
            }
            if std::arch::is_aarch64_feature_detected!("sha2") {
                features.push("sha2");
            }
            if std::arch::is_aarch64_feature_detected!("crc") {
                features.push("crc");
            }
        }
        CpuCapabilities {
            architecture: std::env::consts::ARCH,
            available_features: features,
            selected_backend: if Backend::Avx512.is_supported() {
                Backend::Avx512
            } else if Backend::Avx2.is_supported() {
                Backend::Avx2
            } else {
                Backend::Scalar
            },
            avx2_min_bytes: AVX2_MIN_BYTES,
            avx512_min_bytes: AVX512_MIN_BYTES,
        }
    });
    &CAPABILITIES
}
#[inline]
pub fn selected_backend_for_len(len: usize) -> Backend {
    if len >= AVX512_MIN_BYTES && capabilities().selected_backend == Backend::Avx512 {
        Backend::Avx512
    } else if len >= AVX2_MIN_BYTES && Backend::Avx2.is_supported() {
        Backend::Avx2
    } else {
        Backend::Scalar
    }
}
#[inline]
pub fn make_ascii_lowercase(bytes: &mut [u8]) {
    // Keep tiny inputs independent of feature discovery and indirect dispatch.
    if bytes.len() < AVX2_MIN_BYTES {
        bytes.make_ascii_lowercase();
        return;
    }
    // selected_backend_for_len checks CPU+OS support before entering any kernel.
    let backend = selected_backend_for_len(bytes.len());
    dispatch_supported(bytes, backend);
}
/// Preserves UTF-8: only bytes 0x41..=0x5a change, to their ASCII lowercase form.
/// Non-ASCII leading and continuation bytes are never changed by any kernel.
#[inline]
pub fn lowercase_ascii_string(value: &mut String) {
    // SAFETY: all backends change only ASCII uppercase to ASCII lowercase, so
    // every UTF-8 sequence and the allocation/length remain valid, even on panic.
    unsafe {
        make_ascii_lowercase(value.as_bytes_mut());
    }
}
/// Useful for controlled benchmarking and parity checks. Unsupported kernels
/// return an error before inspecting or modifying any byte of the input.
pub fn make_ascii_lowercase_with_backend(
    bytes: &mut [u8],
    backend: Backend,
) -> Result<(), UnsupportedBackend> {
    if !backend.is_supported() {
        return Err(UnsupportedBackend(backend));
    }
    dispatch_supported(bytes, backend);
    Ok(())
}
#[inline]
fn dispatch_supported(bytes: &mut [u8], backend: Backend) {
    match backend {
        Backend::Scalar => bytes.make_ascii_lowercase(),
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        // SAFETY: private entry reached only after runtime feature detection.
        Backend::Avx2 => unsafe { x86::avx2(bytes) },
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        // SAFETY: private entry reached only after runtime feature detection.
        Backend::Avx512 => unsafe { x86::avx512(bytes) },
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        Backend::Avx2 | Backend::Avx512 => unreachable!("unsupported backend cannot be selected"),
    }
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86 {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn avx2(bytes: &mut [u8]) {
        let mut chunks = bytes.chunks_exact_mut(32);
        for chunk in &mut chunks {
            // SAFETY: chunks_exact_mut provides 32 initialized writable bytes.
            // Unaligned loads/stores require no additional alignment guarantee.
            unsafe {
                let value = _mm256_loadu_si256(chunk.as_ptr().cast());
                let upper = _mm256_and_si256(
                    _mm256_cmpgt_epi8(value, _mm256_set1_epi8(64)),
                    _mm256_cmpgt_epi8(_mm256_set1_epi8(91), value),
                );
                let result = _mm256_add_epi8(value, _mm256_and_si256(upper, _mm256_set1_epi8(32)));
                _mm256_storeu_si256(chunk.as_mut_ptr().cast(), result);
            }
        }
        chunks.into_remainder().make_ascii_lowercase();
    }
    #[target_feature(enable = "avx512f,avx512bw")]
    pub(super) unsafe fn avx512(bytes: &mut [u8]) {
        let mut chunks = bytes.chunks_exact_mut(64);
        for chunk in &mut chunks {
            // SAFETY: each chunk has 64 initialized writable bytes; unaligned
            // access is permitted, and the caller verified AVX-512F+BW support.
            unsafe {
                let value = _mm512_loadu_si512(chunk.as_ptr().cast());
                let upper = _mm512_cmp_epu8_mask::<5>(value, _mm512_set1_epi8(65))
                    & _mm512_cmp_epu8_mask::<2>(value, _mm512_set1_epi8(90));
                let result = _mm512_mask_add_epi8(value, upper, value, _mm512_set1_epi8(32));
                _mm512_storeu_si512(chunk.as_mut_ptr().cast(), result);
            }
        }
        chunks.into_remainder().make_ascii_lowercase();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_byte_unaligned_boundaries_and_tails_match_standard_library() {
        for backend in [Backend::Scalar, Backend::Avx2, Backend::Avx512] {
            if !backend.is_supported() {
                continue;
            }
            for len in 0..=512 {
                for offset in 0..=65 {
                    let mut actual: Vec<u8> =
                        (0..len + offset + 65).map(|n| (n % 256) as u8).collect();
                    let mut expected = actual.clone();
                    expected[offset..offset + len].make_ascii_lowercase();
                    make_ascii_lowercase_with_backend(&mut actual[offset..offset + len], backend)
                        .unwrap();
                    assert_eq!(
                        actual, expected,
                        "{backend:?}, length={len}, offset={offset}"
                    );
                }
            }
        }
        let mut every_byte: Vec<u8> = (0..=255).collect();
        let mut expected = every_byte.clone();
        expected.make_ascii_lowercase();
        make_ascii_lowercase(&mut every_byte);
        assert_eq!(every_byte, expected);
    }
    #[test]
    fn unicode_strings_keep_utf8_and_ascii_only_semantics() {
        for repeats in [0, 1, 2, 8, 32, 100] {
            let mut value = "ABC xyz ÄÖÜ Éß Ελληνικά 中文 📨 \0".repeat(repeats);
            let mut expected = value.clone();
            expected.make_ascii_lowercase();
            lowercase_ascii_string(&mut value);
            assert_eq!(value, expected);
        }
    }
    #[test]
    fn unsupported_forcing_refuses_before_modifying_input() {
        for backend in [Backend::Avx2, Backend::Avx512] {
            if backend.is_supported() {
                continue;
            }
            let mut bytes = b"UNCHANGED".to_vec();
            assert_eq!(
                make_ascii_lowercase_with_backend(&mut bytes, backend),
                Err(UnsupportedBackend(backend))
            );
            assert_eq!(bytes, b"UNCHANGED");
            assert!(make_ascii_lowercase_with_backend(&mut [], backend).is_err());
        }
    }
    #[test]
    fn size_thresholds_never_dispatch_an_unsupported_backend() {
        for size in 0..=1024 {
            let selected = selected_backend_for_len(size);
            assert!(selected.is_supported());
            if size < AVX2_MIN_BYTES {
                assert_eq!(selected, Backend::Scalar);
            }
            if size < AVX512_MIN_BYTES {
                assert_ne!(selected, Backend::Avx512);
            }
            if size >= AVX512_MIN_BYTES && Backend::Avx512.is_supported() {
                assert_eq!(selected, Backend::Avx512);
            } else if size >= AVX2_MIN_BYTES && Backend::Avx2.is_supported() {
                assert_eq!(selected, Backend::Avx2);
            }
        }
    }
    #[test]
    #[ignore = "manual release benchmark; run without concurrent builds"]
    fn lowercase_release_benchmark() {
        assert!(
            !cfg!(debug_assertions),
            "run cargo test -p mw-cpu --release lowercase_release_benchmark -- --ignored --nocapture"
        );
        use std::{hint::black_box, time::Instant};
        for length in [32, 64, 128, 256, 512, 1024, 2048, 4096, 8192] {
            let input: Vec<u8> = (0..length).map(|n| b'A' + (n % 50) as u8).collect();
            let backends: Vec<_> = [
                None,
                Some(Backend::Scalar),
                Some(Backend::Avx2),
                Some(Backend::Avx512),
            ]
            .into_iter()
            .filter(|backend| backend.is_none_or(Backend::is_supported))
            .collect();
            let mut samples: Vec<Vec<f64>> = vec![Vec::new(); backends.len()];
            let mut preparation = Vec::new();
            let rounds = (64 * 1024 * 1024 / length).clamp(1000, 200000);
            let mut expected = input.clone();
            expected.make_ascii_lowercase();
            // Warm every backend and prove parity before recording any timings.
            for backend in &backends {
                let mut bytes = input.clone();
                for _ in 0..32 {
                    bytes.copy_from_slice(&input);
                    match backend {
                        None => make_ascii_lowercase(&mut bytes),
                        Some(backend) => {
                            make_ascii_lowercase_with_backend(&mut bytes, *backend).unwrap()
                        }
                    }
                    assert_eq!(bytes, expected);
                }
            }
            for sample in 0..7 {
                // Rotate and alternate execution order to avoid always favouring
                // the last backend through frequency, cache warmth or drift.
                let mut order: Vec<_> = (0..backends.len()).collect();
                order.rotate_left(sample % backends.len());
                if sample % 2 != 0 {
                    order.reverse();
                }
                for index in order {
                    let mut bytes = input.clone();
                    let start = Instant::now();
                    for _ in 0..rounds {
                        bytes.copy_from_slice(black_box(&input));
                        match backends[index] {
                            None => make_ascii_lowercase(black_box(&mut bytes)),
                            Some(backend) => {
                                make_ascii_lowercase_with_backend(black_box(&mut bytes), backend)
                                    .unwrap()
                            }
                        }
                        black_box(&bytes);
                    }
                    samples[index].push(start.elapsed().as_nanos() as f64 / rounds as f64);
                    assert_eq!(bytes, expected);
                }
                let mut bytes = input.clone();
                let start = Instant::now();
                for _ in 0..rounds {
                    bytes.copy_from_slice(black_box(&input));
                    black_box(&bytes);
                }
                preparation.push(start.elapsed().as_nanos() as f64 / rounds as f64);
            }
            preparation.sort_by(f64::total_cmp);
            println!(
                "lowercase length={length} copy-only median={:.2} ns/op (same preparation included in every backend)",
                preparation[3]
            );
            for (backend, mut values) in backends.into_iter().zip(samples) {
                values.sort_by(f64::total_cmp);
                println!(
                    "lowercase length={length} backend={} median={:.2} ns/op samples=7 rounds={rounds}",
                    backend.map_or("auto", Backend::label),
                    values[3]
                );
            }
        }
    }
}
