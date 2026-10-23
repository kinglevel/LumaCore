// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Step 12a3 (decisions §30 "12a3 hardening"): devices, DeskLock, the normal
//! theme, Admin's fixed password, core keys and user colours — through real
//! dispatch.
//!
//! Owns:          the tests of the devices' words.
//! Doesn't own:   the window toggles' (the kernel's window_bar_tests).
//! Entry points:  the #[test] fns.
//! Decisions:     §30 "12a3", "12c2".

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_objects::AppState;
use lumacore_objects::DispatchResult;
use lumacore_objects::commands::dispatch;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => lumacore_objects::text::strip_ansi(&s),
        _ => panic!("'{line}' did not produce output"),
    }
}

/// A station with a passwordless LD and a device "tab-1" acting as `user`.
fn device(user: &str) -> AppState {
    let mut state = AppState::new();
    run(&mut state, "Store User \"LD\" /Rights Program");
    state.user = user.into();
    state.current_device = Some("tab-1".into());
    state.devices.entry("tab-1".into()).or_default();
    state
}

#[test]
fn a_locked_device_runs_only_the_lock_keys() {
    let mut state = device("LD");
    assert!(run(&mut state, "DeskLock").contains("DeskLock On"));
    assert!(lumacore_objects::devices::status(&state).locked);
    let out = run(&mut state, "Store Group 1");
    assert!(out.contains("E-LOCK-001"), "{out}");
    // The normal theme works while locked.
    assert!(run(&mut state, "Toggle NormalTheme").contains("NormalTheme On"));
    assert!(lumacore_objects::devices::status(&state).normal_theme);
    // LD has no password: the key unlocks.
    assert!(run(&mut state, "DeskLock").contains("DeskLock Off"));
    assert!(!run(&mut state, "Store Group 1").contains("E-LOCK"));
}

#[test]
fn a_user_with_a_password_unlocks_only_with_it() {
    let mut state = device("Admin");
    run(&mut state, "On DeskLock");
    let out = run(&mut state, "Off DeskLock");
    assert!(
        out.contains("E-LOCK-002"),
        "Admin always has a password: {out}"
    );
    assert!(lumacore_objects::devices::is_locked(&state));
    lumacore_objects::devices::unlock(&mut state, "tab-1");
    assert!(!lumacore_objects::devices::is_locked(&state));
}

#[test]
fn desk_lock_station_locks_every_device_and_lines_without_a_device_are_refused() {
    let mut state = device("LD");
    state.devices.entry("app-2".into()).or_default();
    // Every device of the station is Setup's (decisions §30 "12d").
    let out = run(&mut state, "On DeskLock Station");
    assert!(
        out.contains("E-RIGHTS-001") && out.contains("needs Setup"),
        "{out}"
    );
    state.user = "Admin".into();
    run(&mut state, "On DeskLock Station");
    assert!(state.devices.values().all(|d| d.locked));
    state.current_device = None;
    let out = run(&mut state, "DeskLock");
    assert!(out.contains("E-CMD-010"), "a CLI isn't a device: {out}");
}

#[test]
fn the_station_override_applies_to_every_device() {
    let mut state = device("Admin");
    state.user = "Admin".into();
    run(&mut state, "Set Station \"ThemeOverride\" Daylight");
    let st = lumacore_objects::devices::status(&state);
    assert!(st.normal_theme);
    assert_eq!(st.theme_override.as_deref(), Some("Daylight"));
    run(&mut state, "Set Station \"ThemeOverride\" Off");
    assert!(!lumacore_objects::devices::status(&state).normal_theme);
}

/// R1-42: the station records who locked a device; only they or Admin may
/// unlock it; unlocking forgets it.
#[test]
fn the_station_records_who_locked_a_device() {
    let mut state = device("LD");
    state.devices.entry("app-2".into()).or_default();
    run(&mut state, "On DeskLock");
    assert_eq!(state.devices["tab-1"].locked_by.as_deref(), Some("LD"));
    state.user = "Admin".into();
    run(&mut state, "On DeskLock Station");
    assert_eq!(
        state.devices["tab-1"].locked_by.as_deref(),
        Some("LD"),
        "already locked: the first locker stays"
    );
    assert_eq!(state.devices["app-2"].locked_by.as_deref(), Some("Admin"));
    use lumacore_objects::devices::may_unlock;
    assert!(may_unlock(&state, "tab-1", "LD").is_ok());
    assert!(may_unlock(&state, "tab-1", "Admin").is_ok());
    assert!(
        may_unlock(&state, "tab-1", "Busk")
            .unwrap_err()
            .contains("Locked by LD")
    );
    lumacore_objects::devices::unlock(&mut state, "tab-1");
    assert_eq!(state.devices["tab-1"].locked_by, None);
}

/// What a refused line must leave alone: the show, the claims, the station's
/// settings and every device.
fn image(
    state: &AppState,
) -> (
    String,
    String,
    String,
    Vec<(String, lumacore_objects::device_table::DeviceState)>,
) {
    (
        serde_json::to_string(&state.root.show_data).unwrap(),
        format!("{:?}", state.root.runtime.claims),
        serde_json::to_string(&state.root.station_settings).unwrap(),
        state
            .devices
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    )
}

/// R1-41: only the lock words themselves pass a locked device — not any
/// word followed (or preceded) by one (`Shutdown DeskLock /NoConfirmation`).
#[test]
fn a_locked_device_refuses_every_word_around_the_lock_words() {
    use lumacore_objects::commands::registry::{self, Word};
    use lumacore_cmd_station::STATION;
    use lumacore_objects::objects::Verb;
    let mut state = device("LD");
    run(&mut state, "On DeskLock");
    let switch = |w: &Word| matches!(w, Word::Command(c) if matches!(c.object_verb, Some(Verb::On | Verb::Off | Verb::Toggle)));
    let mut seen: Vec<Word> = Vec::new();
    for (name, word) in registry::all_names() {
        if seen.iter().any(|w| w.same(&word)) {
            continue;
        }
        seen.push(word);
        let mut lines = vec![format!("{name} DeskLock /NoConfirmation")];
        if !switch(&word) {
            lines.push(format!("{name} NormalTheme"));
        }
        if !word.same(&Word::Object(&STATION)) {
            lines.push(format!("DeskLock {name}"));
        }
        for line in lines {
            let before = image(&state);
            let DispatchResult::Output(out) = dispatch(&mut state, &line) else {
                panic!("'{line}' from a locked device got past the lock");
            };
            let out = lumacore_objects::text::strip_ansi(&out);
            assert!(
                out.contains("E-LOCK-001"),
                "'{line}' ran from a locked device: {out}"
            );
            assert!(image(&state) == before, "'{line}' changed something");
        }
    }
    for line in [
        "On DeskLock",
        "Toggle NormalTheme",
        "NormalTheme",
        "Off NormalTheme",
        "On DeskLock Station",
    ] {
        let out = run(&mut state, line);
        assert!(!out.contains("E-LOCK-001"), "'{line}' is a lock key: {out}");
    }
    // Two statements are never the lock keys.
    assert!(run(&mut state, "DeskLock; Store Group 1").contains("E-LOCK-001"));
    assert!(run(&mut state, "Off DeskLock").contains("DeskLock Off"));
    run(&mut state, "On DeskLock");
    assert!(run(&mut state, "DeskLock").contains("DeskLock Off"));
}

/// R1-42: while the station is locked, a device that connects later starts
/// locked (a new tab is no way around it); the station lock ends with the
/// last unlocked device.
#[test]
fn a_device_that_connects_while_the_station_is_locked_starts_locked() {
    use lumacore_objects::devices::{is_locked, known, unlock};
    let mut state = device("LD");
    // Locking every device of the station is Setup's (decisions §30 "12d").
    state.user = "Admin".into();
    run(&mut state, "Set User \"LD\" \"Rights\" Setup");
    state.user = "LD".into();
    assert!(run(&mut state, "On DeskLock Station").contains("Locked"));
    // A line from a device the station hasn't seen yet is refused.
    state.current_device = Some("tab-new".into());
    assert!(is_locked(&state));
    assert!(run(&mut state, "Store Group 1").contains("E-LOCK-001"));
    assert!(known(&mut state, "tab-new").locked);
    assert_eq!(state.devices["tab-new"].locked_by.as_deref(), Some("LD"));
    unlock(&mut state, "tab-1");
    assert!(state.station_lock.is_some(), "tab-new is still locked");
    unlock(&mut state, "tab-new");
    assert!(state.station_lock.is_none());
    assert!(!known(&mut state, "tab-later").locked);
}

/// R1-42: `Off DeskLock` from the device follows the same rule as
/// `/api/unlock`: only the user who locked it, or Admin.
#[test]
fn only_the_locker_or_admin_unlocks_with_the_key() {
    let mut state = device("LD");
    state.user = "Admin".into();
    run(&mut state, "Store User \"Busk\" /Role Programmer");
    state.user = "LD".into();
    run(&mut state, "On DeskLock");
    state.user = "Busk".into();
    let out = run(&mut state, "Off DeskLock");
    assert!(out.contains("E-LOCK-002"), "{out}");
    state.user = "LD".into();
    assert!(run(&mut state, "Off DeskLock").contains("DeskLock Off"));
}

#[test]
fn a_device_picks_its_screen_config() {
    let mut state = AppState::new();
    assert!(
        run(&mut state, "Set Device \"ScreenConfig\" 2").contains("E-CMD-007"),
        "only a device"
    );
    state.current_device = Some("tab-1".into());
    assert_eq!(
        lumacore_objects::devices::status(&state).screen_config,
        None,
        "Follow user by default"
    );
    run(&mut state, "Set Device \"ScreenConfig\" 2");
    assert_eq!(
        lumacore_objects::devices::status(&state).screen_config,
        Some(2)
    );
    assert!(run(&mut state, "Set Device \"ScreenConfig\" \"x\"").contains("E-SET-003"));
    run(&mut state, "Set Device \"ScreenConfig\" Follow");
    assert_eq!(
        lumacore_objects::devices::status(&state).screen_config,
        None
    );
}

/// A screen config round-trips as JSON (Export / Import ScreenConfig, a copy with new GUIDs).
#[test]
fn a_screen_config_round_trips_as_json() {
    let mut state = AppState::new();
    let screen = state
        .root
        .runtime
        .live_screens
        .write()
        .unwrap()
        .join_or_create("Admin", None);
    state.current_live_screen = Some(screen);
    lumacore_objects::commands::screens::sync(&mut state);
    assert!(run(&mut state, "Export ScreenConfig 1").contains("screenconfigs/"));
    let name = state.root.show_data.user_profiles[&2]
        .content
        .screen_configs[&1]
        .meta
        .name()
        .unwrap_or_default()
        .to_string();
    let out = run(&mut state, &format!("Import ScreenConfig \"{name}\" 5"));
    assert!(out.contains("Screen config 5"), "{out}");
    assert!(state.root.show_data.identity().duplicates.is_empty());
}
