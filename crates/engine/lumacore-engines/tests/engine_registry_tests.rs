use std::collections::HashSet;

use lumacore_engines::{Engine, EngineContext, EngineRegistry};
use lumacore_wire::{Capability, EngineInfo, EngineStatus};

/// Minimal test engine for registry tests.
struct DummyEngine {
    engine_name: &'static str,
    caps: Vec<Capability>,
    running: bool,
    port: Option<u16>,
}

impl DummyEngine {
    fn new(name: &'static str, caps: Vec<Capability>) -> Self {
        Self {
            engine_name: name,
            caps,
            running: false,
            port: None,
        }
    }

    fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
}

impl Engine for DummyEngine {
    fn name(&self) -> &str {
        self.engine_name
    }

    fn requires(&self) -> &[Capability] {
        &self.caps
    }

    fn status(&self) -> EngineStatus {
        if self.running {
            EngineStatus::Running
        } else {
            EngineStatus::Stopped
        }
    }

    fn info(&self) -> EngineInfo {
        EngineInfo {
            name: self.engine_name.into(),
            status: self.status(),
            requires: self.caps.clone(),
            port: if self.running { self.port } else { None },
        }
    }

    fn start(&mut self, _ctx: &EngineContext) -> Result<(), String> {
        self.running = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        self.running = false;
        Ok(())
    }
}

fn empty_ctx() -> EngineContext {
    EngineContext {
        show_data: None,
        render: None,
        command_tx: None,
        health: None,
    }
}

#[test]
fn registry_start_stop_lifecycle() {
    let caps = HashSet::from([Capability::Http]);
    let mut reg = EngineRegistry::new(caps, empty_ctx());
    reg.register(Box::new(DummyEngine::new("test", vec![Capability::Http])));

    assert!(reg.start("test").is_ok());

    let info = reg.engine_info();
    assert!(matches!(info[0].status, EngineStatus::Running));

    assert!(reg.stop("test").is_ok());

    let info = reg.engine_info();
    assert!(matches!(info[0].status, EngineStatus::Stopped));
}

#[test]
fn registry_capability_check_blocks_start() {
    // Registry has no Gpu capability
    let caps = HashSet::from([Capability::Http]);
    let mut reg = EngineRegistry::new(caps, empty_ctx());
    reg.register(Box::new(DummyEngine::new("gpu_eng", vec![Capability::Gpu])));

    let result = reg.start("gpu_eng");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("missing capabilities"));
}

#[test]
fn registry_start_all_skips_unsupported() {
    let caps = HashSet::from([Capability::Http]);
    let mut reg = EngineRegistry::new(caps, empty_ctx());
    reg.register(Box::new(DummyEngine::new(
        "http_eng",
        vec![Capability::Http],
    )));
    reg.register(Box::new(DummyEngine::new("gpu_eng", vec![Capability::Gpu])));

    let results = reg.start_all();
    assert_eq!(results.len(), 2);
    assert!(results[0].1.is_ok(), "http_eng should start");
    assert!(results[1].1.is_err(), "gpu_eng should fail");
}

#[test]
fn registry_engine_info_reflects_status() {
    let caps = HashSet::from([Capability::Http]);
    let mut reg = EngineRegistry::new(caps, empty_ctx());
    reg.register(Box::new(
        DummyEngine::new("web", vec![Capability::Http]).with_port(8080),
    ));

    // Before start: stopped, no port
    let info = reg.engine_info();
    assert!(matches!(info[0].status, EngineStatus::Stopped));
    assert_eq!(info[0].port, None);

    reg.start("web").unwrap();

    // After start: running, port visible
    let info = reg.engine_info();
    assert!(matches!(info[0].status, EngineStatus::Running));
    assert_eq!(info[0].port, Some(8080));
}

#[test]
fn registry_can_start_checks_capabilities() {
    let caps = HashSet::from([Capability::Http, Capability::Network]);
    let reg = EngineRegistry::new(caps, empty_ctx());

    // Engine exists check — can_start returns false for unknown engines
    assert!(!reg.can_start("nonexistent"));
}
