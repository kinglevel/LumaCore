use lumacore_objects::AppState;

use crate::defs::*;
use crate::harness::*;

fn step_tree_shows_patch(state: &mut AppState) -> Result<(), String> {
    let output = dispatch_output(state, "tree ShowData.Devices.Dmx.Patch")?;
    let plain = strip_ansi(&output);

    if !plain.contains("Fixtures") {
        return Err(format!("tree output missing 'Fixtures': {}", plain));
    }
    if !plain.contains("Universes") {
        return Err(format!("tree output missing 'Universes': {}", plain));
    }

    Ok(())
}

fn step_cat_fixture(state: &mut AppState) -> Result<(), String> {
    let output = dispatch_output(state, "cat ShowData.Devices.Dmx.Patch.Fixtures.1")?;
    let plain = strip_ansi(&output);

    // Cat should display fixture fields — at minimum the fixture should exist
    if plain.is_empty() {
        return Err("cat returned empty output for fixture 1".to_string());
    }

    Ok(())
}

fn step_find_universe(state: &mut AppState) -> Result<(), String> {
    // Navigate into ShowData first, then search for Universe pattern
    run_cmd(state, "cd showdata")?;
    let output = dispatch_output(state, "find \"Universe\"")?;
    let plain = strip_ansi(&output);

    let lower = plain.to_lowercase();
    if !lower.contains("universe") {
        return Err(format!("find output missing universe entries: {}", plain));
    }

    Ok(())
}

pub fn tests() -> Vec<TestDef> {
    vec![
        TestDef {
            name: "Tree Shows Patch Hierarchy",
            user: "Admin",
            gates: vec![GateDef {
                name: "DemoShowExists",
                check: super::gate_demo_show_exists,
            }],
            steps: vec![StepDef {
                name: "tree ShowData.Devices.Dmx.Patch contains Fixtures and Universes",
                kind: StepKind::Verify(step_tree_shows_patch),
            }],
        },
        TestDef {
            name: "Cat Displays Fixture Data",
            user: "Admin",
            gates: vec![GateDef {
                name: "DemoShowExists",
                check: super::gate_demo_show_exists,
            }],
            steps: vec![StepDef {
                name: "cat ShowData.Devices.Dmx.Patch.Fixtures.1 returns non-empty output",
                kind: StepKind::Verify(step_cat_fixture),
            }],
        },
        TestDef {
            name: "Find Locates Nodes By Pattern",
            user: "Admin",
            gates: vec![GateDef {
                name: "DemoShowExists",
                check: super::gate_demo_show_exists,
            }],
            steps: vec![StepDef {
                name: "find ShowData Universe returns universe entries",
                kind: StepKind::Verify(step_find_universe),
            }],
        },
    ]
}
