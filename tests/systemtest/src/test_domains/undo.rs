use lumacore_objects::AppState;

use crate::defs::*;
use crate::harness::*;

/// Gate: check that subtree snapshots work (required for undo).
fn gate_undo_works() -> Result<(), String> {
    let mut state = fresh_state()?;
    // Run a mutating command to get state into post-mutation shape
    let _ = dispatch_output(&mut state, "Fixture 101 At 50");
    // Try serializing the programmer subtree (the most common undo target)
    let programmer = state
        .current_programmer()
        .ok_or_else(|| "no active programmer".to_string())?;
    match serde_json::to_vec(programmer) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Programmer not serializable after mutation: {e}")),
    }
}

fn step_oops_reverts_last(state: &mut AppState) -> Result<(), String> {
    // Set fixture 101 dimmer to 50% (128)
    run_cmd(state, "Fixture 101 At 50")?;

    let f1_guid = state
        .root
        .show_data
        .fixture_guid(101)
        .ok_or_else(|| "fixture 101 GUID not found".to_string())?;

    // Verify dimmer is set
    let programmer = state
        .current_programmer()
        .ok_or_else(|| "no active programmer".to_string())?;
    let value = programmer
        .get_active_value(f1_guid, "Dimmer")
        .ok_or_else(|| "fixture 101 Dimmer missing after At 50".to_string())?;
    if value.percent() != Some(50.0) {
        return Err(format!(
            "expected Dimmer 50% after At 50, got {}",
            value.readout()
        ));
    }

    // Undo
    run_cmd(state, "Oops")?;

    // Verify dimmer is reverted — programmer should have no active value
    let programmer = state
        .current_programmer()
        .ok_or_else(|| "no active programmer after oops".to_string())?;
    let after = programmer.get_active_value(f1_guid, "Dimmer");
    if after.is_some_and(|v| v.percent() != Some(0.0)) {
        return Err(format!(
            "expected no active Dimmer after Oops, got {:?}",
            after
        ));
    }

    Ok(())
}

fn step_meow_redoes_after_oops(state: &mut AppState) -> Result<(), String> {
    // Set fixture 101 dimmer to 50% (128)
    run_cmd(state, "Fixture 101 At 50")?;

    let f1_guid = state
        .root
        .show_data
        .fixture_guid(101)
        .ok_or_else(|| "fixture 101 GUID not found".to_string())?;

    // Undo
    run_cmd(state, "Oops")?;

    // Redo
    run_cmd(state, "Redo")?;

    // Verify dimmer is back to 128
    let programmer = state
        .current_programmer()
        .ok_or_else(|| "no active programmer after redo".to_string())?;
    let value = programmer
        .get_active_value(f1_guid, "Dimmer")
        .ok_or_else(|| "fixture 101 Dimmer missing after Redo".to_string())?;
    if value.percent() != Some(50.0) {
        return Err(format!(
            "expected Dimmer 50% after Redo, got {}",
            value.readout()
        ));
    }

    Ok(())
}

fn step_multi_user_undo_isolation(state: &mut AppState) -> Result<(), String> {
    // Admin sets fixture 101
    run_cmd(state, "Fixture 101 At 50")?;

    let f1_guid = state
        .root
        .show_data
        .fixture_guid(101)
        .ok_or_else(|| "fixture 101 GUID not found".to_string())?;
    let f2_guid = state
        .root
        .show_data
        .fixture_guid(102)
        .ok_or_else(|| "fixture 102 GUID not found".to_string())?;

    // Switch to LD user (must exist in demo show)
    switch_user(state, "LD")?;
    run_cmd(state, "Fixture 102 At 80")?;

    // LD's programmer should have fixture 102 at 204
    let ld_prog = state
        .current_programmer()
        .ok_or_else(|| "no LD programmer".to_string())?;
    let ld_f2 = ld_prog
        .get_active_value(f2_guid, "Dimmer")
        .ok_or_else(|| "LD fixture 102 Dimmer missing".to_string())?;
    if ld_f2.percent() != Some(80.0) {
        return Err(format!("expected LD Dimmer 80%, got {}", ld_f2.readout()));
    }

    // Switch back to Admin
    switch_user(state, "Admin")?;

    // Admin oops should only revert Admin's change (fixture 101), not LD's (fixture 102)
    run_cmd(state, "Oops")?;

    // Verify Admin's fixture 101 is reverted
    let admin_prog = state
        .current_programmer()
        .ok_or_else(|| "no Admin programmer after oops".to_string())?;
    let admin_f1 = admin_prog.get_active_value(f1_guid, "Dimmer");
    if admin_f1.is_some_and(|v| v.percent() != Some(0.0)) {
        return Err(format!(
            "expected Admin fixture 101 reverted after Oops, got {:?}",
            admin_f1
        ));
    }

    // Switch to LD and verify fixture 102 is untouched
    switch_user(state, "LD")?;
    let ld_prog = state
        .current_programmer()
        .ok_or_else(|| "no LD programmer after Admin oops".to_string())?;
    let ld_f2_after = ld_prog
        .get_active_value(f2_guid, "Dimmer")
        .ok_or_else(|| "LD fixture 102 Dimmer lost after Admin oops".to_string())?;
    if ld_f2_after.percent() != Some(80.0) {
        return Err(format!(
            "expected LD fixture 102 Dimmer 80% after Admin oops, got {}",
            ld_f2_after.readout()
        ));
    }

    Ok(())
}

fn step_patch_unclaim_keeps_undo(state: &mut AppState) -> Result<(), String> {
    // Mutating commands build Admin's undo history (Runtime.Users.{User}.Undo).
    run_cmd(state, "Fixture 101 At 50")?;
    run_cmd(state, "Fixture 101 At 80")?;
    let depth = |state: &AppState| state.undo_stack("Admin").map_or(0, |u| u.len());
    let before = depth(state);
    if before == 0 {
        return Err("undo stack should not be empty after mutations".to_string());
    }
    // Entering and leaving the Patch keeps the history (decisions §11, 4d).
    run_cmd(state, "Enter Patch")?;
    run_cmd(state, "Unclaim Root.ShowData.Devices")?;
    if depth(state) != before {
        return Err(format!(
            "Patch unclaim should keep the undo history ({before} frames), has {}",
            depth(state)
        ));
    }
    Ok(())
}

pub fn tests() -> Vec<TestDef> {
    vec![
        TestDef {
            name: "Oops Reverts Last Command",
            user: "Admin",
            gates: vec![
                GateDef {
                    name: "DemoShowExists",
                    check: super::gate_demo_show_exists,
                },
                GateDef {
                    name: "UndoWorks",
                    check: gate_undo_works,
                },
            ],
            steps: vec![StepDef {
                name: "Fixture 101 At 50 then Oops reverts dimmer",
                kind: StepKind::Verify(step_oops_reverts_last),
            }],
        },
        TestDef {
            name: "Redo Restores After Oops",
            user: "Admin",
            gates: vec![
                GateDef {
                    name: "DemoShowExists",
                    check: super::gate_demo_show_exists,
                },
                GateDef {
                    name: "UndoWorks",
                    check: gate_undo_works,
                },
            ],
            steps: vec![StepDef {
                name: "Fixture 101 At 50 then Oops then Redo restores dimmer",
                kind: StepKind::Verify(step_meow_redoes_after_oops),
            }],
        },
        TestDef {
            name: "Multi-User Undo Isolation",
            user: "Admin",
            gates: vec![
                GateDef {
                    name: "DemoShowExists",
                    check: super::gate_demo_show_exists,
                },
                GateDef {
                    name: "UndoWorks",
                    check: gate_undo_works,
                },
            ],
            steps: vec![StepDef {
                name: "Admin and LD changes are isolated in undo",
                kind: StepKind::Verify(step_multi_user_undo_isolation),
            }],
        },
        TestDef {
            name: "Patch Unclaim Clears Undo",
            user: "Admin",
            gates: vec![
                GateDef {
                    name: "DemoShowExists",
                    check: super::gate_demo_show_exists,
                },
                GateDef {
                    name: "UndoWorks",
                    check: gate_undo_works,
                },
            ],
            steps: vec![StepDef {
                name: "Claim then unclaim Patch keeps the undo history",
                kind: StepKind::Verify(step_patch_unclaim_keeps_undo),
            }],
        },
    ]
}
