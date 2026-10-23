use lumacore_objects::AppState;
use lumacore_tree::{SessionClientRuntime, SessionStationRuntime};
use uuid::Uuid;

use crate::defs::*;
use crate::harness::*;

fn step_watch_clock(state: &mut AppState) -> Result<(), String> {
    run_cmd(state, "watch clock")?;
    let watches = state.watches();
    if watches.len() != 1 {
        return Err(format!("expected 1 watch, got {}", watches.len()));
    }
    let watch = &watches[0];
    if watch.command != "clock" {
        return Err(format!("expected clock watch, got {}", watch.command));
    }
    if watch.interval_ms != 200 {
        return Err(format!(
            "expected 200ms interval, got {}",
            watch.interval_ms
        ));
    }
    Ok(())
}

fn step_session_network_surface(state: &mut AppState) -> Result<(), String> {
    let station_name = state.root.station_settings.station_name.clone();
    state.root.runtime.luma_sessions.listeners.insert(
        "Cmdline".to_string(),
        lumacore_tree::NetworkListenerRuntime {
            name: "Cmdline".to_string(),
            transport: "UDP".to_string(),
            bind: "0.0.0.0".to_string(),
            port: 47653,
            active: true,
        },
    );
    state.root.runtime.network_engines.insert(
        "SessionNetwork".to_string(),
        lumacore_tree::EngineRuntime {
            running: true,
            revision: 1,
            last_error: None,
        },
    );
    state.root.runtime.session_engines.insert(
        "SessionAuthority".to_string(),
        lumacore_tree::EngineRuntime {
            running: true,
            revision: 1,
            last_error: None,
        },
    );

    let session = state.root.runtime.user("Admin");
    session.authority = station_name.clone();
    session.attached = true;
    session.stations.insert(
        station_name.clone(),
        SessionStationRuntime {
            name: station_name.clone(),
            role: "Authority".to_string(),
            endpoint: "0.0.0.0:47654".to_string(),
            connected: true,
            ..Default::default()
        },
    );
    let client_id = Uuid::new_v4();
    session.clients.insert(
        client_id.to_string(),
        SessionClientRuntime {
            id: client_id,
            kind: "Shell".to_string(),
            remote_addr: "127.0.0.1:55000".to_string(),
            width: 120,
            height: 40,
            connected: true,
        },
    );

    let runtime = dispatch_output(state, "list runtime")?;
    if !runtime.contains("Users") || !runtime.contains("NetworkEngines") {
        return Err(format!("list runtime missing expected branches: {runtime}"));
    }

    let sessions = dispatch_output(state, "list shell")?;
    if !sessions.contains("Admin") {
        return Err(format!(
            "list shell missing Admin runtime session: {sessions}"
        ));
    }

    let server = dispatch_output(state, "info server")?;
    if !server.contains("Shell") || !server.contains("Users") {
        return Err(format!(
            "info server missing session-network fields: {server}"
        ));
    }

    let session_info = dispatch_output(state, "info shell Admin")?;
    if !session_info.contains("Authority") || !session_info.contains("Clients") {
        return Err(format!(
            "info shell missing authority/client fields: {session_info}"
        ));
    }

    Ok(())
}

pub fn tests() -> Vec<TestDef> {
    vec![
        TestDef {
            name: "Clock Watch Registers Runtime Watch",
            user: "Admin",
            gates: vec![GateDef {
                name: "DemoShowExists",
                check: super::gate_demo_show_exists,
            }],
            steps: vec![StepDef {
                name: "watch clock adds runtime watch entry",
                kind: StepKind::Verify(step_watch_clock),
            }],
        },
        TestDef {
            name: "Session Network Runtime Is Inspectable",
            user: "Admin",
            gates: vec![GateDef {
                name: "DemoShowExists",
                check: super::gate_demo_show_exists,
            }],
            steps: vec![StepDef {
                name: "runtime session network surfaces appear in list and info",
                kind: StepKind::Verify(step_session_network_surface),
            }],
        },
    ]
}
