//  ██████╗ ██████╗ ███╗   ██╗███████╗███████╗███╗   ██╗████████╗
// ██╔════╝██╔═══██╗████╗  ██║██╔════╝██╔════╝████╗  ██║╚══██╔══╝
// ██║     ██║   ██║██╔██╗ ██║███████╗█████╗  ██╔██╗ ██║   ██║
// ██║     ██║   ██║██║╚██╗██║╚════██║██╔══╝  ██║╚██╗██║   ██║
// ╚██████╗╚██████╔╝██║ ╚████║███████║███████╗██║ ╚████║   ██║
//  ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝╚══════╝╚═╝  ╚═══╝   ╚═╝
// FILE: consent

//! `Consent`: the user accepts the Meow License and their login is finished (decisions 0290).
//! The consent dialog's "I consent" runs it; it can be typed too. Until it runs, the login's
//! gate lets through only the users family's words (Consent, Login, Logout) and refuses every
//! other line with E-USER-010.
//!
//! Owns:          the Consent verb, the pending login's gate.
//! Doesn't own:   the license text and the pending logins (`lumacore-state` consent), Login (session).
//! Entry points:  Consent, consent_gate.
//! Decisions:     docs/decisions/0290-meow-license-consent-on-every-login.

use crate::{AppState, DispatchResult, Reply};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Consent` — accept the Meow License: the login waiting here is finished, and the station
/// log says so.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Consent")]
pub struct Consent {}

impl crate::commands::typed::Command for Consent {
    fn run(self, state: &mut AppState) -> DispatchResult {
        let Some(user) = crate::consent::consent(state) else {
            return DispatchResult::Output(Reply::error("No login here waits for the Meow License.", "E-CMD-010"));
        };
        lumacore_sysmon::sysmon_note("Session", format!("{user} consented to the Meow License"));
        DispatchResult::Output(Reply::ok(crate::text::render_success(&format!("Welcome, {user}: the meow is in."))))
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The pending login's gate (a `Hook::Gate` of the users family): until the user consents, a
/// line runs only when it is one statement of the users family's words — Consent, Login, Logout.
pub(crate) fn consent_gate(state: &AppState, line: &str) -> Option<String> {
    let user = crate::consent::pending(state)?;
    if allowed_while_pending(line) {
        return None;
    }
    Some(crate::errors::render_code(
        format!("{user}, accept the Meow License first: Consent, or Logout."),
        "E-USER-010",
    ))
}

fn allowed_while_pending(line: &str) -> bool {
    use crate::commands::registry::{self, Word};
    if line.contains(';') {
        return false;
    }
    let tokens = crate::commands::tokenize(line);
    match tokens.first().and_then(|t| registry::resolve(t).ok()) {
        // By verb: a static's rows needn't sit at one address in every crate that reads them.
        Some(Word::Command(c)) => crate::commands::registry::rows::users::ROWS.iter().any(|r| r.decl.verb == c.verb),
        _ => false,
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
