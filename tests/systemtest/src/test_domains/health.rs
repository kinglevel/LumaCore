use lumacore_objects::AppState;
use std::time::Instant;

use crate::defs::*;
use crate::harness::*;

/// Run a representative command batch and return its wall time (ms).
pub fn run_command_batch(state: &mut AppState) -> u128 {
    let commands = [
        "fixture 1",
        "at 50",
        "fixture 2",
        "at full",
        "fixture thru",
        "at 0",
        "clear",
    ];

    let batch_start = Instant::now();
    for cmd_str in &commands {
        let _ = dispatch_output(state, cmd_str);
    }
    batch_start.elapsed().as_millis()
}

fn step_command_batch_timing(state: &mut AppState) -> Result<(), String> {
    let wall_ms = run_command_batch(state);

    if wall_ms > 2000 {
        return Err(format!(
            "command batch took {}ms, expected < 2000ms",
            wall_ms
        ));
    }

    println!("    command batch wall time: {}ms", wall_ms);

    Ok(())
}

pub fn tests() -> Vec<TestDef> {
    vec![TestDef {
        name: "Command Batch Timing Under Threshold",
        user: "Admin",
        gates: vec![GateDef {
            name: "DemoShowExists",
            check: super::gate_demo_show_exists,
        }],
        steps: vec![StepDef {
            name: "batch of commands completes within wall-time threshold",
            kind: StepKind::Verify(step_command_batch_timing),
        }],
    }]
}
