//! Badge-count command (plan §2.1 "setBadgeCount"; §3 e1).
//!
//! Backs the capability-layer `setBadgeCount(n)` — the unread count on the dock /
//! taskbar / app icon. On macOS and Linux it uses Tauri's core
//! `WebviewWindow::set_badge_count` (the dock badge, the Unity launcher count).
//! Windows has no numeric app badge, so there the count is drawn into a small
//! taskbar overlay icon (`WebviewWindow::set_overlay_icon`; t32 f25): a filled
//! circle with the number, "9+" above nine.
//!
//! Commands registered by e7 (`tauri::generate_handler!`):
//!   * `mw_set_badge_count(window, count: i64)` -> Result<(), String>

use tauri::Runtime;

/// Normalize an unread count into the `set_badge_count` argument: a positive count
/// shows the number, zero-or-negative clears the badge (`None`). Pulled out so the
/// clamping rule is unit-tested without a live window.
pub fn normalize_badge(count: i64) -> Option<i64> {
    if count > 0 { Some(count) } else { None }
}

/// The text of the Windows overlay icon: the count up to 9, then "9+"; `None`
/// clears the overlay.
pub fn overlay_label(count: i64) -> Option<String> {
    normalize_badge(count).map(|n| {
        if n > 9 {
            "9+".to_string()
        } else {
            n.to_string()
        }
    })
}

/// Edge length of the overlay icon in pixels (Windows scales it to the taskbar).
pub const OVERLAY_SIZE: u32 = 32;

/// Circle colour (RGBA): the Windows "critical" red, which keeps white text at a
/// contrast ratio above 4.5:1.
const OVERLAY_FILL: [u8; 4] = [0xC4, 0x2B, 0x1C, 0xFF];
const OVERLAY_TEXT: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

/// 3×5 glyphs, one row per byte, the left column in bit 2.
const DIGITS: [[u8; 5]; 10] = [
    [0b111, 0b101, 0b101, 0b101, 0b111],
    [0b010, 0b110, 0b010, 0b010, 0b111],
    [0b111, 0b001, 0b111, 0b100, 0b111],
    [0b111, 0b001, 0b111, 0b001, 0b111],
    [0b101, 0b101, 0b111, 0b001, 0b001],
    [0b111, 0b100, 0b111, 0b001, 0b111],
    [0b111, 0b100, 0b111, 0b101, 0b111],
    [0b111, 0b001, 0b001, 0b001, 0b001],
    [0b111, 0b101, 0b111, 0b101, 0b111],
    [0b111, 0b101, 0b111, 0b001, 0b111],
];
const PLUS: [u8; 5] = [0b000, 0b010, 0b111, 0b010, 0b000];

/// Draw `label` (digits and `+`) as an `OVERLAY_SIZE`² RGBA image: a filled circle
/// with the glyphs centred on it, transparent outside the circle.
pub fn render_overlay(label: &str) -> Vec<u8> {
    let size = OVERLAY_SIZE as i32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let mut put = |x: i32, y: i32, colour: [u8; 4]| {
        if (0..size).contains(&x) && (0..size).contains(&y) {
            let i = ((y * size + x) * 4) as usize;
            rgba[i..i + 4].copy_from_slice(&colour);
        }
    };
    // Circle: pixel centres within radius size/2 of the image centre.
    let r2 = (size * size) as f32 / 4.0;
    let c = size as f32 / 2.0;
    for y in 0..size {
        for x in 0..size {
            let (dx, dy) = (x as f32 + 0.5 - c, y as f32 + 0.5 - c);
            if dx * dx + dy * dy <= r2 {
                put(x, y, OVERLAY_FILL);
            }
        }
    }
    let glyphs: Vec<[u8; 5]> = label
        .chars()
        .filter_map(|ch| match ch {
            '+' => Some(PLUS),
            d => d.to_digit(10).map(|d| DIGITS[d as usize]),
        })
        .collect();
    if glyphs.is_empty() {
        return rgba;
    }
    let n = glyphs.len() as i32;
    let scale = if n == 1 { 4 } else { 3 };
    let width = n * 3 * scale + (n - 1) * scale;
    let (x0, y0) = ((size - width) / 2, (size - 5 * scale) / 2);
    for (g, rows) in glyphs.iter().enumerate() {
        let gx = x0 + g as i32 * 4 * scale;
        for (row, bits) in rows.iter().enumerate() {
            for col in 0..3 {
                if bits & (0b100 >> col) == 0 {
                    continue;
                }
                for sy in 0..scale {
                    for sx in 0..scale {
                        put(
                            gx + col * scale + sx,
                            y0 + row as i32 * scale + sy,
                            OVERLAY_TEXT,
                        );
                    }
                }
            }
        }
    }
    rgba
}

/// Set (or, for `count <= 0`, clear) the app/taskbar badge to the unread count.
#[tauri::command]
pub async fn mw_set_badge_count<R: Runtime>(
    window: tauri::WebviewWindow<R>,
    count: i64,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        let icon = overlay_label(count).map(|label| {
            tauri::image::Image::new_owned(render_overlay(&label), OVERLAY_SIZE, OVERLAY_SIZE)
        });
        window
            .set_overlay_icon(icon)
            .map_err(|e| format!("set_overlay_icon({count}): {e}"))
    }
    #[cfg(not(windows))]
    {
        window
            .set_badge_count(normalize_badge(count))
            .map_err(|e| format!("set_badge_count({count}): {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(rgba: &[u8], x: u32, y: u32) -> [u8; 4] {
        let i = ((y * OVERLAY_SIZE + x) * 4) as usize;
        rgba[i..i + 4].try_into().unwrap()
    }

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

    #[test]
    fn overlay_label_caps_at_nine_plus_and_clears_at_zero() {
        assert_eq!(overlay_label(0), None);
        assert_eq!(overlay_label(-1), None);
        assert_eq!(overlay_label(1).as_deref(), Some("1"));
        assert_eq!(overlay_label(9).as_deref(), Some("9"));
        assert_eq!(overlay_label(10).as_deref(), Some("9+"));
        assert_eq!(overlay_label(12_345).as_deref(), Some("9+"));
    }

    #[test]
    fn overlay_is_a_filled_circle_with_white_text() {
        let img = render_overlay("8");
        assert_eq!(img.len(), (OVERLAY_SIZE * OVERLAY_SIZE * 4) as usize);
        // Corners are outside the circle: transparent.
        assert_eq!(pixel(&img, 0, 0)[3], 0);
        assert_eq!(pixel(&img, OVERLAY_SIZE - 1, OVERLAY_SIZE - 1)[3], 0);
        // The edge of the circle is the fill colour.
        assert_eq!(pixel(&img, OVERLAY_SIZE / 2, 1), OVERLAY_FILL);
        // "8" at scale 4 starts at x 10, y 6: its top-left block is text.
        assert_eq!(pixel(&img, 10, 6), OVERLAY_TEXT);
        // The centre of an "8" is its middle bar.
        assert_eq!(pixel(&img, 15, 15), OVERLAY_TEXT);
    }

    #[test]
    fn each_label_draws_differently() {
        let labels = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "9+"];
        let images: Vec<_> = labels.iter().map(|l| render_overlay(l)).collect();
        for i in 0..images.len() {
            for j in i + 1..images.len() {
                assert_ne!(images[i], images[j], "{} vs {}", labels[i], labels[j]);
            }
        }
        // Every label puts some text on the circle.
        for (label, img) in labels.iter().zip(&images) {
            let text = img.chunks(4).filter(|p| *p == OVERLAY_TEXT).count();
            assert!(text >= 16, "{label}: {text} text pixels");
        }
    }
}
