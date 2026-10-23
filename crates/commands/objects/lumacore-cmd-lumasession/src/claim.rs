//  ██████╗██╗      █████╗ ██╗███╗   ███╗
// ██╔════╝██║     ██╔══██╗██║████╗ ████║
// ██║     ██║     ███████║██║██╔████╔██║
// ██║     ██║     ██╔══██║██║██║╚██╔╝██║
// ╚██████╗███████╗██║  ██║██║██║ ╚═╝ ██║
//  ╚═════╝╚══════╝╚═╝  ╚═╝╚═╝╚═╝     ╚═╝
// FILE: claim

//! `Claim LumaSession [Station "X"]` asks, then queues the hand-over (decisions §21 "6c");
//! `Pause LumaSession` stops a decision's countdown.
//!
//! Owns:          Claim LumaSession (take the master role, or hand it to a member) and Pause LumaSession.
//! Doesn't own:   the hand-over itself (lumacore-lumasession election, the outbox).
//! Entry points:  cmd_claim, cmd_pause.
//! Decisions:     §21 "6c", §30 "12d".

use lumacore_objects::AppState;
use lumacore_objects::session_link::{Role, SessionRequest};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// `Claim LumaSession [Station "X"] [/NoConfirmation]`: take the master
/// role, or — on the master — hand it to member X. Both ask first
/// (decisions §30 "12d": claiming the master is a station ask).
pub fn cmd_claim(state: &mut AppState, args: &[&str]) -> lumacore_objects::DispatchResult {
    use lumacore_objects::DispatchResult;
    use lumacore_objects::errors::render_code;
    use lumacore_objects::text::render_success;
    let confirmed = lumacore_objects::commands::flags::has(
        args,
        &lumacore_objects::commands::flags::NO_CONFIRMATION,
    );
    let words: Vec<&str> = args
        .iter()
        .copied()
        .filter(|a| !a.starts_with('/'))
        .collect();
    let ask = |title: String, text: String, button: &str, line: String| {
        DispatchResult::Ask(Box::new(
            lumacore_objects::dialogs::Ask::new(title, text)
                .meta("LumaSession", Some(lumacore_wire::Tone::Warn))
                .button(button, lumacore_wire::ButtonKind::Danger, line)
                .cancel("Cancel")
                .flags("/NoConfirmation"),
        ))
    };
    match words.as_slice() {
        [] => {}
        [word, name]
            if lumacore_objects::objects::find(word)
                .is_some_and(|o| std::ptr::eq(o, &lumacore_cmd_station::STATION)) =>
        {
            let name = name.trim_matches('"').to_string();
            let me = state.root.station_settings.station_name.clone();
            if name.eq_ignore_ascii_case(&me) {
                return cmd_claim(
                    state,
                    &args
                        .iter()
                        .copied()
                        .filter(|a| a.starts_with('/'))
                        .collect::<Vec<_>>(),
                );
            }
            if let Some(link) = &state.link {
                return DispatchResult::Output(lumacore_objects::Reply::error(
                    format!(
                        "Only the master ({}) hands the role to another station; this station can take it: Claim LumaSession.",
                        link.master
                    ),
                    "E-CMD-010",
                ));
            }
            let members = &state.root.runtime.luma_sessions.members;
            let Some(member) = members
                .values()
                .find(|m| m.station.eq_ignore_ascii_case(&name))
                .map(|m| m.station.clone())
            else {
                return DispatchResult::Output(if members.is_empty() {
                    (render_code("This station isn't a LumaSession master.", "E-CMD-010")).into()
                } else {
                    (render_code(
                        format!("{name} isn't a member of this LumaSession (List LumaSession)."),
                        "E-CMD-007",
                    ))
                    .into()
                });
            };
            if !confirmed {
                return ask(
                    format!("Hand the master role to {member}"),
                    format!(
                        "{member} becomes the master of this LumaSession (a new epoch); {me} and every other member follow it. DMX is sent by {member} from then on."
                    ),
                    "Hand over",
                    format!(
                        "Claim LumaSession Station {} /NoConfirmation",
                        lumacore_objects::stations::word(&member)
                    ),
                );
            }
            state
                .session_requests
                .push(SessionRequest::HandTo(member.clone()));
            return DispatchResult::Output(
                (render_success(&format!("Handing the master role to {member}…"))).into(),
            );
        }
        _ => {
            return DispatchResult::Output(lumacore_objects::Reply::error(
                "Usage: Claim LumaSession [Station \"name\"] [/NoConfirmation]",
                "E-CMD-006",
            ));
        }
    }
    if let Some(link) = &state.link
        && !confirmed
    {
        let master = link.master.clone();
        let me = state.root.station_settings.station_name.clone();
        return ask(
            "Take the master role".to_string(),
            format!(
                "{me} asks {master} to hand the master role over: {me} becomes the master (a new epoch) and sends DMX; every other station follows it."
            ),
            "Take master",
            "Claim LumaSession /NoConfirmation".to_string(),
        );
    }
    DispatchResult::Output((claim_now(state)).into())
}

fn claim_now(state: &mut AppState) -> String {
    use lumacore_objects::errors::render_code;
    use lumacore_objects::text::render_success;
    if let Some(link) = &state.link {
        // Setup may take the master role (declared, `rights::setup`;
        // decisions §30 "12d": the station-priority bypass for lower levels
        // is gone — the level decides).
        state.session_requests.push(SessionRequest::HandOver);
        return render_success(&format!(
            "Asking {} to hand over the master role…",
            link.master
        ));
    }
    match &state.session_role {
        Role::Fenced { .. } | Role::IdleMaster { .. } => {
            state.session_requests.push(SessionRequest::UseMine);
            render_success("Use Mine: this station becomes master; the other side takes this show.")
        }
        Role::Normal if !state.root.runtime.luma_sessions.members.is_empty() => {
            render_success("This station is already the master.")
        }
        Role::Normal => render_code("This station isn't in a LumaSession.", "E-CMD-010"),
    }
}

/// `Pause LumaSession`: stop the decision's countdown (like a tap on the pop-up).
pub fn cmd_pause(state: &mut AppState) -> String {
    use lumacore_objects::errors::render_code;
    use lumacore_objects::text::render_success;
    match &mut state.decision {
        Some(d) if d.deadline_ms.is_some() => {
            d.deadline_ms = None;
            d.paused = true;
            render_success("The decision waits for you now (countdown paused).")
        }
        Some(_) => render_success("The decision has no countdown."),
        None => render_code("There is no LumaSession decision to pause.", "E-CMD-009"),
    }
}
