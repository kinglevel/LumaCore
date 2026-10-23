//! The profile's look as concrete colours on the station (decisions §30
//! "12b"): the TUI the station draws for a shell follows the user's
//! UserProfile theme like the web page does. The presets are the ones in
//! the WebUI's `web/kit/tokens.css` and the adjusters do what `adjust()` in
//! `web/kit/theme.js` does; the tests check both against those files.
//! This is the theme module on the station side: the one place a colour is
//! written as a number.

use lumacore_tree::media::Theme;

use crate::AppState;

pub type Rgb = (u8, u8, u8);

/// The 14 palette colours plus the roles a terminal needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub bg: Rgb,
    pub surface: Rgb,
    pub raised: Rgb,
    pub line: Rgb,
    pub text: Rgb,
    pub text_dim: Rgb,
    pub accent: Rgb,
    pub info: Rgb,
    pub ok: Rgb,
    pub warn: Rgb,
    pub alert: Rgb,
    pub programmer: Rgb,
    pub blind: Rgb,
    pub park: Rgb,
    pub on_accent: Rgb,
    /// Between windows and under a terminal (tokens.css `--lc-gap`).
    pub gap: Rgb,
    /// Selected rows (`--lc-selected`).
    pub selected: Rgb,
    /// Emphasis (`--lc-strong`).
    pub strong: Rgb,
}

const fn hex(v: u32) -> Rgb {
    ((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

/// `(name, [bg, surface, raised, line, text, text_dim, accent, info, ok,
/// warn, alert, programmer, blind, park, on_accent])` as in tokens.css.
pub const PRESETS: [(&str, [u32; 15]); 3] = [
    (
        "Dark",
        [
            0x121212, 0x1c1c1c, 0x303030, 0x424242, 0xd1d1d1, 0x8a8a8a, 0x9a7a08, 0x758ba0,
            0x00a60a, 0xc27a12, 0xe10000, 0xd86a7c, 0x8a80c8, 0x3f9a90, 0xf3ecd3,
        ],
    ),
    (
        "Daylight",
        [
            0xe4e4e4, 0xf0f0f0, 0xfbfbfb, 0xbdbdbd, 0x161616, 0x5a5a5a, 0x7c6200, 0x3d5a78,
            0x007a07, 0x9a5600, 0xc00000, 0xa8384e, 0x5a4fb0, 0x1f756c, 0xffffff,
        ],
    ),
    (
        "Night",
        [
            0x0b0505, 0x140909, 0x1d0d0d, 0x2e1515, 0xd07a68, 0x8a4a40, 0xe0862e, 0xb0705f,
            0x9aa040, 0xe0862e, 0xff4a3a, 0xe0607a, 0x9a6ab0, 0x4a9a8a, 0x0b0505,
        ],
    ),
];

fn mix(a: Rgb, b: Rgb, a_share: f64) -> Rgb {
    let f = |x: u8, y: u8| (f64::from(x) * a_share + f64::from(y) * (1.0 - a_share)).round() as u8;
    (f(a.0, b.0), f(a.1, b.1), f(a.2, b.2))
}

const BLACK: Rgb = (0, 0, 0);
const WHITE: Rgb = (255, 255, 255);

/// A preset as shipped (unknown names → Dark).
pub fn preset(name: &str) -> Palette {
    let (name, c) = PRESETS
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .copied()
        .unwrap_or(PRESETS[0]);
    let c = c.map(hex);
    let mut p = Palette {
        bg: c[0],
        surface: c[1],
        raised: c[2],
        line: c[3],
        text: c[4],
        text_dim: c[5],
        accent: c[6],
        info: c[7],
        ok: c[8],
        warn: c[9],
        alert: c[10],
        programmer: c[11],
        blind: c[12],
        park: c[13],
        on_accent: c[14],
        gap: BLACK,
        selected: BLACK,
        strong: BLACK,
    };
    derive(&mut p, name);
    p
}

/// The derived roles (tokens.css per preset).
fn derive(p: &mut Palette, preset: &str) {
    p.gap = mix(p.bg, BLACK, if preset == "Daylight" { 0.80 } else { 0.45 });
    p.selected = mix(p.accent, p.raised, 0.24);
    p.strong = match preset {
        "Daylight" => BLACK,
        "Night" => p.text,
        _ => mix(p.text, WHITE, 0.70),
    };
}

fn parse_hex(s: &str) -> Option<Rgb> {
    let s = s.trim().trim_start_matches('#');
    let s = match s.len() {
        3 => s.chars().flat_map(|c| [c, c]).collect::<String>(),
        6 => s.to_string(),
        _ => return None,
    };
    let v = u32::from_str_radix(&s, 16).ok()?;
    Some(hex(v))
}

fn to_hsl((r, g, b): Rgb) -> (f64, f64, f64) {
    let (r, g, b) = (f64::from(r) / 255.0, f64::from(g) / 255.0, f64::from(b) / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if max == min {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h * 60.0, s, l)
}

fn from_hsl(h: f64, s: f64, l: f64) -> Rgb {
    let a = s * l.min(1.0 - l);
    let f = |n: f64| {
        let k = (n + h / 30.0) % 12.0;
        let v = l - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0);
        (v * 255.0).round() as u8
    };
    (f(0.0), f(8.0), f(4.0))
}

fn clamp(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

/// One colour through the adjusters — `adjust()` in kit/theme.js. A neutral (Background,
/// Surface, Raised, Line) first blends toward the background hue at its own lightness.
fn adjust(rgb: Rgb, a: &lumacore_tree::media::ThemeAdjust, is_accent: bool, is_neutral: bool) -> Rgb {
    let (mut h, mut s, mut l) = to_hsl(rgb);
    if is_neutral && a.background_saturation != 0 {
        let t = f64::from(a.background_saturation) / 100.0 * 0.55;
        let to = from_hsl(f64::from(a.background_hue), 1.0, l);
        let mix = |from: u8, to: u8| (f64::from(from) + (f64::from(to) - f64::from(from)) * t).round() as u8;
        (h, s, l) = to_hsl((mix(rgb.0, to.0), mix(rgb.1, to.1), mix(rgb.2, to.2)));
    }
    l = clamp(l + f64::from(a.brightness) / 250.0);
    l = clamp(0.5 + (l - 0.5) * (1.0 + f64::from(a.contrast) / 100.0));
    s = clamp(s * (1.0 + f64::from(a.saturation) / 100.0));
    if is_accent && a.accent_hue != 0 {
        h = (h + f64::from(a.accent_hue) + 360.0) % 360.0;
    }
    from_hsl(h, s, l)
}

/// A profile's theme as colours (`applyLook` in kit/theme.js).
pub fn resolve(theme: &Theme) -> Palette {
    let base = preset(&theme.preset);
    let a = &theme.adjust;
    // The background hue alone changes nothing (`adjusted` in kit/theme.js).
    let adjusted = a.brightness != 0
        || a.contrast != 0
        || a.saturation != 0
        || a.accent_hue != 0
        || a.background_saturation != 0;
    let pal = &theme.palette;
    let mut p = base;
    {
        let slots: [(&mut Rgb, &Option<String>, bool); 14] = [
            (&mut p.bg, &pal.background, false),
            (&mut p.surface, &pal.surface, false),
            (&mut p.raised, &pal.raised, false),
            (&mut p.line, &pal.line, false),
            (&mut p.text, &pal.text, false),
            (&mut p.text_dim, &pal.text_dim, false),
            (&mut p.accent, &pal.accent, true),
            (&mut p.info, &pal.info, false),
            (&mut p.ok, &pal.ok, false),
            (&mut p.warn, &pal.warn, false),
            (&mut p.alert, &pal.alert, false),
            (&mut p.programmer, &pal.programmer, false),
            (&mut p.blind, &pal.blind, false),
            (&mut p.park, &pal.park, false),
        ];
        // The first four are the neutrals the background adjusters move.
        for (i, (slot, own, is_accent)) in slots.into_iter().enumerate() {
            let colour = own.as_deref().and_then(parse_hex).unwrap_or(*slot);
            *slot = if adjusted { adjust(colour, a, is_accent, i < 4) } else { colour };
        }
    }
    let name = PRESETS
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(&theme.preset))
        .map_or("Dark", |(n, _)| n);
    derive(&mut p, name);
    p
}

/// The colours the executing client's terminal is drawn in: the user's
/// profile theme, or the factory preset while its device shows the normal
/// theme (Meow+F9, the station's `ThemeOverride`; the station's own console
/// follows `ThemeOverride`).
pub fn for_current(state: &AppState) -> Palette {
    let override_preset = state.root.station_settings.theme_override.clone();
    let normal = match &state.current_device {
        Some(d) => state.devices.get(d).is_some_and(|d| d.normal_theme) || override_preset.is_some(),
        None => state.current_client.is_none() && override_preset.is_some(),
    };
    if normal {
        return preset(override_preset.as_deref().unwrap_or("Dark"));
    }
    let user = state.effective_user_name();
    state
        .root
        .show_data
        .profile_for_user(user)
        .map_or_else(|| preset("Dark"), |p| resolve(&p.settings.theme))
}

#[cfg(test)]
#[path = "look_tests.rs"]
mod tests;
