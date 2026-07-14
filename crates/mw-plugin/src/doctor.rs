//! `doctor` posture output for the plugin subsystem (plan §2.1, SPEC §7.5).
//!
//! Surfaces the security-relevant state an operator needs: which backend the jail
//! uses (JIT vs interpreter), whether any loaded plugin is **unsigned** (a
//! persistent-banner condition), and the per-plugin capability grants.

use crate::PluginHandle;

/// A one-shot posture snapshot for `mailwoman doctor` / the admin UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginPosture {
    /// The codegen backend: `"cranelift-jit"` or `"pulley-interpreter"`.
    pub backend: &'static str,
    /// Per-plugin rows.
    pub plugins: Vec<PluginPostureRow>,
}

/// One plugin's posture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginPostureRow {
    pub id: String,
    /// True ⇒ loaded unsigned under `allow_unsigned` — the UI MUST show a banner.
    pub unsigned: bool,
    /// The effective (kebab-case) granted capabilities.
    pub granted: Vec<String>,
}

/// The codegen backend selected at build time.
#[must_use]
pub fn backend_name() -> &'static str {
    if cfg!(feature = "pulley") {
        "pulley-interpreter"
    } else {
        "cranelift-jit"
    }
}

/// Build a posture snapshot from the currently-loaded handles.
#[must_use]
pub fn posture(handles: &[&PluginHandle]) -> PluginPosture {
    PluginPosture {
        backend: backend_name(),
        plugins: handles
            .iter()
            .map(|h| PluginPostureRow {
                id: h.id().to_string(),
                unsigned: h.is_unsigned(),
                granted: h
                    .granted()
                    .iter()
                    .map(|c| {
                        serde_json::to_value(c)
                            .ok()
                            .and_then(|v| v.as_str().map(str::to_string))
                            .unwrap_or_else(|| format!("{c:?}"))
                    })
                    .collect(),
            })
            .collect(),
    }
}

/// A human-readable one-liner per plugin (for the CLI `doctor` text output).
#[must_use]
pub fn render(posture: &PluginPosture) -> String {
    let mut out = format!("plugin jail: backend={}\n", posture.backend);
    if posture.plugins.is_empty() {
        out.push_str("  (no plugins loaded)\n");
    }
    for p in &posture.plugins {
        out.push_str(&format!(
            "  {} — {} — caps=[{}]\n",
            p.id,
            if p.unsigned {
                "UNSIGNED (banner)"
            } else {
                "signed"
            },
            p.granted.join(", ")
        ));
    }
    out
}
