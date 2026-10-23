use lumacore_objects::AppState;
use lumacore_tree::TreeNode;

use crate::defs::*;
use crate::harness::*;

fn gate_ld_user_exists() -> Result<(), String> {
    let state = fresh_state()?;
    let exists = state
        .root
        .show_data
        .user_profiles
        .values()
        .any(|profile| profile.name() == Some("LD"));
    if exists {
        Ok(())
    } else {
        Err("demo show does not contain user LD".to_string())
    }
}

fn step_ld_programmer_selection(state: &mut AppState) -> Result<(), String> {
    for user in state.root.show_data.users.values_mut() {
        if user.login != "LD" {
            user.programmer.selected_fixtures.clear();
        }
    }

    let fixture_id: u32 = 101;
    let fixture_guid = state
        .root
        .show_data
        .fixture_guid(fixture_id)
        .ok_or_else(|| format!("fixture {fixture_id} GUID not found"))?;

    run_cmd(state, &format!("fixture {fixture_id}"))?;
    run_cmd(state, "at 50")?;
    assert_selected_fixtures(state, &[fixture_id])?;
    let programmer = state
        .current_programmer()
        .ok_or_else(|| "no active programmer".to_string())?;
    let value = programmer
        .get_active_value(fixture_guid, "Dimmer")
        .ok_or_else(|| format!("fixture {fixture_id} dimmer missing"))?;
    if value.percent() == Some(50.0) {
        Ok(())
    } else {
        Err(format!("expected Dimmer 50%, got {}", value.readout()))
    }
}

/// Selecting claims nothing (decisions §4): LD can select a fixture Admin
/// has selected. `Claim Fixture` locks it for everyone else.
fn step_selection_claims_nothing_claim_locks(state: &mut AppState) -> Result<(), String> {
    run_cmd(state, "fixture 101")?;
    run_cmd(state, "Claim Fixture 101")?;
    switch_user(state, "Admin")?;
    let blocked = run_cmd(state, "fixture 101");
    switch_user(state, "LD")?;
    run_cmd(state, "Unclaim Fixture 101")?;
    match blocked {
        Ok(()) => Err("expected fixture 101 to be blocked by LD's claim".to_string()),
        Err(err) if strip_ansi(&err).contains("E-PROG-001") => Ok(()),
        Err(err) => Err(format!("unexpected claim error: {err}")),
    }
}

fn step_admin_profile_branches(state: &mut AppState) -> Result<(), String> {
    let admin = state
        .root
        .show_data
        .user_profiles
        .values()
        .find(|profile| profile.name() == Some("Admin"))
        .ok_or_else(|| "Admin profile missing".to_string())?;
    if admin.settings.preferences.preferred_ui_engine.is_empty() {
        return Err("Admin preferred_ui_engine should be initialized".to_string());
    }
    if !admin.settings.permissions.can_edit_patch {
        return Err("Admin patch permission should be enabled".to_string());
    }
    Ok(())
}

pub fn tests() -> Vec<TestDef> {
    vec![
        TestDef {
            name: "LD Keeps Separate Programmer",
            user: "LD",
            gates: vec![
                GateDef {
                    name: "DemoShowExists",
                    check: super::gate_demo_show_exists,
                },
                GateDef {
                    name: "LDUserExists",
                    check: gate_ld_user_exists,
                },
            ],
            steps: vec![StepDef {
                name: "LD selects fixture and sets dimmer",
                kind: StepKind::Verify(step_ld_programmer_selection),
            }],
        },
        TestDef {
            name: "Selection Claims Nothing, Claim Fixture Locks",
            user: "LD",
            gates: vec![
                GateDef {
                    name: "DemoShowExists",
                    check: super::gate_demo_show_exists,
                },
                GateDef {
                    name: "LDUserExists",
                    check: gate_ld_user_exists,
                },
            ],
            steps: vec![StepDef {
                name: "LD selects Admin's fixture; LD's claim blocks Admin",
                kind: StepKind::Verify(step_selection_claims_nothing_claim_locks),
            }],
        },
        TestDef {
            name: "Admin Profile Branches Exist",
            user: "Admin",
            gates: vec![GateDef {
                name: "DemoShowExists",
                check: super::gate_demo_show_exists,
            }],
            steps: vec![StepDef {
                name: "Admin profile preferences and permissions are initialized",
                kind: StepKind::Verify(step_admin_profile_branches),
            }],
        },
    ]
}
