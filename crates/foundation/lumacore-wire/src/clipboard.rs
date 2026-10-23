//  ██████╗██╗     ██╗██████╗ ██████╗  ██████╗  █████╗ ██████╗ ██████╗
// ██╔════╝██║     ██║██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██╔══██╗██╔══██╗
// ██║     ██║     ██║██████╔╝██████╔╝██║   ██║███████║██████╔╝██║  ██║
// ██║     ██║     ██║██╔═══╝ ██╔══██╗██║   ██║██╔══██║██╔══██╗██║  ██║
// ╚██████╗███████╗██║██║     ██████╔╝╚██████╔╝██║  ██║██║  ██║██████╔╝
//  ╚═════╝╚══════╝╚═╝╚═╝     ╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝
// FILE: clipboard

//! The user's clipboard, as a terminal reaches it: the OSC 52 escape sequence a
//! terminal client writes to its own terminal when the station sends
//! [`crate::UiEvent::CopyText`] (a GUID, a path, a Set line copied in the NAV
//! inspector). Nothing here is the station's object clipboard (`Copy` / `Paste`
//! of show objects, decisions clipboard-of-live-guids): that one never leaves the
//! station.
//!
//! Owns:          the OSC 52 encoder, its length cap and its refusal
//! Doesn't own:   which text is copied (the TUI's `repl::clipboard::copy`), the
//!                page's and the app's clipboard writes (index.html, lumacore-app)
//! Entry points:  osc52, OSC52_MAX_TEXT, CopyRefused
//! Decisions:     docs/decisions/wire-crate-clients

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The longest text (bytes of UTF-8) sent through OSC 52: its base64 is
/// 100 000 bytes, the most common terminals take in one sequence.
pub const OSC52_MAX_TEXT: usize = 75_000;

/// Why a text can't go through the terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopyRefused {
    /// Nothing to copy.
    Empty,
    /// Longer than [`OSC52_MAX_TEXT`].
    TooLong { len: usize, max: usize },
}

impl std::fmt::Display for CopyRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CopyRefused::Empty => write!(f, "Nothing to copy"),
            CopyRefused::TooLong { len, max } => write!(
                f,
                "Too long to copy through the terminal ({len} bytes; the terminal takes {max})"
            ),
        }
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The OSC 52 sequence that puts `text` on the clipboard of the terminal it is
/// written to (`ESC ] 52 ; c ; <base64> BEL`). It travels over SSH; tmux passes
/// it on with `set-clipboard on`.
pub fn osc52(text: &str) -> Result<String, CopyRefused> {
    if text.is_empty() {
        return Err(CopyRefused::Empty);
    }
    if text.len() > OSC52_MAX_TEXT {
        return Err(CopyRefused::TooLong { len: text.len(), max: OSC52_MAX_TEXT });
    }
    Ok(format!("\x1b]52;c;{}\x07", base64(text.as_bytes())))
}

/// Standard base64 (RFC 4648, with padding).
pub fn base64(bytes: &[u8]) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], chunk.get(1).copied().unwrap_or(0), chunk.get(2).copied().unwrap_or(0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for (i, shift) in [18u32, 12, 6, 0].into_iter().enumerate() {
            if i <= chunk.len() {
                out.push(ABC[((n >> shift) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
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
