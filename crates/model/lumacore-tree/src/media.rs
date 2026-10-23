//! Files a show carries, and the look a profile carries (decisions §30
//! "12a2 / 12a3 hardening", §8).
//!
//! **Embedded files** (images and icons; ≤ [`MAX_FILE_BYTES`] each, ≤
//! [`MAX_SHOW_BYTES`] per show) are pool objects whose manifest
//! ([`MediaFile`]: GUID, name, SHA-256, type, size, width × height, source
//! name) lives in the tree; the bytes sit beside it in the `.luma` and once
//! per SHA-256 in the station's store. The show's `Images` pool is for show
//! data; a profile's `Icons` and `Images` pools are for that profile's own UI,
//! so a profile imported into another show looks the same.
//!
//! **Asset drives** hold large show-critical media outside the show: the show
//! names [`AssetRoot`]s, each station maps a root to a folder of its own (like
//! a network to a NIC), and an [`Asset`] is root + relative path + SHA-256 +
//! size. Missing assets are listed, never a crash.

use std::collections::BTreeMap;

use lumacore_base::NodeMeta;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The largest embedded file (images, icons, GDTF).
pub const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;

/// The most a show may embed in all.
pub const MAX_SHOW_BYTES: u64 = 256 * 1024 * 1024;

/// The file types an image or icon may be (lowercase extensions).
pub const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "svg"];

/// One embedded file's manifest (`image.json` / `icon.json` in the `.luma`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaFile {
    pub meta: NodeMeta,
    /// SHA-256 of the bytes (lowercase hex): the file's identity on disk and
    /// on the wire.
    pub sha256: String,
    /// Lowercase extension (`png`, `jpg`, `webp`, `gif`, `svg`).
    pub ext: String,
    pub size: u64,
    /// Pixels; 0 when unknown (an SVG without a size).
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    /// The file name it was imported from.
    #[serde(default)]
    pub source: String,
}

impl MediaFile {
    /// `<sha256>.<ext>`: the name in the station's store.
    pub fn stored_name(&self) -> String {
        format!("{}.{}", self.sha256, self.ext)
    }

    /// The MIME type of its extension.
    pub fn mime(&self) -> &'static str {
        mime_of(&self.ext)
    }
}

/// The MIME type of an image extension.
pub fn mime_of(ext: &str) -> &'static str {
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

/// Width and height from an image's header (PNG, JPEG, GIF, WebP; SVG's
/// `width` / `height` attributes). None = not a picture this knows.
pub fn image_size(bytes: &[u8], ext: &str) -> Option<(u32, u32)> {
    let be16 = |b: &[u8], i: usize| Some(u16::from_be_bytes([*b.get(i)?, *b.get(i + 1)?]) as u32);
    let le16 = |b: &[u8], i: usize| Some(u16::from_le_bytes([*b.get(i)?, *b.get(i + 1)?]) as u32);
    let be32 = |b: &[u8], i: usize| Some(u32::from_be_bytes(b.get(i..i + 4)?.try_into().ok()?));
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some((be32(bytes, 16)?, be32(bytes, 20)?));
    }
    if bytes.starts_with(b"GIF8") {
        return Some((le16(bytes, 6)?, le16(bytes, 8)?));
    }
    if bytes.starts_with(b"\xff\xd8") {
        // Walk the JPEG segments to the first start-of-frame.
        let mut i = 2;
        while i + 9 < bytes.len() {
            if bytes[i] != 0xff {
                i += 1;
                continue;
            }
            let marker = bytes[i + 1];
            let len = be16(bytes, i + 2)? as usize;
            if matches!(marker, 0xc0..=0xc3 | 0xc5..=0xc7 | 0xc9..=0xcb | 0xcd..=0xcf) {
                return Some((be16(bytes, i + 7)?, be16(bytes, i + 5)?));
            }
            i += 2 + len;
        }
        return None;
    }
    if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        let le24 = |i: usize| -> Option<u32> {
            Some(u32::from_le_bytes([*bytes.get(i)?, *bytes.get(i + 1)?, *bytes.get(i + 2)?, 0]))
        };
        return match bytes.get(12..16)? {
            b"VP8 " => Some((le16(bytes, 26)? & 0x3fff, le16(bytes, 28)? & 0x3fff)),
            b"VP8L" => {
                let b = u32::from_le_bytes(bytes.get(21..25)?.try_into().ok()?);
                Some(((b & 0x3fff) + 1, ((b >> 14) & 0x3fff) + 1))
            }
            b"VP8X" => Some((le24(24)? + 1, le24(27)? + 1)),
            _ => None,
        };
    }
    if ext == "svg" {
        let text = std::str::from_utf8(bytes).ok()?;
        let tag = &text[text.find("<svg")?..];
        let tag = &tag[..tag.find('>')?];
        let attr = |name: &str| -> Option<u32> {
            let at = tag.find(&format!(" {name}=\""))? + name.len() + 3;
            let value = &tag[at..at + tag[at..].find('"')?];
            let digits: String = value.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
            digits.parse::<f32>().ok().map(|v| v.round() as u32)
        };
        return Some((attr("width").unwrap_or(0), attr("height").unwrap_or(0)));
    }
    None
}

// ── The look of a profile ────────────────────────────────────────────────────

/// The built-in theme presets (the kit's `tokens.css`).
pub const THEME_PRESETS: &[&str] = &["Dark", "Daylight", "Night"];

/// A profile's theme (decisions §30 "Themes"): a preset, five adjusters and
/// only the palette colours changed from the preset. The kit derives every
/// other colour.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    #[serde(default = "default_preset")]
    pub preset: String,
    #[serde(default)]
    pub adjust: ThemeAdjust,
    #[serde(default)]
    pub palette: ThemePalette,
}

fn default_preset() -> String {
    THEME_PRESETS[0].to_string()
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            preset: default_preset(),
            adjust: ThemeAdjust::default(),
            palette: ThemePalette::default(),
        }
    }
}

/// The six adjusters (REAPER style; decisions theme-on-profile, archive §30 "12g"). Brightness,
/// Contrast and Saturation are −100…100, AccentHue turns the accent −180…180°; BackgroundHue
/// (0…360°) and BackgroundSaturation (0…100) blend the neutral greys (Background, Surface, Raised,
/// Line) toward one hue at their own lightness — Saturation 0 leaves them as the preset's. They
/// replaced Tint. 0 = the preset as shipped.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct ThemeAdjust {
    #[serde(default)]
    pub brightness: i32,
    #[serde(default)]
    pub contrast: i32,
    #[serde(default)]
    pub saturation: i32,
    #[serde(default)]
    pub accent_hue: i32,
    #[serde(default)]
    pub background_hue: i32,
    #[serde(default)]
    pub background_saturation: i32,
}

/// The 14 palette colours (`#rrggbb`); None = the preset's.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ThemePalette {
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub surface: Option<String>,
    #[serde(default)]
    pub raised: Option<String>,
    #[serde(default)]
    pub line: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub text_dim: Option<String>,
    #[serde(default)]
    pub accent: Option<String>,
    #[serde(default)]
    pub info: Option<String>,
    #[serde(default)]
    pub ok: Option<String>,
    #[serde(default)]
    pub warn: Option<String>,
    #[serde(default)]
    pub alert: Option<String>,
    #[serde(default)]
    pub programmer: Option<String>,
    #[serde(default)]
    pub blind: Option<String>,
    #[serde(default)]
    pub park: Option<String>,
}

/// A profile's lock screen (decisions §30): the background (a picture of the
/// profile's own Images pool), the clock, a line of text and auto-lock.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockScreen {
    /// GUID link into the profile's `Images`; None = the built-in background.
    #[serde(default)]
    pub image: Option<Uuid>,
    #[serde(default = "yes")]
    pub show_clock: bool,
    #[serde(default)]
    pub text: Option<String>,
    /// Lock a device after this many idle minutes; 0 = never.
    #[serde(default)]
    pub auto_lock_minutes: u32,
}

fn yes() -> bool {
    true
}

impl Default for LockScreen {
    fn default() -> Self {
        Self {
            image: None,
            show_clock: true,
            text: None,
            auto_lock_minutes: 0,
        }
    }
}

// ── Asset drives ─────────────────────────────────────────────────────────────

/// `ShowData.Assets`: the roots the show names and its large assets.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AssetsData {
    #[serde(default)]
    pub roots: BTreeMap<u32, AssetRoot>,
    #[serde(default)]
    pub files: BTreeMap<u32, Asset>,
}

/// A named place for large media (`Video`, `Textures`); each station maps it
/// to a folder or drive of its own (`Root.StationSettings`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetRoot {
    pub meta: NodeMeta,
    /// The root's name: each station maps it to a folder by this key (`AssetRoot` mapped per
    /// station, §8).
    pub name: String,
}

/// A large file on an asset drive: where it is and what it must be.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    pub meta: NodeMeta,
    /// GUID link to its [`AssetRoot`].
    pub root: Uuid,
    /// Path inside the root (forward slashes).
    pub path: String,
    pub sha256: String,
    pub size: u64,
}

macro_rules! tree_node {
    ($($t:ty => $kind:literal),* $(,)?) => {$(
        impl crate::TreeNode for $t {
            fn guid(&self) -> Uuid {
                self.meta.identity.guid
            }
            fn name(&self) -> Option<&str> {
                self.meta.name()
            }
            fn node_type(&self) -> &'static str {
                $kind
            }
        }
    )*};
}

tree_node! {
    MediaFile => "Image",
    Asset => "Asset",
}

/// A root is listed and named by its key (each station maps it by that name).
impl crate::TreeNode for AssetRoot {
    fn guid(&self) -> Uuid {
        self.meta.identity.guid
    }
    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }
    fn node_type(&self) -> &'static str {
        "AssetRoot"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picture_sizes_come_from_the_header() {
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend(640u32.to_be_bytes());
        png.extend(480u32.to_be_bytes());
        assert_eq!(image_size(&png, "png"), Some((640, 480)));
        let gif = b"GIF89a\x20\x03\x58\x02";
        assert_eq!(image_size(gif, "gif"), Some((800, 600)));
        let svg = br#"<?xml version="1.0"?><svg xmlns="x" width="24" height="12.4px">"#;
        assert_eq!(image_size(svg, "svg"), Some((24, 12)));
        let jpeg = [
            0xff, 0xd8, 0xff, 0xe0, 0, 4, 0, 0, 0xff, 0xc0, 0, 11, 8, 0x01, 0xe0, 0x02, 0x80, 3, 0, 0,
        ];
        assert_eq!(image_size(&jpeg, "jpg"), Some((640, 480)));
        assert_eq!(image_size(b"nope", "png"), None);
    }
}
