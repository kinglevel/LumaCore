// ██████╗  █████╗ ██╗███╗   ██╗████████╗
// ██╔══██╗██╔══██╗██║████╗  ██║╚══██╔══╝
// ██████╔╝███████║██║██╔██╗ ██║   ██║
// ██╔═══╝ ██╔══██║██║██║╚██╗██║   ██║
// ██║     ██║  ██║██║██║ ╚████║   ██║
// ╚═╝     ╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝   ╚═╝
// FILE: paint

//! An object's Paint (decisions object-paint-palette-free-theme-links): its colour on screen,
//! sRGB without alpha (`Color` is light, decisions 0144). A Paint is a hex colour or a link to one
//! of the viewer's theme paint slots (`Theme "Position"`); no Paint (`none`) means the object's
//! tags, then its pool's colour. The palette's named colours (the 12 base hues, Black, White) are
//! words for their hex and are stored as it. This is the one parser every Paint goes through —
//! objects, tags, feature groups, a theme's slots; a user's colour and the window settings read
//! hex through [`hex_rgb`] too.
//!
//! Owns:          Paint, PALETTE, GREYS, NAMED, parse, hex_rgb, hex_of, palette_name.
//! Doesn't own:   where a Paint is resolved to a colour for a viewer (lumacore-state `paint`), the
//!                theme's slots (lumacore-tree `media`).
//! Entry points:  Paint::parse, Paint::word, hex_rgb.
//! Decisions:     docs/decisions/0294-object-paint-palette-free-theme-links, 0144, 0205.

use serde::{Deserialize, Serialize};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A Paint: a colour, or a link to a theme paint slot by its name. Saved as `"#rrggbb"` or
/// `{"Theme": "<slot>"}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "PaintJson", into = "PaintJson")]
pub enum Paint {
    Rgb([u8; 3]),
    /// A theme paint slot (`Theme "Position"`): each viewer sees their own theme's colour; a slot
    /// their theme doesn't have is a broken link, kept, and the object falls back.
    Theme(String),
}

/// The palette (decisions object-paint-palette-free-theme-links): 12 hues, each in four tones
/// from dark to pale; the second tone carries the hue's name.
pub const PALETTE: [(&str, [&str; 4]); 12] = [
    ("Red", ["#612925", "#9e342c", "#c7615a", "#e6a5a1"]),
    ("Orange", ["#613e25", "#9e5c2c", "#c7875a", "#e6bda1"]),
    ("Amber", ["#614d25", "#9e782c", "#c7a25a", "#e6cfa1"]),
    ("Yellow", ["#615925", "#9e8f2c", "#c7b85a", "#e6dca1"]),
    ("Lime", ["#4d6125", "#789e2c", "#a2c75a", "#cfe6a1"]),
    ("Green", ["#25612f", "#2c9e3f", "#5ac76c", "#a1e6ac"]),
    ("Teal", ["#256155", "#2c9e87", "#5ac7b1", "#a1e6d8"]),
    ("Cyan", ["#255961", "#2c8f9e", "#5ab8c7", "#a1dce6"]),
    ("Sky", ["#254861", "#2c6f9e", "#5a99c7", "#a1c9e6"]),
    ("Blue", ["#253761", "#2c4e9e", "#5a7ac7", "#a1b5e6"]),
    ("Violet", ["#412561", "#612c9e", "#8d5ac7", "#c1a1e6"]),
    ("Magenta", ["#612559", "#9e2c8f", "#c75ab8", "#e6a1dc"]),
];

/// The tone of each hue that carries its name.
pub const NAMED_TONE: usize = 1;

/// The palette's greys, black to white; the ends are named.
pub const GREYS: [&str; 8] = ["#000000", "#262626", "#4a4a4a", "#6e6e6e", "#939393", "#b8b8b8", "#dcdcdc", "#ffffff"];

/// Every colour word a Paint takes (stored as its hex): the hues, then Black and White.
pub const NAMED: [(&str, &str); 14] = [
    ("Red", PALETTE[0].1[NAMED_TONE]),
    ("Orange", PALETTE[1].1[NAMED_TONE]),
    ("Amber", PALETTE[2].1[NAMED_TONE]),
    ("Yellow", PALETTE[3].1[NAMED_TONE]),
    ("Lime", PALETTE[4].1[NAMED_TONE]),
    ("Green", PALETTE[5].1[NAMED_TONE]),
    ("Teal", PALETTE[6].1[NAMED_TONE]),
    ("Cyan", PALETTE[7].1[NAMED_TONE]),
    ("Sky", PALETTE[8].1[NAMED_TONE]),
    ("Blue", PALETTE[9].1[NAMED_TONE]),
    ("Violet", PALETTE[10].1[NAMED_TONE]),
    ("Magenta", PALETTE[11].1[NAMED_TONE]),
    ("Black", GREYS[0]),
    ("White", GREYS[7]),
];

/// The longest slot name a theme link takes.
pub const SLOT_NAME_MAX: usize = 32;

/// How a Paint is saved.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum PaintJson {
    Hex(String),
    Theme {
        #[serde(rename = "Theme")]
        theme: String,
    },
    /// Format 36 and older: RGBA bytes (read for safety; the upgrade step rewrites them).
    Bytes(Vec<u8>),
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Paint {
    /// A Paint as typed: `#rrggbb` (the `#` optional), a palette word (`Blue`, `White`),
    /// `Theme "Position"` (the quotes optional), or `none` / empty = no Paint (Ok(None)). Any case.
    /// Err = why it isn't one (the caller adds its code).
    pub fn parse(raw: &str) -> Result<Option<Paint>, String> {
        let text = raw.trim();
        let text = text.strip_prefix('"').and_then(|t| t.strip_suffix('"')).unwrap_or(text).trim();
        if text.is_empty() || text.eq_ignore_ascii_case("none") {
            return Ok(None);
        }
        if let Some(rest) = theme_rest(text) {
            let slot = rest.trim().trim_matches('"').trim();
            return check_slot(slot).map(|s| Some(Paint::Theme(s.to_string())));
        }
        if let Some(rgb) = hex_rgb(text) {
            return Ok(Some(Paint::Rgb(rgb)));
        }
        if let Some((_, hex)) = NAMED.iter().find(|(n, _)| n.eq_ignore_ascii_case(text)) {
            return Ok(hex_rgb(hex).map(Paint::Rgb));
        }
        Err(format!("'{text}' isn't a paint: #rrggbb, a palette colour ({}), Theme \"<slot>\" or none", palette_words()))
    }

    /// As the line writes it: `#rrggbb` or `Theme "Position"`.
    pub fn word(&self) -> String {
        match self {
            Paint::Rgb(rgb) => hex_of(*rgb),
            Paint::Theme(slot) => format!("Theme \"{slot}\""),
        }
    }

    /// The colour itself when it is one (a theme link has none of its own).
    pub fn rgb(&self) -> Option<[u8; 3]> {
        match self {
            Paint::Rgb(rgb) => Some(*rgb),
            Paint::Theme(_) => None,
        }
    }

    /// The slot it links to.
    pub fn slot(&self) -> Option<&str> {
        match self {
            Paint::Theme(s) => Some(s),
            Paint::Rgb(_) => None,
        }
    }
}

/// `#rrggbb` (or `rrggbb`, any case) → its bytes. The one hex reading every colour goes through.
pub fn hex_rgb(text: &str) -> Option<[u8; 3]> {
    let hex = text.trim().trim_matches('"').trim_start_matches('#');
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some([byte(0)?, byte(2)?, byte(4)?])
}

/// Bytes → `#rrggbb`.
pub fn hex_of([r, g, b]: [u8; 3]) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// The palette word for a colour, when it is one (`#2c4e9e` → Blue).
pub fn palette_name(rgb: [u8; 3]) -> Option<&'static str> {
    let hex = hex_of(rgb);
    NAMED.iter().find(|(_, h)| *h == hex).map(|(n, _)| *n)
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The text after a leading `Theme` word (`Theme "Position"`, `theme Position`).
fn theme_rest(text: &str) -> Option<&str> {
    let head = text.get(..5)?;
    let rest = &text[5..];
    (head.eq_ignore_ascii_case("theme") && (rest.starts_with(char::is_whitespace) || rest.starts_with('"'))).then_some(rest)
}

/// A slot name as a link takes it.
pub fn check_slot(slot: &str) -> Result<&str, String> {
    if slot.is_empty() {
        return Err("Theme needs a slot: Theme \"Position\"".to_string());
    }
    if slot.chars().count() > SLOT_NAME_MAX || slot.contains(['"', ';', '`']) {
        return Err(format!("'{slot}' isn't a slot name (up to {SLOT_NAME_MAX} characters, no quotes)"));
    }
    Ok(slot)
}

fn palette_words() -> String {
    NAMED.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(", ")
}

impl From<Paint> for PaintJson {
    fn from(p: Paint) -> Self {
        match p {
            Paint::Rgb(rgb) => PaintJson::Hex(hex_of(rgb)),
            Paint::Theme(theme) => PaintJson::Theme { theme },
        }
    }
}

impl TryFrom<PaintJson> for Paint {
    type Error = String;
    fn try_from(j: PaintJson) -> Result<Self, String> {
        match j {
            PaintJson::Hex(h) => hex_rgb(&h).map(Paint::Rgb).ok_or_else(|| format!("'{h}' isn't a colour #rrggbb")),
            PaintJson::Theme { theme } => Ok(Paint::Theme(theme)),
            PaintJson::Bytes(b) if b.len() >= 3 => Ok(Paint::Rgb([b[0], b[1], b[2]])),
            PaintJson::Bytes(_) => Err("a colour needs three bytes".to_string()),
        }
    }
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
