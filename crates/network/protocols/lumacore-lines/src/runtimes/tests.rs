// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `runtimes`.
//!
//! Owns:          the unit tests of runtimes.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §11 "Step 4b/4c hardening", §21 "6c", §34 22.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;
use lumacore_protocols::InputPlan;
use lumacore_tree::LineOps;

fn nic() -> Vec<Nic> {
    vec![Nic {
        name: "en1".into(),
        ip: Ipv4Addr::new(10, 0, 0, 5),
        netmask: Ipv4Addr::new(255, 255, 255, 0),
        broadcast: Ipv4Addr::new(10, 0, 0, 255),
        index: None,
    }]
}

fn with_line(state: &mut AppState, protocol: &str) -> u32 {
    let show = &mut state.root.show_data;
    let mut ev = Vec::new();
    lumacore_tree::UniverseOps::create_universe(show, None, &mut ev);
    let line = LineOps::create_line(show, protocol, Direction::Out, None, None, "Rig").unwrap();
    LineOps::set_run(show, protocol, line, Direction::Out, &[1], Some(1), &mut ev).unwrap();
    line
}

#[test]
fn unmapped_networks_never_send() {
    let mut state = AppState::new();
    with_line(&mut state, "Sacn");
    let d = desired(&state, &nic());
    assert!(d.drivers.is_empty());
    assert_eq!(d.status[0].state, LineState::NotMapped);
    // Mapped to a NIC this machine doesn't have: still silent.
    state
        .root
        .station_settings
        .networks
        .insert("rig".into(), "en9".into());
    assert_eq!(
        desired(&state, &nic()).status[0].state,
        LineState::NoInterface("en9".into())
    );
}

#[test]
fn mapped_lines_send_on_their_nic_unless_not_master() {
    let mut state = AppState::new();
    let line = with_line(&mut state, "ArtNet");
    state
        .root
        .station_settings
        .networks
        .insert("rig".into(), "10.0.0.0/24".into());
    let d = desired(&state, &nic());
    let plan = &d.drivers[&("ArtNet".to_string(), Ipv4Addr::new(10, 0, 0, 5))];
    assert_eq!(plan.nic.broadcast, Ipv4Addr::new(10, 0, 0, 255));
    assert_eq!(plan.outputs[0].line, line);
    assert_eq!(plan.outputs[0].routes, vec![(1, 1)]);
    // A joined station that isn't master stays silent.
    state.root.runtime.joined_sessions.insert(
        "s".into(),
        lumacore_tree::JoinedSessionRuntime {
            target: String::new(),
            user: String::new(),
            authority: String::new(),
            cmd_port: 0,
            shell_port: 0,
            connected: true,
            last_error: None,
        },
    );
    state.is_authority = false;
    let d = desired(&state, &nic());
    assert!(d.drivers.is_empty());
    assert_eq!(d.status[0].state, LineState::NotMaster);
}

#[test]
fn input_lines_join_the_driver_of_their_nic() {
    let mut state = AppState::new();
    let out = with_line(&mut state, "Sacn");
    let show = &mut state.root.show_data;
    let input = LineOps::create_line(show, "Sacn", Direction::In, None, None, "Rig").unwrap();
    let mut ev = Vec::new();
    LineOps::set_run(show, "Sacn", input, Direction::In, &[1], Some(7), &mut ev).unwrap();
    show.protocols.lines_mut("Sacn").unwrap().get_mut(&input).unwrap().input_priority =
        lumacore_protocols::InputPriority::Super;
    // Unmapped: neither sends nor listens.
    let d = desired(&state, &nic());
    assert!(d.drivers.is_empty());
    assert_eq!(d.status[1].state, LineState::NotMapped);
    state
        .root
        .station_settings
        .networks
        .insert("rig".into(), "en1".into());
    let d = desired(&state, &nic());
    let plan = &d.drivers[&("Sacn".to_string(), Ipv4Addr::new(10, 0, 0, 5))];
    assert_eq!(plan.outputs[0].line, out);
    assert_eq!(
        plan.inputs,
        vec![InputPlan {
            line: input,
            source: None,
            band: lumacore_protocols::InputPriority::Super,
            routes: vec![(1, 7)],
        }]
    );
    assert!(matches!(
        d.status[1].state,
        LineState::Receiving { universes: 1, live: 0, .. }
    ));
    assert!(d.status[1].runtime().state.starts_with("no source"));
}

/// OSC input listens only where DMX input does (R1-25): a broadcast OSC
/// message must run once, on the station that sends.
#[test]
fn osc_listens_only_on_the_station_that_sends() {
    let mut state = AppState::new();
    let show = &mut state.root.show_data;
    let osc = LineOps::create_line(show, "Osc", Direction::In, None, None, "Rig").unwrap();
    // An OSC line names its user; Admin chosen explicitly is fine (decisions §31 "OSC line users").
    let user = show.user("Admin").unwrap().meta.identity.guid;
    let line = show.protocols.lines_mut("Osc").unwrap().get_mut(&osc).unwrap();
    line.settings.insert("Port".into(), 8000.into());
    line.user = Some(user);
    state.root.station_settings.networks.insert("rig".into(), "en1".into());
    let state_of = |state: &AppState| {
        let d = desired(state, &nic());
        (d.control.contains_key(&("Osc".to_string(), osc)), d.status[0].state.clone())
    };
    // Standalone: listens.
    let (listens, st) = state_of(&state);
    assert!(listens, "{st:?}");
    assert!(matches!(st, LineState::Listening { port: 8000, .. }), "{st:?}");
    // A member: silent.
    let (handle, _rx) = lumacore_objects::session_link::LinkHandle::test_handle("10.0.0.1:47658");
    state.link = Some(handle);
    state.is_authority = false;
    assert_eq!(state_of(&state), (false, LineState::NotMaster));
    state.link = None;
    state.is_authority = true;
    // A fenced master: silent.
    state.session_role = lumacore_objects::session_link::Role::Fenced { by: "FOH-3".into() };
    assert_eq!(state_of(&state), (false, LineState::NotMaster));
    // An IdleMaster: only with output when alone.
    state.session_role = lumacore_objects::session_link::Role::IdleMaster {
        diverged: false,
        lost: "FOH-1".into(),
        quiet: false,
    };
    assert_eq!(state_of(&state), (false, LineState::NotMaster));
    state.root.station_settings.output_when_alone = true;
    assert!(state_of(&state).0);
    // Offline: silent.
    state.root.station_settings.online = false;
    assert_eq!(state_of(&state), (false, LineState::Offline));
}
