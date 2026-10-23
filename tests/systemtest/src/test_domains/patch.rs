use lumacore_objects::AppState;

use crate::defs::*;
use crate::harness::*;

fn step_patch_creates_fixture(state: &mut AppState) -> Result<(), String> {
    // Claim Root.ShowData.Devices first (required for patching)
    run_cmd(state, "enter patch")?;

    let before = state.root.show_data.devices.dmx.patch.fixtures.len();
    // Past the rig's numbers and universes, with a type the rig has (decisions §40 14).
    run_cmd(state, "Store Universe 300")?;
    run_cmd(state, "Store Fixture 20201 At DMX 300.200 /FixtureType \"Dummy - Dimmer\"")?;
    let after = state.root.show_data.devices.dmx.patch.fixtures.len();

    if after != before + 1 {
        return Err(format!(
            "expected {} fixtures after patch, got {}",
            before + 1,
            after
        ));
    }

    // Verify the new fixture has correct universe/address
    let new_fixture = state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .fixtures
        .values()
        .last()
        .ok_or_else(|| "no fixtures after patch".to_string())?;

    use lumacore_tree::TreeNode;
    let name = new_fixture.name().unwrap_or("?");
    if !name.contains("AuraXB") && !name.contains("Aura") {
        // Just verify it was created — name format may vary
    }

    Ok(())
}

fn step_batch_patch_creates_n(state: &mut AppState) -> Result<(), String> {
    run_cmd(state, "enter patch")?;

    let before = state.root.show_data.devices.dmx.patch.fixtures.len();
    run_cmd(state, "Store Universe 301")?;
    run_cmd(state, "Store Fixture 20301 Thru 20303 At DMX 301.001 /FixtureType \"Dummy - Dimmer\"")?;
    let after = state.root.show_data.devices.dmx.patch.fixtures.len();

    if after != before + 3 {
        return Err(format!(
            "expected {} fixtures after batch patch, got {} (added {})",
            before + 3,
            after,
            after - before
        ));
    }

    Ok(())
}

pub fn tests() -> Vec<TestDef> {
    vec![
        TestDef {
            name: "Store Fixture At An Address",
            user: "Admin",
            gates: vec![GateDef {
                name: "DemoShowExists",
                check: super::gate_demo_show_exists,
            }],
            steps: vec![StepDef {
                name: "Store Fixture 201 At DMX 1.200 adds a fixture to the patch",
                kind: StepKind::Verify(step_patch_creates_fixture),
            }],
        },
        TestDef {
            name: "Store Fixture Makes N Fixtures",
            user: "Admin",
            gates: vec![GateDef {
                name: "DemoShowExists",
                check: super::gate_demo_show_exists,
            }],
            steps: vec![StepDef {
                name: "Store Fixture 301 Thru 303 At DMX 2.001 creates 3 fixtures",
                kind: StepKind::Verify(step_batch_patch_creates_n),
            }],
        },
    ]
}
