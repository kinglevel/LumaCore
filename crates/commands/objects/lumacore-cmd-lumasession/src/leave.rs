// ██╗     ███████╗ █████╗ ██╗   ██╗███████╗
// ██║     ██╔════╝██╔══██╗██║   ██║██╔════╝
// ██║     █████╗  ███████║██║   ██║█████╗
// ██║     ██╔══╝  ██╔══██║╚██╗ ██╔╝██╔══╝
// ███████╗███████╗██║  ██║ ╚████╔╝ ███████╗
// ╚══════╝╚══════╝╚═╝  ╚═╝  ╚═══╝  ╚══════╝
// FILE: leave

//! `Leave LumaSession [/NoConfirmation]`: a member asks first; a fenced master's Cancel doesn't.
//!
//! Owns:          Leave LumaSession's ask (leaving holds this station's DMX output).
//! Doesn't own:   leaving itself (the kernel's session::leave_lumasession).
//! Entry points:  cmd_leave_lumasession.
//! Decisions:     §30 "12d".

use lumacore_objects::AppState;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// `Leave LumaSession`: leave the master; DMX output is held until the
/// operator resumes it.
/// `Leave LumaSession [/NoConfirmation]`: a member asks first (decisions
/// §30 "12d": leaving holds DMX output and is a station ask); a fenced
/// master's / IdleMaster's Cancel doesn't (its decision was the ask).
pub fn cmd_leave_lumasession(
    state: &mut AppState,
    args: &[&str],
) -> lumacore_objects::DispatchResult {
    let confirmed = lumacore_objects::commands::flags::has(
        args,
        &lumacore_objects::commands::flags::NO_CONFIRMATION,
    );
    if !confirmed && let Some(link) = &state.link {
        let master = link.master.clone();
        return lumacore_objects::DispatchResult::Ask(Box::new(
            lumacore_objects::dialogs::Ask::new(
                "Leave the LumaSession",
                format!("This station leaves {master}'s LumaSession and holds its DMX output until you resume it (Set Station \"HoldOutput\" Off)."),
            )
            .meta("this station", Some(lumacore_wire::Tone::Warn))
            .button("Leave", lumacore_wire::ButtonKind::Danger, "Leave LumaSession /NoConfirmation")
            .cancel("Cancel")
            .flags("/NoConfirmation"),
        ));
    }
    lumacore_objects::DispatchResult::Output(
        (lumacore_objects::commands::session::leave_lumasession(state)).into(),
    )
}
