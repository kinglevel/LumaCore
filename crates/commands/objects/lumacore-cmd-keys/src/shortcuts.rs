// ███████╗██╗  ██╗ ██████╗ ██████╗ ████████╗ ██████╗██╗   ██╗████████╗███████╗
// ██╔════╝██║  ██║██╔═══██╗██╔══██╗╚══██╔══╝██╔════╝██║   ██║╚══██╔══╝██╔════╝
// ███████╗███████║██║   ██║██████╔╝   ██║   ██║     ██║   ██║   ██║   ███████╗
// ╚════██║██╔══██║██║   ██║██╔══██╗   ██║   ██║     ██║   ██║   ██║   ╚════██║
// ███████║██║  ██║╚██████╔╝██║  ██║   ██║   ╚██████╗╚██████╔╝   ██║   ███████║
// ╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚═════╝    ╚═╝   ╚══════╝
// FILE: shortcuts

//! `Shortcuts [Keyboard|Programmer|Navigator|On|Off]`: cycle or set the shortcut mode.
//!
//! Owns:          Shortcuts: the shortcut mode (Keyboard, Programmer, Navigator).
//! Doesn't own:   what each mode does with a key (the terminal).
//! Entry points:  shortcuts.
//! Decisions:     §17.

use lumacore_objects::text::render_success;
use lumacore_objects::{AppState, DispatchResult, Reply};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Shortcuts — cycle or set shortcut mode (KBD/PRG/NAV).
pub(crate) fn shortcuts(state: &mut AppState, args: &[&str]) -> DispatchResult {
    use lumacore_objects::app_state::ShortcutMode;
    let new_mode = match args.first().map(|s| s.to_lowercase()).as_deref() {
        Some("keyboard") | Some("kbd") | Some("off") => ShortcutMode::Keyboard,
        Some("programmer") | Some("prg") | Some("on") => ShortcutMode::Programmer,
        Some("navigator") | Some("nav") => ShortcutMode::Navigator,
        _ => state.shortcut_mode.next(),
    };
    state.shortcut_mode = new_mode;
    DispatchResult::Output(Reply::ok(render_success(&format!(
        "Shortcuts: {}",
        new_mode.label()
    ))))
}
