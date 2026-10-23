use lumacore_objects::AppState;
use serde::Serialize;

#[derive(Serialize)]
pub struct RunSummary {
    pub build_manifest: String,
    pub selected_groups: Vec<String>,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub groups: Vec<GroupResult>,
    /// Wall time of the representative command batch (ms).
    pub command_batch_wall_ms: Option<u128>,
}

/// Metrics snapshot written to target/systemtest/metrics.json after each run.
#[derive(Serialize)]
pub struct MetricsSnapshot {
    pub timestamp: String,
    pub git_commit: String,
    pub git_branch: String,
    pub command_batch_wall_ms: u128,
    pub verdict: String,
}

#[derive(Serialize)]
pub struct GroupResult {
    pub name: String,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub tests: Vec<TestResult>,
}

#[derive(Serialize)]
pub struct TestResult {
    pub name: String,
    pub status: String,
    pub user: String,
    pub reason: Option<String>,
    pub steps: Vec<StepResult>,
}

#[derive(Serialize)]
pub struct StepResult {
    pub name: String,
    pub status: String,
    pub detail: Option<String>,
}

pub type StepAction = fn(&mut AppState) -> Result<(), String>;

pub struct StepDef {
    pub name: &'static str,
    pub kind: StepKind,
}

pub enum StepKind {
    Verify(StepAction),
    SyntaxCommand(&'static str),
}

pub type GateCheck = fn() -> Result<(), String>;

pub struct GateDef {
    pub name: &'static str,
    pub check: GateCheck,
}

pub struct TestDef {
    pub name: &'static str,
    pub user: &'static str,
    pub gates: Vec<GateDef>,
    pub steps: Vec<StepDef>,
}

pub struct GroupDef {
    pub name: &'static str,
    pub tests: Vec<TestDef>,
}
