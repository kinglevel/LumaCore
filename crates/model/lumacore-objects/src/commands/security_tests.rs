//! Step 5 security (decisions §19 "Step 5 hardening") — through real dispatch.

use super::dispatch;
use crate::AppState;
use crate::DispatchResult;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        _ => panic!("'{line}' did not produce output"),
    }
}

/// A show with a Guest "Visitor" and a Programmer "LD" (password "pw",
/// Remote Login off).
fn show() -> AppState {
    let mut state = AppState::new();
    run(&mut state, "Store User \"Visitor\" /Rights Guest");
    run(&mut state, "Store User \"LD\" /Rights Program /Password \"pw\"");
    run(&mut state, "Set User \"LD\" \"RemoteLogin\" Off");
    state
}

#[test]
fn security_one_rights_gate_for_every_line() {
    let mut state = show();
    state.user = "Visitor".into();
    let out = run(&mut state, "Store Group 1");
    assert!(out.contains("E-RIGHTS-001") && out.contains("Guest"), "{out}");
    assert!(!run(&mut state, "List User").contains("E-RIGHTS"));
    // A Guest looks around and can log in as someone else.
    for line in ["Info User", "List Root", "Help Store"] {
        let out = run(&mut state, line);
        assert!(!out.contains("E-RIGHTS"), "{line}: {out}");
    }
    assert!(run(&mut state, "Help Store").contains("Needs:  Program"));
    // Each part of a `;` line is checked.
    let out = run(&mut state, "List User; Store Group 2");
    assert!(out.contains("E-RIGHTS-001"), "{out}");
    let out = run(&mut state, "Login \"LD\" \"pw\"");
    assert!(!out.contains("Error"), "{out}");
    // The login is finished once the Meow License is accepted (decisions 0290).
    assert!(!run(&mut state, "Consent").contains("Error"));
    // Admin-only words refuse a Program user; the message names both levels.
    state.user = "LD".into();
    let out = run(&mut state, "TestAsk");
    assert!(out.contains("E-RIGHTS-001") && out.contains("needs Admin — you are Program"), "{out}");
    // The internal root user passes.
    state.user = crate::INTERNAL_ROOT_USER.into();
    assert!(!run(&mut state, "List User").contains("E-RIGHTS"));
}

#[test]
fn security_remote_login_holds_for_in_session_login() {
    let mut state = show();
    state.remote_client = true;
    let out = run(&mut state, "Login \"LD\" \"pw\"");
    assert!(out.contains("E-RIGHTS-002"), "{out}");
    assert_eq!(state.effective_user_name(), "Admin");
    // On this machine the same login works.
    state.remote_client = false;
    let out = run(&mut state, "Login \"LD\" \"pw\"");
    assert!(!out.contains("Error"), "{out}");
    // The shell's own login check refuses it from afar too.
    let refused = crate::passwords::authenticate(&state, "LD", Some("pw"), true);
    assert!(refused.unwrap_err().contains("E-RIGHTS-002"));
}

#[test]
fn security_passwords_are_masked_before_journal_undo_and_replication() {
    let mut state = AppState::new();
    state.replication_outbox.clear();
    run(&mut state, "Store User \"Op\" /Rights Program /Password \"hunter2\"");
    run(&mut state, "Set User \"Op\" \"Password\" \"hunter3\"");
    let journal: Vec<&str> = state
        .root
        .runtime
        .journal
        .entries()
        .map(|e| e.command.as_str())
        .collect();
    let journal = journal.join("\n");
    let outbox: Vec<&str> = state
        .replication_outbox
        .iter()
        .map(|op| op.command.as_str())
        .collect();
    let undo = serde_json::to_string(&state.root.show_data.user_profiles).unwrap();
    for (what, text) in [("journal", journal), ("outbox", outbox.join("\n")), ("undo", undo)] {
        assert!(
            !text.contains("hunter2") && !text.contains("hunter3"),
            "password in {what}: {text}"
        );
    }
}

#[test]
fn security_logout_ends_a_token() {
    let mut state = show();
    state.local_settings.token_secret = "station-secret".into();
    let token = crate::passwords::issue_token(&state, "Visitor").unwrap();
    assert_eq!(
        crate::passwords::verify_token(&state, &token).as_deref(),
        Some("Visitor")
    );
    assert_eq!(
        crate::passwords::revoke_token(&mut state, &token).as_deref(),
        Some("Visitor")
    );
    assert_eq!(crate::passwords::verify_token(&state, &token), None);
    // A new login — even in the same second — gets its own token that works
    // (each token carries a nonce; the revoked one stays revoked).
    let fresh = crate::passwords::issue_token(&state, "Visitor").unwrap();
    assert_ne!(fresh, token);
    assert!(crate::passwords::verify_token(&state, &fresh).is_some());
    assert_eq!(crate::passwords::verify_token(&state, &token), None);
    // A token that isn't valid changes nothing.
    assert_eq!(crate::passwords::revoke_token(&mut state, "v1.x.y"), None);
}

/// R1-46: no word shows a password hash — not `cat`, `Info`, `Dump`, `List`
/// on a user or the users, whoever asks.
#[test]
fn security_no_word_shows_a_password_hash() {
    let mut state = show();
    let hash = state.root.show_data.user("LD").unwrap().password_hash.clone().unwrap();
    let salt = hash.split('$').nth(2).unwrap().to_string();
    let no = state.root.show_data.user_no("LD").unwrap();
    for user in ["Visitor", "Admin"] {
        state.user = user.into();
        for line in [
            format!("Info Root.ShowData.Users.{no}"),
            "Info User \"LD\"".to_string(),
            "Dump User \"LD\"".to_string(),
            "List User".to_string(),
        ] {
            let out = run(&mut state, &line);
            assert!(!out.contains(&salt), "{user}: '{line}' shows the hash: {out}");
        }
        // The lines do show the user (so the test isn't empty).
        assert!(run(&mut state, "Info User \"LD\"").contains("LD"));
    }
}

/// Issue 04: Admin-only refusals carry E-RIGHTS-001 (or E-CLAIM-001 for
/// another user's claim), wherever they are checked.
#[test]
fn security_admin_refusals_carry_the_rights_code() {
    let mut state = show();
    run(&mut state, "Claim Root.ShowData.Devices");
    state.user = "LD".into();
    let out = run(&mut state, "Store User \"X\"");
    assert!(out.contains("E-RIGHTS-001"), "{out}");
    // Another user's claim is Admin's: an ownership rule through
    // `rights::require` (decisions §30 "12d"), so the rights code.
    let out = run(&mut state, "Unclaim Root.ShowData.Devices");
    assert!(out.contains("E-RIGHTS-001"), "{out}");
}
