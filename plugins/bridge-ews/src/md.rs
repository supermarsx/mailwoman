//! Minimal, pure-Rust MD4 / MD5 / HMAC-MD5 — the only primitives NTLMv2 needs
//! (MS-NLMP §3.3.2). Hand-rolled (plan §5/§6 R2 sanctions this) so the EWS bridge
//! pulls **zero** new dependencies and stays `wasm32-wasip2`-clean: no `-sys` C, no
//! `getrandom`, no RustCrypto version-alignment churn. Correctness is pinned by the
//! RFC 1320 (MD4) / RFC 1321 (MD5) / RFC 2202 (HMAC-MD5) known-answer vectors in the
//! tests below, plus the MS-NLMP §4.2.4 NTOWFv2 vector in [`crate::ntlm`].
//!
//! These are legacy digests used ONLY where the NTLM wire protocol mandates them;
//! nothing in Mailwoman uses MD4/MD5 for security decisions of its own.

// ── MD4 (RFC 1320) ─────────────────────────────────────────────────────────────

/// MD4 digest of `msg`.
#[must_use]
pub fn md4(msg: &[u8]) -> [u8; 16] {
    let mut a: u32 = 0x6745_2301;
    let mut b: u32 = 0xefcd_ab89;
    let mut c: u32 = 0x98ba_dcfe;
    let mut d: u32 = 0x1032_5476;

    for block in padded_blocks(msg) {
        let x = words_le(&block);
        let (aa, bb, cc, dd) = (a, b, c, d);

        // Round 1: F(x,y,z) = (x & y) | (!x & z)
        let f = |x: u32, y: u32, z: u32| (x & y) | (!x & z);
        let s1 = [3u32, 7, 11, 19];
        for i in 0..4 {
            let k = i * 4;
            a = a
                .wrapping_add(f(b, c, d))
                .wrapping_add(x[k])
                .rotate_left(s1[0]);
            d = d
                .wrapping_add(f(a, b, c))
                .wrapping_add(x[k + 1])
                .rotate_left(s1[1]);
            c = c
                .wrapping_add(f(d, a, b))
                .wrapping_add(x[k + 2])
                .rotate_left(s1[2]);
            b = b
                .wrapping_add(f(c, d, a))
                .wrapping_add(x[k + 3])
                .rotate_left(s1[3]);
        }

        // Round 2: G(x,y,z) = (x & y) | (x & z) | (y & z), constant 0x5a827999
        let g = |x: u32, y: u32, z: u32| (x & y) | (x & z) | (y & z);
        let s2 = [3u32, 5, 9, 13];
        for i in 0..4 {
            let k = i as usize;
            a = a
                .wrapping_add(g(b, c, d))
                .wrapping_add(x[k])
                .wrapping_add(0x5a82_7999)
                .rotate_left(s2[0]);
            d = d
                .wrapping_add(g(a, b, c))
                .wrapping_add(x[k + 4])
                .wrapping_add(0x5a82_7999)
                .rotate_left(s2[1]);
            c = c
                .wrapping_add(g(d, a, b))
                .wrapping_add(x[k + 8])
                .wrapping_add(0x5a82_7999)
                .rotate_left(s2[2]);
            b = b
                .wrapping_add(g(c, d, a))
                .wrapping_add(x[k + 12])
                .wrapping_add(0x5a82_7999)
                .rotate_left(s2[3]);
        }

        // Round 3: H(x,y,z) = x ^ y ^ z, constant 0x6ed9eba1
        let h = |x: u32, y: u32, z: u32| x ^ y ^ z;
        let s3 = [3u32, 9, 11, 15];
        let order = [0usize, 8, 4, 12, 2, 10, 6, 14, 1, 9, 5, 13, 3, 11, 7, 15];
        for i in 0..4 {
            let k = i * 4;
            a = a
                .wrapping_add(h(b, c, d))
                .wrapping_add(x[order[k]])
                .wrapping_add(0x6ed9_eba1)
                .rotate_left(s3[0]);
            d = d
                .wrapping_add(h(a, b, c))
                .wrapping_add(x[order[k + 1]])
                .wrapping_add(0x6ed9_eba1)
                .rotate_left(s3[1]);
            c = c
                .wrapping_add(h(d, a, b))
                .wrapping_add(x[order[k + 2]])
                .wrapping_add(0x6ed9_eba1)
                .rotate_left(s3[2]);
            b = b
                .wrapping_add(h(c, d, a))
                .wrapping_add(x[order[k + 3]])
                .wrapping_add(0x6ed9_eba1)
                .rotate_left(s3[3]);
        }

        a = a.wrapping_add(aa);
        b = b.wrapping_add(bb);
        c = c.wrapping_add(cc);
        d = d.wrapping_add(dd);
    }

    let mut out = [0u8; 16];
    out[0..4].copy_from_slice(&a.to_le_bytes());
    out[4..8].copy_from_slice(&b.to_le_bytes());
    out[8..12].copy_from_slice(&c.to_le_bytes());
    out[12..16].copy_from_slice(&d.to_le_bytes());
    out
}

// ── MD5 (RFC 1321) ─────────────────────────────────────────────────────────────

const MD5_S: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9,
    14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15,
    21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

const MD5_K: [u32; 64] = [
    0xd76a_a478,
    0xe8c7_b756,
    0x2420_70db,
    0xc1bd_ceee,
    0xf57c_0faf,
    0x4787_c62a,
    0xa830_4613,
    0xfd46_9501,
    0x6980_98d8,
    0x8b44_f7af,
    0xffff_5bb1,
    0x895c_d7be,
    0x6b90_1122,
    0xfd98_7193,
    0xa679_438e,
    0x49b4_0821,
    0xf61e_2562,
    0xc040_b340,
    0x265e_5a51,
    0xe9b6_c7aa,
    0xd62f_105d,
    0x0244_1453,
    0xd8a1_e681,
    0xe7d3_fbc8,
    0x21e1_cde6,
    0xc337_07d6,
    0xf4d5_0d87,
    0x455a_14ed,
    0xa9e3_e905,
    0xfcef_a3f8,
    0x676f_02d9,
    0x8d2a_4c8a,
    0xfffa_3942,
    0x8771_f681,
    0x6d9d_6122,
    0xfde5_380c,
    0xa4be_ea44,
    0x4bde_cfa9,
    0xf6bb_4b60,
    0xbebf_bc70,
    0x289b_7ec6,
    0xeaa1_27fa,
    0xd4ef_3085,
    0x0488_1d05,
    0xd9d4_d039,
    0xe6db_99e5,
    0x1fa2_7cf8,
    0xc4ac_5665,
    0xf429_2244,
    0x432a_ff97,
    0xab94_23a7,
    0xfc93_a039,
    0x655b_59c3,
    0x8f0c_cc92,
    0xffef_f47d,
    0x8584_5dd1,
    0x6fa8_7e4f,
    0xfe2c_e6e0,
    0xa301_4314,
    0x4e08_11a1,
    0xf753_7e82,
    0xbd3a_f235,
    0x2ad7_d2bb,
    0xeb86_d391,
];

/// MD5 digest of `msg`.
#[must_use]
pub fn md5(msg: &[u8]) -> [u8; 16] {
    let mut a0: u32 = 0x6745_2301;
    let mut b0: u32 = 0xefcd_ab89;
    let mut c0: u32 = 0x98ba_dcfe;
    let mut d0: u32 = 0x1032_5476;

    for block in padded_blocks(msg) {
        let m = words_le(&block);
        let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);

        for i in 0..64 {
            let (f, g) = match i {
                0..=15 => ((b & c) | (!b & d), i),
                16..=31 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                32..=47 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let tmp = d;
            d = c;
            c = b;
            let sum = a
                .wrapping_add(f)
                .wrapping_add(MD5_K[i])
                .wrapping_add(m[g])
                .rotate_left(MD5_S[i]);
            b = b.wrapping_add(sum);
            a = tmp;
        }

        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut out = [0u8; 16];
    out[0..4].copy_from_slice(&a0.to_le_bytes());
    out[4..8].copy_from_slice(&b0.to_le_bytes());
    out[8..12].copy_from_slice(&c0.to_le_bytes());
    out[12..16].copy_from_slice(&d0.to_le_bytes());
    out
}

/// HMAC-MD5 (RFC 2104) of `data` under `key`.
#[must_use]
pub fn hmac_md5(key: &[u8], data: &[u8]) -> [u8; 16] {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        k[..16].copy_from_slice(&md5(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let mut inner = Vec::with_capacity(64 + data.len());
    inner.extend_from_slice(&ipad);
    inner.extend_from_slice(data);
    let inner_digest = md5(&inner);
    let mut outer = Vec::with_capacity(80);
    outer.extend_from_slice(&opad);
    outer.extend_from_slice(&inner_digest);
    md5(&outer)
}

// ── shared padding / word helpers (identical MD4/MD5 Merkle–Damgård framing) ────

/// The classic length-padded 512-bit block stream (append `0x80`, zero-fill to
/// 56 mod 64, then the 64-bit little-endian bit length).
fn padded_blocks(msg: &[u8]) -> Vec<[u8; 64]> {
    let bit_len = (msg.len() as u64).wrapping_mul(8);
    let mut data = msg.to_vec();
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_le_bytes());
    data.chunks_exact(64)
        .map(|c| {
            let mut b = [0u8; 64];
            b.copy_from_slice(c);
            b
        })
        .collect()
}

fn words_le(block: &[u8; 64]) -> [u32; 16] {
    let mut w = [0u32; 16];
    for (i, wi) in w.iter_mut().enumerate() {
        let j = i * 4;
        *wi = u32::from_le_bytes([block[j], block[j + 1], block[j + 2], block[j + 3]]);
    }
    w
}

/// Encode `s` as UTF-16LE bytes (NTLM wire encoding of names/passwords).
#[must_use]
pub fn utf16le(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() * 2);
    for u in s.encode_utf16() {
        out.extend_from_slice(&u.to_le_bytes());
    }
    out
}

/// Lowercase hex encoding (test/debug helper).
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn md4_rfc1320_vectors() {
        assert_eq!(hex(&md4(b"")), "31d6cfe0d16ae931b73c59d7e0c089c0");
        assert_eq!(hex(&md4(b"a")), "bde52cb31de33e46245e05fbdbd6fb24");
        assert_eq!(hex(&md4(b"abc")), "a448017aaf21d8525fc10ae87aa6729d");
        assert_eq!(
            hex(&md4(b"message digest")),
            "d9130a8164549fe818874806e1c7014b"
        );
    }

    #[test]
    fn md5_rfc1321_vectors() {
        assert_eq!(hex(&md5(b"")), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(hex(&md5(b"a")), "0cc175b9c0f1b6a831c399e269772661");
        assert_eq!(hex(&md5(b"abc")), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(
            hex(&md5(b"message digest")),
            "f96b697d7cb7938d525a2f31aaf161d0"
        );
        assert_eq!(
            hex(&md5(b"abcdefghijklmnopqrstuvwxyz")),
            "c3fcd3d76192e4007dfb496cca67e13b"
        );
    }

    #[test]
    fn hmac_md5_rfc2202_vectors() {
        // Case 1: key = 0x0b*16, data = "Hi There".
        assert_eq!(
            hex(&hmac_md5(&[0x0b; 16], b"Hi There")),
            "9294727a3638bb1c13f48ef8158bfc9d"
        );
        // Case 2: key = "Jefe", data = "what do ya want for nothing?".
        assert_eq!(
            hex(&hmac_md5(b"Jefe", b"what do ya want for nothing?")),
            "750c783e6ab0b503eaa86e310a5db738"
        );
    }

    #[test]
    fn utf16le_encodes_ascii_as_lo_hi() {
        assert_eq!(utf16le("AB"), vec![0x41, 0x00, 0x42, 0x00]);
    }
}
