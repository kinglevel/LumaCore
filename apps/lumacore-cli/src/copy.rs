//  ██████╗ ██████╗ ██████╗ ██╗   ██╗
// ██╔════╝██╔═══██╗██╔══██╗╚██╗ ██╔╝
// ██║     ██║   ██║██████╔╝ ╚████╔╝
// ██║     ██║   ██║██╔═══╝   ╚██╔╝
// ╚██████╗╚██████╔╝██║        ██║
//  ╚═════╝ ╚═════╝ ╚═╝        ╚═╝
// FILE: copy

//! The CLI's side of a copy (`UiEvent::CopyText` from the station's TUI, the NAV
//! inspector's copy keys): the text goes on the user's clipboard of the computer
//! the CLI runs in. OSC 52 to this terminal first (it travels over SSH; tmux
//! passes it on with `set-clipboard on`), and the desktop clipboard too when the
//! CLI runs on a desktop, for terminals that don't take OSC 52. Never the
//! station's object clipboard (`Copy` / `Paste`).
//!
//! Owns:          UserClipboard (OSC 52 + the native clipboard), write_osc52
//! Doesn't own:   the encoder (`lumacore_wire::clipboard`), what is copied (the station's TUI)
//! Entry points:  UserClipboard::copy, write_osc52
//! Decisions:     docs/decisions/wire-crate-clients

use std::io::Write;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The user's clipboard for one shell session. The native clipboard opens on
/// the first copy and stays open for the session (on X11 the text lives as long
/// as its owner, or until a clipboard manager takes it).
#[derive(Default)]
pub struct UserClipboard {
    native: Option<arboard::Clipboard>,
    /// The native clipboard was tried (no desktop: never tried again).
    tried: bool,
}

/// What a copy reached.
#[derive(Debug, PartialEq, Eq)]
pub struct Copied {
    /// OSC 52 written, or why not.
    pub terminal: Result<(), String>,
    /// The desktop clipboard took it.
    pub native: bool,
}

// ██╗ ██████╗
// ██║██╔═══██╗
// ██║██║   ██║
// ██║██║   ██║
// ██║╚██████╔╝
// ╚═╝ ╚═════╝
// PART: IO

impl UserClipboard {
    /// Put `text` on the user's clipboard: OSC 52 to `out` (the terminal) and
    /// the desktop clipboard when there is one. Never fails loudly: a terminal
    /// without OSC 52 ignores the sequence, a CLI without a desktop has no
    /// native clipboard.
    pub fn copy(&mut self, out: &mut impl Write, text: &str) -> Copied {
        let terminal = write_osc52(out, text);
        if !self.tried {
            self.tried = true;
            self.native = arboard::Clipboard::new().ok();
        }
        let native = self.native.as_mut().is_some_and(|c| c.set_text(text).is_ok());
        Copied { terminal, native }
    }
}

/// Write `text` as OSC 52 to `out`; a text past the cap is refused with a
/// message that says so (`lumacore_wire::clipboard::OSC52_MAX_TEXT`).
pub fn write_osc52(out: &mut impl Write, text: &str) -> Result<(), String> {
    let seq = lumacore_wire::clipboard::osc52(text).map_err(|refused| refused.to_string())?;
    out.write_all(seq.as_bytes()).and_then(|()| out.flush()).map_err(|e| e.to_string())
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
