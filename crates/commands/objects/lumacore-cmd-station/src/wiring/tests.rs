// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The station's network line in, reply out (decisions §34 25.17): station settings, interface
//! services, the NIC per network, wiring rows for this and other stations, the asks, the menus.
//!
//! Owns:          the tests of the station's settings and wiring.
//! Doesn't own:   lines that need another crate's words too (tests/catalog wiring).
//! Entry points:  the #[test] fns.
//! Decisions:     §11, §30 "12d part 4", §34 25.17.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use crate::settings::{PRIORITY_CHOICES, THEME_OVERRIDE_CHOICES};
use lumacore_objects::commands::dispatch;
use lumacore_objects::{AppState, DispatchResult};
use lumacore_tree::OopsCategory;
use lumacore_wire::UiEvent;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => lumacore_objects::text::strip_ansi(&s),
        _ => panic!("'{line}' did not produce output"),
    }
}

fn last_menu(state: &AppState) -> (String, Option<String>, bool, String) {
    match state.ui_requests.last() {
        Some(UiEvent::OpenMenu {
            menu,
            tab,
            toggle,
            grids,
            ..
        }) => (menu.clone(), tab.clone(), *toggle, grids.clone()),
        other => panic!("no OpenMenu: {other:?}"),
    }
}

fn steps(state: &AppState) -> usize {
    state.undo_stack("Admin").map_or(0, |u| u.frames.len())
}

fn ui() -> AppState {
    let mut state = AppState::new();
    state.ui_client = true;
    state
}

fn newest(state: &AppState) -> (OopsCategory, String, bool) {
    let f = state
        .undo_stack("Admin")
        .and_then(|u| u.frames.back())
        .expect("a step");
    (f.category, f.description.clone(), f.main)
}

fn main_guid(s: &AppState) -> uuid::Uuid {
    s.root
        .show_data
        .protocols
        .networks
        .values()
        .find(|n| n.meta.identity.name.as_deref() == Some("Main"))
        .unwrap()
        .meta
        .identity.guid
}

fn station() -> AppState {
    let mut s = AppState::new();
    s.local_settings.station_id = "0b7c8a2e-5d4f-4c3b-9a1e-2f3d4c5b6a7e".into();
    s.root.station_settings.station_name = "FOH".into();
    run(&mut s, "Store Network \"Main\"");
    s
}

#[test]
fn lumasession_interface_services_are_station_settings() {
    let Some(nic) = lumacore_net::nics::list()
        .into_iter()
        .find(|n| !n.ip.is_loopback())
    else {
        return; // no NIC to switch in this environment
    };
    let mut state = AppState::new();
    let out = run(
        &mut state,
        &format!("Set Interface \"{}\" \"HTTP\" No", nic.name),
    );
    assert!(out.contains("HTTP No"), "{out}");
    let s = state.root.station_settings.services_of(&nic.name);
    assert!(!s.http && s.shell && s.lumasession);
    assert!(run(&mut state, "Set Interface \"nope9\" \"HTTP\" No").contains("E-CMD-007"));
}

#[test]
fn a_networks_nic_is_a_typed_station_setting() {
    let mut state = AppState::new();
    run(&mut state, "Store Network \"Rig\"");
    assert!(run(&mut state, "Set Network \"Rig\" \"Interface\" \"en1\"").contains("→ en1"));
    assert_eq!(
        state
            .root
            .station_settings
            .networks
            .get("rig")
            .map(String::as_str),
        Some("en1")
    );
    // The network's own props are still tree props.
    let id = lumacore_objects::objects::protocols::network_id(&state.root, "Rig").unwrap();
    let out = run(&mut state, &format!("Set Network {id} \"Note\" \"main\""));
    assert!(out.contains("Note"), "{out}");
    assert!(run(&mut state, "Set Network \"Rig\" \"Interface\" None").contains("unmapped"));
}

#[test]
fn interface_services_take_many_rows() {
    let nics: Vec<String> = lumacore_net::nics::list()
        .into_iter()
        .map(|n| n.name)
        .collect();
    let [a, b, ..] = nics.as_slice() else {
        return; // fewer than two interfaces here
    };
    let mut state = AppState::new();
    let out = run(
        &mut state,
        &format!("Set Interface \"{a}\" + \"{b}\" \"Shell\" No"),
    );
    assert!(out.contains("Shell = No"), "{out}");
    assert!(
        !state.root.station_settings.services_of(a).shell
            && !state.root.station_settings.services_of(b).shell
    );
    // A name that isn't there is skipped (the one range rule).
    let out = run(
        &mut state,
        &format!("Set Interface \"{a}\" + \"nope9\" \"HTTP\" No"),
    );
    assert!(out.contains("HTTP No") && !out.contains("nope9"), "{out}");
}

#[test]
fn menu_opens_at_the_last_tab_and_the_same_line_closes() {
    let mut state = ui();
    let before = steps(&state);
    assert_eq!(run(&mut state, "Menu \"Settings\""), "");
    assert_eq!(
        last_menu(&state),
        ("Settings".into(), None, true, String::new())
    );
    assert_eq!(run(&mut state, "Menu Network /Tab networks"), "");
    let (menu, tab, toggle, _) = last_menu(&state);
    assert_eq!(
        (menu.as_str(), tab.as_deref(), toggle),
        ("Network", Some("Networks"), false)
    );
    // The profile keeps it; a bare line opens there and toggles.
    run(&mut state, "Menu \"Network\"");
    assert_eq!(last_menu(&state).1.as_deref(), Some("Networks"));
    assert!(last_menu(&state).2);
    let profile = state.root.show_data.profile_for_user("Admin").unwrap();
    assert_eq!(
        profile
            .settings
            .ui
            .menu_tabs
            .get("Network")
            .map(String::as_str),
        Some("Networks")
    );
    assert_eq!(steps(&state), before, "a tab is never an Oops step");
    // `Toggle Menu` = the ⚙ menu (decisions §33 38; `Menu` alone sets your default object, 39).
    run(&mut state, "Toggle Menu");
    assert_eq!(last_menu(&state).0, "Menu");
    run(&mut state, "Menu power");
    assert_eq!(last_menu(&state).0, "Shutdown");
    assert!(run(&mut state, "Menu \"Nope\"").contains("E-CMD-007"));
    assert!(run(&mut state, "Menu Network /Tab Nope").contains("E-CMD-007"));
    // Without a UI it says so.
    let mut plain = AppState::new();
    assert!(run(&mut plain, "Menu Settings").contains("E-CMD-010"));
}

#[test]
fn a_station_step_is_put_back_on_its_station() {
    let mut state = AppState::new();
    state.local_settings.station_id = "this-station".into();
    run(&mut state, "Set Station \"Location\" \"Stage left\"");
    let (category, _, _) = newest(&state);
    assert_eq!(category, OopsCategory::Station);
    run(&mut state, "Oops");
    assert_eq!(state.root.station_settings.location, "");
    run(&mut state, "Redo");
    assert_eq!(state.root.station_settings.location, "Stage left");

    // Another station's part is left alone here.
    let frame = state
        .undo_stack("Admin")
        .unwrap()
        .frames
        .back()
        .unwrap()
        .clone();
    state.local_settings.station_id = "other-station".into();
    lumacore_objects::commands::oops::restore_local(
        &mut state,
        &frame.local,
        lumacore_objects::commands::oops::Side::Before,
    );
    assert_eq!(state.root.station_settings.location, "Stage left");
}

#[test]
fn a_stations_wiring_row_is_owner_first() {
    let mut state = AppState::new();
    run(&mut state, "Set Station \"Name\" \"FOH\"");
    run(&mut state, "Store Network \"Rig\"");
    run(&mut state, "Store Station \"Tower\"");
    let out = run(
        &mut state,
        "Set Station \"Tower\" Network \"Rig\" \"Interface\" \"en7\"",
    );
    assert!(out.contains("en7") && out.contains("Tower"), "{out}");
    let tower = state
        .root
        .show_data
        .stations
        .values()
        .find(|r| r.meta.name() == Some("Tower"))
        .unwrap();
    assert_eq!(
        tower.networks.values().next().map(String::as_str),
        Some("en7")
    );
    // A network's own property isn't a station's wiring.
    assert!(
        run(
            &mut state,
            "Set Station \"Tower\" Network \"Rig\" \"Note\" \"x\""
        )
        .contains("E-SET-004")
    );
}

#[test]
fn keyed_rows_resolve_by_name() {
    let Some(nic) = lumacore_net::nics::list()
        .into_iter()
        .map(|n| n.name)
        .next()
    else {
        return;
    };
    let mut state = AppState::new();
    let out = run(&mut state, &format!("Set Interface \"{nic}\" \"Shell\" No"));
    assert!(!out.contains("Error"), "{out}");
    assert!(!state.root.station_settings.services_of(&nic).shell);
    assert!(run(&mut state, "Set Interface \"nope9\" \"Shell\" No").contains("E-CMD-007"));
}

#[test]
fn the_dangerous_ones_ask_and_the_answer_is_quoted() {
    let mut state = AppState::new();
    let DispatchResult::Ask(ask) = dispatch(&mut state, "Set Station \"HoldOutput\" On") else {
        panic!("asks")
    };
    let line = ask.line_of("Hold output").unwrap();
    assert_eq!(line, "Set Station \"HoldOutput\" On /NoConfirmation");
    assert!(run(&mut state, &line).contains("held"));
}

#[test]
fn a_row_made_in_advance_is_claimed_by_its_station_and_saved_with_the_show() {
    let mut s = station();
    run(&mut s, "Store Station \"FOH\"");
    run(
        &mut s,
        "Set Station \"FOH\" Network \"Main\" \"Interface\" \"en5\"",
    );
    // The row names this station: it applies at once and is claimed.
    assert_eq!(
        s.root
            .station_settings
            .networks
            .get("main")
            .map(String::as_str),
        Some("en5")
    );
    run(&mut s, "Set Interface \"lo0\" \"HTTP\" No");
    let row = &s.root.show_data.stations[&1];
    assert_eq!(row.station_id, s.local_settings.station_id);
    // Saved and read back: Stations.json in the .luma.
    let (bytes, _) = lumacore_luma::show_file::encode(&s.root.show_data).unwrap();
    let back = lumacore_luma::show_file::from_bytes(&bytes).unwrap();
    assert_eq!(back.stations, s.root.show_data.stations);
}

#[test]
fn a_wiring_change_writes_this_stations_row_in_the_show_and_applies_it() {
    let mut s = station();
    assert!(s.root.show_data.stations.is_empty());
    let out = run(&mut s, "Set Network \"Main\" \"Interface\" \"en9\"");
    assert!(out.contains("en9") && out.contains("this station"), "{out}");
    let (no, row) = s
        .root
        .show_data
        .stations
        .iter()
        .next()
        .expect("the own row");
    assert_eq!(
        (row.meta.name().unwrap_or_default(), row.station_id.as_str()),
        ("FOH", s.local_settings.station_id.as_str())
    );
    assert_eq!(
        row.meta.identity.guid,
        lumacore_tree::StationWiring::guid_for(&s.local_settings.station_id)
    );
    assert_eq!(
        row.networks.get(&main_guid(&s)).map(String::as_str),
        Some("en9")
    );
    // The station applies it (its last-applied copy, by network name).
    assert_eq!(
        s.root
            .station_settings
            .networks
            .get("main")
            .map(String::as_str),
        Some("en9")
    );
    let no = *no;
    // Interfaces' services live in the same row. A real interface of this machine
    // (`lo0` on macOS, `lo` on Linux): a name the station doesn't have is refused.
    let nic = lumacore_net::nics::list()
        .into_iter()
        .next()
        .expect("an interface")
        .name;
    let out = run(&mut s, &format!("Set Interface \"{nic}\" \"Shell\" No"));
    assert!(
        !s.root.show_data.stations[&no].services_of(&nic).shell,
        "{out}"
    );
    assert!(!s.root.station_settings.services_of(&nic).shell);
    // Oops puts the row back, and the station follows it.
    run(&mut s, "Oops");
    assert!(s.root.show_data.stations[&no].services_of(&nic).shell);
    lumacore_objects::stations::apply_own_row(&mut s);
    assert!(s.root.station_settings.services_of(&nic).shell);
    // It is one show subtree: identity clean, a Station step.
    assert!(
        s.root.show_data.identity().is_clean(),
        "{:?}",
        s.root.show_data.identity()
    );
}

#[test]
fn another_stations_identity_changes_on_that_station() {
    let mut s = station();
    run(&mut s, "Store Station \"Tower\"");
    // Offline: refused with what does change here.
    let out = run(&mut s, "Set Station \"Tower\" \"Priority\" High");
    assert!(
        out.contains("isn't reachable") && out.contains("E-CMD-010"),
        "{out}"
    );
    // Seen by discovery: sent to its command port as this user.
    s.root.runtime.luma_sessions.discovery.insert(
        "10.0.0.7:47653".into(),
        lumacore_tree::DiscoveryPeerRuntime {
            name: "Tower".into(),
            ip: "10.0.0.7".into(),
            cmd_port: 47653,
            station_id: "7a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d".into(),
            ..Default::default()
        },
    );
    let out = run(
        &mut s,
        "Set Station \"FOH\" + \"Tower\" \"Location\" \"Stage left\"",
    );
    assert!(out.contains("Station \"Tower\""), "{out}");
    assert_eq!(
        s.root.station_settings.location, "Stage left",
        "this station's own changes here"
    );
    let sent: Vec<String> = s
        .session_requests
        .iter()
        .filter_map(|r| match r {
            lumacore_objects::session_link::SessionRequest::EditStation {
                request: lumacore_wire::RemoteRequest::Command { cmd, .. },
                ip,
                ..
            } => Some(format!("{ip} {cmd}")),
            _ => None,
        })
        .collect();
    assert_eq!(
        sent,
        ["10.0.0.7 Set Station \"Location\" \"Stage left\" /NoConfirmation"]
    );
    // A discovered station without a row gets its own row's GUID when wired.
    run(&mut s, "Delete Station \"Tower\" /NoConfirmation");
    run(
        &mut s,
        "Set Station \"Tower\" Network \"Main\" \"Interface\" \"en1\"",
    );
    let row = s
        .root
        .show_data
        .stations
        .values()
        .find(|r| r.meta.name() == Some("Tower"))
        .unwrap();
    assert_eq!(
        row.meta.identity.guid,
        lumacore_tree::StationWiring::guid_for("7a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d")
    );
}

#[test]
fn the_theme_override_choices_are_off_and_the_presets() {
    assert_eq!(THEME_OVERRIDE_CHOICES[0], "Off");
    assert_eq!(
        &THEME_OVERRIDE_CHOICES[1..],
        lumacore_tree::media::THEME_PRESETS
    );
}

#[test]
fn the_priority_choices_parse() {
    for p in PRIORITY_CHOICES {
        assert_eq!(
            p.parse::<lumacore_tree::StationPriority>()
                .unwrap()
                .to_string(),
            *p
        );
    }
}
