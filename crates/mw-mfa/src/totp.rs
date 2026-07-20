//! RFC 6238 TOTP over HMAC-SHA1 (the authenticator-app default), with a
//! configurable step, digit count, and validation window. Verification is
//! constant-time and scans the whole ±`skew` window without early-out.

use hmac::{Hmac, KeyInit, Mac};
use rand::RngCore;
use rand::rngs::OsRng;
use sha1::Sha1;

use crate::ct_eq;

type HmacSha1 = Hmac<Sha1>;

/// Bytes of entropy in a generated TOTP shared secret (160-bit, the SHA-1 block-
/// aligned RFC 6238 recommendation).
pub const SECRET_BYTES: usize = 20;

/// Generate a fresh random TOTP shared secret.
pub fn generate_secret() -> [u8; SECRET_BYTES] {
    let mut b = [0u8; SECRET_BYTES];
    OsRng.fill_bytes(&mut b);
    b
}

/// RFC 4648 base32 (uppercase, no padding) — the encoding authenticator apps and
/// `otpauth://` URIs use for the shared secret.
pub fn base32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut out = String::new();
    let mut buffer = 0u32;
    let mut bits = 0u32;
    for &byte in data {
        buffer = (buffer << 8) | u32::from(byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let idx = ((buffer >> bits) & 0x1f) as usize;
            out.push(ALPHABET[idx] as char);
        }
    }
    if bits > 0 {
        let idx = ((buffer << (5 - bits)) & 0x1f) as usize;
        out.push(ALPHABET[idx] as char);
    }
    out
}

/// Decode an RFC 4648 base32 string (case-insensitive, padding and spaces ignored).
/// Returns `None` on any non-alphabet character.
pub fn base32_decode(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u32;
    for c in s.chars() {
        if c == '=' || c.is_whitespace() {
            continue;
        }
        let v = match c.to_ascii_uppercase() {
            'A'..='Z' => (c.to_ascii_uppercase() as u8) - b'A',
            '2'..='7' => (c as u8) - b'2' + 26,
            _ => return None,
        };
        buffer = (buffer << 5) | u32::from(v);
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
        }
    }
    Some(out)
}

/// Build an `otpauth://totp/...` provisioning URI (the QR-code payload) for a
/// secret. `issuer` and `account` are percent-encoded into the label/params.
pub fn provisioning_uri(secret: &[u8], issuer: &str, account: &str, params: &TotpParams) -> String {
    let enc = |s: &str| {
        let mut out = String::new();
        for b in s.bytes() {
            if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
                out.push(b as char);
            } else {
                out.push_str(&format!("%{b:02X}"));
            }
        }
        out
    };
    let label = format!("{}:{}", enc(issuer), enc(account));
    format!(
        "otpauth://totp/{label}?secret={secret}&issuer={issuer}&algorithm=SHA1&digits={digits}&period={period}",
        secret = base32_encode(secret),
        issuer = enc(issuer),
        digits = params.digits,
        period = params.step,
    )
}

/// TOTP parameters. The Mailwoman defaults (DQ2) are a 30s step, 6 digits, and a
/// ±1-step validation window.
#[derive(Debug, Clone, Copy)]
pub struct TotpParams {
    /// Time step in seconds.
    pub step: u64,
    /// Number of decimal digits in the code.
    pub digits: u32,
    /// Number of steps to check on each side of the current step.
    pub skew: u64,
}

impl Default for TotpParams {
    fn default() -> Self {
        TotpParams {
            step: 30,
            digits: 6,
            skew: 1,
        }
    }
}

/// Compute the TOTP code for `secret` at absolute `unix_time` seconds.
pub fn totp_at(secret: &[u8], unix_time: u64, params: &TotpParams) -> String {
    let counter = unix_time / params.step;
    hotp(secret, counter, params.digits)
}

/// Verify `code` against `secret` at `unix_time`, accepting any step within the
/// ±`skew` window. Returns the RFC 6238 step counter that matched (`Some`) so the
/// caller can enforce a replay guard by remembering the last-consumed step, or
/// `None` if no step in the window matched. The whole window is scanned without an
/// early-out and each candidate is compared constant-time, so a match's position is
/// not leaked by timing.
pub fn totp_verify(secret: &[u8], code: &str, unix_time: u64, params: &TotpParams) -> Option<u64> {
    let code = code.trim();
    // A well-formed code is exactly `digits` ASCII digits.
    if code.len() != params.digits as usize || !code.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let center = unix_time / params.step;
    let lo = center.saturating_sub(params.skew);
    let hi = center.saturating_add(params.skew);
    let mut matched = None;
    for counter in lo..=hi {
        let candidate = hotp(secret, counter, params.digits);
        if ct_eq(candidate.as_bytes(), code.as_bytes()) {
            matched = Some(counter);
        }
    }
    matched
}

/// RFC 4226 HOTP: HMAC-SHA1 over the 8-byte big-endian counter, dynamic
/// truncation, reduced mod 10^digits and zero-padded.
fn hotp(secret: &[u8], counter: u64, digits: u32) -> String {
    let mut mac = HmacSha1::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(&counter.to_be_bytes());
    let hs = mac.finalize().into_bytes();

    // Dynamic truncation (RFC 4226 §5.3).
    let offset = (hs[hs.len() - 1] & 0x0f) as usize;
    let bin = ((u32::from(hs[offset]) & 0x7f) << 24)
        | ((u32::from(hs[offset + 1]) & 0xff) << 16)
        | ((u32::from(hs[offset + 2]) & 0xff) << 8)
        | (u32::from(hs[offset + 3]) & 0xff);

    let modulo = 10u32.pow(digits);
    let value = bin % modulo;
    format!("{value:0width$}", width = digits as usize)
}
