//! Safe runtime dispatch for small byte-oriented operations. Feature discovery
//! reports availability, not a claim that every feature is used by Mailwoman.
//! No global target-cpu flags or unsupported instructions reach portable callers.
#![deny(unsafe_op_in_unsafe_fn)]
use serde::Serialize;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KernelUsage {
    pub calls: u64,
    pub bytes: u64,
}

/// Observed lowercase work in this process since startup, not third-party crypto
/// or SQLite instruction usage. SIMD byte totals exclude portable tails.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuUsage {
    pub scalar: KernelUsage,
    pub sse2: KernelUsage,
    pub sse3: KernelUsage,
    pub ssse3: KernelUsage,
    pub sse41: KernelUsage,
    pub sse42: KernelUsage,
    pub avx2: KernelUsage,
    pub avx512: KernelUsage,
}

#[repr(align(64))]
struct UsageCounter {
    calls: AtomicU64,
    bytes: AtomicU64,
}
impl UsageCounter {
    const fn new() -> Self {
        Self {
            calls: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
        }
    }
    fn record(&self, bytes: usize) {
        if bytes != 0 {
            self.bytes.fetch_add(bytes as u64, Ordering::Relaxed);
            self.calls.fetch_add(1, Ordering::Relaxed);
        }
    }
    fn snapshot(&self) -> KernelUsage {
        KernelUsage {
            calls: self.calls.load(Ordering::Relaxed),
            bytes: self.bytes.load(Ordering::Relaxed),
        }
    }
}
static SCALAR_USAGE: UsageCounter = UsageCounter::new();
static SSE2_USAGE: UsageCounter = UsageCounter::new();
static SSE3_USAGE: UsageCounter = UsageCounter::new();
static SSSE3_USAGE: UsageCounter = UsageCounter::new();
static SSE41_USAGE: UsageCounter = UsageCounter::new();
static SSE42_USAGE: UsageCounter = UsageCounter::new();
static AVX2_USAGE: UsageCounter = UsageCounter::new();
static AVX512_USAGE: UsageCounter = UsageCounter::new();

/// Lock-free approximate counters; concurrent calls may advance during sampling.
pub fn usage() -> CpuUsage {
    CpuUsage {
        scalar: SCALAR_USAGE.snapshot(),
        sse2: SSE2_USAGE.snapshot(),
        sse3: SSE3_USAGE.snapshot(),
        ssse3: SSSE3_USAGE.snapshot(),
        sse41: SSE41_USAGE.snapshot(),
        sse42: SSE42_USAGE.snapshot(),
        avx2: AVX2_USAGE.snapshot(),
        avx512: AVX512_USAGE.snapshot(),
    }
}

fn record_usage(backend: Backend, len: usize) {
    let (counter, width) = match backend {
        Backend::Scalar => {
            SCALAR_USAGE.record(len);
            return;
        }
        Backend::Sse2 => (&SSE2_USAGE, 16),
        Backend::Sse3 => (&SSE3_USAGE, 16),
        Backend::Ssse3 => (&SSSE3_USAGE, 16),
        Backend::Sse41 => (&SSE41_USAGE, 16),
        Backend::Sse42 => (&SSE42_USAGE, 16),
        Backend::Avx2 => (&AVX2_USAGE, 32),
        Backend::Avx512 => (&AVX512_USAGE, 64),
    };
    counter.record(len / width * width);
    SCALAR_USAGE.record(len % width);
}

pub const AVX2_MIN_BYTES: usize = 256;
pub const AVX512_MIN_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Backend {
    /// Portable Rust implementation; the compiler may itself vectorize it.
    Scalar,
    Sse2,
    Sse3,
    Ssse3,
    Sse41,
    Sse42,
    Avx2,
    Avx512,
}
impl Backend {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Scalar => "portable",
            Self::Sse2 => "sse2",
            Self::Sse3 => "sse3",
            Self::Ssse3 => "ssse3",
            Self::Sse41 => "sse4.1",
            Self::Sse42 => "sse4.2",
            Self::Avx2 => "avx2",
            Self::Avx512 => "avx512f+bw",
        }
    }
    pub fn is_supported(self) -> bool {
        match self {
            Self::Scalar => true,
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Self::Sse2 => std::is_x86_feature_detected!("sse2"),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Self::Sse3 => std::is_x86_feature_detected!("sse3"),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Self::Ssse3 => std::is_x86_feature_detected!("ssse3"),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Self::Sse41 => std::is_x86_feature_detected!("sse4.1"),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Self::Sse42 => std::is_x86_feature_detected!("sse4.2"),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Self::Avx2 => std::is_x86_feature_detected!("avx2"),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Self::Avx512 => {
                std::is_x86_feature_detected!("avx512f")
                    && std::is_x86_feature_detected!("avx512bw")
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            Self::Sse2
            | Self::Sse3
            | Self::Ssse3
            | Self::Sse41
            | Self::Sse42
            | Self::Avx2
            | Self::Avx512 => false,
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
    /// Preferred kernel for large inputs; higher ISA availability alone does
    /// not establish a faster implementation. Short calls use size thresholds.
    pub selected_backend: Backend,
    pub highest_available_backend: Backend,
    /// No automatic SSE cutoff is qualified. Forced SSE kernels remain
    /// available independently of automatic dispatch policy.
    pub sse_min_bytes: Option<usize>,
    pub avx2_min_bytes: usize,
    pub avx512_min_bytes: usize,
}
fn preferred_large_backend(highest: Backend) -> Backend {
    match highest {
        Backend::Avx512 | Backend::Avx2 => highest,
        _ => Backend::Scalar,
    }
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
            feature!("sse3");
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
        let highest = [
            Backend::Avx512,
            Backend::Avx2,
            Backend::Sse42,
            Backend::Sse41,
            Backend::Ssse3,
            Backend::Sse3,
            Backend::Sse2,
        ]
        .into_iter()
        .find(|backend| backend.is_supported())
        .unwrap_or(Backend::Scalar);
        CpuCapabilities {
            architecture: std::env::consts::ARCH,
            available_features: features,
            // No measured consistent SSE gain over compiler-vectorized Rust.
            selected_backend: preferred_large_backend(highest),
            highest_available_backend: highest,
            sse_min_bytes: None,
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
    if bytes.len() < AVX2_MIN_BYTES.min(AVX512_MIN_BYTES) {
        bytes.make_ascii_lowercase();
        record_usage(Backend::Scalar, bytes.len());
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
        // SAFETY: each private kernel is reached only after its runtime check.
        Backend::Sse2 => unsafe { x86::sse2(bytes) },
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        Backend::Sse3 => unsafe { x86::sse3(bytes) },
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        Backend::Ssse3 => unsafe { x86::ssse3(bytes) },
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        Backend::Sse41 => unsafe { x86::sse41(bytes) },
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        Backend::Sse42 => unsafe { x86::sse42(bytes) },
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        // SAFETY: private entry reached only after runtime feature detection.
        Backend::Avx2 => unsafe { x86::avx2(bytes) },
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        // SAFETY: private entry reached only after runtime feature detection.
        Backend::Avx512 => unsafe { x86::avx512(bytes) },
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        Backend::Sse2
        | Backend::Sse3
        | Backend::Ssse3
        | Backend::Sse41
        | Backend::Sse42
        | Backend::Avx2
        | Backend::Avx512 => unreachable!("unsupported backend cannot be selected"),
    }
    record_usage(backend, bytes.len());
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86 {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;
    #[target_feature(enable = "sse2")]
    pub(super) unsafe fn sse2(bytes: &mut [u8]) {
        let mut chunks = bytes.chunks_exact_mut(16);
        for chunk in &mut chunks {
            // SAFETY: exactly sixteen initialized writable bytes, no alignment
            // requirement. Signed comparison excludes every non-ASCII byte.
            unsafe {
                let value = _mm_loadu_si128(chunk.as_ptr().cast());
                let upper = _mm_and_si128(
                    _mm_cmpgt_epi8(value, _mm_set1_epi8(64)),
                    _mm_cmpgt_epi8(_mm_set1_epi8(91), value),
                );
                let result = _mm_add_epi8(value, _mm_and_si128(upper, _mm_set1_epi8(32)));
                _mm_storeu_si128(chunk.as_mut_ptr().cast(), result);
            }
        }
        chunks.into_remainder().make_ascii_lowercase();
    }
    #[target_feature(enable = "sse3")]
    pub(super) unsafe fn sse3(bytes: &mut [u8]) {
        let mut chunks = bytes.chunks_exact_mut(16);
        for chunk in &mut chunks {
            // SAFETY: LDDQU consumes the full sixteen-byte chunk; all stores
            // remain within it. Classification/tails have SSE2 semantics.
            unsafe {
                let value = _mm_lddqu_si128(chunk.as_ptr().cast());
                let upper = _mm_and_si128(
                    _mm_cmpgt_epi8(value, _mm_set1_epi8(64)),
                    _mm_cmpgt_epi8(_mm_set1_epi8(91), value),
                );
                let result = _mm_add_epi8(value, _mm_and_si128(upper, _mm_set1_epi8(32)));
                _mm_storeu_si128(chunk.as_mut_ptr().cast(), result);
            }
        }
        chunks.into_remainder().make_ascii_lowercase();
    }
    #[target_feature(enable = "ssse3")]
    pub(super) unsafe fn ssse3(bytes: &mut [u8]) {
        let mut chunks = bytes.chunks_exact_mut(16);
        for chunk in &mut chunks {
            // SAFETY: only in-bounds sixteen-byte loads/stores. PSHUFB uses
            // bounded nibble indices; non-ASCII high nibbles map to zero.
            unsafe {
                let value = _mm_loadu_si128(chunk.as_ptr().cast());
                let nibble = _mm_set1_epi8(15);
                let low = _mm_and_si128(value, nibble);
                let high = _mm_and_si128(_mm_srli_epi16::<4>(value), nibble);
                // High nibble4 admits low1..15; high5 admits low0..10.
                let lows = _mm_setr_epi8(2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 1, 1, 1, 1, 1);
                let highs = _mm_setr_epi8(0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
                let tags =
                    _mm_and_si128(_mm_shuffle_epi8(lows, low), _mm_shuffle_epi8(highs, high));
                let upper = _mm_cmpgt_epi8(tags, _mm_setzero_si128());
                _mm_storeu_si128(
                    chunk.as_mut_ptr().cast(),
                    _mm_add_epi8(value, _mm_and_si128(upper, _mm_set1_epi8(32))),
                );
            }
        }
        chunks.into_remainder().make_ascii_lowercase();
    }
    #[target_feature(enable = "sse4.1")]
    pub(super) unsafe fn sse41(bytes: &mut [u8]) {
        let mut chunks = bytes.chunks_exact_mut(16);
        for chunk in &mut chunks {
            // SAFETY: exact sixteen-byte chunk, runtime SSE4.1 gate. PTEST
            // avoids stores when the chunk has no uppercase ASCII; the
            // optional blend changes only bytes whose ASCII mask is FF.
            unsafe {
                let value = _mm_loadu_si128(chunk.as_ptr().cast());
                let upper = _mm_and_si128(
                    _mm_cmpgt_epi8(value, _mm_set1_epi8(64)),
                    _mm_cmpgt_epi8(_mm_set1_epi8(91), value),
                );
                if _mm_testz_si128(upper, upper) == 0 {
                    let result =
                        _mm_blendv_epi8(value, _mm_add_epi8(value, _mm_set1_epi8(32)), upper);
                    _mm_storeu_si128(chunk.as_mut_ptr().cast(), result);
                }
            }
        }
        chunks.into_remainder().make_ascii_lowercase();
    }
    #[target_feature(enable = "sse4.2")]
    pub(super) unsafe fn sse42(bytes: &mut [u8]) {
        let mut chunks = bytes.chunks_exact_mut(16);
        for chunk in &mut chunks {
            // SAFETY: explicit lengths2/16 make NUL an ordinary input byte,
            // not a terminator. Unsigned ranges exclude all non-ASCII values.
            // No implicit string read, masked overread or pointer past a chunk.
            unsafe {
                let value = _mm_loadu_si128(chunk.as_ptr().cast());
                let range = _mm_setr_epi8(65, 90, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
                let upper = _mm_cmpestrm::<
                    {
                        _SIDD_UBYTE_OPS
                            | _SIDD_CMP_RANGES
                            | _SIDD_POSITIVE_POLARITY
                            | _SIDD_UNIT_MASK
                    },
                >(range, 2, value, 16);
                let result = _mm_add_epi8(value, _mm_and_si128(upper, _mm_set1_epi8(32)));
                _mm_storeu_si128(chunk.as_mut_ptr().cast(), result);
            }
        }
        chunks.into_remainder().make_ascii_lowercase();
    }
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
    const BACKENDS: [Backend; 8] = [
        Backend::Scalar,
        Backend::Sse2,
        Backend::Sse3,
        Backend::Ssse3,
        Backend::Sse41,
        Backend::Sse42,
        Backend::Avx2,
        Backend::Avx512,
    ];
    #[test]
    fn empty_work_is_not_counted_and_local_counter_totals_are_exact() {
        let counter = UsageCounter::new();
        counter.record(0);
        assert_eq!(counter.snapshot().calls, 0);
        counter.record(17);
        counter.record(31);
        assert_eq!(counter.snapshot().calls, 2);
        assert_eq!(counter.snapshot().bytes, 48);
    }
    #[test]
    fn executed_kernels_and_portable_tails_advance_runtime_counters() {
        for backend in BACKENDS {
            if !backend.is_supported() {
                continue;
            }
            let before = usage();
            let mut bytes = vec![b'A'; 513];
            make_ascii_lowercase_with_backend(&mut bytes, backend).unwrap();
            let after = usage();
            assert!(bytes.iter().all(|byte| *byte == b'a'));
            let (first, last, byte_count) = match backend {
                Backend::Scalar => (&before.scalar, &after.scalar, 513),
                Backend::Sse2 => (&before.sse2, &after.sse2, 512),
                Backend::Sse3 => (&before.sse3, &after.sse3, 512),
                Backend::Ssse3 => (&before.ssse3, &after.ssse3, 512),
                Backend::Sse41 => (&before.sse41, &after.sse41, 512),
                Backend::Sse42 => (&before.sse42, &after.sse42, 512),
                Backend::Avx2 => (&before.avx2, &after.avx2, 512),
                Backend::Avx512 => (&before.avx512, &after.avx512, 512),
            };
            // Other tests may run concurrently; assert this call's minimum delta.
            assert!(last.calls >= first.calls + 1);
            assert!(last.bytes >= first.bytes + byte_count);
            if backend != Backend::Scalar {
                assert!(after.scalar.bytes >= before.scalar.bytes + 1);
            }
        }
        let before = usage();
        make_ascii_lowercase(&mut [b'A'; 17]);
        assert!(usage().scalar.bytes >= before.scalar.bytes + 17);
    }
    #[test]
    fn every_byte_unaligned_boundaries_and_tails_match_standard_library() {
        for backend in BACKENDS {
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
            for backend in BACKENDS
                .into_iter()
                .filter(|backend| backend.is_supported())
            {
                let mut bytes = "ABC xyz ÄÖÜ Éß Ελληνικά 中文 📨 \0"
                    .repeat(repeats)
                    .into_bytes();
                make_ascii_lowercase_with_backend(&mut bytes, backend).unwrap();
                assert_eq!(String::from_utf8(bytes).unwrap(), expected, "{backend:?}");
            }
        }
    }
    #[test]
    fn embedded_nul_never_terminates_an_explicit_length_chunk() {
        for backend in BACKENDS
            .into_iter()
            .filter(|backend| backend.is_supported())
        {
            for nul in 0..16 {
                let mut bytes = [b'X'; 16];
                bytes[nul] = 0;
                let mut expected = bytes;
                expected.make_ascii_lowercase();
                make_ascii_lowercase_with_backend(&mut bytes, backend).unwrap();
                assert_eq!(bytes, expected, "{backend:?}, nul={nul}");
            }
        }
    }
    #[test]
    fn unsupported_forcing_refuses_before_modifying_input() {
        for backend in BACKENDS {
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
            if size < AVX2_MIN_BYTES.min(AVX512_MIN_BYTES) {
                assert_eq!(selected, Backend::Scalar);
            }
            if size < AVX512_MIN_BYTES {
                assert_ne!(selected, Backend::Avx512);
            }
            if size >= AVX512_MIN_BYTES && Backend::Avx512.is_supported() {
                assert_eq!(selected, Backend::Avx512);
            } else if size >= AVX2_MIN_BYTES && Backend::Avx2.is_supported() {
                assert_eq!(selected, Backend::Avx2);
            } else {
                assert_eq!(selected, Backend::Scalar);
            }
        }
        for highest in [
            Backend::Scalar,
            Backend::Sse2,
            Backend::Sse3,
            Backend::Ssse3,
            Backend::Sse41,
            Backend::Sse42,
        ] {
            assert_eq!(
                preferred_large_backend(highest),
                Backend::Scalar,
                "SSE availability alone must not replace portable dispatch"
            );
        }
        assert_eq!(preferred_large_backend(Backend::Avx2), Backend::Avx2);
        assert_eq!(preferred_large_backend(Backend::Avx512), Backend::Avx512);
        assert!(capabilities().sse_min_bytes.is_none());
        assert!(capabilities().highest_available_backend.is_supported());
        assert!(capabilities().selected_backend.is_supported());
        if Backend::Sse3.is_supported() {
            assert!(capabilities().available_features.contains(&"sse3"));
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
        println!(
            "Observed API cost includes runtime gating, usage counters and input copy; not an HTTP/RPS or application speed claim."
        );
        println!(
            "cpu architecture={} highestAvailable={} selectedLarge={} sseMinBytes={:?} avx2MinBytes={} avx512MinBytes={} features={:?}",
            capabilities().architecture,
            capabilities().highest_available_backend.label(),
            capabilities().selected_backend.label(),
            capabilities().sse_min_bytes,
            AVX2_MIN_BYTES,
            AVX512_MIN_BYTES,
            capabilities().available_features
        );
        for corpus in ["mixed-ascii", "already-lowercase", "mixed-high-bytes"] {
            for length in [
                8, 15, 16, 17, 31, 32, 64, 128, 255, 256, 512, 1024, 2048, 4096, 8192,
            ] {
                let input: Vec<u8> = (0..length)
                    .map(|n| match corpus {
                        "already-lowercase" => b'a' + (n % 26) as u8,
                        "mixed-high-bytes" => match n % 4 {
                            0 => 0,
                            1 => 0xc3,
                            2 => 0x89,
                            _ => b'A' + (n % 26) as u8,
                        },
                        _ => b'A' + (n % 50) as u8,
                    })
                    .collect();
                let backends: Vec<_> = [
                    None,
                    Some(Backend::Scalar),
                    Some(Backend::Sse2),
                    Some(Backend::Sse3),
                    Some(Backend::Ssse3),
                    Some(Backend::Sse41),
                    Some(Backend::Sse42),
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
                                Some(backend) => make_ascii_lowercase_with_backend(
                                    black_box(&mut bytes),
                                    backend,
                                )
                                .unwrap(),
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
                    "lowercase corpus={corpus} length={length} copy-only median={:.2} ns/op (same preparation included in every backend)",
                    preparation[3]
                );
                for (backend, mut values) in backends.into_iter().zip(samples) {
                    let observed = values.clone();
                    values.sort_by(f64::total_cmp);
                    println!(
                        "lowercase corpus={corpus} length={length} backend={} median={:.2} ns/op samples=7 rounds={rounds} samplesNsPerOp={observed:?}",
                        backend.map_or("auto", Backend::label),
                        values[3]
                    );
                }
            }
        }
    }
}
