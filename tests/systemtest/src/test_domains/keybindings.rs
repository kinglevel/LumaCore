use lumacore_objects::AppState;

use crate::defs::*;
use crate::harness::*;

fn step_bind_and_unbind(state: &mut AppState) -> Result<(), String> {
    // Bind F5 to a command line (written in backticks, lum.md)
    run_cmd(state, "Bind F5 `Fixture Thru`")?;

    // Verify binding exists in Admin profile
    use lumacore_tree::TreeNode;
    let profile = state
        .root
        .show_data
        .user_profiles
        .values()
        .find(|p| p.name() == Some("Admin"))
        .ok_or_else(|| "Admin profile not found".to_string())?;

    let has_f5 = profile
        .content.keybindings
        .values()
        .any(|b| b.key.to_lowercase().contains("f5"));
    if !has_f5 {
        return Err(format!(
            "F5 binding not found in keybindings: {:?}",
            profile.content.keybindings.keys().collect::<Vec<_>>()
        ));
    }

    // Unbind F5
    run_cmd(state, "Unbind F5")?;

    // Verify binding is removed
    let profile = state
        .root
        .show_data
        .user_profiles
        .values()
        .find(|p| p.name() == Some("Admin"))
        .ok_or_else(|| "Admin profile not found after unbind".to_string())?;

    let still_has_f5 = profile
        .content.keybindings
        .values()
        .any(|b| b.key.to_lowercase().contains("f5"));
    if still_has_f5 {
        return Err("F5 binding still present after Unbind".to_string());
    }

    Ok(())
}

pub fn tests() -> Vec<TestDef> {
    vec![TestDef {
        name: "Bind And Unbind Key",
        user: "Admin",
        gates: vec![GateDef {
            name: "DemoShowExists",
            check: super::gate_demo_show_exists,
        }],
        steps: vec![StepDef {
            name: "Bind F5 then Unbind F5 adds and removes keybinding",
            kind: StepKind::Verify(step_bind_and_unbind),
        }],
    }]
}
