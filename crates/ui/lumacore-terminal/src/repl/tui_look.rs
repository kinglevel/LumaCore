// ██╗      ██████╗  ██████╗ ██╗  ██╗
// ██║     ██╔═══██╗██╔═══██╗██║ ██╔╝
// ██║     ██║   ██║██║   ██║█████╔╝
// ██║     ██║   ██║██║   ██║██╔═██╗
// ███████╗╚██████╔╝╚██████╔╝██║  ██╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝
// FILE: look

//! The TUI in the user's theme (decisions §30 "12b"): drawing code names
//! colours by role ([`role`]) — the 16 terminal colour slots — and one pass
//! at the end of each frame ([`paint`]) turns every cell into the user's
//! UserProfile theme as truecolor, the same slots xterm gets on the web page
//! (`terminalTheme()` in kit/theme.js), with the theme's background painted.
//! The only file in `terminal/` that names a colour (tidy).
//!
//! Owns:          the TUI's colours by role and the theme pass
//! Doesn't own:   the theme itself (the user profile)
//! Entry points:  role, paint
//! Decisions:     §30 "12b"

use ratatui::buffer::Buffer;
use ratatui::style::Color;

use crate::core::AppState;
use crate::core::look::{Palette, Rgb};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Colours by what they mean; [`paint`] makes them the theme's.
pub(super) mod role {
    use ratatui::style::Color;
    /// The accent, as text (gold in Dark).
    pub const ACCENT: Color = Color::Yellow;
    pub const ACCENT_BRIGHT: Color = Color::LightYellow;
    pub const OK: Color = Color::Green;
    pub const ALERT: Color = Color::Red;
    pub const INFO: Color = Color::Blue;
    pub const PROGRAMMER: Color = Color::Magenta;
    pub const DIM: Color = Color::DarkGray;
    pub const STRONG: Color = Color::White;
    /// The ground (under text, a fill's text colour).
    pub const GROUND: Color = Color::Black;
}


/// The exact theme colours, for drawing that needs more than the slots (the
/// dialog window: surface, lines, fills).
#[derive(Debug, Clone, Copy)]
pub(super) struct TuiLook {
    pub(super) surface: Color,
    pub(super) raised: Color,
    pub(super) text: Color,
    pub(super) dim: Color,
    pub(super) accent: Color,
    pub(super) on_accent: Color,
    pub(super) ok: Color,
    pub(super) warn: Color,
    pub(super) alert: Color,
    pub(super) info: Color,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn rgb(c: Rgb) -> Color {
    Color::Rgb(c.0, c.1, c.2)
}

impl TuiLook {
    pub(super) fn new(state: &AppState) -> Self {
        let p = crate::core::look::for_current(state);
        Self {
            surface: rgb(p.surface),
            raised: rgb(p.raised),
            text: rgb(p.text),
            dim: rgb(p.text_dim),
            accent: rgb(p.accent),
            on_accent: rgb(p.on_accent),
            ok: rgb(p.ok),
            warn: rgb(p.warn),
            alert: rgb(p.alert),
            info: rgb(p.info),
        }
    }

    /// A tone of the dialog (its meta, a detail row).
    pub(super) fn tone(&self, tone: Option<lumacore_wire::Tone>) -> Color {
        use lumacore_wire::Tone;
        match tone {
            None => self.text,
            Some(Tone::Dim) => self.dim,
            Some(Tone::Info) => self.info,
            Some(Tone::Ok) => self.ok,
            Some(Tone::Warn) => self.warn,
            Some(Tone::Alert) => self.alert,
        }
    }

    /// `a` over `b` (a dimmed shell behind a dialog).
    pub(super) fn blend(a: Color, b: Color, a_share: f32) -> Color {
        match (a, b) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
                let f = |x: u8, y: u8| (f32::from(x) * a_share + f32::from(y) * (1.0 - a_share)).round() as u8;
                Color::Rgb(f(r1, r2), f(g1, g2), f(b1, b2))
            }
            _ => a,
        }
    }
}

/// An object's colour as the station resolved it for this user (decisions 0294: its Paint, its
/// tags', its theme slot), `#rrggbb` → a terminal colour drawn as it is. The one door a show
/// object's colour takes into the TUI; None = no colour of its own.
pub(super) fn object_paint(hex: Option<&str>) -> Option<Color> {
    lumacore_base::paint::hex_rgb(hex?).map(|[r, g, b]| Color::Rgb(r, g, b))
}

fn lift(c: Rgb, amount: f64) -> Rgb {
    let f = |v: u8| (f64::from(v) + (255.0 - f64::from(v)) * amount).round() as u8;
    (f(c.0), f(c.1), f(c.2))
}

/// A slot (or any colour) as the theme's colour; `background` = a cell's
/// background (Reset → `ground`).
fn themed(c: Color, p: &Palette, background: bool, ground: Rgb) -> Color {
    let out = match c {
        Color::Reset if background => ground,
        Color::Reset => p.text,
        Color::Black => p.gap,
        Color::Red => p.alert,
        Color::Green => p.ok,
        Color::Yellow => lift(p.accent, 0.35),
        Color::Blue => p.info,
        Color::Magenta => p.programmer,
        Color::Cyan => p.park,
        Color::Gray => p.text,
        Color::DarkGray => p.text_dim,
        Color::LightRed => lift(p.alert, 0.25),
        Color::LightGreen => lift(p.ok, 0.25),
        Color::LightYellow => lift(p.accent, 0.55),
        Color::LightBlue => lift(p.info, 0.25),
        Color::LightMagenta => lift(p.programmer, 0.25),
        Color::LightCyan => lift(p.park, 0.25),
        Color::White => p.strong,
        Color::Indexed(n) if n < 16 => {
            return themed(SLOTS[usize::from(n)], p, background, ground);
        }
        other => return other,
    };
    rgb(out)
}

/// xterm's 16 colours in order (an indexed colour 0–15 is a slot too).
const SLOTS: [Color; 16] = [
    Color::Black,
    Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::Gray,
    Color::DarkGray,
    Color::LightRed,
    Color::LightGreen,
    Color::LightYellow,
    Color::LightBlue,
    Color::LightMagenta,
    Color::LightCyan,
    Color::White,
];

/// Where a frame is drawn, which decides its ground (decisions §30 "12b2").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Ground {
    /// A terminal: the gap colour, as the web page's terminal windows.
    Terminal,
    /// The screen's command line bar (the input layer alone): the bar's own
    /// background, so the line shows no darker strip.
    Bar,
}

/// The frame in the theme of the client it is drawn for.
pub(super) fn paint(buffer: &mut Buffer, state: &AppState, ground: Ground) {
    let p = crate::core::look::for_current(state);
    let ground = match ground {
        Ground::Terminal => p.gap,
        Ground::Bar => p.bg,
    };
    for cell in buffer.content.iter_mut() {
        cell.fg = themed(cell.fg, &p, false, ground);
        cell.bg = themed(cell.bg, &p, true, ground);
    }
}
