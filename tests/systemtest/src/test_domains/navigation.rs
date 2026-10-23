use lumacore_objects::{AppState, ReplContext};

use crate::defs::*;
use crate::harness::*;
use lumacore_objects::navigation::Navigation;

fn step_cd_showdata(state: &mut AppState) -> Result<(), String> {
    run_cmd(state, "cd showdata")?;
    if state.path_string() != "/ShowData" {
        return Err(format!("unexpected path {}", state.path_string()));
    }
    if state.context != ReplContext::Fixture {
        return Err(format!("unexpected context {}", state.context));
    }
    Ok(())
}

pub fn tests() -> Vec<TestDef> {
    vec![TestDef {
        name: "CD ShowData Updates Path",
        user: "Admin",
        gates: vec![GateDef {
            name: "DemoShowExists",
            check: super::gate_demo_show_exists,
        }],
        steps: vec![StepDef {
            name: "cd showdata updates path and context",
            kind: StepKind::Verify(step_cd_showdata),
        }],
    }]
}
