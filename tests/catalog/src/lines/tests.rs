// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The protocol lines with the station's network (decisions §34 25.16, 25.17, "34f7c hardened"): a
//! line sends only once its network is mapped, gateways on their own NICs, output hold, one run per
//! line.
//!
//!
//! Owns:          the cross-crate lines tests.
//! Doesn't own:   each crate's own words' tests.
//! Entry points:  the #[test] fns.
//! Decisions:     §11, §34 25.16, 25.17.

use lumacore_protocols::{InputPriority, MergeMode};

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_net::nics::Nic;
use lumacore_objects::commands::dispatch;
use lumacore_objects::protocol_plan::{LineState, desired};
use lumacore_objects::{AppState, DispatchResult};
use std::net::Ipv4Addr;

fn nic() -> Vec<Nic> {
    vec![Nic {
        name: "en1".into(),
        ip: Ipv4Addr::new(10, 0, 0, 5),
        netmask: Ipv4Addr::new(255, 255, 255, 0),
        broadcast: Ipv4Addr::new(10, 0, 0, 255),
        index: None,
    }]
}

fn nics() -> Vec<Nic> {
    vec![
        Nic {
            name: "en1".into(),
            ip: Ipv4Addr::new(10, 0, 0, 5),
            netmask: Ipv4Addr::new(255, 255, 255, 0),
            broadcast: Ipv4Addr::new(10, 0, 0, 255),
            index: None,
        },
        Nic {
            name: "en2".into(),
            ip: Ipv4Addr::new(10, 1, 0, 5),
            netmask: Ipv4Addr::new(255, 255, 255, 0),
            broadcast: Ipv4Addr::new(10, 1, 0, 255),
            index: None,
        },
    ]
}

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => lumacore_objects::text::strip_ansi(&s),
        _ => panic!("'{line}' did not produce output"),
    }
}

fn sacn(
    d: &lumacore_objects::protocol_plan::Desired,
    ip: [u8; 4],
) -> &lumacore_objects::protocol_plan::DriverPlan {
    &d.drivers[&("Sacn".to_string(), Ipv4Addr::from(ip))]
}

fn with_universes(n: u32) -> AppState {
    let mut state = AppState::new();
    let mut ev = Vec::new();
    for _ in 0..n {
        lumacore_tree::UniverseOps::create_universe(&mut state.root.show_data, None, &mut ev);
    }
    state
}

#[test]
fn protocol_assign_creates_a_line_that_sends_only_once_mapped() {
    let mut state = with_universes(6);
    // A line is a protocol's object (decisions §34 "34f7c hardened" 1).
    assert!(run(&mut state, "Store Sacn 1").contains("Sacn 1 (Output)"));
    let out = run(&mut state, "Assign Universe 1 Thru 6 At Sacn 1 /Universe 1");
    assert!(out.contains("1–6 → 1–6") && out.contains("isn't mapped"), "{out}");
    let d = desired(&state, &nics());
    assert!(d.drivers.is_empty());
    assert_eq!(d.status[0].state, LineState::NotMapped);
    // The station maps Network "Default": now it sends six universes on en1.
    assert!(run(&mut state, "Set Network \"Default\" \"Interface\" \"en1\"").contains("en1"));
    let d = desired(&state, &nics());
    let plan = &sacn(&d, [10, 0, 0, 5]).outputs;
    assert_eq!(plan[0].routes.len(), 6);
    assert_eq!(plan[0].routes[5], (6, 6));
    // Unmapping stops it again.
    run(&mut state, "Set Network \"Default\" \"Interface\" none");
    assert!(desired(&state, &nics()).drivers.is_empty());
    // Assign onto a line that isn't there yet makes it, on the show's only network.
    let out = run(&mut state, "Assign Universe 1 At ArtNet 4 /Universe 0.0.3");
    assert!(out.contains("ArtNet 4: 1 → 0.0.3"), "{out}");
}

#[test]
fn protocol_gateway_networks_send_on_their_own_nics() {
    let mut state = with_universes(6);
    run(&mut state, "Store Network \"Rig\"");
    run(&mut state, "Store Sacn 1 \"Out to rig\" /Network \"Rig\"");
    run(&mut state, "Store ArtNet 1 \"Nodes\" /Network \"Guest\"");
    run(&mut state, "Assign Universe 1 Thru 6 At Sacn 1 /Universe 1");
    run(&mut state, "Assign Universe 1 At ArtNet 1 /Universe 0.1.2");
    run(&mut state, "Set Network \"Rig\" \"Interface\" \"en1\"");
    run(&mut state, "Set Network \"Guest\" \"Interface\" \"10.1.0.0/24\"");
    let d = desired(&state, &nics());
    assert_eq!(sacn(&d, [10, 0, 0, 5]).outputs[0].routes.len(), 6);
    let art = &d.drivers[&("ArtNet".to_string(), Ipv4Addr::new(10, 1, 0, 5))];
    assert_eq!(art.nic.broadcast, Ipv4Addr::new(10, 1, 0, 255));
    assert_eq!(art.outputs[0].routes, vec![(1, 0x12)]);
    // Line settings are live: unicast target and sACN's own priority.
    run(&mut state, "Set Sacn 1 \"Target\" \"10.0.0.50\"");
    run(&mut state, "Set Sacn 1 \"Priority\" 150");
    let d = desired(&state, &nics());
    let plan = &sacn(&d, [10, 0, 0, 5]).outputs[0];
    assert_eq!((plan.unicast, plan.priority), (Some(Ipv4Addr::new(10, 0, 0, 50)), 150));
    assert!(run(&mut state, "Set Sacn 1 \"Target\" \"banana\"").contains("E-SET-003"));
    // Art-Net has no Priority: it simply isn't one of its properties.
    assert!(run(&mut state, "Set ArtNet 1 \"Priority\" 5").contains("E-SET-"));
}

/// The festival gateway (decisions §11): guest sACN 1–6 in on one NIC,
/// merged into local 1–6, out as sACN 1–6 to the rig on another NIC.
#[test]
fn protocol_gateway_takes_guest_input_on_one_nic_and_sends_on_another() {
    let mut state = with_universes(6);
    run(&mut state, "Store Sacn 1 \"Rig\" /Network \"Rig\"");
    let out = run(&mut state, "Store Sacn 2 \"Guest\" /Network \"Guest\" /Mode Input");
    assert!(out.contains("Sacn 2 (Input)") && out.contains("At Sacn 2"), "{out}");
    run(&mut state, "Assign Universe 1 Thru 6 At Sacn 1 /Universe 1");
    // A line onto universes receives (Assign's order is the data flow).
    let out = run(&mut state, "Assign Sacn 2 At Universe 1 Thru 6 /Universe 1");
    assert!(out.contains("1–6 ← 1–6") && out.contains("received"), "{out}");
    run(&mut state, "Set Network \"Rig\" \"Interface\" \"en1\"");
    run(&mut state, "Set Network \"Guest\" \"Interface\" \"en2\"");
    let d = desired(&state, &nics());
    let rig = sacn(&d, [10, 0, 0, 5]);
    let guest = sacn(&d, [10, 1, 0, 5]);
    assert_eq!((rig.outputs.len(), rig.inputs.len()), (1, 0));
    assert_eq!((guest.outputs.len(), guest.inputs.len()), (0, 1));
    assert_eq!(guest.inputs[0].routes.len(), 6);
    assert_eq!(guest.inputs[0].band, InputPriority::Ltp);
    // Settings of the line; Target filters the source.
    run(&mut state, "Set Sacn 2 \"InputPriority\" super");
    run(&mut state, "Set Sacn 2 \"Target\" \"10.1.0.9\"");
    let d = desired(&state, &nics());
    let plan = &sacn(&d, [10, 1, 0, 5]).inputs[0];
    assert_eq!(plan.band, InputPriority::Super);
    assert_eq!(plan.source, Some(Ipv4Addr::new(10, 1, 0, 9)));
    assert!(run(&mut state, "Set Sacn 2 \"InputPriority\" \"Loud\"").contains("E-SET-003"));
    // sACN's Priority is an output line's.
    assert!(run(&mut state, "Set Sacn 2 \"Priority\" 150").contains("E-SET-001"));
    // Universes come off the end of a run; the output keeps its own.
    assert!(run(&mut state, "Unassign Universe 6 At Sacn 2").contains("Took 1 universe off"));
    assert_eq!(state.root.show_data.protocols.line("Sacn", 2).unwrap().amount, 5);
    assert_eq!(state.root.show_data.protocols.line("Sacn", 1).unwrap().amount, 6);
    let info = run(&mut state, "Info Sacn 2");
    assert!(info.contains("Super") && info.contains("only 10.1.0.9") && info.contains("none heard"), "{info}");
    let listed = run(&mut state, "List Sacn");
    assert!(listed.contains("Super from 10.1.0.9") && listed.contains("1–5 ← 1–5"), "{listed}");
}

#[test]
fn lumasession_station_settings_and_output_hold() {
    let mut state = AppState::new();
    let mut ev = Vec::new();
    lumacore_tree::UniverseOps::create_universe(&mut state.root.show_data, None, &mut ev);
    run(&mut state, "Store Sacn 1");
    run(&mut state, "Assign Universe 1 At Sacn 1 /Universe 1");
    run(&mut state, "Set Network \"Default\" \"Interface\" \"en1\"");
    assert!(matches!(
        desired(&state, &nic()).status[0].state,
        LineState::Sending { .. }
    ));

    assert!(run(&mut state, "Set Station \"LumaSession\" \"Festival\"").contains("Festival"));
    run(&mut state, "Set Station \"Location\" \"FOH\"");
    run(&mut state, "Set Station \"Name\" \"FOH-1\"");
    run(&mut state, "Set Station \"Invite\" Off");
    let st = &state.root.station_settings;
    assert_eq!(
        (
            st.lumasession.as_str(),
            st.location.as_str(),
            st.station_name.as_str(),
            st.invite
        ),
        ("Festival", "FOH", "FOH-1", false)
    );
    // Held output and LumaSession off silence every line; both ask first
    // (decisions §30 "12d part 4": dangerous actions are station asks).
    assert!(matches!(
        dispatch(&mut state, "Set Station \"HoldOutput\" On"),
        DispatchResult::Ask(_)
    ));
    assert!(matches!(
        dispatch(&mut state, "Set Station \"Online\" Off"),
        DispatchResult::Ask(_)
    ));
    assert!(
        !state.root.station_settings.output_held && state.root.station_settings.online,
        "an ask changes nothing"
    );
    run(&mut state, "Set Station \"HoldOutput\" On /NoConfirmation");
    assert_eq!(desired(&state, &nic()).status[0].state, LineState::Held);
    run(&mut state, "Set Station \"HoldOutput\" Off");
    run(&mut state, "Set Station \"Online\" Off /NoConfirmation");
    assert_eq!(desired(&state, &nic()).status[0].state, LineState::Offline);
    run(&mut state, "Set Station \"Online\" On");
    assert!(matches!(
        desired(&state, &nic()).status[0].state,
        LineState::Sending { .. }
    ));
    assert!(run(&mut state, "Set Station \"Priority\" \"Loud\"").contains("Error"));
}

fn names(s: &lumacore_objects::commands::line::Statement) -> Vec<(&str, Option<&str>)> {
    s.flags
        .iter()
        .map(|f| (f.def.name, f.value.as_deref()))
        .collect()
}

fn one(text: &str) -> lumacore_objects::commands::line::Statement {
    let line = lumacore_objects::commands::line::Line::parse(text).expect("lexes");
    assert_eq!(line.statements.len(), 1, "{text}");
    line.statements.into_iter().next().unwrap()
}

fn ask(state: &mut AppState, line: &str) -> lumacore_objects::dialogs::Ask {
    match dispatch(state, line) {
        DispatchResult::Ask(a) => *a,
        DispatchResult::Output(s) => panic!(
            "'{line}' didn't ask: {}",
            lumacore_objects::text::strip_ansi(&s)
        ),
        _ => panic!("'{line}' didn't ask"),
    }
}

#[test]
fn protocol_merge_mode_lives_on_the_universe() {
    let mut state = with_universes(3);
    run(&mut state, "Store ArtNet 1 /Network \"Guest\" /Mode Input");
    run(&mut state, "Assign ArtNet 1 At Universe 1 Thru 2 /Universe 0.0.1");
    // MergeMode's one home is the universe (decisions §11 "Issue 11 hardening" 4): a line has none.
    assert!(run(&mut state, "Set ArtNet 1 \"MergeMode\" HTP").contains("E-SET-"));
    // Universes are in the Devices draft, MergeMode with them (§11 "Issue 11 hardening").
    assert!(run(&mut state, "Set Universe 1 \"MergeMode\" htp").contains("Claim Root.ShowData.Devices"));
    run(&mut state, "Claim Root.ShowData.Devices");
    run(&mut state, "Set Universe 1 Thru 2 \"MergeMode\" htp");
    run(&mut state, "Set Universe 2 \"MergeMode\" LowTP");
    let unis = &state.root.show_data.devices.dmx.patch.universes;
    assert_eq!(
        (unis[&1].merge_mode, unis[&2].merge_mode, unis[&3].merge_mode),
        (MergeMode::Htp, MergeMode::LowTp, MergeMode::Prio)
    );
    assert!(run(&mut state, "Info ArtNet 1").contains("…"), "two universes, two modes");
    assert!(run(&mut state, "Set Universe 2 \"MergeMode\" \"Loud\"").contains("E-SET-003"));
    // The show keeps the mode and the line's run across a save and load.
    let json = serde_json::to_string(&state.root.show_data).unwrap();
    assert!(json.contains("\"LowTP\""));
    let mut loaded = AppState::new();
    lumacore_objects::objects::replace_show(&mut loaded, serde_json::from_str(&json).unwrap());
    assert_eq!(loaded.root.show_data.devices.dmx.patch.universes[&2].merge_mode, MergeMode::LowTp);
    let line = loaded.root.show_data.protocols.line("ArtNet", 1).unwrap();
    assert_eq!((line.protocol.as_str(), line.mode, line.local_universe, line.amount), ("ArtNet", lumacore_protocols::Direction::In, 1, 2));
}

#[test]
fn protocol_osc_lines_run_as_their_user_and_listen_on_their_network() {
    let mut state = AppState::new();
    // An unknown user is refused: a line never falls back to Admin.
    assert!(run(&mut state, "Store Osc 1 \"Companion\" /Network \"Rig\" /Port 9000 /User \"Ghost\"").contains("No user"));
    run(&mut state, "Store Osc 1 \"Companion\" /Network \"Rig\" /Port 9000 /User \"Admin\"");
    run(&mut state, "Set Network \"Rig\" \"Interface\" \"en1\"");
    fn osc(state: &mut AppState) -> &mut lumacore_protocols::Line {
        state.root.show_data.protocols.lines_mut("Osc").unwrap().get_mut(&1).unwrap()
    }
    // Its user is gone (a broken link): the line doesn't listen.
    osc(&mut state).user = Some(uuid::Uuid::new_v4());
    assert_eq!(desired(&state, &nics()).status[0].state, LineState::NoUser);
    // No user set: the line doesn't start — it never runs as Admin
    // (decisions §30 "12d"), and storing one without a user is refused.
    osc(&mut state).user = None;
    let d = desired(&state, &nics());
    assert_eq!(d.status[0].state, LineState::NoUser);
    assert!(!d.control.contains_key(&("Osc".to_string(), 1)));
    let out = run(&mut state, "Store Osc 3 \"Bare\" /Network \"Rig\" /Port 9001");
    assert!(out.contains("E-RIGHTS-003"), "{out}");
    // OSC lines are inputs and carry no universes.
    assert!(run(&mut state, "Store Osc 4 /Mode Output /User \"Admin\"").contains("E-CMD-008"));
    assert!(run(&mut state, "Set Osc 1 \"Mode\" Output").contains("Osc lines are Input"));
    // A line runs as a user up to your own level.
    run(&mut state, "Store User \"Desk\" /Rights Setup");
    run(&mut state, "Store User \"Boss\" /Rights Admin");
    state.user = "Desk".into();
    let out = run(&mut state, "Store Osc 3 \"Up\" /Network \"Rig\" /Port 9001 /User \"Boss\"");
    assert!(out.contains("E-RIGHTS-001") && out.contains("needs Admin"), "{out}");
    let out = run(&mut state, "Store Osc 3 \"Same\" /Network \"Rig\" /Port 9001 /User \"Desk\"");
    assert!(!out.contains("Error"), "{out}");
}

#[test]
fn protocol_a_line_carries_one_run_and_needs_no_claim() {
    let mut state = with_universes(4);
    run(&mut state, "Store Sacn 1");
    assert!(run(&mut state, "Assign Universe 1 Thru 2 At Sacn 1 /Universe 10").contains("1–2 → 10–11"));
    // Another run asks (Replace / New line / Cancel), changing nothing yet.
    let a = ask(&mut state, "Assign Universe 4 At Sacn 1");
    assert!(a.view().text.contains("carries 1–2 → 10–11"), "{}", a.view().text);
    assert_eq!(state.root.show_data.protocols.line("Sacn", 1).unwrap().local_universe, 1);
    // A gap isn't a run.
    run(&mut state, "Unassign Sacn 1");
    assert_eq!(state.root.show_data.protocols.line("Sacn", 1).unwrap().amount, 0);
    let out = run(&mut state, "Assign Universe 1 + 3 At Sacn 1");
    assert!(out.contains("E-OUTPUT-007"), "{out}");
    assert!(run(&mut state, "Assign Universe 1 Thru 3 At Sacn 1 /Universe 63998").contains("E-OUTPUT-005"));
    run(&mut state, "Assign Universe 1 Thru 3 At Sacn 1 /Universe 10");
    assert!(run(&mut state, "Unassign Universe 2").contains("E-OUTPUT-007"), "the middle of a run");
    assert!(run(&mut state, "Unassign Universe 1").contains("Took 1"));
    assert_eq!(lumacore_tree::LineOps::routes_of(&state.root.show_data, "Sacn", 1), vec![(2, 11), (3, 12)]);
    // The run's settings are the line's own.
    run(&mut state, "Set Sacn 1 \"Amount\" 3");
    run(&mut state, "Set Sacn 1 \"SacnUniverse\" 20");
    assert_eq!(lumacore_tree::LineOps::routes_of(&state.root.show_data, "Sacn", 1), vec![(2, 20), (3, 21), (4, 22)]);
    assert!(run(&mut state, "Set Sacn 1 \"SacnUniverse\" 63998").contains("E-OUTPUT-005"));
    // Lines are created live, without the Devices claim.
    assert!(run(&mut state, "Store ArtNet 5").contains("ArtNet 5"));
    let listed = run(&mut state, "List Sacn");
    assert!(listed.contains("2–4 → 20–22") && listed.contains("not mapped"), "{listed}");
}

#[test]
fn protocol_lines_are_numbered_per_protocol_and_deleted_alone() {
    let mut state = with_universes(1);
    run(&mut state, "Store Sacn 1");
    run(&mut state, "Store ArtNet 1");
    assert!(matches!(dispatch(&mut state, "Delete ArtNet 1"), DispatchResult::Ask(_)), "deleting a line asks");
    assert!(run(&mut state, "Delete ArtNet 1 /NoConfirmation").contains("ArtNet 1 deleted"));
    assert!(state.root.show_data.protocols.line("Sacn", 1).is_some());
    assert!(run(&mut state, "Delete ArtNet 1 /nc").contains("E-CMD-007"), "the one resolver answers");
    // OSC lines take no universes.
    run(&mut state, "Store Osc 2 /User \"Admin\"");
    assert!(run(&mut state, "Assign Universe 1 At Osc 2 /Universe 1").contains("E-OUTPUT-006"));
    // The tree has each protocol under its kind's group, its lines by number.
    let dmx = run(&mut state, "ls Root.ShowData.Protocols.DMXProtocols");
    assert!(dmx.contains("Sacn") && dmx.contains("ArtNet") && !dmx.contains("Osc"), "{dmx}");
    let data = run(&mut state, "ls Root.ShowData.Protocols.DataProtocols.Osc");
    assert!(data.contains('2'), "{data}");
}

/// R1-54: two output lines of one protocol can't send the same wire
/// universe on one network to the same destination; unicast lines to
/// different IPs can.
#[test]
fn two_lines_never_send_one_wire_universe_to_one_destination() {
    let mut state = with_universes(2);
    run(&mut state, "Store Sacn 1 /Network \"Rig\"");
    run(&mut state, "Store Sacn 2 /Network \"Rig\"");
    assert!(run(&mut state, "Assign Universe 1 At Sacn 1 /Universe 5").contains("1 → 5"));
    let out = run(&mut state, "Assign Universe 2 At Sacn 2 /Universe 5");
    assert!(out.contains("E-OUTPUT-003") && out.contains("Sacn 1"), "{out}");
    // Another wire universe, another protocol, or another network is fine.
    assert!(run(&mut state, "Assign Universe 2 At Sacn 2 /Universe 6").contains("2 → 6"));
    run(&mut state, "Store ArtNet 3 /Network \"Rig\"");
    assert!(run(&mut state, "Assign Universe 2 At ArtNet 3 /Universe 5").contains("2 → 0.0.5"));
    run(&mut state, "Store Sacn 4 /Network \"Other\"");
    assert!(run(&mut state, "Assign Universe 2 At Sacn 4 /Universe 5").contains("2 → 5"));
    // The same local universes on two lines are fine (decisions §34 "34f7c hardened" 4).
    run(&mut state, "Unassign Sacn 2");
    run(&mut state, "Set Sacn 1 \"Target\" \"10.0.0.50\"");
    run(&mut state, "Set Sacn 2 \"Target\" \"10.0.0.51\"");
    assert!(run(&mut state, "Assign Universe 1 At Sacn 2 /Universe 5").contains("1 → 5"));
    // Changing a line's Target into a clash is refused, the line unchanged.
    let out = run(&mut state, "Set Sacn 2 \"Target\" \"10.0.0.50\"");
    assert!(out.contains("E-OUTPUT-003") && !out.contains("E-SET"), "{out}");
    assert_eq!(state.root.show_data.protocols.line("Sacn", 2).unwrap().target, Some(Ipv4Addr::new(10, 0, 0, 51)), "unchanged");
}

#[test]
fn values_follow_the_declaration() {
    let s = one("Store Osc 2 \"Companion\" /Network \"Rig\" /User \"LD\" /Port 8000");
    assert_eq!(
        names(&s),
        vec![("Network", Some("Rig")), ("User", Some("LD")), ("Port", Some("8000"))]
    );
    // A switch doesn't take the next word.
    let s = one("Store Group 3 /Overwrite \"Front\"");
    assert_eq!(names(&s), vec![("Overwrite", None)]);
    // Words up to the next flag; inline values.
    let s = one("Menu \"DMX Protocols\" /Tab Lines In /nc");
    assert_eq!(
        names(&s),
        vec![("Tab", Some("Lines In")), ("NoConfirmation", None)]
    );
    let s = one("Store Window 5 /Screen=2");
    assert_eq!(names(&s), vec![("Screen", Some("2"))]);
    let s = one("Fixture 1 /Watch=500");
    assert_eq!(names(&s), vec![("Watch", Some("500"))]);
}

#[test]
fn help_lists_a_words_options_and_the_global_ones() {
    let mut state = AppState::new();
    let out = run(&mut state, "Help Group");
    assert!(
        out.contains("Options:")
            && out.contains("/Overwrite (/o)")
            && out.contains("[Store, Copy, Paste]"),
        "{out}"
    );
    assert!(
        out.contains("On every line:") && out.contains("/NoConfirmation (/nc)"),
        "{out}"
    );
    let out = run(&mut state, "Help Sacn");
    assert!(out.contains("/SacnUniverse <universe>") && out.contains("[Store]"), "{out}");
}

/// Decisions §31 "OSC line users": deleting a user an input line runs as asks first and names
/// the line; going on stops it (no user) and the broken link is listed.
#[test]
fn deleting_a_user_an_osc_line_runs_as_names_the_line() {
    let mut state = AppState::new();
    for line in [
        "Store Network \"Rig\"",
        "Store User \"Osc\" /Rights Program",
        "Store User \"Spare\" /Rights Program",
        "Store Osc 3 /Network \"Rig\" /Port 9000 /User \"Osc\"",
    ] {
        run(&mut state, line);
    }
    let rows = |a: &lumacore_objects::dialogs::Ask| -> Vec<(String, String)> {
        a.view()
            .rows
            .iter()
            .map(|r| (r.k.clone(), r.v.clone()))
            .collect()
    };
    let one = rows(&ask(&mut state, "Delete User \"Osc\""));
    assert!(
        one.iter()
            .any(|(k, v)| k == "Lines" && v.contains("used by Osc 3")),
        "{one:?}"
    );
    let several = rows(&ask(&mut state, "Delete User \"Osc\" + \"Spare\""));
    assert!(
        several
            .iter()
            .any(|(k, v)| k == "User Osc" && v.contains("Osc 3")),
        "{several:?}"
    );
    assert!(
        !several.iter().any(|(k, _)| k == "User Spare"),
        "{several:?}"
    );
    let out = run(&mut state, "Delete User \"Osc\" /NoConfirmation");
    assert!(out.contains("Osc 3 stopped: no user"), "{out}");
    let broken = run(&mut state, "List Reference /Broken");
    assert!(
        broken.contains("Protocols.DataProtocols.Osc.Lines.3.User → missing User"),
        "{broken}"
    );
}

#[test]
fn patch_draft_refuses_other_users_patch_edits() {
    let mut state = AppState::new();
    let mut ev = Vec::new();
    lumacore_tree::UniverseOps::create_universe(&mut state.root.show_data, None, &mut ev);
    run(&mut state, "Claim Root.ShowData.Devices");
    state.user = "Debug".to_string();
    state.root.runtime.user("Debug");
    let out = run(&mut state, "Set Universe 1 \"MergeMode\" HTP");
    assert!(
        out.contains("E-PATCH-008") && out.contains("Admin"),
        "{out}"
    );
    assert_eq!(
        state.root.show_data.devices.dmx.patch.universes[&1].merge_mode,
        lumacore_protocols::MergeMode::Prio
    );
    // Protocol lines aren't patch: they stay live for everyone.
    assert!(run(&mut state, "Store Sacn 1").contains("Sacn 1"));
}

/// The question a line asks (its dialog as JSON) and its first button's line.
fn asked(state: &mut AppState, line: &str) -> (String, String) {
    match dispatch(state, line) {
        DispatchResult::Ask(a) => (serde_json::to_string(a.view()).unwrap_or_default(), a.line_of(&a.view().buttons[0].label).unwrap_or_default()),
        _ => panic!("'{line}' must ask"),
    }
}

/// Issue 11 points 4–5 (decisions §11 "Issue 11 hardening", 34g4 part 2b): in the Devices draft
/// the lines stay on their slots — Move asks when inputs stay behind, Delete names the fixtures
/// (they lose their address) and the lines, Keep names lines left on empty slots.
#[test]
fn the_draft_moves_and_deletes_universes_the_lines_stay_on_their_slots() {
    let mut state = with_universes(3);
    run(&mut state, "Store Sacn 1 \"Rig\" /Network \"Rig\"");
    run(&mut state, "Store Sacn 2 \"In\" /Network \"Rig\" /Mode Input");
    run(&mut state, "Assign Universe 1 Thru 3 At Sacn 1 /Universe 1");
    run(&mut state, "Assign Sacn 2 At Universe 2 /Universe 10");
    run(&mut state, "Claim Root.ShowData.Devices");
    // Move: the input on slot 2 stays behind.
    let (text, answer) = asked(&mut state, "Move Universe 2 At 5");
    assert!(text.contains("Sacn 2") && text.contains("MergeMode goes with the universe"), "{text}");
    assert!(answer.ends_with("/NoConfirmation"), "{answer}");
    run(&mut state, &answer);
    let universes = &state.root.show_data.devices.dmx.patch.universes;
    assert!(universes.contains_key(&5) && !universes.contains_key(&2), "moved in the draft");
    let sacn2 = state.root.show_data.protocols.node("Sacn").unwrap().lines[&2].local_universe;
    assert_eq!(sacn2, 2, "the line stays on its slot");
    // Delete: the lines on the slot are named; the answer deletes it.
    let (text, answer) = asked(&mut state, "Delete Universe 1");
    assert!(text.contains("Sacn 1"), "{text}");
    run(&mut state, &answer);
    assert!(!state.root.show_data.devices.dmx.patch.universes.contains_key(&1));
    // Keep's question names the lines left on slots without a universe.
    let (text, _) = asked(&mut state, "Unclaim Root.ShowData.Devices");
    assert!(text.contains("without a universe") && text.contains("Sacn 1") && text.contains("Sacn 2"), "{text}");
}
