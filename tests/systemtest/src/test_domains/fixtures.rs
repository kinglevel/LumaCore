use lumacore_objects::AppState;

use crate::defs::*;
use crate::harness::*;

fn step_fixture_thru_selects_all(state: &mut AppState) -> Result<(), String> {
    run_cmd(state, "fixture thru")?;
    // Every patched fixture, whatever its number (the demo's are 101–106).
    let expected: Vec<u32> = state.root.show_data.devices.dmx.patch.fids().into_iter().map(|(n, _)| n).collect();
    assert_selected_fixtures(state, &expected)
}

fn step_fixture_thru_at_full(state: &mut AppState) -> Result<(), String> {
    run_cmd(state, "fixture thru at full")?;
    let programmer = state
        .current_programmer()
        .ok_or_else(|| "no active programmer".to_string())?;
    let ids: Vec<u32> = state.root.show_data.devices.dmx.patch.fids().into_iter().map(|(n, _)| n).collect();
    // The rig has fixtures without a Dimmer (a folder's type, a marker): every fixture that has
    // one is at full, and most have one.
    let mut dimmers = 0;
    for id in ids {
        let guid = state
            .root
            .show_data
            .fixture_guid(id)
            .ok_or_else(|| format!("fixture {id} GUID not found"))?;
        let Some(value) = programmer.get_active_value(guid, "Dimmer") else { continue };
        dimmers += 1;
        if value.percent() != Some(100.0) {
            return Err(format!(
                "fixture {id} expected Dimmer 100%, got {}",
                value.readout()
            ));
        }
    }
    if dimmers < 1000 {
        return Err(format!("expected ≥1000 fixtures with a Dimmer at full, got {dimmers}"));
    }
    Ok(())
}

pub fn tests() -> Vec<TestDef> {
    vec![
        TestDef {
            name: "Fixture Thru Selects All Fixtures",
            user: "Admin",
            gates: vec![GateDef {
                name: "DemoShowExists",
                check: super::gate_demo_show_exists,
            }],
            steps: vec![StepDef {
                name: "Fixture Thru selects all patched fixtures",
                kind: StepKind::Verify(step_fixture_thru_selects_all),
            }],
        },
        TestDef {
            name: "Fixture Thru At Full Sets Dimmer",
            user: "Admin",
            gates: vec![GateDef {
                name: "DemoShowExists",
                check: super::gate_demo_show_exists,
            }],
            steps: vec![StepDef {
                name: "Fixture Thru At Full writes Dimmer values",
                kind: StepKind::Verify(step_fixture_thru_at_full),
            }],
        },
    ]
}
