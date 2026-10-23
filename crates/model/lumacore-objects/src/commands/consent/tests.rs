// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The Meow License on every login (decisions 0290), through real dispatch on the rig: a login
//! waits for the consent, its lines are refused until then, `Consent` finishes it and the log
//! says so, `Logout` drops it, and a holder's wait is its own.
//!
//! Owns:          the tests of Consent and the pending login's gate.
//! Doesn't own:   the e2e flow (tests/e2e login-sysmon.spec.ts).
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/0290-meow-license-consent-on-every-login.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use crate::commands::dispatch;
use crate::dialogs::{self, Audience, Who};
use crate::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        DispatchResult::Logout => "LOGOUT".to_string(),
        DispatchResult::Ask(a) => format!("ASK {}", a.title()),
        _ => panic!("'{line}' did not produce output"),
    }
}

/// The rig, with a user of its own to log in as (no password).
fn rig() -> AppState {
    let mut state = crate::demo::fresh_state();
    let out = run(&mut state, "Store User \"Purrl\" /Rights Program");
    assert!(!out.contains("Error"), "{out}");
    state
}

fn shown(state: &AppState) -> Vec<lumacore_wire::Dialog> {
    dialogs::for_client(state, &Who::current(state)).0
}

#[test]
fn a_login_waits_for_the_meow_license_and_consent_finishes_it() {
    let mut state = rig();
    let out = run(&mut state, "Login \"Purrl\"");
    assert!(out.contains("Logged in as Purrl"), "{out}");
    assert_eq!(crate::consent::pending(&state).as_deref(), Some("Purrl"));

    // The one dialog, first, with the text from its one door.
    let d = shown(&state).into_iter().next().expect("the consent dialog");
    assert_eq!(d.title, crate::consent::TITLE);
    assert_eq!(d.topic.as_deref(), Some(crate::consent::TOPIC));
    assert!(d.text.starts_with(crate::consent::HEADING) && d.text.ends_with(crate::consent::BODY));
    assert_eq!(d.details[0].k, crate::consent::WARNING);
    assert_eq!(d.details[0].tone, Some(lumacore_wire::Tone::Alert));
    assert_eq!(d.buttons.iter().map(|b| b.label.as_str()).collect::<Vec<_>>(), [crate::consent::CONSENT_BUTTON, crate::consent::NOT_ME]);
    assert_eq!(d.note.as_deref(), Some(crate::consent::FINE_PRINT));
    let ask = crate::consent::ask("Purrl");
    assert_eq!(ask.line_of(crate::consent::CONSENT_BUTTON).as_deref(), Some("Consent"));
    assert_eq!(ask.line_of(crate::consent::NOT_ME).as_deref(), Some("Logout"));

    // Every other line is refused and changes nothing; a `;` line too.
    for line in ["Fixture 1 At 50", "List User", "Consent; Fixture 1 At 50"] {
        let out = run(&mut state, line);
        assert!(out.contains("E-USER-010") && out.contains("Meow License"), "{line}: {out}");
    }
    assert!(state.current_programmer().is_none_or(|p| p.selected_fixtures.is_empty()));

    // Consent finishes the login, closes the dialog and writes the log line.
    let out = run(&mut state, "Consent");
    assert!(out.contains("Purrl") && !out.contains("Error"), "{out}");
    assert_eq!(crate::consent::pending(&state), None);
    assert!(shown(&state).iter().all(|d| d.topic.as_deref() != Some(crate::consent::TOPIC)));
    assert!(
        lumacore_sysmon::sysmon_history_snapshot().iter().any(|l| l.contains("Purrl consented to the Meow License")),
        "the station log notes the consent"
    );
    assert!(!run(&mut state, "List User").contains("E-USER-010"));
    // Nothing waits any more.
    assert!(run(&mut state, "Consent").contains("E-CMD-010"));
}

#[test]
fn not_me_logs_out_and_drops_the_waiting_login() {
    let mut state = rig();
    run(&mut state, "Login \"Purrl\"");
    assert_eq!(run(&mut state, "Logout"), "LOGOUT");
    assert_eq!(crate::consent::pending(&state), None);
    assert!(shown(&state).iter().all(|d| d.topic.as_deref() != Some(crate::consent::TOPIC)));
}

#[test]
fn a_login_while_waiting_asks_again_for_the_new_user() {
    let mut state = rig();
    run(&mut state, "Login \"Purrl\"");
    let out = run(&mut state, "Login \"Admin\" \"admin\"");
    assert!(out.contains("Logged in as Admin"), "{out}");
    assert_eq!(crate::consent::pending(&state).as_deref(), Some("Admin"));
    let consents = shown(&state).into_iter().filter(|d| d.topic.as_deref() == Some(crate::consent::TOPIC)).count();
    assert_eq!(consents, 1, "one dialog, the new login's");
}

#[test]
fn each_holder_waits_on_its_own_and_its_token_with_it() {
    let mut state = rig();
    let token = crate::passwords::issue_token(&state, "Purrl").unwrap();
    crate::consent::await_token(&mut state, &token);
    assert!(crate::consent::token_waits(&state, &token));
    assert!(crate::consent::token_pending(&state.consent.token_signatures(), &token));

    // A shell client of its own logged in with the token: the station's console runs on.
    let client = uuid::Uuid::new_v4();
    crate::consent::begin(&mut state, Audience::Client(client), "Purrl", Some(&token));
    assert!(!run(&mut state, "List User").contains("E-USER-010"));

    // The client's lines wait; its consent finishes the token too.
    state.current_client = Some(client);
    state.acting_user = Some("Purrl".into());
    assert!(run(&mut state, "List User").contains("E-USER-010"));
    assert!(!run(&mut state, "Consent").contains("Error"));
    assert!(!crate::consent::token_waits(&state, &token));

    // A revoked token waits no more either.
    let other = crate::passwords::issue_token(&state, "Purrl").unwrap();
    crate::consent::await_token(&mut state, &other);
    crate::passwords::revoke_token(&mut state, &other);
    assert!(!crate::consent::token_waits(&state, &other));
}
