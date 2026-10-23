// ██╗    ██╗ ██████╗ ██████╗ ██████╗ ███████╗
// ██║    ██║██╔═══██╗██╔══██╗██╔══██╗██╔════╝
// ██║ █╗ ██║██║   ██║██████╔╝██║  ██║███████╗
// ██║███╗██║██║   ██║██╔══██╗██║  ██║╚════██║
// ╚███╔███╔╝╚██████╔╝██║  ██║██████╔╝███████║
//  ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═╝╚═════╝ ╚══════╝
// FILE: words

//! The station's small words: Shutoff (asks, then exits), Echo (the command line and the
//! System Monitor), Debug (Admin; asks).
//!
//! Owns:          Shutoff, Echo, Debug.
//! Doesn't own:   Restart / Shutdown (lumacore-cmd-showfile cmd_power), Disconnect (the kernel's run: it shares Logout's leave).
//! Entry points:  shutoff, echo, debug.
//! Decisions:     §30 "12b".

use lumacore_objects::{AppState, DispatchResult};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Shutoff — same as shutdown but asks for confirmation first
/// (`Shutoff /NoConfirmation` shuts down at once).
pub(crate) fn shutoff(_state: &mut AppState, args: &[&str]) -> DispatchResult {
    if lumacore_objects::commands::flags::has(
        args,
        &lumacore_objects::commands::flags::NO_CONFIRMATION,
    ) {
        return DispatchResult::Exit;
    }
    DispatchResult::Ask(Box::new(
        lumacore_objects::dialogs::Ask::new("Shutoff", "Shut down the server?")
            .button(
                "Shut down",
                lumacore_wire::ButtonKind::Danger,
                "Shutoff /NoConfirmation",
            )
            .cancel("Cancel")
            .topic("power:Shutdown"),
    ))
}

/// Echo — feedback in the command line output and the System Monitor.
pub(crate) fn echo(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let text = args.join(" ");
    let text = text.trim_matches('"').trim_matches('\'').to_string();
    let user = state.effective_user_name().to_string();
    state
        .sysmon_pending
        .push(("Echo".to_string(), format!("[{user}] {text}")));
    lumacore_objects::objects::out(text)
}

/// Debug — Admin can toggle debug mode with confirmation
/// (`Debug /NoConfirmation` toggles at once).
pub(crate) fn debug(state: &mut AppState, args: &[&str]) -> DispatchResult {
    if lumacore_objects::commands::flags::has(
        args,
        &lumacore_objects::commands::flags::NO_CONFIRMATION,
    ) {
        state.debug_mode = !state.debug_mode;
        return DispatchResult::Output(lumacore_objects::Reply::ok(
            lumacore_objects::text::render_success(if state.debug_mode {
                "Debug mode enabled."
            } else {
                "Debug mode disabled."
            }),
        ));
    }
    let (title, ok) = if state.debug_mode {
        ("Disable debug mode?", "Disable")
    } else {
        ("Enable debug mode?", "Enable")
    };
    DispatchResult::Ask(Box::new(
        lumacore_objects::dialogs::Ask::new("Debug", title)
            .button(
                ok,
                lumacore_wire::ButtonKind::Primary,
                "Debug /NoConfirmation",
            )
            .cancel("Cancel"),
    ))
}
