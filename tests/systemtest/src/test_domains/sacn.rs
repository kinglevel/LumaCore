use lumacore_objects::AppState;

use crate::defs::*;

fn step_demo_show_has_sacn(state: &mut AppState) -> Result<(), String> {
    let show = &state.root.show_data;
    let routed = show
        .protocols
        .lines("Sacn")
        .into_iter()
        .flatten()
        .filter(|(_, l)| l.sends())
        .map(|(&l, _)| lumacore_tree::LineOps::routes_of(show, "Sacn", l).len())
        .sum::<usize>();
    if routed < 2 {
        return Err(format!(
            "demo show should route at least two universes to sACN, got {routed}"
        ));
    }
    Ok(())
}

/// Decisions §11: a line on a network this station hasn't mapped never
/// sends; mapped to an interface it does.
fn step_unmapped_network_is_silent(state: &mut AppState) -> Result<(), String> {
    use lumacore_objects::protocol_plan::{LineState, desired};
    let nics = lumacore_net::nics::list();
    let saved = state.root.station_settings.networks.clone();
    state.root.station_settings.networks.clear();
    let d = desired(state, &nics);
    if !d.drivers.is_empty() {
        return Err("nothing may send with no network mapped".into());
    }
    if !d.status.iter().any(|s| s.state == LineState::NotMapped) {
        return Err(format!("expected a NotMapped line, got {:?}", d.status));
    }
    let Some(lo) = nics.iter().find(|n| n.ip.is_loopback()) else {
        state.root.station_settings.networks = saved;
        return Ok(()); // no loopback interface to map in this environment
    };
    let networks: Vec<String> = state
        .root
        .show_data
        .protocols
        .all_lines()
        .filter(|(_, _, l)| l.sends())
        .map(|(_, _, l)| state.root.show_data.protocols.network_name(l.network).to_lowercase())
        .collect();
    for n in networks {
        state
            .root
            .station_settings
            .networks
            .insert(n, lo.ip.to_string());
    }
    let d = desired(state, &nics);
    state.root.station_settings.networks = saved;
    if d.drivers.is_empty() {
        return Err(format!("mapped sACN lines should send: {:?}", d.status));
    }
    Ok(())
}

pub fn tests() -> Vec<TestDef> {
    vec![
        TestDef {
            name: "Demo Show Ships sACN Outputs",
            user: "Admin",
            gates: vec![GateDef {
                name: "DemoShowExists",
                check: super::gate_demo_show_exists,
            }],
            steps: vec![StepDef {
                name: "demo show routes universes to an sACN line",
                kind: StepKind::Verify(step_demo_show_has_sacn),
            }],
        },
        TestDef {
            name: "Unmapped Networks Never Send",
            user: "Admin",
            gates: vec![GateDef {
                name: "DemoShowExists",
                check: super::gate_demo_show_exists,
            }],
            steps: vec![StepDef {
                name: "lines send only once their network is mapped to an interface",
                kind: StepKind::Verify(step_unmapped_network_is_silent),
            }],
        },
    ]
}
