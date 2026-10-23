//! Moonlight / Deep Space: shared semantic colors for widgets, charts and the Lab Map.
use iced::Color;

pub const fn rgb(hex: u32) -> Color {
    Color::from_rgb(
        ((hex >> 16) & 255) as f32 / 255.0,
        ((hex >> 8) & 255) as f32 / 255.0,
        (hex & 255) as f32 / 255.0,
    )
}

pub const BG: Color = rgb(0xF2F3F8);
pub const PAPER: Color = Color::WHITE;
pub const INK: Color = rgb(0x24293A);
pub const MUTED: Color = rgb(0x5B667A);
pub const LINE: Color = rgb(0xD9DEEA);
pub const ACCENT: Color = rgb(0x5757BF);
pub const SELECTED: Color = rgb(0xE7E8FA);
pub const HOVER: Color = rgb(0xECEFF7);
pub const AMBER: Color = rgb(0x89580E);
pub const WARNING_BG: Color = rgb(0xFFF4DF);
pub const HEALTHY: Color = rgb(0x216D82);
pub const DANGER: Color = rgb(0xAD3A50);
pub const UNKNOWN: Color = rgb(0x5A6477);
pub const CONNECTING: Color = rgb(0x4A5EA8);
pub const CPU: Color = rgb(0x6461CB);
pub const MEMORY: Color = rgb(0x9560A4);
pub const GPU: Color = rgb(0x297A96);
pub const GROUND: Color = rgb(0xEAEDF5);

// The navigation rail has the same night-sky surface in both themes.
pub const NAV: Color = rgb(0x0D1220);
pub const NAV_TEXT: Color = rgb(0xCCD4E7);
pub const NAV_MUTED: Color = rgb(0x929FB9);
pub const NAV_SELECTED: Color = rgb(0x242B47);
pub const NAV_HOVER: Color = rgb(0x1C2437);
pub const ACCENT_SOFT: Color = rgb(0xD0CCFF);

const NIGHT: [(Color, Color); 18] = [
    (BG, rgb(0x0B0E18)),
    (PAPER, rgb(0x141927)),
    (INK, rgb(0xEDF0FA)),
    (MUTED, rgb(0xA5B0C7)),
    (LINE, rgb(0x303B52)),
    (ACCENT, rgb(0xB6B2FF)),
    (SELECTED, rgb(0x262842)),
    (HOVER, rgb(0x20283A)),
    (AMBER, rgb(0xE7BB75)),
    (WARNING_BG, rgb(0x302619)),
    (HEALTHY, rgb(0x7ACBE1)),
    (DANGER, rgb(0xF29CA8)),
    (UNKNOWN, rgb(0xA2ACC2)),
    (CONNECTING, rgb(0xAABCF7)),
    (CPU, rgb(0xAAA7FF)),
    (MEMORY, rgb(0xDBA5CF)),
    (GPU, rgb(0x79C7E3)),
    (GROUND, rgb(0x101624)),
];

pub fn resolve(color: Color, dark: bool) -> Color {
    if dark {
        for (day, night) in NIGHT {
            if [color.r, color.g, color.b] == [day.r, day.g, day.b] {
                return Color {
                    a: color.a,
                    ..night
                };
            }
        }
    }
    color
}

pub fn rgba(color: Color) -> [f32; 4] {
    [color.r, color.g, color.b, color.a]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luminance(c: Color) -> f32 {
        let linear = |v: f32| {
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(c.r) + 0.7152 * linear(c.g) + 0.0722 * linear(c.b)
    }
    fn contrast(a: Color, b: Color) -> f32 {
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    #[test]
    fn text_status_and_series_remain_legible_in_both_themes() {
        for dark in [false, true] {
            for background in [BG, PAPER, SELECTED, HOVER] {
                for foreground in [
                    INK, MUTED, ACCENT, HEALTHY, DANGER, UNKNOWN, CONNECTING, AMBER,
                ] {
                    let ratio = contrast(resolve(foreground, dark), resolve(background, dark));
                    assert!(
                        ratio >= 4.5,
                        "Text contrast {ratio:.2}, dark={dark}, {foreground:?} on {background:?}"
                    );
                }
            }
            for series in [CPU, MEMORY, GPU] {
                assert!(contrast(resolve(series, dark), resolve(PAPER, dark)) >= 3.0);
            }
            assert!(contrast(resolve(AMBER, dark), resolve(WARNING_BG, dark)) >= 4.5);
        }
        for background in [NAV, NAV_SELECTED, NAV_HOVER] {
            for foreground in [NAV_TEXT, NAV_MUTED, ACCENT_SOFT] {
                assert!(contrast(foreground, background) >= 4.5);
            }
        }
    }
}
