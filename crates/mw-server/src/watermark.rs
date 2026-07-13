//! Web screen-capture watermark — the HONEST overlay (SPEC §7.6, plan §3 e7 /
//! risk #13). The server exposes a config flag + the viewer's identity + a
//! server timestamp; the SPA renders a tiled, low-opacity name/time overlay over
//! sensitive views. This is a **visual deterrent, not a security control**, and the
//! response always carries [`HONEST_NOTE`] so the overlay can never ship without
//! the honesty statement.
//!
//! Why honest: a web browser cannot prevent, block, or detect screenshots or
//! screen recording (there is no such web API), so no browser overlay can protect
//! content from capture. True capture protection needs the native desktop app
//! (`SetWindowDisplayAffinity` / `WDA_EXCLUDEFROMCAPTURE`), which is a later
//! milestone (V5) — deliberately OUT of this release (plan §0). The overlay is
//! CSP-safe: it is pure DOM/CSS rendered by the SPA under the existing
//! `script-src 'self'` policy, pulling no external resource.

/// The mandatory honesty statement returned with every watermark config, and the
/// canonical wording e9 mirrors into `docs/security/`. It must NOT overclaim.
pub const HONEST_NOTE: &str = "This watermark is a visual deterrent only. A web \
    browser cannot prevent, block, or detect screenshots or screen recordings, so \
    this overlay cannot stop this content from being captured. It stamps the \
    viewer's identity and the time across the view to discourage casual sharing \
    and to make a leaked screenshot attributable — it is not a security control. \
    Genuine screen-capture protection requires the native desktop application, \
    planned for a later release.";

/// Watermark overlay configuration (env-sourced, plan §3 e7).
#[derive(Debug, Clone)]
pub struct WatermarkConfig {
    /// Whether the SPA should render the deterrent overlay.
    pub enabled: bool,
    /// Overlay tile opacity (0.0–1.0); low by default so it does not impede
    /// reading while remaining visible in a capture.
    pub opacity: f32,
}

impl Default for WatermarkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            opacity: 0.08,
        }
    }
}

impl WatermarkConfig {
    /// The client-facing config payload: the flag, opacity, the viewer identity to
    /// tile, a server timestamp, and — always — the honesty note. `enabled` is a
    /// deterrent toggle, never a protection guarantee.
    pub fn payload(&self, identity: &str) -> serde_json::Value {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        serde_json::json!({
            "enabled": self.enabled,
            "opacity": self.opacity.clamp(0.0, 1.0),
            "identity": identity,
            "serverTimeUnixMs": now_ms,
            "honest": true,
            "note": HONEST_NOTE,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_always_carries_the_honest_note() {
        let cfg = WatermarkConfig {
            enabled: true,
            opacity: 0.1,
        };
        let p = cfg.payload("alice@example.org");
        assert_eq!(p["enabled"], true);
        assert_eq!(p["honest"], true);
        assert_eq!(p["identity"], "alice@example.org");
        assert!(p["serverTimeUnixMs"].as_u64().unwrap() > 0);
        let note = p["note"].as_str().unwrap();
        assert!(note.contains("cannot prevent"));
        assert!(note.contains("deterrent"));
        // Must not overclaim.
        assert!(!note.to_lowercase().contains("prevents screenshots"));
    }

    #[test]
    fn opacity_is_clamped() {
        let cfg = WatermarkConfig {
            enabled: true,
            opacity: 5.0,
        };
        assert_eq!(cfg.payload("x")["opacity"], 1.0);
    }
}
