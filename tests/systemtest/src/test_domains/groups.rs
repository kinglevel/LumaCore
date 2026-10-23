use lumacore_objects::AppState;

use crate::defs::*;
use crate::harness::*;

fn step_store_and_recall_group(state: &mut AppState) -> Result<(), String> {
    // Select the demo's first two fixtures (101 + 102)
    run_cmd(state, "fixture 101 thru 102")?;
    assert_selected_fixtures(state, &[101, 102])?;

    // Store as Group 1010 (past the rig's 77; storing onto one asks Merge / Overwrite)
    run_cmd(state, "Store Group 1010")?;

    // Clear selection
    run_cmd(state, "reset")?;

    // Recall Group 1010
    run_cmd(state, "Group 1010")?;
    assert_selected_fixtures(state, &[101, 102])?;

    Ok(())
}

fn step_delete_group(state: &mut AppState) -> Result<(), String> {
    // Store a group first
    run_cmd(state, "fixture 101")?;
    run_cmd(state, "Store Group 1011")?;

    // Verify group exists by recalling it
    run_cmd(state, "reset")?;
    run_cmd(state, "Group 1011")?;
    assert_selected_fixtures(state, &[101])?;

    // Delete it
    run_cmd(state, "Delete Group 1011 /nc")?;

    // Verify group is gone — recall should fail
    let output = dispatch_output(state, "Group 1011")?;
    let lower = strip_ansi(&output).to_lowercase();
    if !lower.contains("not found") {
        return Err(format!(
            "expected 'not found' after delete, got: {}",
            strip_ansi(&output)
        ));
    }

    Ok(())
}

pub fn tests() -> Vec<TestDef> {
    vec![
        TestDef {
            name: "Store And Recall Group",
            user: "Admin",
            gates: vec![GateDef {
                name: "DemoShowExists",
                check: super::gate_demo_show_exists,
            }],
            steps: vec![StepDef {
                name: "store group 1010 then recall restores selection",
                kind: StepKind::Verify(step_store_and_recall_group),
            }],
        },
        TestDef {
            name: "Delete Group Removes Entry",
            user: "Admin",
            gates: vec![GateDef {
                name: "DemoShowExists",
                check: super::gate_demo_show_exists,
            }],
            steps: vec![StepDef {
                name: "Delete Group 1011 removes group from show data",
                kind: StepKind::Verify(step_delete_group),
            }],
        },
    ]
}
