//! [`Secret`] — a plaintext password held transiently, zeroized on drop.

use zeroize::Zeroizing;

/// A secret (old/new plaintext password) held only transiently in memory.
///
/// The inner buffer is zeroized on drop ([`Zeroizing`]); [`Debug`](std::fmt::Debug)
/// never renders the plaintext, and the type is not `Serialize` — it cannot be
/// accidentally logged or persisted. Backends that must transmit the password
/// upstream call [`Secret::expose`].
#[derive(Clone)]
pub struct Secret(Zeroizing<String>);

impl Secret {
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(Zeroizing::new(s.into()))
    }

    /// Expose the plaintext to a backend that must transmit it upstream.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(***)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_never_prints_plaintext() {
        let s = Secret::new("hunter2");
        assert_eq!(format!("{s:?}"), "Secret(***)");
        assert_eq!(s.expose(), "hunter2");
        // A clone exposes the same plaintext but is an independent zeroized buffer.
        assert_eq!(s.clone().expose(), "hunter2");
    }
}
