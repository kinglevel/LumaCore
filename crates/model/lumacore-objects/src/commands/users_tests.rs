//! Step 12c2 (decisions §30 "12c2 hardening / design locked"): the locked
//! template Default, new users and profiles from station asks, the safe
//! user delete (ask, logout, claims, one Oops step), rights only from the
//! user, renames, the shared hold-full-screen and the device's screen config.

use super::dispatch;
use crate::{AppState, DispatchResult};
use lumacore_tree::UserRights;
use crate::claims::Claims;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        DispatchResult::Ask(a) => format!("ASK {}", a.title()),
        _ => panic!("'{line}' did not produce output"),
    }
}

fn ask(state: &mut AppState, line: &str) -> crate::dialogs::Ask {
    match dispatch(state, line) {
        DispatchResult::Ask(a) => *a,
        DispatchResult::Output(s) => panic!("'{line}' didn't ask: {}", crate::text::strip_ansi(&s)),
        _ => panic!("'{line}' didn't ask"),
    }
}

fn as_user(state: &mut AppState, user: &str) {
    state.user = user.to_string();
}

fn profile_no(state: &AppState, name: &str) -> Option<u32> {
    state
        .root
        .show_data
        .user_profiles
        .iter()
        .find(|(_, p)| p.meta.identity.name.as_deref() == Some(name))
        .map(|(n, _)| *n)
}

fn default_json(state: &AppState) -> String {
    serde_json::to_string(state.root.show_data.default_profile().unwrap().1).unwrap()
}

#[test]
fn default_is_a_locked_template_nobody_holds() {
    let mut state = AppState::new();
    let (no, p) = state.root.show_data.default_profile().expect("every show has Default");
    assert_eq!((no, p.meta.identity.name.as_deref(), p.locked), (1, Some("Default"), true));
    let before = default_json(&state);
    for line in [
        "Set UserProfile 1 Theme \"Base\" Night",
        "Set UserProfile \"Default\" \"MeowKey\" Alt",
        "Set UserProfile 1 \"HoldFollows\" Off",
        "Label UserProfile 1 \"Mine\"",
        "Delete UserProfile 1 /NoConfirmation",
        "Delete UserProfile \"Default\"",
    ] {
        let out = run(&mut state, line);
        assert!(out.contains("E-USER-002"), "{line}: {out}");
    }
    assert_eq!(default_json(&state), before, "Default never changes");
    // Nobody works in it.
    assert!(run(&mut state, "Store User \"Ann\" /Profile \"Default\"").contains("E-USER-008"));
    assert!(state.root.show_data.user("Ann").is_none());
    run(&mut state, "Store User \"Bob\"");
    assert!(run(&mut state, "Set User \"Bob\" \"Profile\" \"Default\"").contains("E-USER-008"));
    assert!(run(&mut state, "Set User \"Bob\" \"Profile\" 1").contains("E-USER-008"));
    // The pick lists never offer it.
    let a = ask(&mut state, "Store User");
    let lumacore_wire::InputKind::Link { pick } = &a.view().inputs[1].kind else { panic!("a pick list") };
    assert_eq!(pick.items[0].name, "Own — a copy of Default");
    assert!(pick.items.iter().all(|i| i.name != "Default"), "{:?}", pick.items);
}

#[test]
fn a_new_user_gets_their_own_copy_of_default_or_shares_one() {
    let mut state = AppState::new();
    let out = run(&mut state, "Store User \"Anna\" /Rights Program /Password \"pw\" /Colour Teal");
    assert!(out.contains("a copy of Default"), "{out}");
    let show = &state.root.show_data;
    let anna = show.user("Anna").unwrap();
    let (pno, p) = show.profile_of(anna).unwrap();
    assert_eq!((p.meta.identity.name.as_deref(), p.locked), (Some("Anna"), false));
    assert_ne!(p.meta.identity.guid, show.default_profile().unwrap().1.meta.identity.guid, "a copy: new GUID");
    assert_eq!(anna.color, "#3f9a90");
    assert!(anna.password_hash.is_some());
    // The same name taken: the next free one.
    run(&mut state, "Store UserProfile \"Bob\"");
    run(&mut state, "Store User \"Bob\" /Profile New");
    let bob = state.root.show_data.user("Bob").unwrap();
    assert_eq!(state.root.show_data.profile_of(bob).unwrap().1.meta.identity.name.as_deref(), Some("Bob 2"));
    // Sharing.
    run(&mut state, &format!("Store User \"Phone\" /Profile {pno}"));
    assert_eq!(state.root.show_data.user("Phone").unwrap().profile, state.root.show_data.user("Anna").unwrap().profile);
    assert!(run(&mut state, "Store User \"anna\"").contains("E-USER-007"), "names are unique");
    assert!(state.root.show_data.identity().duplicates.is_empty());
    // Only an Admin (the rights gate, from the registry).
    as_user(&mut state, "Anna");
    let out = run(&mut state, "Store User \"X\"");
    assert!(out.contains("E-RIGHTS-001"), "{out}");
    assert!(run(&mut state, "Delete User \"Phone\" /NoConfirmation").contains("E-RIGHTS-001"));
    assert!(run(&mut state, "Store UserProfile \"Y\"").contains("E-RIGHTS-001"));
}

#[test]
fn a_bare_store_user_asks_for_everything() {
    let mut state = AppState::new();
    let a = ask(&mut state, "Store User");
    let names: Vec<&str> = a.view().inputs.iter().map(|i| i.name.as_str()).collect();
    assert_eq!(names, ["name", "profile", "password", "colour"]);
    assert_eq!(a.view().selectors[0].value, "Program");
    // The level pick: the six levels, lowest first (decisions §30 "12d").
    let options: Vec<&str> = a.view().selectors[0].choices.iter().map(String::as_str).collect();
    assert_eq!(options, ["Guest", "Playback", "Presets", "Program", "Setup", "Admin"]);
    assert!(matches!(a.view().inputs[2].kind, lumacore_wire::InputKind::Password));
    let line = a.primary_line().unwrap();
    assert_eq!(line, "Store User \"User 1\" /Rights Program /Profile New /Password \"\" /Colour Slate");
    let out = run(&mut state, &line);
    assert!(out.contains("User 1"), "{out}");
    let u = state.root.show_data.user("User 1").unwrap();
    assert!(u.password_hash.is_none(), "an empty password is none");
    assert_eq!(state.root.show_data.profile_of(u).unwrap().1.meta.identity.name.as_deref(), Some("User 1"));
}

#[test]
fn profiles_are_made_from_default_a_copy_or_a_file_never_blank() {
    let mut state = AppState::new();
    run(&mut state, "Set UserProfile \"Admin\" \"MeowKey\" Alt");
    let out = run(&mut state, "Store UserProfile \"Stage\"");
    assert!(out.contains("a copy of \"Default\""), "{out}");
    run(&mut state, "Store UserProfile \"Stage\"");
    assert!(profile_no(&state, "Stage 2").is_some(), "duplicate names get the next free");
    run(&mut state, "Store UserProfile \"Desk\" /CopyOf \"Admin\"");
    let desk = profile_no(&state, "Desk").unwrap();
    assert_eq!(state.root.show_data.user_profiles[&desk].settings.preferences.meow_key, "Alt");
    // Copy … At a number or a name (it used to answer E-CMD-006).
    let out = run(&mut state, "Copy UserProfile \"Admin\" At 20");
    assert!(out.contains("UserProfile 20 \"Admin 2\""), "{out}");
    let out = run(&mut state, "Copy UserProfile 2 At \"Show\"");
    assert!(out.contains("\"Show\""), "{out}");
    assert!(run(&mut state, "Copy UserProfile 2 At 20").contains("E-CMD-010"));
    let ids = state.root.show_data.identity();
    assert!(ids.duplicates.is_empty(), "copies get new GUIDs: {:?}", ids.duplicates);
    // The asks.
    let a = ask(&mut state, "Store UserProfile");
    assert_eq!(a.view().selectors[0].choices, ["From Default", "Copy of…", "Import file…"]);
    assert_eq!(a.primary_line().as_deref(), Some("Store UserProfile \"Profile\""));
    let a = ask(&mut state, "Store UserProfile \"X\" /CopyOf");
    assert!(a.primary_line().unwrap().starts_with("Store UserProfile \"X\" /CopyOf "));
    // Rename: bare asks, a taken name is refused.
    let a = ask(&mut state, &format!("Label UserProfile {desk}"));
    assert_eq!(a.primary_line(), Some(format!("Label UserProfile {desk} \"Desk\"")));
    assert!(run(&mut state, &format!("Label UserProfile {desk} \"Stage\"")).contains("E-USER-007"));
    run(&mut state, &format!("Label UserProfile {desk} \"Front\""));
    assert!(profile_no(&state, "Front").is_some());
}

#[test]
fn a_profile_file_imports_as_a_named_copy() {
    let mut state = AppState::new();
    let out = run(&mut state, "Export UserProfile \"Admin\" \"t12c2-admin\"");
    assert!(out.contains("t12c2-admin.lumaprofile"), "{out}");
    let out = run(&mut state, "Import UserProfile \"t12c2-admin\" /As \"Stage\"");
    assert!(out.contains("a copy, new GUIDs"), "{out}");
    let stage = profile_no(&state, "Stage").unwrap();
    assert_ne!(state.root.show_data.user_profiles[&stage].meta.identity.guid, state.root.show_data.user_profiles[&2].meta.identity.guid);
    assert!(state.root.show_data.identity().duplicates.is_empty());
    // A bare Import asks for the file: the files element (decisions §38 5), its pick `<drive>:<file>`
    // rendered as `"file" [/Drive "usb1"]`.
    let a = ask(&mut state, "Import UserProfile /As \"Desk\"");
    assert!(matches!(a.view().inputs[0].kind, lumacore_wire::InputKind::File { ref files } if files == "userprofiles"));
    let mut answer = lumacore_wire::DialogAnswer::default();
    answer.inputs.insert("file".into(), "usb1:t12c2-admin".into());
    answer.inputs.insert("name".into(), "Desk".into());
    assert_eq!(a.line_for(a.view().primary, &answer).unwrap().unwrap(), "Import UserProfile \"t12c2-admin\" /Drive \"usb1\" /As \"Desk\"");
    answer.inputs.insert("file".into(), "local-library:t12c2-admin".into());
    assert_eq!(a.line_for(a.view().primary, &answer).unwrap().unwrap(), "Import UserProfile \"t12c2-admin\" /As \"Desk\"");
    answer.inputs.insert("file".into(), String::new());
    assert!(a.line_for(a.view().primary, &answer).unwrap_err().contains("Which file?"));
}

/// A user with work: a programmer value, a selection and a claim.
fn busy_ld() -> AppState {
    let mut state = super::coverage_tests::with_fixture();
    run(&mut state, "Store User \"LD\" /Rights Program");
    as_user(&mut state, "LD");
    run(&mut state, "Fixture 1 At 50");
    run(&mut state, "Fixture 1");
    let out = run(&mut state, "Claim Fixture 1");
    assert!(out.contains("Claimed"), "{out}");
    state.root.runtime.user("LD").attached = true;
    as_user(&mut state, "Admin");
    state
}

#[test]
fn deleting_a_user_asks_what_goes() {
    let mut state = busy_ld();
    let a = ask(&mut state, "Delete User \"LD\"");
    let v = a.view();
    assert_eq!(v.title, "Delete User \"LD\"");
    assert!(v.text.contains("LD is logged in"), "{}", v.text);
    let rows: Vec<(String, String)> = v.rows.iter().map(|r| (r.k.clone(), r.v.clone())).collect();
    assert!(rows.iter().any(|(k, v)| k == "Logged in" && v.contains("logged out now")), "{rows:?}");
    assert!(rows.iter().any(|(k, v)| k == "Programmer" && v.starts_with("1 value")), "{rows:?}");
    assert!(rows.iter().any(|(k, v)| k == "Selection" && v.starts_with("1 fixture")), "{rows:?}");
    assert!(rows.iter().any(|(k, v)| k == "Claims" && v.contains("released")), "{rows:?}");
    assert_eq!(v.toggles[0].label, "Also delete profile \"LD\" (nobody else uses it)");
    assert_eq!(a.line_of("Delete LD").as_deref(), Some("Delete User \"LD\" /WithProfile /NoConfirmation"));
    assert!(state.root.show_data.user("LD").is_some(), "nothing before the answer");
}

#[test]
fn a_deleted_user_is_logged_out_their_claims_released_and_oops_brings_them_back() {
    let mut state = busy_ld();
    state.local_settings.token_secret = "t12c2-secret".into();
    let token = crate::passwords::issue_token(&state, "LD").unwrap();
    assert!(crate::passwords::verify_token(&state, &token).is_some());
    let profiles = state.root.show_data.user_profiles.len();
    let out = run(&mut state, "Delete User \"LD\" /WithProfile /NoConfirmation");
    assert!(out.contains("Deleted user") && out.contains("deleted"), "{out}");
    let show = &state.root.show_data;
    assert!(show.user("LD").is_none());
    assert_eq!(show.user_profiles.len(), profiles - 1, "the profile went too");
    assert!(state.root.runtime.claims.values().all(|c| c.user != "LD"), "claims released");
    assert!(!state.root.runtime.users.contains_key("LD"), "runtime ended");
    assert!(crate::passwords::verify_token(&state, &token).is_none());
    // One Oops step: user, programmer, selection and profile — not the session.
    assert!(run(&mut state, "Oops").contains("Undone"));
    let show = &state.root.show_data;
    let ld = show.user("LD").expect("back");
    assert_eq!(ld.programmer.values.len(), 1);
    assert_eq!(ld.programmer.selected_fixtures.len(), 1);
    assert_eq!(show.profile_of(ld).unwrap().1.meta.identity.name.as_deref(), Some("LD"));
    assert_eq!(show.user_profiles.len(), profiles);
    assert!(state.root.runtime.claims.values().all(|c| c.user != "LD"), "claims stay released");
    assert!(crate::passwords::verify_token(&state, &token).is_none(), "old tokens stay ended");
}

#[test]
fn deleting_refuses_admin_yourself_the_last_admin_and_shared_profiles_stay() {
    let mut state = AppState::new();
    assert!(run(&mut state, "Delete User \"Admin\" /NoConfirmation").contains("E-USER-005"));
    as_user(&mut state, "Debug");
    assert!(run(&mut state, "Delete User \"Debug\" /NoConfirmation").contains("E-USER-003"));
    as_user(&mut state, "Admin");
    // The user Admin is always Admin (decisions §30 "12d").
    assert!(run(&mut state, "Set User \"Admin\" \"Rights\" Program").contains("E-USER-005"));
    // The last-Admin rule stays as a backstop (a show whose Admin user lost
    // the level some other way).
    state.root.show_data.user_mut("Admin").unwrap().rights = UserRights::Program;
    state.switch_to_internal_root();
    assert!(run(&mut state, "Delete User \"Debug\" /NoConfirmation").contains("E-USER-004"));
    assert!(run(&mut state, "Set User \"Debug\" \"Rights\" Guest").contains("E-USER-004"));
    let root = crate::INTERNAL_ROOT_USER;
    assert!(run(&mut state, &format!("Delete User \"{root}\"")).contains("E-USER-009"));
    // A shared profile: no toggle, and /WithProfile keeps it.
    as_user(&mut state, "Debug");
    run(&mut state, "Store User \"A\"");
    run(&mut state, "Store User \"B\" /Profile \"A\"");
    let a = ask(&mut state, "Delete User \"A\"");
    assert!(a.view().toggles.is_empty());
    assert!(a.view().rows.iter().any(|r| r.k == "Profile" && r.v.contains("used by B")));
    run(&mut state, "Delete User \"A\" /WithProfile /NoConfirmation");
    assert!(profile_no(&state, "A").is_some(), "B still works in it");
}

#[test]
fn a_profile_in_use_is_refused_an_unused_one_asks() {
    let mut state = AppState::new();
    run(&mut state, "Store User \"Bob\"");
    run(&mut state, "Store User \"Phone\" /Profile \"Bob\"");
    let out = run(&mut state, "Delete UserProfile \"Bob\" /NoConfirmation");
    assert!(out.contains("E-USER-006") && out.contains("Bob, Phone"), "{out}");
    run(&mut state, "Store UserProfile \"Spare\"");
    let a = ask(&mut state, "Delete UserProfile \"Spare\"");
    assert!(a.view().text.contains("Nobody uses \"Spare\""));
    assert!(a.view().rows.iter().any(|r| r.k == "Window presets"));
    let no = profile_no(&state, "Spare").unwrap();
    assert_eq!(a.line_of("Delete").unwrap(), format!("Delete UserProfile {no} /NoConfirmation"));
    run(&mut state, &format!("Delete UserProfile {no} /NoConfirmation"));
    assert!(profile_no(&state, "Spare").is_none());
    assert!(run(&mut state, "Oops").contains("Undone"));
    assert!(profile_no(&state, "Spare").is_some(), "one Oops step");
}

#[test]
fn rights_come_only_from_the_user() {
    let mut state = AppState::new();
    // A deleted Admin kept rights through the same-named profile (12c2 finding).
    run(&mut state, "Store User \"Boss\" /Rights Admin");
    run(&mut state, "Delete User \"Boss\" /NoConfirmation");
    assert!(profile_no(&state, "Boss").is_some(), "the profile stays");
    as_user(&mut state, "Boss");
    assert!(!state.is_admin());
    assert_eq!(state.root.show_data.rights_of("Boss"), None);
    let out = run(&mut state, "Store User \"X\"");
    assert!(out.contains("E-RIGHTS-001"), "{out}");
    assert!(crate::rights::check(&state, "testask").is_err());
    // Admin-only Set answers E-RIGHTS-001 too; your own colour is yours.
    as_user(&mut state, "Admin");
    run(&mut state, "Store User \"LD\"");
    as_user(&mut state, "LD");
    assert!(run(&mut state, "Set User \"LD\" Role Admin").contains("E-RIGHTS-001"));
    assert!(!run(&mut state, "Set User \"LD\" \"Paint\" \"Sky\"").contains("Error"));
    assert_eq!(state.root.show_data.user("LD").unwrap().rights, UserRights::Program);
}

#[test]
fn renaming_a_user_keeps_their_guid_and_password_asks_masked() {
    let mut state = AppState::new();
    run(&mut state, "Store User \"LD\"");
    let guid = state.root.show_data.user("LD").unwrap().meta.identity.guid;
    let out = run(&mut state, "Set User \"LD\" \"Name\" \"Anna\"");
    assert!(out.contains("renamed"), "{out}");
    assert_eq!(state.root.show_data.user("Anna").unwrap().meta.identity.guid, guid);
    assert!(run(&mut state, "Set User \"Anna\" \"Name\" \"Debug\"").contains("E-USER-007"));
    assert!(run(&mut state, "Set User \"Admin\" \"Name\" \"Boss\"").contains("E-USER-005"));
    let a = ask(&mut state, "Set User \"Anna\" \"Name\" Ask");
    assert_eq!(a.primary_line().as_deref(), Some("Set User \"Anna\" \"Name\" \"Anna\""));
    let a = ask(&mut state, "Set User \"Anna\" \"Password\" Ask");
    assert!(matches!(a.view().inputs[0].kind, lumacore_wire::InputKind::Password));
    assert_eq!(a.primary_line().as_deref(), Some("Set User \"Anna\" \"Password\" \"\""), "empty = no password");
    run(&mut state, "Set User \"Anna\" \"Password\" \"pw\"");
    assert!(state.root.show_data.user("Anna").unwrap().password_hash.is_some());
    let out = run(&mut state, "Set User \"Anna\" \"Password\" \"\"");
    assert!(state.root.show_data.user("Anna").unwrap().password_hash.is_none(), "{out}");
}

#[test]
fn a_held_pane_maximises_on_the_shared_screen() {
    let mut state = AppState::new();
    let screen = state.root.runtime.live_screens.write().unwrap().join_or_create("Admin", None);
    state.current_live_screen = Some(screen);
    let maximized = |s: &AppState| s.root.runtime.live_screens.read().unwrap().get(screen).unwrap().maximized;
    let out = run(&mut state, &format!("Set Window {screen}.2 \"Maximized\" On"));
    assert!(out.contains("maximized"), "{out}");
    assert_eq!(maximized(&state), Some(2));
    run(&mut state, &format!("Set Window {screen}.2 \"Maximized\" Toggle"));
    assert_eq!(maximized(&state), None);
    run(&mut state, &format!("Set Window {screen}.1 \"Maximized\" On"));
    run(&mut state, &format!("Delete Window {screen}.1 /NoConfirmation"));
    assert_eq!(maximized(&state), None, "a closed window isn't held");
    assert!(run(&mut state, &format!("Set Window {screen}.2 \"Maximized\" \"Big\"")).contains("E-SYN-014"));
    // The profile's setting the page reads (default On).
    assert!(state.root.show_data.profile_for_user("Admin").unwrap().settings.preferences.hold_follows);
    run(&mut state, "Set UserProfile \"HoldFollows\" Off");
    assert!(!state.root.show_data.profile_for_user("Admin").unwrap().settings.preferences.hold_follows);
    assert_eq!(crate::commands::media::profile_section("HoldFollows"), "Input & keys");
}

#[test]
fn no_path_changes_default_even_if_someone_ended_up_in_it() {
    let mut state = AppState::new();
    // A user forced into Default (never through a command): their own
    // profile writes are rolled back by the change path's lock.
    let default = state.root.show_data.default_profile().unwrap().1.meta.identity.guid;
    state.root.show_data.user_mut("Debug").unwrap().profile = default;
    let before = default_json(&state);
    as_user(&mut state, "Debug");
    let out = run(&mut state, "Set UserProfile \"MeowKey\" Alt");
    assert!(out.contains("E-USER-002"), "{out}");
    assert_eq!(default_json(&state), before);
}

#[test]
fn admin_always_asks_for_admin() {
    let state = AppState::new();
    let auth = |pw: Option<&str>| crate::passwords::authenticate(&state, "Admin", pw, false);
    assert!(auth(None).is_err());
    assert!(auth(Some("nope")).is_err());
    assert_eq!(auth(Some("admin")).unwrap(), "Admin");
    let mut state = AppState::new();
    let out = run(&mut state, "Set User \"Admin\" \"Password\" \"x\"");
    assert!(out.contains("E-USER-001"), "{out}");
}

#[test]
fn users_have_their_own_colour() {
    let mut state = AppState::new();
    run(&mut state, "Store User \"Bob\"");
    let show = &state.root.show_data;
    assert_eq!(show.user("Admin").unwrap().color, "#9a7a08");
    let bob = show.user("Bob").unwrap();
    // The next colour nobody has (decisions §30 "12c2").
    assert_eq!(bob.color, lumacore_tree::USER_COLORS[0]);
    run(&mut state, "Store User \"Ann\" /Colour Teal");
    assert_eq!(state.root.show_data.user("Ann").unwrap().color, "#3f9a90");
    run(&mut state, "Set User \"Bob\" \"Paint\" \"#758BA0\"");
    assert_eq!(state.root.show_data.user("Bob").unwrap().color, "#758ba0");
    assert!(run(&mut state, "Set User \"Bob\" \"Paint\" \"red\"").contains("E-CMD-008"));
}
