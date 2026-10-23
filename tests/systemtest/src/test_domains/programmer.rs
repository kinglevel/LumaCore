use lumacore_objects::AppState;

use crate::defs::*;
use crate::harness::*;

fn step_clear_resets_programmer(state: &mut AppState) -> Result<(), String> {
    // Set a value
    run_cmd(state, "Fixture 101 At 50")?;

    let fixture_guid = state
        .root
        .show_data
        .fixture_guid(101)
        .ok_or_else(|| "fixture 101 GUID not found".to_string())?;

    // Verify value is set
    let programmer = state
        .current_programmer()
        .ok_or_else(|| "no active programmer".to_string())?;
    if programmer
        .get_active_value(fixture_guid, "Dimmer")
        .is_none()
    {
        return Err("Dimmer not set after At 50".to_string());
    }

    // Clear three times to fully reset (selection → deactivate → clear)
    run_cmd(state, "reset")?;
    run_cmd(state, "reset")?;
    run_cmd(state, "reset")?;

    // Verify programmer has no active values
    let programmer = state
        .current_programmer()
        .ok_or_else(|| "no active programmer after clear".to_string())?;
    if programmer
        .get_active_value(fixture_guid, "Dimmer")
        .is_some()
    {
        return Err("Dimmer still active after 3x reset".to_string());
    }
    if !programmer.selected_fixtures.is_empty() {
        return Err(format!(
            "selection not empty after 3x reset: {:?}",
            programmer.selected_fixtures
        ));
    }

    Ok(())
}

fn step_park_holds_value(state: &mut AppState) -> Result<(), String> {
    // Select fixture 101
    run_cmd(state, "Fixture 101 At Full")?;

    // Park fixture 101 at 100% (explicit At value ensures non-zero parked channels)
    run_cmd(state, "Park Fixture 101 At 100")?;

    // Parked values live in the show, per fixture attribute.
    let park = &state.root.show_data.playback.park;
    if park.fixtures.is_empty() {
        return Err("nothing parked after Park Fixture 101 At 100".to_string());
    }
    // The Dimmer is parked at 100%.
    let full = park
        .fixtures
        .iter()
        .any(|p| p.attribute == "Dimmer" && p.value.percent() == Some(100.0));
    if !full {
        return Err(format!(
            "expected the Dimmer parked at 100%, got: {:?}",
            park.fixtures
                .iter()
                .map(|p| format!("{} {}", p.attribute, p.value.readout()))
                .collect::<Vec<_>>()
        ));
    }

    Ok(())
}

pub fn tests() -> Vec<TestDef> {
    vec![
        TestDef {
            name: "Clear Resets Programmer State",
            user: "Admin",
            gates: vec![GateDef {
                name: "DemoShowExists",
                check: super::gate_demo_show_exists,
            }],
            steps: vec![StepDef {
                name: "Fixture 101 At 50 then 3x reset clears programmer",
                kind: StepKind::Verify(step_clear_resets_programmer),
            }],
        },
        TestDef {
            name: "Park Holds Fixture Value",
            user: "Admin",
            gates: vec![GateDef {
                name: "DemoShowExists",
                check: super::gate_demo_show_exists,
            }],
            steps: vec![StepDef {
                name: "Fixture 101 At Full then Park creates parked channels",
                kind: StepKind::Verify(step_park_holds_value),
            }],
        },
    ]
}
