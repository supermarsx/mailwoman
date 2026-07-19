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
        let (major, arg) = self.head()?;
        match major {
            MAJOR_UINT | MAJOR_NINT | MAJOR_SIMPLE => Ok(()),
            MAJOR_BYTES | MAJOR_TEXT => {
                self.take(arg as usize)?;
                Ok(())
            }
            MAJOR_ARRAY => {
                for _ in 0..arg {
                    self.skip()?;
                }
                Ok(())
            }
            MAJOR_MAP => {
                for _ in 0..arg {
                    self.skip()?; // key
                    self.skip()?; // value
                }
                Ok(())
            }
            MAJOR_TAG => self.skip(),
            _ => Err(Self::err("unskippable item")),
        }
    }
}
