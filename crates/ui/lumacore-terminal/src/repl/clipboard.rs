//  ██████╗██╗     ██╗██████╗ ██████╗  ██████╗  █████╗ ██████╗ ██████╗
// ██╔════╝██║     ██║██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██╔══██╗██╔══██╗
// ██║     ██║     ██║██████╔╝██████╔╝██║   ██║███████║██████╔╝██║  ██║
// ██║     ██║     ██║██╔═══╝ ██╔══██╗██║   ██║██╔══██║██╔══██╗██║  ██║
// ╚██████╗███████╗██║██║     ██████╔╝╚██████╔╝██║  ██║██║  ██║██████╔╝
//  ╚═════╝╚══════╝╚═╝╚═╝     ╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝
// FILE: clipboard

//! The one door to the user's clipboard from the TUI: a GUID, a path, an object
//! target, a name or a Set line the NAV inspector copies goes to the computer of
//! the client that owns this TUI, as one `UiEvent::CopyText` beside the
//! session's other UI events. The CLI writes it as OSC 52 (and sets the desktop
//! clipboard when it has one), the page with the browser's clipboard, the app
//! natively; the station's own console writes OSC 52 here. Never the station's
//! object clipboard (`Copy` / `Cut` / `Paste` of show objects,
//! `Runtime.Users.{User}.clipboard`).
//!
//! Owns:          copy (queue a text for the user's clipboard), the console's OSC 52 write
//! Doesn't own:   what the inspector copies (the NAV inspector), the encoder
//!                (`lumacore_wire::clipboard`), the clients' writes (lumacore-cli,
//!                index.html, lumacore-app)
//! Entry points:  copy, write_to_own_terminal
//! Decisions:     docs/decisions/wire-crate-clients, docs/decisions/app-thin-client

use std::io::Write;

use lumacore_wire::UiEvent;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Queue `text` for the user's clipboard; the client says "Copied `label`"
/// (`label`: what it is, `GUID`, `Path`, `Set line`). `outbox` is where the TUI
/// queues its UI events: `TuiApp.ui_outbox` in key handling, or
/// `AppState.ui_requests`; both reach the client with the key's reply. An empty
/// text queues nothing.
pub fn copy(outbox: &mut Vec<UiEvent>, label: &str, text: &str) {
    if text.is_empty() {
        return;
    }
    outbox.push(UiEvent::CopyText { label: label.to_string(), text: text.to_string() });
}

// ██╗ ██████╗
// ██║██╔═══██╗
// ██║██║   ██║
// ██║██║   ██║
// ██║╚██████╔╝
// ╚═╝ ╚═════╝
// PART: IO

/// The station's own console (`run_repl`) is its own client: it takes the queued
/// copies out of `queues` and writes each as OSC 52 to `out`, its terminal. The
/// other events stay where they are. Returns what to tell the user: a line per
/// copy, "Copied …" or why the terminal can't take it.
pub fn write_to_own_terminal(queues: [&mut Vec<UiEvent>; 2], out: &mut impl Write) -> Vec<String> {
    let mut said = Vec::new();
    for queue in queues {
        queue.retain(|event| {
            let UiEvent::CopyText { label, text } = event else {
                return true;
            };
            said.push(match lumacore_wire::clipboard::osc52(text) {
                Ok(seq) if out.write_all(seq.as_bytes()).and_then(|()| out.flush()).is_ok() => format!("Copied {label}"),
                Ok(_) => format!("Couldn't copy {label}: the terminal didn't take it"),
                Err(refused) => format!("Couldn't copy {label}: {refused}"),
            });
            false
        });
    }
    said
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
