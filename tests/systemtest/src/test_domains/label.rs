use lumacore_objects::AppState;
use lumacore_tree::TreeNode;

use crate::defs::*;
use crate::harness::*;

// ── Single-object labeling ──────────────────────────────────────────────────

fn step_label_fixture_single(state: &mut AppState) -> Result<(), String> {
    run_cmd(state, "Label Fixture 101 \"FixtureLabel\"")?;
    let name = state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .fixture(101)
        .ok_or("Fixture 101 not found")?
        .name();
    if name != Some("FixtureLabel") {
        return Err(format!("expected 'FixtureLabel', got {:?}", name));
    }
    Ok(())
}

fn step_label_universe_single(state: &mut AppState) -> Result<(), String> {
    run_cmd(state, "Label Universe 1 \"simpleName\"")?;
    let name = state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .universes
        .get(&1)
        .ok_or("Universe 1 not found")?
        .name();
    if name != Some("simpleName") {
        return Err(format!("expected 'simpleName', got {:?}", name));
    }
    Ok(())
}

fn step_label_group_single(state: &mut AppState) -> Result<(), String> {
    // Store a group first
    run_cmd(state, "fixture 101")?;
    run_cmd(state, "Store Group 1")?;
    run_cmd(state, "Label Group 1 \"MyGroup\"")?;

    let dp_id = state.selected_datapool();
    let name = state
        .root
        .show_data
        .datapools
        .get(&dp_id)
        .and_then(|dp| dp.groups.get(&1))
        .ok_or("Group 1 not found")?
        .name();
    if name != Some("MyGroup") {
        return Err(format!("expected 'MyGroup', got {:?}", name));
    }
    Ok(())
}

// ── Name alias ──────────────────────────────────────────────────────────────

fn step_name_alias_works(state: &mut AppState) -> Result<(), String> {
    run_cmd(state, "Name Fixture 101 \"AliasTest\"")?;
    let name = state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .fixture(101)
        .ok_or("Fixture 101 not found")?
        .name();
    if name != Some("AliasTest") {
        return Err(format!("expected 'AliasTest', got {:?}", name));
    }
    Ok(())
}

// ── Empty string clears label ───────────────────────────────────────────────

fn step_label_clear(state: &mut AppState) -> Result<(), String> {
    // Set a name first, then clear it
    run_cmd(state, "Label Fixture 101 \"TempName\"")?;
    let name = state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .fixture(101)
        .ok_or("Fixture 101 not found")?
        .name();
    if name != Some("TempName") {
        return Err(format!("pre-clear: expected 'TempName', got {:?}", name));
    }

    // A bare Label asks for the new name; /Clear clears without asking
    run_cmd(state, "Label Fixture 101 /Clear")?;
    let name = state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .fixture(101)
        .ok_or("Fixture 101 not found")?
        .name();
    if name.is_some() {
        return Err(format!("expected None after clear, got {:?}", name));
    }
    Ok(())
}

// ── Thru range with auto-enumeration ────────────────────────────────────────

fn step_label_universe_thru_enumeration(state: &mut AppState) -> Result<(), String> {
    // Demo show should have universes — label 1 thru however many exist
    let universe_count = state.root.show_data.devices.dmx.patch.universes.len() as u32;
    if universe_count < 2 {
        return Err(format!("need at least 2 universes, got {universe_count}"));
    }
    let end = universe_count.min(10);

    let cmd = format!("Label Universe 1 Thru {end} \"hello 0\"");
    run_cmd(state, &cmd)?;

    // Verify: "hello 0", "hello 1", ..., "hello {end-1}"
    for id in 1..=end {
        let name = state
            .root
            .show_data
            .devices
            .dmx
            .patch
            .universes
            .get(&id)
            .ok_or_else(|| format!("Universe {id} not found"))?
            .name()
            .ok_or_else(|| format!("Universe {id} has no name"))?;
        let expected = format!("hello {}", id - 1);
        if name != expected {
            return Err(format!(
                "Universe {id}: expected '{}', got '{}'",
                expected, name
            ));
        }
    }
    Ok(())
}

fn step_label_fixture_thru_enumeration(state: &mut AppState) -> Result<(), String> {
    let fixture_count = state.root.show_data.devices.dmx.patch.fixtures.len() as u32;
    if fixture_count < 3 {
        return Err(format!("need at least 3 fixtures, got {fixture_count}"));
    }
    let end = fixture_count.min(5);

    // The demo's fixtures are 101–106.
    let cmd = format!("Label Fixture 101 Thru {} \"Wash 1\"", 100 + end);
    run_cmd(state, &cmd)?;

    // Verify: "Wash 1", "Wash 2", ..., "Wash {end}"
    for n in 1..=end {
        let id = 100 + n;
        let name = state
            .root
            .show_data
            .devices
            .dmx
            .patch
            .fixture(id)
            .ok_or_else(|| format!("Fixture {id} not found"))?
            .name()
            .ok_or_else(|| format!("Fixture {id} has no name"))?;
        let expected = format!("Wash {n}");
        if name != expected {
            return Err(format!(
                "Fixture {id}: expected '{}', got '{}'",
                expected, name
            ));
        }
    }
    Ok(())
}

// ── Character filtering ─────────────────────────────────────────────────────

fn step_label_strips_restricted_chars(state: &mut AppState) -> Result<(), String> {
    let output = dispatch_output(state, "Label Fixture 101 \"test$name\"")?;
    let plain = strip_ansi(&output).to_lowercase();
    if !plain.contains("restricted") {
        return Err(format!("expected stripping note, got: {output}"));
    }

    let name = state
        .root
        .show_data
        .devices
        .dmx
        .patch
        .fixture(101)
        .ok_or("Fixture 101 not found")?
        .name();
    if name != Some("testname") {
        return Err(format!("expected 'testname', got {:?}", name));
    }
    Ok(())
}

// ── Error cases ─────────────────────────────────────────────────────────────

fn step_label_not_found_returns_error(state: &mut AppState) -> Result<(), String> {
    let output = dispatch_output(state, "Label Fixture 99999 \"Ghost\"")?;
    let plain = strip_ansi(&output).to_lowercase();
    // The one resolver's reply (decisions §33 57).
    if !plain.contains("no such fixture") || !plain.contains("e-cmd-007") {
        return Err(format!("expected 'No such Fixture' [E-CMD-007], got: {output}"));
    }
    Ok(())
}

// ── Test definitions ────────────────────────────────────────────────────────

fn demo_gate() -> Vec<GateDef> {
    vec![GateDef {
        name: "DemoShowExists",
        check: super::gate_demo_show_exists,
    }]
}

pub fn tests() -> Vec<TestDef> {
    vec![
        TestDef {
            name: "Label Fixture Single",
            user: "Admin",
            gates: demo_gate(),
            steps: vec![StepDef {
                name: "Label Fixture 101 \"FixtureLabel\" sets name",
                kind: StepKind::Verify(step_label_fixture_single),
            }],
        },
        TestDef {
            name: "Label Universe Single",
            user: "Admin",
            gates: demo_gate(),
            steps: vec![StepDef {
                name: "Label Universe 1 \"simpleName\" sets name",
                kind: StepKind::Verify(step_label_universe_single),
            }],
        },
        TestDef {
            name: "Label Group Single",
            user: "Admin",
            gates: demo_gate(),
            steps: vec![StepDef {
                name: "Label Group 1 \"MyGroup\" sets name",
                kind: StepKind::Verify(step_label_group_single),
            }],
        },
        TestDef {
            name: "Name Alias Works",
            user: "Admin",
            gates: demo_gate(),
            steps: vec![StepDef {
                name: "Name Fixture 101 \"AliasTest\" uses label handler",
                kind: StepKind::Verify(step_name_alias_works),
            }],
        },
        TestDef {
            name: "Label Clear Removes Name",
            user: "Admin",
            gates: demo_gate(),
            steps: vec![StepDef {
                name: "Bare Label Fixture 101 clears the name",
                kind: StepKind::Verify(step_label_clear),
            }],
        },
        TestDef {
            name: "Label Universe Thru With Enumeration",
            user: "Admin",
            gates: demo_gate(),
            steps: vec![StepDef {
                name: "Label Universe 1 Thru N \"hello 0\" enumerates",
                kind: StepKind::Verify(step_label_universe_thru_enumeration),
            }],
        },
        TestDef {
            name: "Label Fixture Thru With Enumeration",
            user: "Admin",
            gates: demo_gate(),
            steps: vec![StepDef {
                name: "Label Fixture 101 Thru N \"Wash 1\" enumerates",
                kind: StepKind::Verify(step_label_fixture_thru_enumeration),
            }],
        },
        TestDef {
            name: "Label Strips Restricted Characters",
            user: "Admin",
            gates: demo_gate(),
            steps: vec![StepDef {
                name: "Label Fixture 101 \"test$name\" strips $",
                kind: StepKind::Verify(step_label_strips_restricted_chars),
            }],
        },
        TestDef {
            name: "Label Not Found Returns Error",
            user: "Admin",
            gates: demo_gate(),
            steps: vec![StepDef {
                name: "Label Fixture 99999 returns not found",
                kind: StepKind::Verify(step_label_not_found_returns_error),
            }],
        },
    ]
}
