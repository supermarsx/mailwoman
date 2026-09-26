//! Minimal CBOR (RFC 8949) reader — just enough to decode a WebAuthn
//! `attestationObject` and a COSE_Key.
//!
//! We deliberately do NOT pull a general CBOR crate (e.g. `ciborium`): the whole
//! 2FA lane's net-new dependency budget is exactly `sha1` (t16 plan), and the CBOR
//! we must read is a small, well-specified, definite-length subset. This reader
//! supports unsigned ints, negative ints, byte strings, text strings, arrays and
//! maps, plus a `skip` over any single item (used to step over the empty `attStmt`).
//! Indefinite-length items and the reserved additional-info values are rejected —
//! WebAuthn/COSE use canonical, definite-length encodings.

use crate::MfaError;

/// Cursor over a CBOR byte buffer. `pos` is public-in-crate so the WebAuthn decoder
/// can measure how many bytes a COSE_Key occupied inside `authData`.
pub(crate) struct Reader<'a> {
    buf: &'a [u8],
    pub(crate) pos: usize,
}

/// Maximum nesting [`Reader::skip`] will descend into before reporting the input
/// malformed.
///
/// `skip` is the only recursive routine in this reader, and it is reached from
/// `webauthn`'s `attestationObject` decode and `cose`'s key decode — both over
/// bytes a client supplies. Nesting there costs **one stack frame per input
/// byte** at worst (`0x81` is a one-element array; `0xc6` a tag), which is the
/// worst frames-per-byte ratio of any parser in the tree, so the input length
/// bound is no bound at all.
///
/// **32 because the structures are flat.** The deepest thing WebAuthn asks `skip`
/// to step over is an `attStmt` map holding an `x5c` array of byte strings —
/// three levels. A COSE_Key is one map of scalars. 32 is an order of magnitude of
/// headroom over both and far below any plausible stack ceiling (t25-e3 measured
/// the *fattest* recursive parser in this tree, `mw-autoconfig`'s XML reader,
/// overflowing a 2 MiB tokio worker stack at ~1 150 levels in a debug build; CBOR
/// frames are smaller, so its own ceiling is higher still).
///
/// The refusal is an ordinary [`MfaError::Cbor`], so an over-nested attestation
/// fails registration the same way a truncated one does.
pub(crate) const MAX_DEPTH: usize = 32;

/// CBOR major types we care about.
pub(crate) const MAJOR_UINT: u8 = 0;
pub(crate) const MAJOR_NINT: u8 = 1;
pub(crate) const MAJOR_BYTES: u8 = 2;
pub(crate) const MAJOR_TEXT: u8 = 3;
pub(crate) const MAJOR_ARRAY: u8 = 4;
pub(crate) const MAJOR_MAP: u8 = 5;
pub(crate) const MAJOR_TAG: u8 = 6;
pub(crate) const MAJOR_SIMPLE: u8 = 7;

impl<'a> Reader<'a> {
    pub(crate) fn new(buf: &'a [u8]) -> Self {
        Reader { buf, pos: 0 }
    }

    fn err(msg: &str) -> MfaError {
        MfaError::Cbor(msg.to_string())
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], MfaError> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| Self::err("length overflow"))?;
        if end > self.buf.len() {
            return Err(Self::err("unexpected end of input"));
        }
        let out = &self.buf[self.pos..end];
        self.pos = end;
        Ok(out)
    }

    /// Read one item head: returns `(major_type, argument)`. Rejects
    /// indefinite-length (additional info 31) and the reserved values 28–30.
    fn head(&mut self) -> Result<(u8, u64), MfaError> {
        let b = self.take(1)?[0];
        let major = b >> 5;
        let ai = b & 0x1f;
        let arg = match ai {
            0..=23 => u64::from(ai),
            24 => u64::from(self.take(1)?[0]),
            25 => {
                let s = self.take(2)?;
                u64::from(u16::from_be_bytes([s[0], s[1]]))
            }
            26 => {
                let s = self.take(4)?;
                u64::from(u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
            }
            27 => {
                let s = self.take(8)?;
                u64::from_be_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]])
            }
            _ => return Err(Self::err("indefinite-length or reserved encoding")),
        };
        Ok((major, arg))
    }

    /// Expect a map header, returning its entry count.
    pub(crate) fn map_len(&mut self) -> Result<u64, MfaError> {
        let (major, arg) = self.head()?;
        if major != MAJOR_MAP {
            return Err(Self::err("expected a map"));
        }
        Ok(arg)
    }

    /// Read a definite-length text string.
    pub(crate) fn text(&mut self) -> Result<&'a str, MfaError> {
        let (major, arg) = self.head()?;
        if major != MAJOR_TEXT {
            return Err(Self::err("expected a text string"));
        }
        let bytes = self.take(arg as usize)?;
        std::str::from_utf8(bytes).map_err(|_| Self::err("invalid utf-8 in text string"))
    }

    /// Read a definite-length byte string.
    pub(crate) fn bytes(&mut self) -> Result<&'a [u8], MfaError> {
        let (major, arg) = self.head()?;
        if major != MAJOR_BYTES {
            return Err(Self::err("expected a byte string"));
        }
        self.take(arg as usize)
    }

    /// Read an integer that may be a COSE label: unsigned (major 0) or negative
    /// (major 1, value `-1 - arg`). Returns it as an `i64`.
    pub(crate) fn int(&mut self) -> Result<i64, MfaError> {
        let (major, arg) = self.head()?;
        match major {
            MAJOR_UINT => i64::try_from(arg).map_err(|_| Self::err("integer out of range")),
            MAJOR_NINT => {
                let n = i64::try_from(arg).map_err(|_| Self::err("integer out of range"))?;
                Ok(-1 - n)
            }
            _ => Err(Self::err("expected an integer")),
        }
    }

    /// Skip exactly one complete data item (recursively for arrays/maps/tags).
    pub(crate) fn skip(&mut self) -> Result<(), MfaError> {
        self.skip_at(0)
    }

    /// `skip` with the nesting depth carried explicitly. Every nested
    /// array/map/tag costs one stack frame, and the cheapest encoding of a level
    /// is **one byte** (`0x81`, a one-element array), so an attestation object of
    /// N bytes buys N frames. See [`MAX_DEPTH`] — without it, a few kilobytes of
    /// `0x81` in `attStmt` aborts the process instead of failing the assertion.
    fn skip_at(&mut self, depth: usize) -> Result<(), MfaError> {
        if depth > MAX_DEPTH {
            return Err(Self::err("nested deeper than the CBOR depth limit"));
        }
        let (major, arg) = self.head()?;
        match major {
            MAJOR_UINT | MAJOR_NINT | MAJOR_SIMPLE => Ok(()),
            MAJOR_BYTES | MAJOR_TEXT => {
                self.take(arg as usize)?;
                Ok(())
            }
            MAJOR_ARRAY => {
                for _ in 0..arg {
                    self.skip_at(depth + 1)?;
                }
                Ok(())
            }
            MAJOR_MAP => {
                for _ in 0..arg {
                    self.skip_at(depth + 1)?; // key
                    self.skip_at(depth + 1)?; // value
                }
                Ok(())
            }
            // A tag wraps exactly one item, so it is a level of nesting like any
            // other — counting it is what stops a run of `0xc6` bytes.
            MAJOR_TAG => self.skip_at(depth + 1),
            _ => Err(Self::err("unskippable item")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression for the 26.20 one-frame-per-byte recursion (t25-e3): 64 KiB of
    /// `0x81` — a 65 536-deep chain of one-element arrays — must be refused by
    /// [`MAX_DEPTH`] rather than recursing 65 536 times.
    ///
    /// Run on a thread sized to tokio's default 2 MiB worker stack, not libtest's
    /// larger one, so that removing the cap makes this abort rather than pass.
    #[test]
    fn a_deep_array_chain_is_refused_not_fatal() {
        let err = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                let bomb = vec![0x81u8; 64 * 1024];
                Reader::new(&bomb).skip().unwrap_err().to_string()
            })
            .expect("spawn a 2 MiB probe thread")
            .join()
            .expect("the probe thread must return — a stack overflow aborts the process");
        assert!(
            err.contains("nested deeper"),
            "the depth guard must be what refused it, got: {err}"
        );

        // The same shape built from tags (`0xc6`), which recurse through a
        // different arm.
        let tags = vec![0xc6u8; 64 * 1024];
        let err = Reader::new(&tags).skip().unwrap_err().to_string();
        assert!(err.contains("nested deeper"), "got: {err}");
    }

    /// Control: the cap must refuse only absurd nesting, not the shapes WebAuthn
    /// actually sends. Without this, the test above would pass for a `skip` that
    /// rejects everything.
    #[test]
    fn realistic_nesting_still_skips() {
        // {1: "packed", 2: [h'AA', h'BB'], 3: 0} — an attStmt-shaped map with an
        // x5c-shaped array inside, three levels deep.
        let item = [
            0xa3, // map(3)
            0x01, 0x66, b'p', b'a', b'c', b'k', b'e', b'd', // 1: "packed"
            0x02, 0x82, 0x41, 0xaa, 0x41, 0xbb, // 2: [h'AA', h'BB']
            0x03, 0x00, // 3: 0
        ];
        let mut r = Reader::new(&item);
        r.skip().expect("a realistic attStmt must skip cleanly");
        assert_eq!(r.pos, item.len(), "skip must consume exactly one item");

        // Nesting right at the cap is accepted; one level past it is not. Pinning
        // both sides means an off-by-one cannot pass unnoticed.
        let mut at_cap = vec![0x81u8; MAX_DEPTH];
        at_cap.push(0x00); // the innermost element
        assert!(
            Reader::new(&at_cap).skip().is_ok(),
            "the cap must be allowed"
        );
        let mut over = vec![0x81u8; MAX_DEPTH + 1];
        over.push(0x00);
        assert!(
            Reader::new(&over).skip().is_err(),
            "one level over must be refused"
        );
    }
}
