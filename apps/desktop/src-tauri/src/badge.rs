//! Badge-count command (plan §2.1 "setBadgeCount"; §3 e1).
//!
//! Backs the capability-layer `setBadgeCount(n)` — the unread count on the dock /
//! taskbar / app icon. Uses Tauri's core `WebviewWindow::set_badge_count`, which
//! maps to the dock badge on macOS and the Unity launcher count on Linux; on
//! Windows the OS has no numeric app badge, so the call degrades (the browser
//! fallback's tab-title/favicon badge, wired by e6, covers that surface).
//!
//! Commands registered by e7 (`tauri::generate_handler!`):
//!   * `mw_set_badge_count(window, count: i64)` -> Result<(), String>

use tauri::Runtime;

/// Normalize an unread count into the `set_badge_count` argument: a positive count
/// shows the number, zero-or-negative clears the badge (`None`). Pulled out so the
/// clamping rule is unit-tested without a live window.
pub fn normalize_badge(count: i64) -> Option<i64> {
    if count > 0 {
        Some(count)
    } else {
        None
    }
}

/// Set (or, for `count <= 0`, clear) the app/taskbar badge to the unread count.
#[tauri::command]
pub async fn mw_set_badge_count<R: Runtime>(
    window: tauri::WebviewWindow<R>,
    count: i64,
) -> Result<(), String> {
    window
        .set_badge_count(normalize_badge(count))
        .map_err(|e| format!("set_badge_count({count}): {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_counts_pass_through() {
        assert_eq!(normalize_badge(1), Some(1));
        assert_eq!(normalize_badge(42), Some(42));
    }

    #[test]
    fn zero_and_negative_clear_the_badge() {
        assert_eq!(normalize_badge(0), None);
        assert_eq!(normalize_badge(-5), None);
    }
}
